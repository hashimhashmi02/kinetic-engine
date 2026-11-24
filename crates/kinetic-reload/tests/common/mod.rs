//! Helpers shared by the reload integration tests.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The workspace root, without the `\\?\` prefix Windows adds to canonical paths,
/// which cargo cannot read in a path dependency.
pub fn workspace() -> PathBuf {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root");
    let text = root.to_string_lossy();
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

/// Where test scene builds go: inside the workspace target directory, but separate from
/// the build running these tests, so the nested cargo never waits on the outer one.
pub fn target_dir() -> PathBuf {
    workspace().join("target").join("reload-tests")
}

/// A standalone scene crate under the test target directory, with `body` as its scene
/// function and the `kinetic_scene!` export. Rewritten on every call.
pub fn fixture(name: &str, body: &str) -> PathBuf {
    let dir = target_dir().join("fixtures").join(name);
    std::fs::create_dir_all(dir.join("src")).expect("fixture dir");
    let core = workspace().join("crates/kinetic-core");
    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n\
         [workspace]\n\n[lib]\ncrate-type = [\"cdylib\", \"rlib\"]\n\n\
         [dependencies]\nkinetic-core = {{ path = {:?} }}\n\n\
         [profile.dev.package.\"*\"]\nopt-level = 3\n",
        core.to_string_lossy().replace('\\', "/"),
    );
    write_if_changed(&dir.join("Cargo.toml"), &manifest);
    let source =
        format!("use kinetic_core::*;\n\n{body}\n\nkinetic_core::kinetic_scene!(scene);\n");
    write_if_changed(&dir.join("src/lib.rs"), &source);
    dir
}

/// Writes only when the content differs, so cargo's freshness checks stay useful.
pub fn write_if_changed(path: &Path, content: &str) {
    if std::fs::read_to_string(path).ok().as_deref() != Some(content) {
        std::fs::write(path, content).expect("write fixture");
    }
}
