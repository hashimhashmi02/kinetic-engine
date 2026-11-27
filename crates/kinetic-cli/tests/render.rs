//! `kinetic render` end to end: a scene crate in, a video ffprobe can read out.
//!
//! Needs ffmpeg and ffprobe; without them the video test is skipped with a notice, unless
//! `KINETIC_REQUIRE_FFMPEG=1` (set in CI).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn hello() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenes/hello")
}

fn kinetic(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kinetic"))
        .args(args)
        .env("KINETIC_ADAPTER", "software")
        .output()
        .expect("run kinetic")
}

fn have_ffmpeg() -> bool {
    let works = |program: &str| {
        Command::new(program).arg("-version").output().is_ok_and(|o| o.status.success())
    };
    let found = works("ffmpeg") && works("ffprobe");
    if !found {
        assert!(
            std::env::var("KINETIC_REQUIRE_FFMPEG").as_deref() != Ok("1"),
            "ffmpeg or ffprobe is missing and KINETIC_REQUIRE_FFMPEG=1"
        );
        eprintln!("SKIPPED: ffmpeg or ffprobe not found");
    }
    found
}

#[test]
fn renders_a_scene_crate_to_a_video() {
    if !have_ffmpeg() {
        return;
    }
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("render").join("hello.mp4");
    let _ = std::fs::remove_file(&out);
    let out_text = out.to_string_lossy();
    let scene = hello();
    let output = kinetic(&[
        "render",
        &scene.to_string_lossy(),
        "-o",
        &out_text,
        "--size",
        "320x180",
        "--fps",
        "12",
        "--range",
        "1..2",
        "--crf",
        "23",
    ]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("rendering 13 frames at 320x180"), "{stderr}");

    let probe = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-count_frames", "-show_entries"])
        .args(["stream=width,height,nb_read_frames,r_frame_rate", "-of", "default=nw=1"])
        .arg(&out)
        .output()
        .expect("run ffprobe");
    let info = String::from_utf8_lossy(&probe.stdout).replace('\r', "");
    for line in ["width=320", "height=180", "nb_read_frames=13", "r_frame_rate=12/1"] {
        assert!(info.lines().any(|l| l == line), "missing {line} in\n{info}");
    }
}

#[test]
fn renders_a_scene_crate_to_png_frames_without_ffmpeg() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("render").join("hello-frames");
    let _ = std::fs::remove_dir_all(&dir);
    let scene = hello();
    let output = Command::new(env!("CARGO_BIN_EXE_kinetic"))
        .args(["render", &scene.to_string_lossy(), "--frames", &dir.to_string_lossy()])
        .args(["--size", "161x91", "--fps", "10", "--range", "0.95..1.3"])
        .env("KINETIC_ADAPTER", "software")
        // PNG output must not need ffmpeg at all.
        .env("KINETIC_FFMPEG", "kinetic-no-such-ffmpeg")
        .output()
        .expect("run kinetic");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");

    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .expect("list frames")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    // Frames keep their index in the full timeline: 1.0 s at 10 fps is frame 10.
    assert_eq!(names, ["frame_00010.png", "frame_00011.png", "frame_00012.png", "frame_00013.png"]);
    let file = std::fs::File::open(dir.join("frame_00010.png")).expect("open frame");
    let info = png::Decoder::new(std::io::BufReader::new(file)).read_info().expect("png info");
    assert_eq!((info.info().width, info.info().height), (161, 91), "odd sizes are fine for PNG");
}

#[test]
fn bad_settings_fail_before_the_scene_is_built() {
    let scene = hello();
    let output = kinetic(&["render", &scene.to_string_lossy(), "-o", "x.mp4", "--size", "333x200"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("334x200"), "{stderr}");
    assert!(!stderr.contains("building"), "{stderr}");
}

#[test]
fn a_folder_without_a_scene_is_named() {
    let output = kinetic(&["render", "no/such/scene", "-o", "x.mp4"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("has no Cargo.toml"), "{stderr}");
}
