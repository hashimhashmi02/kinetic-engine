//! The `text` example (shaped text, glyph by glyph) renders the same bytes twice.

mod common;

#[test]
fn text_renders_the_same_twice() {
    common::determinism::renders_the_same_twice("text", text::build);
}
