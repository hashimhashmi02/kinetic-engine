//! Loads real scene libraries built by cargo.

mod common;

use kinetic_reload::{load, LoadError, SceneCrate};

fn scratch(name: &str) -> std::path::PathBuf {
    common::target_dir().join("scratch").join(name)
}

#[test]
fn loaded_hello_matches_the_in_process_build() {
    let library = SceneCrate::open(common::workspace().join("scenes/hello"))
        .expect("open")
        .build(&common::target_dir())
        .expect("build");
    let timeline = load(&library, &scratch("hello")).expect("load");
    assert_eq!(timeline, hello::build());
}

#[test]
fn loading_leaves_the_built_library_free_to_overwrite() {
    let library = SceneCrate::open(common::workspace().join("scenes/hello"))
        .expect("open")
        .build(&common::target_dir())
        .expect("build");
    load(&library, &scratch("overwrite")).expect("load");
    // Cargo replaces the library on every build; that must not be blocked by a lock.
    let bytes = std::fs::read(&library).expect("read");
    std::fs::write(&library, &bytes).expect("library is not locked");
    // And the scratch copy is gone.
    let leftovers = std::fs::read_dir(scratch("overwrite")).map(|d| d.count()).unwrap_or(0);
    assert_eq!(leftovers, 0);
}

#[test]
fn a_panicking_scene_reports_its_message() {
    let dir =
        common::fixture("panics", "fn scene(_: &mut Scene) { panic!(\"no circles today\"); }");
    let library =
        SceneCrate::open(&dir).expect("open").build(&common::target_dir()).expect("build");
    match load(&library, &scratch("panics")) {
        Err(LoadError::Panicked(message)) => assert_eq!(message, "no circles today"),
        other => panic!("expected a panic report, got {other:?}"),
    }
}

#[test]
fn reloading_a_changed_scene_gives_the_new_timeline() {
    let one = "fn scene(s: &mut Scene) { let c = s.add(Circle::new(1.0)); s.play(c.move_to((1.0, 0.0)), 1.0); }";
    let two = "fn scene(s: &mut Scene) { let c = s.add(Circle::new(1.0)); s.play(c.move_to((1.0, 0.0)), 2.5); }";
    let target = common::target_dir();
    let dir = common::fixture("changes", one);
    let first = load(
        &SceneCrate::open(&dir).expect("open").build(&target).expect("build"),
        &scratch("changes"),
    )
    .expect("first load");
    common::fixture("changes", two);
    let second = load(
        &SceneCrate::open(&dir).expect("open").build(&target).expect("build"),
        &scratch("changes"),
    )
    .expect("second load");
    assert_eq!((first.duration, second.duration), (1.0, 2.5));
}
