//! Loading a built scene library and reading its timeline.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use kinetic_core::abi::{SceneBytes, ABI_VERSION, STATUS_ENCODE, STATUS_OK, STATUS_PANIC};
use kinetic_core::wire::{self, WireError};
use kinetic_core::Timeline;
use libloading::Library;

/// The library could not be loaded or did not produce a timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// Copying or opening the library failed.
    Io(String),
    /// The library lacks the scene entry points; it was not built with `kinetic_scene!`.
    NotAScene(String),
    /// The library was built against a different scene ABI.
    Abi { found: u32, expected: u32 },
    /// The scene function panicked. Holds the panic message.
    Panicked(String),
    /// The scene's timeline could not be encoded or decoded.
    Wire(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(why) => write!(f, "could not load the scene library: {why}"),
            Self::NotAScene(why) => {
                write!(f, "the library is not a Kinetic scene (add kinetic_scene!): {why}")
            }
            Self::Abi { found, expected } => write!(
                f,
                "the scene was built for scene ABI {found}, but this editor speaks {expected}; \
                 rebuild it against the same kinetic-core"
            ),
            Self::Panicked(message) => write!(f, "the scene panicked: {message}"),
            Self::Wire(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<WireError> for LoadError {
    fn from(error: WireError) -> Self {
        Self::Wire(error.to_string())
    }
}

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type BuildFn = unsafe extern "C" fn(*mut SceneBytes) -> u32;
type FreeFn = unsafe extern "C" fn(SceneBytes);

/// Loads the scene library at `library` and returns its timeline.
///
/// The library is copied into `scratch` first and the copy is loaded: Windows keeps a
/// loaded library locked, and cargo must be free to overwrite the original on the next
/// build. Only bytes cross the boundary and they are copied out before the library is
/// unloaded, so nothing returned here points into the library.
pub fn load(library: &Path, scratch: &Path) -> Result<Timeline, LoadError> {
    let copy = scratch_copy(library, scratch)?;
    let result = call_scene(&copy);
    // The copy is only removable once the library is unloaded, which call_scene has done.
    let _ = std::fs::remove_file(&copy);
    let (status, bytes) = result?;
    match status {
        STATUS_OK => Ok(wire::decode(&bytes)?),
        STATUS_PANIC => Err(LoadError::Panicked(String::from_utf8_lossy(&bytes).into_owned())),
        STATUS_ENCODE => Err(LoadError::Wire(String::from_utf8_lossy(&bytes).into_owned())),
        other => Err(LoadError::Wire(format!("the scene returned unknown status {other}"))),
    }
}

/// Removes library copies left behind by earlier runs, for example after a crash.
pub fn clean_scratch(scratch: &Path) {
    let Ok(entries) = std::fs::read_dir(scratch) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("scene-") {
            // Copies still loaded by another running editor stay locked and are skipped.
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn scratch_copy(library: &Path, scratch: &Path) -> Result<PathBuf, LoadError> {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    std::fs::create_dir_all(scratch).map_err(|e| LoadError::Io(e.to_string()))?;
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let name = format!("scene-{}-{n}.{}", std::process::id(), std::env::consts::DLL_EXTENSION);
    let copy = scratch.join(name);
    std::fs::copy(library, &copy)
        .map_err(|e| LoadError::Io(format!("{}: {e}", library.display())))?;
    Ok(copy)
}

/// Loads `path`, checks the ABI, runs the scene and copies its bytes out. The library is
/// unloaded when this returns.
fn call_scene(path: &Path) -> Result<(u32, Vec<u8>), LoadError> {
    // SAFETY: loading runs the library's initialisers. Scene libraries are built from
    // the user's own scene crates by this tool, and are trusted as much as that code is.
    let library = unsafe { Library::new(path) }.map_err(|e| LoadError::Io(e.to_string()))?;
    let not_a_scene = |e: libloading::Error| LoadError::NotAScene(e.to_string());
    // SAFETY: the symbol types match the signatures kinetic_scene! exports.
    let (abi, build, free) = unsafe {
        (
            library.get::<AbiVersionFn>(b"kinetic_scene_abi_version").map_err(not_a_scene)?,
            library.get::<BuildFn>(b"kinetic_scene_build").map_err(not_a_scene)?,
            library.get::<FreeFn>(b"kinetic_scene_free").map_err(not_a_scene)?,
        )
    };
    // SAFETY: the version function takes nothing and only returns a number.
    let found = unsafe { abi() };
    if found != ABI_VERSION {
        return Err(LoadError::Abi { found, expected: ABI_VERSION });
    }
    let mut out = SceneBytes::EMPTY;
    // SAFETY: out is a valid SceneBytes the library may write to. The bytes are copied
    // before being handed back, once, to the library that allocated them.
    let (status, bytes) = unsafe {
        let status = build(&mut out);
        let bytes = out.as_slice().to_vec();
        free(out);
        (status, bytes)
    };
    // The symbols borrow the library; their last use above ends that, so it can close.
    library.close().map_err(|e| LoadError::Io(e.to_string()))?;
    Ok((status, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_is_an_io_error() {
        let scratch = std::env::temp_dir().join("kinetic-load-test");
        let error = load(Path::new("definitely/not/here.dll"), &scratch).expect_err("must fail");
        assert!(matches!(error, LoadError::Io(_)), "{error:?}");
    }

    #[test]
    fn abi_message_names_both_versions() {
        let text = LoadError::Abi { found: 7, expected: ABI_VERSION }.to_string();
        assert!(text.contains('7') && text.contains(&ABI_VERSION.to_string()));
    }
}
