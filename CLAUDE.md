# Kinetic

Native GPU motion engine in Rust. Scene code compiles to a timeline; every frame is a pure function of time. Read `docs/ARCHITECTURE.md` before changing crate boundaries and `docs/MILESTONES.md` to see what is done and what is next.

## Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p kinetic-cli -- preview scenes/hello      # from M3
cargo run -p kinetic-cli -- render scenes/hello -o out/hello.mp4   # from M6
pnpm --dir site dev                                    # from M9
```

## Rules

1. **Determinism is the product.** Evaluation code never reads the wall clock, uses unseeded randomness, iterates a `HashMap`, or depends on thread timing. `evaluate(t)` twice gives identical output.
2. **`kinetic-core` stays pure.** No GPU, window, file or process dependencies. Anything that needs IO lives in a higher crate.
3. **Dependencies point one way:** core <- vector <- text; core, vector <- render <- export, editor <- cli. Never add a reverse edge.
4. **Golden images are never edited by hand.** They change only through `cargo xtask bless`, and the commit message says why the pixels changed.
5. **Verify before every commit:** fmt, clippy with `-D warnings`, tests. Use the `verify` skill. A red check is fixed before anything else.
6. **No `unwrap`/`expect` on runtime input** (files, user scenes, GPU, ffmpeg). Return errors with context. They are fine in tests and for proven invariants with a comment.
7. **Public API gets a doc comment and a test.** Animation math gets property tests (continuity, endpoints, monotonicity where it applies).
8. **Time is `f64` seconds** everywhere in core. Frame index to time is `frame as f64 / fps`, never accumulated.
9. **Colours are linear premultiplied RGBA** inside the engine; sRGB only at the API edge and at output.

## Git workflow

- One logical change per commit, pushed to `main` immediately after it passes verify.
- Conventional Commits with a crate scope: `feat(core): add cubic bezier easing`, `test(vector): cover morph winding`, `docs: ...`, `ci: ...`.
- Commits are real units of work. Never split a change artificially or commit noise.
- Tag `v0.N.0` when milestone N's checklist in `docs/MILESTONES.md` is fully ticked, and tick the boxes in the same commit as the last piece of work.

## Working style

- Start a milestone with `/milestone N`; it reads the checklist and works through it in order.
- Keep the main context for decisions. Hand long test or CI output to the `log-reader` agent.
- When a design decision is not covered by the docs, pick the option that keeps frames pure and scrubbable, then record it in `docs/ARCHITECTURE.md` under "Decisions".
- The landing page (`site/`) follows `docs/LANDING.md`. Load the `impeccable` and `frontend-design` skills before touching it.
