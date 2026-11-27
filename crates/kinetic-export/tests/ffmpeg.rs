//! The ffmpeg encoder against a real ffmpeg: files that play, colours that survive, and
//! failures that say why.
//!
//! These tests need ffmpeg and ffprobe on `PATH`. Without them they are skipped with a
//! notice, unless `KINETIC_REQUIRE_FFMPEG=1` (set in CI), which makes them fail instead.

use std::path::{Path, PathBuf};
use std::process::Command;

use kinetic_export::{ffmpeg_program, Encoder, ExportError, VideoSettings};

fn have_ffmpeg() -> bool {
    let works = |program: &std::ffi::OsStr| {
        Command::new(program).arg("-version").output().is_ok_and(|o| o.status.success())
    };
    let found = works(&ffmpeg_program()) && works("ffprobe".as_ref());
    if !found {
        assert!(
            std::env::var("KINETIC_REQUIRE_FFMPEG").as_deref() != Ok("1"),
            "ffmpeg or ffprobe is missing and KINETIC_REQUIRE_FFMPEG=1"
        );
        eprintln!("SKIPPED: ffmpeg or ffprobe not found");
    }
    found
}

fn out_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ffmpeg");
    std::fs::create_dir_all(&dir).expect("create output dir");
    dir
}

fn solid(settings: &VideoSettings, rgb: [u8; 3]) -> Vec<u8> {
    let pixels = (settings.width * settings.height) as usize;
    std::iter::repeat_n([rgb[0], rgb[1], rgb[2], 255], pixels).flatten().collect()
}

fn probe(path: &Path) -> String {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-count_frames", "-show_entries"])
        .arg("stream=codec_name,width,height,pix_fmt,nb_read_frames,color_space,r_frame_rate")
        .args(["-of", "default=noprint_wrappers=1"])
        .arg(path)
        .output()
        .expect("run ffprobe");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).replace('\r', "")
}

#[test]
fn encodes_a_clip_that_ffprobe_reads_back() {
    if !have_ffmpeg() {
        return;
    }
    let path = out_dir().join("clip.mp4");
    let settings = VideoSettings { width: 96, height: 54, fps: 30.0, crf: 18 };
    let mut encoder = Encoder::spawn(ffmpeg_program(), &path, &settings).expect("spawn");
    for i in 0..45u8 {
        encoder.write_frame(&solid(&settings, [i * 5, 100, 255 - i * 5])).expect("frame");
    }
    encoder.finish().expect("finish");

    let info = probe(&path);
    for line in [
        "codec_name=h264",
        "width=96",
        "height=54",
        "pix_fmt=yuv420p",
        "color_space=bt709",
        "r_frame_rate=30/1",
        "nb_read_frames=45",
    ] {
        assert!(info.lines().any(|l| l == line), "missing {line} in\n{info}");
    }
}

#[test]
fn colours_survive_the_round_trip() {
    if !have_ffmpeg() {
        return;
    }
    let path = out_dir().join("colour.mp4");
    let settings = VideoSettings { width: 64, height: 64, fps: 24.0, crf: 0 };
    let colours = [[200, 80, 40], [30, 160, 220], [242, 160, 61]];
    let mut encoder = Encoder::spawn(ffmpeg_program(), &path, &settings).expect("spawn");
    for rgb in colours {
        encoder.write_frame(&solid(&settings, rgb)).expect("frame");
    }
    encoder.finish().expect("finish");

    // Decode back to sRGB; the BT.709 tags tell ffmpeg which matrix to undo.
    let output = Command::new(ffmpeg_program())
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "pipe:1"])
        .output()
        .expect("decode");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let frame = 64 * 64 * 3;
    assert_eq!(output.stdout.len(), frame * colours.len());
    for (k, want) in colours.iter().enumerate() {
        let centre = k * frame + (32 * 64 + 32) * 3;
        let got = &output.stdout[centre..centre + 3];
        for (g, w) in got.iter().zip(want) {
            assert!(g.abs_diff(*w) <= 3, "frame {k}: got {got:?}, rendered {want:?}");
        }
    }
}

#[test]
fn a_failing_ffmpeg_reports_what_it_said() {
    if !have_ffmpeg() {
        return;
    }
    let path = out_dir().join("no-such-folder").join("lost.mp4");
    let settings = VideoSettings { width: 32, height: 32, fps: 30.0, crf: 20 };
    let mut encoder = Encoder::spawn(ffmpeg_program(), &path, &settings).expect("spawn");
    let frame = solid(&settings, [0, 0, 0]);
    // ffmpeg gives up on the output and stops reading, so either a write or finish fails.
    let error = (0..200)
        .find_map(|_| encoder.write_frame(&frame).err())
        .unwrap_or_else(|| encoder.finish().expect_err("cannot write into a missing folder"));
    let message = error.to_string();
    assert!(matches!(error, ExportError::Ffmpeg(_)), "{message}");
    assert!(message.contains("lost.mp4"), "the log names the file it could not open: {message}");
}

#[test]
fn a_frame_of_the_wrong_size_is_refused() {
    if !have_ffmpeg() {
        return;
    }
    let path = out_dir().join("wrong-size.mp4");
    let settings = VideoSettings { width: 32, height: 32, fps: 30.0, crf: 20 };
    let mut encoder = Encoder::spawn(ffmpeg_program(), &path, &settings).expect("spawn");
    let error = encoder.write_frame(&[0; 16]).expect_err("too small");
    assert!(error.to_string().contains("4096"), "{error}");
    // Dropping the unfinished encoder stops ffmpeg instead of leaving it waiting.
    drop(encoder);
}
