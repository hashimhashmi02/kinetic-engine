//! The `morph` example (shapes morphing, drawing on and trimming) renders the same bytes twice.

mod common;

#[test]
fn morph_renders_the_same_twice() {
    common::determinism::renders_the_same_twice("morph", morph::build);
}
