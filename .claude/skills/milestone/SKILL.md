---
name: milestone
description: Drive one Kinetic milestone from docs/MILESTONES.md end to end. Use when the user types /milestone N or asks to start, continue or finish a milestone.
---

# Milestone driver

Argument: the milestone number `N`. If missing, pick the first milestone with unticked boxes.

## Steps

1. Read `docs/MILESTONES.md` section `M<N>`, plus the parts of `docs/ARCHITECTURE.md` it touches. For M9 read `docs/LANDING.md` and load the `impeccable` and `frontend-design` skills.
2. Run `git status` and `git log --oneline -10`. The tree must be clean and on `main`. If earlier milestones have unticked boxes, say so and ask before skipping ahead.
3. Put the unticked boxes on the task list, in order.
4. For each box:
   - Write the smallest change that completes it, with tests in the same commit or the one right after.
   - Run the `verify` skill. Fix anything red before moving on.
   - Tick the box in `docs/MILESTONES.md` in the commit that completes it.
   - Commit with a Conventional Commit message scoped to the crate, then `git push`.
   - A box is usually one to three commits. Split by logical unit (a type, a function family, a test suite), never artificially.
5. If a design question is not answered by the docs, choose the option that keeps frames pure and scrubbable, and add one line under "Decisions" in `docs/ARCHITECTURE.md`.
6. When every box is ticked:
   - Run the milestone's "done when" check by hand (run the binary, look at the output, open the page in the browser).
   - Check CI with `gh run list --limit 3`. Hand failures to the `log-reader` agent.
   - Tag `v0.<N>.0` and push the tag.
   - Summarise what shipped, what was deferred, and anything not verified.

## Rules

- Do not start the next milestone in the same session unless asked.
- Do not tick a box whose behaviour was not actually exercised.
- New dependencies: add with `cargo add` so the latest compatible version is resolved, and note why in the commit body.
