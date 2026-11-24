//! Builds real scene crates with cargo. The first run compiles kinetic-core and its
//! dependencies into a separate target directory, so it takes a while.

mod common;

use kinetic_reload::{BuildError, SceneCrate};

#[test]
fn builds_the_hello_scene_into_a_dynamic_library() {
    let scene = SceneCrate::open(common::workspace().join("scenes/hello")).expect("open");
    let library = scene.build(&common::target_dir()).expect("build");
    assert!(library.is_file(), "{}", library.display());
    assert_eq!(library.extension().and_then(|e| e.to_str()), Some(std::env::consts::DLL_EXTENSION));
}

#[test]
fn reports_compile_errors_as_the_compiler_wrote_them() {
    let dir = common::fixture("broken", "fn scene(s: &mut Scene) { let x: u32 = \"text\"; }");
    let error =
        SceneCrate::open(&dir).expect("open").build(&common::target_dir()).expect_err("must fail");
    match error {
        BuildError::Compile(text) => {
            assert!(text.contains("mismatched types"), "{text}");
            assert!(text.contains("src"), "should point at the source: {text}");
        }
        other => panic!("expected a compile error, got {other:?}"),
    }
}
