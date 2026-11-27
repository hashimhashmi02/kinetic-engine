//! `kinetic render`: builds a scene crate and exports it to a video file.

use std::path::{Path, PathBuf};
use std::time::Instant;

use kinetic_core::Timeline;
use kinetic_export::{ffmpeg_program, AdapterChoice, Encoder, Export, Frames, Gpu, VideoSettings};
use kinetic_reload::{load, SceneCrate};

pub const USAGE: &str = "usage: kinetic render <scene-dir> -o <file.mp4> [options]

options:
  -o, --output <file>   video file to write; the extension picks the container
  --fps <rate>          frames per second (default 60)
  --size <W>x<H>        image size in pixels (default: the scene's size)
  --crf <0-51>          x264 quality, lower is better (default 16)
  --range <from>..<to>  only the frames between these times in seconds; either end
                        may be left out, as in 2.5.. or ..4

KINETIC_ADAPTER=software renders on the CPU; KINETIC_FFMPEG picks the ffmpeg program.";

const DEFAULT_FPS: f64 = 60.0;
const DEFAULT_CRF: u8 = 16;

/// What `kinetic render` was asked to do.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderArgs {
    pub scene: PathBuf,
    pub output: PathBuf,
    pub fps: f64,
    pub size: Option<(u32, u32)>,
    pub crf: u8,
    pub range: Option<(f64, f64)>,
}

/// Reads the arguments that follow `kinetic render`.
pub fn parse(args: &[&str]) -> Result<RenderArgs, String> {
    let mut scene = None;
    let mut output = None;
    let (mut fps, mut size, mut crf, mut range) = (DEFAULT_FPS, None, DEFAULT_CRF, None);
    let mut rest = args.iter();
    while let Some(&arg) = rest.next() {
        let mut value = || rest.next().copied().ok_or_else(|| format!("{arg} needs a value"));
        match arg {
            "-o" | "--output" => output = Some(PathBuf::from(value()?)),
            "--fps" => fps = parse_number(arg, value()?)?,
            "--size" => size = Some(parse_size(value()?)?),
            "--crf" => crf = parse_number(arg, value()?)?,
            "--range" => range = Some(parse_range(value()?)?),
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(format!("unknown option {flag}"));
            }
            path if scene.is_none() => scene = Some(PathBuf::from(path)),
            extra => return Err(format!("unexpected argument {extra}")),
        }
    }
    if crf > 51 {
        return Err(format!("--crf {crf} must be from 0 to 51"));
    }
    let scene = scene.ok_or(
        "render needs a scene directory, for example: kinetic render scenes/hello -o out/hello.mp4",
    )?;
    let output = output.ok_or("render needs an output file: -o out/hello.mp4")?;
    if output.extension().is_none() {
        return Err(format!(
            "{} has no extension; add one such as .mp4 so ffmpeg knows the format",
            output.display()
        ));
    }
    Ok(RenderArgs { scene, output, fps, size, crf, range })
}

fn parse_number<T: std::str::FromStr>(flag: &str, text: &str) -> Result<T, String> {
    text.parse().map_err(|_| format!("{flag} {text} is not a valid number"))
}

fn parse_size(text: &str) -> Result<(u32, u32), String> {
    let bad = || format!("size {text} should look like 1920x1080");
    let (w, h) = text.split_once(['x', 'X']).ok_or_else(bad)?;
    let (w, h): (u32, u32) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if w == 0 || h == 0 {
        return Err(format!("size {text} has no pixels"));
    }
    Ok((w, h))
}

fn parse_range(text: &str) -> Result<(f64, f64), String> {
    let bad = || format!("range {text} should look like 1.5..4, 2.. or ..3 (seconds)");
    let (from, to) = text.split_once("..").ok_or_else(bad)?;
    let end = |s: &str, default: f64| -> Result<f64, String> {
        if s.is_empty() {
            Ok(default)
        } else {
            s.parse::<f64>().ok().filter(|x| x.is_finite()).ok_or_else(bad)
        }
    };
    let (from, to) = (end(from, 0.0)?, end(to, f64::MAX)?);
    if from > to {
        return Err(format!("range {text} ends before it starts"));
    }
    Ok((from, to))
}

/// The scene's own size in pixels at one pixel per unit, rounded to even numbers as 4:2:0
/// video needs.
fn default_size(timeline: &Timeline) -> (u32, u32) {
    let even = |x: f64| ((x.round().max(2.0) as u32) + 1) & !1;
    (even(timeline.size.x), even(timeline.size.y))
}

/// Builds the scene crate in `dir` and loads its timeline.
pub fn load_scene(dir: &Path) -> Result<Timeline, String> {
    let scene = SceneCrate::open(dir).map_err(|e| e.to_string())?;
    eprintln!("building {}", dir.display());
    let library = scene
        .build(&SceneCrate::default_target_dir())
        .map_err(|e| format!("could not build {}:\n{e}", dir.display()))?;
    let scratch = std::env::temp_dir().join("kinetic").join("scratch");
    load(&library, &scratch).map_err(|e| format!("could not load {}: {e}", dir.display()))
}

/// Runs `kinetic render`.
pub fn run(args: &RenderArgs) -> Result<(), String> {
    // Check what can be checked before the scene build, which may take a while.
    Frames::new(0.0, args.fps).map_err(|e| e.to_string())?;
    if let Some((width, height)) = args.size {
        VideoSettings { width, height, fps: args.fps, crf: args.crf }
            .validate()
            .map_err(|e| e.to_string())?;
    }

    let timeline = load_scene(&args.scene)?;
    let mut frames = Frames::new(timeline.duration, args.fps).map_err(|e| e.to_string())?;
    if let Some((from, to)) = args.range {
        frames = frames.range(from, to).map_err(|e| e.to_string())?;
    }
    let (width, height) = args.size.unwrap_or_else(|| default_size(&timeline));
    let export = Export::new(&timeline, frames, width, height).map_err(|e| e.to_string())?;
    let settings = VideoSettings { width, height, fps: args.fps, crf: args.crf };
    settings.validate().map_err(|e| e.to_string())?;

    let gpu = Gpu::headless(AdapterChoice::from_env()).map_err(|e| e.to_string())?;
    if let Some(dir) = args.output.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    eprintln!(
        "rendering {} frames at {width}x{height}, {} fps, on {}",
        frames.len(),
        args.fps,
        gpu.adapter_name
    );
    let started = Instant::now();
    let mut encoder =
        Encoder::spawn(ffmpeg_program(), &args.output, &settings).map_err(|e| e.to_string())?;
    export.run(&gpu, |_, rgba| encoder.write_frame(rgba)).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    eprintln!(
        "wrote {}: {} frames ({:.2} s of video) in {:.1} s",
        args.output.display(),
        frames.len(),
        frames.len() as f64 / args.fps,
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(args: &str) -> RenderArgs {
        parse(&args.split_whitespace().collect::<Vec<_>>()).expect("parses")
    }

    fn err(args: &str) -> String {
        parse(&args.split_whitespace().collect::<Vec<_>>()).expect_err("refused")
    }

    #[test]
    fn defaults_fill_in_what_is_not_given() {
        let args = ok("scenes/hello -o out/hello.mp4");
        assert_eq!(args.scene, PathBuf::from("scenes/hello"));
        assert_eq!(args.output, PathBuf::from("out/hello.mp4"));
        assert_eq!((args.fps, args.size, args.crf, args.range), (60.0, None, 16, None));
    }

    #[test]
    fn every_option_is_read() {
        let args =
            ok("--fps 30 scenes/text --size 1920x1080 --crf 20 --output a.mov --range 1.5..4");
        assert_eq!(args.scene, PathBuf::from("scenes/text"));
        assert_eq!(args.output, PathBuf::from("a.mov"));
        assert_eq!(args.fps, 30.0);
        assert_eq!(args.size, Some((1920, 1080)));
        assert_eq!(args.crf, 20);
        assert_eq!(args.range, Some((1.5, 4.0)));
    }

    #[test]
    fn a_range_may_leave_out_either_end() {
        assert_eq!(ok("s -o a.mp4 --range 2..").range, Some((2.0, f64::MAX)));
        assert_eq!(ok("s -o a.mp4 --range ..3").range, Some((0.0, 3.0)));
    }

    #[test]
    fn mistakes_are_named() {
        assert!(err("-o a.mp4").contains("scene directory"));
        assert!(err("scenes/hello").contains("-o"));
        assert!(err("s -o out").contains("extension"));
        assert!(err("s -o a.mp4 --fps").contains("--fps needs a value"));
        assert!(err("s -o a.mp4 --fps fast").contains("not a valid number"));
        assert!(err("s -o a.mp4 --size 1920").contains("1920x1080"));
        assert!(err("s -o a.mp4 --size 0x10").contains("no pixels"));
        assert!(err("s -o a.mp4 --crf 300").contains("--crf"));
        assert!(err("s -o a.mp4 --crf 52").contains("0 to 51"));
        assert!(err("s -o a.mp4 --range 4..2").contains("ends before"));
        assert!(err("s -o a.mp4 --range 2").contains("1.5..4"));
        assert!(err("s -o a.mp4 --speed 2").contains("unknown option --speed"));
        assert!(err("s t -o a.mp4").contains("unexpected argument t"));
    }

    #[test]
    fn the_default_size_is_the_scene_rounded_to_even_pixels() {
        let mut timeline = kinetic_core::Scene::new().build();
        assert_eq!(default_size(&timeline), (1280, 720));
        timeline.size = kinetic_core::DVec2::new(333.4, 199.0);
        assert_eq!(default_size(&timeline), (334, 200));
    }
}
