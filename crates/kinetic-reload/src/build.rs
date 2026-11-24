//! Building a scene crate with cargo.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A scene crate: a directory with a `Cargo.toml` whose library is built as a `cdylib`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneCrate {
    dir: PathBuf,
}

/// The scene crate could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// The directory has no `Cargo.toml`.
    NotACrate(PathBuf),
    /// `cargo` could not be started.
    Cargo(String),
    /// The compiler reported errors. Holds them as cargo would print them.
    Compile(String),
    /// The build succeeded but produced no loadable library.
    NoLibrary,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotACrate(dir) => write!(f, "{} has no Cargo.toml", dir.display()),
            Self::Cargo(why) => write!(f, "could not run cargo: {why}"),
            Self::Compile(errors) => write!(f, "{errors}"),
            Self::NoLibrary => write!(
                f,
                "the scene built, but not as a library; add `crate-type = [\"cdylib\", \"rlib\"]` \
                 under [lib] in its Cargo.toml"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

impl SceneCrate {
    /// The scene crate in `dir`.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, BuildError> {
        let dir = dir.as_ref().to_path_buf();
        if !dir.join("Cargo.toml").is_file() {
            return Err(BuildError::NotACrate(dir));
        }
        Ok(Self { dir })
    }

    /// The crate's directory.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The directory scene builds go to, separate from any workspace target directory so a
    /// scene build never waits on, or invalidates, the build of the editor running it.
    pub fn default_target_dir() -> PathBuf {
        std::env::temp_dir().join("kinetic").join("target")
    }

    /// Builds the crate's library and returns the path of the built dynamic library.
    pub fn build(&self, target_dir: &Path) -> Result<PathBuf, BuildError> {
        let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
        let output = Command::new(cargo)
            .arg("build")
            .arg("--lib")
            .arg("--message-format=json")
            .arg("--manifest-path")
            .arg(self.dir.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(target_dir)
            .output()
            .map_err(|e| BuildError::Cargo(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut library = None;
        let mut errors = String::new();
        for line in stdout.lines() {
            let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            match message["reason"].as_str() {
                Some("compiler-artifact") => {
                    if let Some(path) = cdylib_path(&message) {
                        library = Some(path);
                    }
                }
                Some("compiler-message") if message["message"]["level"] == "error" => {
                    if let Some(rendered) = message["message"]["rendered"].as_str() {
                        errors.push_str(rendered);
                    }
                }
                _ => {}
            }
        }

        if !output.status.success() {
            if errors.is_empty() {
                // Manifest and dependency problems come from cargo itself, on stderr.
                errors = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            return Err(BuildError::Compile(errors));
        }
        library.ok_or(BuildError::NoLibrary)
    }
}

/// The dynamic library among an artifact message's files, if the artifact is a cdylib.
fn cdylib_path(message: &serde_json::Value) -> Option<PathBuf> {
    let kinds = message["target"]["kind"].as_array()?;
    if !kinds.iter().any(|k| k == "cdylib") {
        return None;
    }
    let wanted = std::env::consts::DLL_EXTENSION;
    message["filenames"]
        .as_array()?
        .iter()
        .filter_map(|f| f.as_str())
        .map(PathBuf::from)
        .find(|p| p.extension().is_some_and(|e| e == wanted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_without_a_manifest_is_refused() {
        let dir = std::env::temp_dir();
        assert_eq!(SceneCrate::open(&dir), Err(BuildError::NotACrate(dir)));
    }

    #[test]
    fn picks_the_dynamic_library_from_an_artifact_message() {
        let ext = std::env::consts::DLL_EXTENSION;
        let message = serde_json::json!({
            "reason": "compiler-artifact",
            "target": { "kind": ["cdylib", "rlib"] },
            "filenames": ["/t/libhello.rlib", format!("/t/hello.{ext}"), "/t/hello.pdb"],
        });
        assert_eq!(cdylib_path(&message), Some(PathBuf::from(format!("/t/hello.{ext}"))));
    }

    #[test]
    fn ignores_artifacts_that_are_not_cdylibs() {
        let message = serde_json::json!({
            "reason": "compiler-artifact",
            "target": { "kind": ["lib"] },
            "filenames": ["/t/libserde.rlib"],
        });
        assert_eq!(cdylib_path(&message), None);
    }
}
