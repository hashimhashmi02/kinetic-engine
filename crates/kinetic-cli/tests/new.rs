//! `kinetic new` produces a scene crate that builds and loads as it stands.

use std::path::{Path, PathBuf};
use std::process::Command;

use kinetic_reload::{load, SceneCrate};

fn workspace() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("root");
    let text = root.to_string_lossy();
    PathBuf::from(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

#[test]
fn new_scene_builds_and_loads() {
    let target = workspace().join("target").join("reload-tests");
    let dir = target.join("new").join("kite-test");
    let _ = std::fs::remove_dir_all(&dir);

    let output = Command::new(env!("CARGO_BIN_EXE_kinetic"))
        .args(["new", &dir.to_string_lossy()])
        .output()
        .expect("run kinetic new");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("next: kinetic preview"), "{stdout}");

    let library = SceneCrate::open(&dir).expect("open").build(&target).expect("build");
    let timeline = load(&library, &target.join("scratch").join("new")).expect("load");
    assert_eq!(timeline.nodes.len(), 1);
    assert!(timeline.duration > 1.7, "{}", timeline.duration);

    // A second run must not overwrite the crate it just made.
    let again = Command::new(env!("CARGO_BIN_EXE_kinetic"))
        .args(["new", &dir.to_string_lossy()])
        .output()
        .expect("run kinetic new again");
    assert!(!again.status.success());
    assert!(String::from_utf8_lossy(&again.stderr).contains("not empty"));
}
