//! `kinetic new`: scaffolds a scene crate.

use std::fmt;
use std::path::{Component, Path, PathBuf};

/// Where scene crates outside this repository get `kinetic-core` from.
pub const REPOSITORY: &str = "https://github.com/hashimhashmi02/kinetic-engine";

/// The scene crate could not be created.
#[derive(Debug, PartialEq, Eq)]
pub enum NewError {
    /// The directory already exists and is not empty.
    Exists(PathBuf),
    /// Reading the current directory or writing a file failed.
    Io(String),
}

impl fmt::Display for NewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exists(dir) => write!(f, "{} already exists and is not empty", dir.display()),
            Self::Io(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for NewError {}

/// Where the new crate's `kinetic-core` dependency comes from.
#[derive(Debug, PartialEq, Eq)]
enum Core {
    /// A relative path into a checkout of this repository.
    Path(String),
    /// The repository on GitHub.
    Git,
}

const TEMPLATE: &str = r#"//! A Kinetic scene. Run `kinetic preview` on this folder, then edit and save this file:
//! the preview rebuilds and reloads it, keeping the playhead where it was.

use kinetic_core::color::palette;
use kinetic_core::{spring, Circle, Ease, Scene};

/// Describes the animation. It runs once; the editor then scrubs the timeline it builds.
pub fn scene(s: &mut Scene) {
    let dot = s.add(Circle::new(60.0).fill(palette::blue()).opacity(0.0));

    s.play(dot.fade_to(1.0), 0.4);
    s.play(dot.move_to((300.0, 0.0)), (0.8, Ease::OutBack));
    s.play(dot.move_to((-300.0, 0.0)), spring().stiffness(180.0).damping(12.0));
    s.wait(0.5);
}

kinetic_core::kinetic_scene!(scene);
"#;

/// Creates a scene crate in `dir` and returns its absolute path.
pub fn create(dir: &Path) -> Result<PathBuf, NewError> {
    let dir = absolute(dir).map_err(|e| NewError::Io(e.to_string()))?;
    if std::fs::read_dir(&dir).map(|mut entries| entries.next().is_some()).unwrap_or(false) {
        return Err(NewError::Exists(dir));
    }
    let raw = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let name = crate_name(&raw);
    let parent = dir.parent().unwrap_or(&dir);
    let workspace = find_workspace(parent);
    let core = match &workspace {
        Some(root) => Core::Path(relative(&dir, &root.join("crates").join("kinetic-core"))),
        None => Core::Git,
    };
    // The repository's workspace takes every crate under scenes/ as a member.
    let member = workspace.as_ref().is_some_and(|root| parent == root.join("scenes"));

    let write = |path: PathBuf, text: &str| {
        std::fs::write(&path, text).map_err(|e| NewError::Io(format!("{}: {e}", path.display())))
    };
    std::fs::create_dir_all(dir.join("src")).map_err(|e| NewError::Io(e.to_string()))?;
    write(dir.join("Cargo.toml"), &manifest(&name, &core, member))?;
    write(dir.join("src").join("lib.rs"), TEMPLATE)?;
    Ok(dir)
}

fn manifest(name: &str, core: &Core, member: bool) -> String {
    let dependency = match core {
        Core::Path(path) => format!("kinetic-core = {{ path = \"{path}\" }}"),
        Core::Git => format!("kinetic-core = {{ git = \"{REPOSITORY}\" }}"),
    };
    let mut text = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\npublish = false\n\n\
         [lib]\n# cdylib: loaded and hot reloaded by `kinetic preview`. rlib: usable from tests.\n\
         crate-type = [\"cdylib\", \"rlib\"]\n\n[dependencies]\n{dependency}\n"
    );
    if !member {
        text.push_str(
            "\n# The scene builds on its own, outside any workspace around it.\n[workspace]\n\n\
             # Dependencies compile once with optimisations; the scene itself rebuilds fast.\n\
             [profile.dev]\nopt-level = 1\n\n[profile.dev.package.\"*\"]\nopt-level = 3\n",
        );
    }
    text
}

/// A valid package name from a folder name: lowercase, letters, digits, `-` and `_`.
fn crate_name(raw: &str) -> String {
    let mut name: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c.to_ascii_lowercase() } else { '-' })
        .collect();
    name = name.trim_matches('-').to_string();
    if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        name.insert_str(0, "scene-");
    }
    name
}

/// The nearest ancestor of `start` that is a checkout of this repository.
fn find_workspace(start: &Path) -> Option<PathBuf> {
    start.ancestors().find_map(|dir| {
        let core = dir.join("crates").join("kinetic-core").join("Cargo.toml");
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).ok()?;
        (core.is_file() && manifest.contains("[workspace]")).then(|| dir.to_path_buf())
    })
}

/// `path` made absolute against the current directory, with `.` and `..` resolved by hand,
/// since the directory may not exist yet.
fn absolute(path: &Path) -> std::io::Result<PathBuf> {
    let joined =
        if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir()?.join(path) };
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// The path from directory `from` to `to`, with forward slashes for Cargo.toml.
fn relative(from: &Path, to: &Path) -> String {
    let from: Vec<_> = from.components().collect();
    let to: Vec<_> = to.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".to_string(); from.len() - common];
    parts.extend(to[common..].iter().map(|c| c.as_os_str().to_string_lossy().into_owned()));
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("kinetic-new-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn fake_repo(root: &Path) {
        std::fs::create_dir_all(root.join("crates/kinetic-core")).expect("repo");
        std::fs::write(root.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/*\"]\n")
            .expect("ws");
        std::fs::write(root.join("crates/kinetic-core/Cargo.toml"), "[package]\n").expect("core");
    }

    #[test]
    fn names_are_made_valid() {
        assert_eq!(crate_name("My Scene!"), "my-scene");
        assert_eq!(crate_name("kite_film"), "kite_film");
        assert_eq!(crate_name("2nd take"), "scene-2nd-take");
    }

    #[test]
    fn relative_paths_climb_then_descend() {
        let from = Path::new("/ws/scenes/kite");
        assert_eq!(
            relative(from, Path::new("/ws/crates/kinetic-core")),
            "../../crates/kinetic-core"
        );
    }

    #[test]
    fn outside_the_repository_it_uses_git_and_stands_alone() {
        let dir = scratch("outside").join("kite");
        let made = create(&dir).expect("create");
        let manifest = std::fs::read_to_string(made.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains(REPOSITORY), "{manifest}");
        assert!(manifest.contains("[workspace]"));
        assert!(manifest.contains("crate-type = [\"cdylib\", \"rlib\"]"));
        let source = std::fs::read_to_string(made.join("src/lib.rs")).expect("lib.rs");
        assert!(source.contains("kinetic_core::kinetic_scene!(scene);"));
    }

    #[test]
    fn under_scenes_in_the_repository_it_joins_the_workspace() {
        let root = scratch("repo-member");
        fake_repo(&root);
        let made = create(&root.join("scenes").join("kite")).expect("create");
        let manifest = std::fs::read_to_string(made.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains("path = \"../../crates/kinetic-core\""), "{manifest}");
        assert!(!manifest.contains("[workspace]"));
    }

    #[test]
    fn elsewhere_in_the_repository_it_uses_a_path_but_stands_alone() {
        let root = scratch("repo-elsewhere");
        fake_repo(&root);
        let made = create(&root.join("experiments").join("one").join("kite")).expect("create");
        let manifest = std::fs::read_to_string(made.join("Cargo.toml")).expect("manifest");
        assert!(manifest.contains("path = \"../../../crates/kinetic-core\""), "{manifest}");
        assert!(manifest.contains("[workspace]"));
    }

    #[test]
    fn refuses_a_folder_that_is_not_empty() {
        let dir = scratch("busy");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(dir.join("notes.txt"), "mine").expect("file");
        assert_eq!(create(&dir), Err(NewError::Exists(dir.clone())));
        assert_eq!(std::fs::read_to_string(dir.join("notes.txt")).expect("kept"), "mine");
    }
}
