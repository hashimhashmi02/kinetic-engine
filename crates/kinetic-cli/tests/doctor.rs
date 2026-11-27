//! `kinetic doctor` on this machine, which must be able to build and render (as CI is).

use std::process::Command;

fn doctor(ffmpeg: Option<&str>) -> (bool, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kinetic"));
    command.arg("doctor");
    if let Some(program) = ffmpeg {
        command.env("KINETIC_FFMPEG", program);
    }
    let output = command.output().expect("run kinetic doctor");
    (output.status.success(), String::from_utf8_lossy(&output.stdout).replace('\r', ""))
}

#[test]
fn reports_every_tool_and_is_ready() {
    let (ok, text) = doctor(None);
    assert!(ok, "{text}");
    for name in ["cargo", "rustc", "gpu", "software", "ffmpeg"] {
        assert!(
            text.lines().any(|l| l.split_whitespace().nth(1) == Some(name)),
            "no {name} line in\n{text}"
        );
    }
    assert!(text.lines().any(|l| l.starts_with("  ok    software")), "{text}");
    assert!(text.ends_with("ready\n"), "{text}");
}

#[test]
fn a_missing_ffmpeg_warns_but_does_not_fail() {
    let (ok, text) = doctor(Some("kinetic-no-such-ffmpeg"));
    assert!(ok, "PNG export still works: {text}");
    assert!(text.contains("  warn  ffmpeg    kinetic-no-such-ffmpeg not found"), "{text}");
}
