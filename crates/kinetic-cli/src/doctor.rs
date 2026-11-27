//! `kinetic doctor`: checks that this machine can build, preview and export scenes.

use std::ffi::OsStr;
use std::process::Command;

use kinetic_export::{ffmpeg_program, AdapterChoice, Gpu};

/// How one check came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Present and working.
    Ok,
    /// Missing, but something else still works without it.
    Warn,
    /// Missing, and Kinetic cannot work without it.
    Fail,
}

/// One line of the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
}

impl Check {
    fn new(name: &'static str, status: Status, detail: impl Into<String>) -> Self {
        Self { name, status, detail: detail.into() }
    }
}

/// Runs every check.
pub fn checks() -> Vec<Check> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut checks = vec![
        tool("cargo", &cargo, "builds scene crates; install Rust from https://rustup.rs"),
        tool(
            "rustc",
            OsStr::new("rustc"),
            "compiles scene crates; install Rust from https://rustup.rs",
        ),
    ];
    checks.extend(adapters());
    checks.push(ffmpeg(&ffmpeg_program()));
    checks
}

/// Formats the report, and says whether Kinetic can work on this machine.
pub fn report(checks: &[Check]) -> (String, bool) {
    let mut out = format!("kinetic {}\n", env!("CARGO_PKG_VERSION"));
    for check in checks {
        let mark = match check.status {
            Status::Ok => "ok  ",
            Status::Warn => "warn",
            Status::Fail => "FAIL",
        };
        out += &format!("  {mark}  {:<9} {}\n", check.name, check.detail);
    }
    let healthy = checks.iter().all(|c| c.status != Status::Fail);
    out += if healthy { "ready" } else { "not ready: fix the FAIL lines above" };
    (out, healthy)
}

/// First line of `program --version`, or why it could not run.
fn tool(name: &'static str, program: &OsStr, purpose: &str) -> Check {
    match first_line(program, &["--version"]) {
        Ok(version) => Check::new(name, Status::Ok, version),
        Err(why) => Check::new(name, Status::Fail, format!("{why}; it {purpose}")),
    }
}

/// The GPU an export or preview gets, and the software adapter golden tests use.
fn adapters() -> Vec<Check> {
    let describe = |gpu: &Gpu| {
        let kind = match gpu.adapter.get_info().device_type {
            kinetic_export::wgpu::DeviceType::DiscreteGpu => "discrete GPU",
            kinetic_export::wgpu::DeviceType::IntegratedGpu => "integrated GPU",
            kinetic_export::wgpu::DeviceType::VirtualGpu => "virtual GPU",
            kinetic_export::wgpu::DeviceType::Cpu => "runs on the CPU",
            kinetic_export::wgpu::DeviceType::Other => "unknown kind",
        };
        format!("{}, {kind}", gpu.adapter_name)
    };
    let hardware = Gpu::headless(AdapterChoice::HighPerformance);
    let software = Gpu::headless(AdapterChoice::Software);
    let usable = hardware.is_ok() || software.is_ok();
    let missing = if usable { Status::Warn } else { Status::Fail };
    vec![
        match &hardware {
            Ok(gpu) => Check::new("gpu", Status::Ok, describe(gpu)),
            Err(e) => Check::new("gpu", missing, format!("{e}; update the graphics driver")),
        },
        match &software {
            Ok(gpu) => Check::new("software", Status::Ok, describe(gpu)),
            Err(e) => Check::new(
                "software",
                missing,
                format!("{e}; golden image tests need it (WARP on Windows, lavapipe on Linux)"),
            ),
        },
    ]
}

/// ffmpeg's version, and whether it has the H.264 encoder video export uses.
fn ffmpeg(program: &OsStr) -> Check {
    // Name the program only when KINETIC_FFMPEG points somewhere else.
    let shown = match program.to_string_lossy() {
        name if name == "ffmpeg" => String::new(),
        name => format!(" ({name})"),
    };
    let version = match first_line(program, &["-hide_banner", "-version"]) {
        // "ffmpeg version 8.1 Copyright (c) ...": keep the version.
        Ok(line) => line.split(" Copyright").next().unwrap_or_default().to_owned(),
        Err(why) => {
            return Check::new(
                "ffmpeg",
                Status::Warn,
                format!(
                    "{why}; video export needs it (install ffmpeg or set KINETIC_FFMPEG), \
                     PNG frames do not"
                ),
            )
        }
    };
    let encoders = Command::new(program).args(["-hide_banner", "-encoders"]).output();
    let has_x264 =
        encoders.map(|o| String::from_utf8_lossy(&o.stdout).contains("libx264")).unwrap_or(false);
    if has_x264 {
        Check::new("ffmpeg", Status::Ok, format!("{version}{shown}, with libx264"))
    } else {
        Check::new(
            "ffmpeg",
            Status::Warn,
            format!("{version}{shown} has no libx264, so video export fails; use a full build"),
        )
    }
}

fn first_line(program: &OsStr, args: &[&str]) -> Result<String, String> {
    let shown = program.to_string_lossy();
    let output = Command::new(program).args(args).output().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!("{shown} not found"),
        _ => format!("could not run {shown}: {e}"),
    })?;
    if !output.status.success() {
        return Err(format!("{shown} failed ({})", output.status));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().next().unwrap_or_default().trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_ffmpeg_is_a_warning_not_a_failure() {
        let check = ffmpeg(OsStr::new("kinetic-no-such-ffmpeg"));
        assert_eq!(check.status, Status::Warn);
        assert!(check.detail.contains("not found"), "{}", check.detail);
        assert!(check.detail.contains("PNG frames do not"), "{}", check.detail);
    }

    #[test]
    fn a_missing_tool_fails_and_says_where_to_get_it() {
        let check = tool("cargo", OsStr::new("kinetic-no-such-cargo"), "builds scene crates");
        assert_eq!(check.status, Status::Fail);
        assert!(check.detail.contains("kinetic-no-such-cargo not found"), "{}", check.detail);
    }

    #[test]
    fn the_report_is_ready_only_without_failures() {
        let ok = Check::new("cargo", Status::Ok, "cargo 1.0");
        let warn = Check::new("ffmpeg", Status::Warn, "not found");
        let fail = Check::new("gpu", Status::Fail, "none");
        let (text, healthy) = report(&[ok.clone(), warn.clone()]);
        assert!(healthy);
        assert!(text.contains("  ok    cargo     cargo 1.0"), "{text}");
        assert!(text.contains("  warn  ffmpeg    not found"), "{text}");
        assert!(text.ends_with("ready"));
        let (text, healthy) = report(&[ok, warn, fail]);
        assert!(!healthy);
        assert!(text.contains("FAIL  gpu"), "{text}");
        assert!(text.ends_with("fix the FAIL lines above"));
    }
}
