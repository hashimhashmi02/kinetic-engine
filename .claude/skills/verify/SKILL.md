---
name: verify
description: Run Kinetic's pre-commit checks (format, clippy, tests, determinism, site build). Use before every commit and whenever asked whether the tree is green.
---

# Verify

Run in this order and stop at the first failure.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Then, only if the change touches these areas:

| Changed | Extra check |
|---|---|
| `kinetic-render`, shaders, tessellation | Golden tests ran (not skipped for lack of an adapter). If pixels changed on purpose: `cargo xtask bless`, look at the new images, say why in the commit |
| `kinetic-export` (from M6) | `cargo test -p kinetic-export determinism` |
| `site/` (from M9) | `pnpm --dir site build` |
| Public API | `cargo doc --workspace --no-deps` has no warnings |

## Reporting

- Green: one line, for example "verify: fmt, clippy, 142 tests ok".
- Red: the failing command and the first error only. If output is long, give it to the `log-reader` agent rather than pasting it.
- Never commit on red, and never weaken a check (`#[allow]`, `#[ignore]`, raising a tolerance) to get to green without saying so and why.
