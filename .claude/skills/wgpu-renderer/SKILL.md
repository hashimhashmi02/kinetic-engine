---
name: wgpu-renderer
description: Reference for Kinetic's wgpu renderer - device setup, MSAA, colour spaces, premultiplied blending, texture readback, post-processing passes, golden image tests on a software adapter. Use when writing or debugging code in kinetic-render or kinetic-export, or shaders.
---

# wgpu renderer notes

wgpu's API changes between releases. Before writing against it, check the version resolved in `Cargo.lock` and read that version's docs or examples (`cargo doc -p wgpu --open`, or the source under `~/.cargo/registry`). Do not write wgpu calls from memory.

## Setup

- One `Renderer` owns device, queue, pipelines and caches. It renders a `FrameState` into a caller-supplied `TextureView`; it does not own a surface. The editor passes a surface view, export and tests pass an offscreen texture.
- Adapter selection: `HighPerformance` by default; `KINETIC_ADAPTER=software` forces the fallback adapter (WARP on Windows, lavapipe on Linux). Tests always force software.
- If no adapter exists (bare CI runner), golden tests must fail loudly unless `KINETIC_ALLOW_NO_GPU=1` is set, in which case they print a skip notice. CI installs lavapipe so they really run.

## Colour

- Engine colours are linear premultiplied RGBA `f32`.
- Render target format is `Rgba8UnormSrgb`, so the GPU encodes to sRGB on write and blending happens in linear space.
- Surface formats vary; pick an sRGB surface format when available, otherwise render to an intermediate sRGB texture and blit.
- egui expects to draw into a specific kind of target; follow the `egui-wgpu` docs for the resolved version instead of assuming.

## Blending

Premultiplied alpha: `src = One`, `dst = OneMinusSrcAlpha` for both colour and alpha. Clear to the scene background colour, premultiplied.

## MSAA

- Sample count 4. Draw into a multisampled colour texture with `resolve_target` set to the single-sample target.
- The multisampled texture is recreated on resize and never read back.
- Tessellation tolerance is in device pixels: scale lyon's tolerance by the current pixel scale so curves stay smooth when zoomed.

## Geometry

- Vertex: `position: [f32; 2]` in scene units. Per-draw uniforms: 3x2 affine transform (as a mat3 or two vec3 rows), colour, opacity.
- Scene coordinates: origin at the centre, +x right, +y up, units are pixels at the scene's nominal resolution. A single view-projection maps the scene rect to clip space; the camera modifies that matrix.
- Batch by pipeline. Start simple (one draw per node with a dynamic uniform offset), measure, then merge meshes only if a real scene needs it.
- Mesh cache key: hash of path data and stroke/fill style, not transform or colour.

## Readback (export and tests)

- Copy texture to buffer. `bytes_per_row` must be a multiple of `COPY_BYTES_PER_ROW_ALIGNMENT` (256): pad it, then strip the padding per row when copying out.
- Map asynchronously, poll the device until the callback fires, copy, unmap. Reuse the buffer across frames.
- Output rows are top to bottom, RGBA8, sRGB-encoded: exactly what ffmpeg's `rgba` rawvideo input and PNG encoders want.

## Post stack

- Two full-size textures, ping-pong. Each effect is a fullscreen-triangle pass sampling the previous one.
- Gaussian blur: separable, two passes, optionally at half resolution for large radii. Weights computed on the CPU and passed as uniforms.
- Glow: bright-pass threshold, blur, additive composite over the original.
- Motion blur: render `n` sub-frames at `t + shutter * (i + 0.5) / n - shutter / 2` and average in a float accumulation target. Deterministic because the sample times are fixed.

## Golden tests

- Small targets (for example 256x144) keep PNGs tiny and tests fast.
- Compare with per-channel tolerance (start at 2/255) and a cap on the number of differing pixels. On failure write `<name>.actual.png` and `<name>.diff.png` beside the golden (both git-ignored) and print their paths.
- `cargo xtask bless` regenerates goldens. Always open the new images before committing them.
- A golden that only passes on one machine is a renderer bug or a tolerance problem; investigate before loosening anything.

## Debugging checklist

- Black frame: check the clear colour, the view-projection (y flip), winding and cull mode (use `cull_mode: None` for 2D), and that the index buffer format matches (`u32`).
- Washed out or too dark: a double or missing sRGB conversion.
- Dark fringes on edges: straight alpha blended as premultiplied, or the reverse.
- Validation errors: read the first one only; later ones are usually consequences.
