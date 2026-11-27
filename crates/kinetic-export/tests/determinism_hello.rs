//! The `hello` example (shapes moving, scaling and fading) renders the same bytes twice.

mod common;

#[test]
fn hello_renders_the_same_twice() {
    common::determinism::renders_the_same_twice("hello", hello::build);
}
