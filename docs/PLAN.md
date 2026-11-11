# Plan

## What Kinetic is

A motion engine for people who would rather write animations than keyframe them by hand. You describe a scene in Rust, see it in a live preview, scrub a timeline, and export an MP4. It sits in the same space as Manim and Motion Canvas, but it is native, GPU-rendered and hot-reloading.

## Goals

- A scene API that reads like a storyboard: add things, then play, parallel, stagger.
- Instant feedback: save a file, see the change, keep the playhead where it was.
- Exact scrubbing: any frame can be evaluated directly, in any order.
- Export that matches the preview pixel for pixel.
- Morphing between arbitrary shapes and between pieces of text.

## Non-goals (for 1.0)

- A general-purpose 2D game engine or UI toolkit.
- 3D scenes.
- A node-based or drag-and-drop authoring tool. The editor previews and inspects; scenes are code.
- Audio synthesis. An audio track for export is a post-1.0 feature.

## Stack

| Concern | Choice | Reason |
|---|---|---|
| GPU | wgpu | Portable (Vulkan, Metal, DX12), offscreen rendering, safe API |
| Windowing | winit | Standard pairing with wgpu |
| Editor UI | egui (egui-wgpu, egui-winit) | Immediate mode fits a tool that redraws every frame |
| Paths | kurbo | Robust bezier maths, arc length, flattening |
| Tessellation | lyon | Fill and stroke tessellation to triangle meshes |
| Text | cosmic-text | Shaping, fallback and layout in pure Rust |
| Hot reload | libloading + notify | Scenes as `cdylib`, reload on save |
| Export | ffmpeg (external process) | H.264 without linking codec libraries |
| Site | Astro, hand-written SVG and scroll timeline | Static output, paper cut-out scroll journey |

Exact crate versions are pinned in the root `Cargo.toml` as each one is first used.

## Risks

| Risk | Mitigation |
|---|---|
| Morphing shapes with different topology looks wrong | Start point and winding alignment; subpath pairing by area and centroid; goldens for hard pairs (ring to disc, "o" to "l") |
| Text antialiasing looks soft or uneven | Tessellate at device resolution with 4x MSAA first; measure against a reference rasteriser; move to analytic coverage only if needed |
| GPU output differs between machines, breaking goldens | Render tests run on a software adapter with a per-pixel tolerance; core tests compare numeric state exactly |
| Hot reload crashes or leaks on Windows | Only serialised bytes cross the dylib boundary; DLL is copied before loading; old libraries are unloaded only after their data is dropped |
| Scene compile time makes reload slow | Scenes depend only on `kinetic-core`; dependencies are optimised in dev profile; incremental builds |
| ffmpeg missing or mismatched | `kinetic doctor` checks for it; clear error with install hint; PNG sequence fallback |

## Order of work

Milestones are in `MILESTONES.md`. The engine is built bottom-up (core maths, then vectors, text, export, editor, effects). The landing page depends on nothing but its own spec and can be built in parallel once the API shape is stable (after M2).
