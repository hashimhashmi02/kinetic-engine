//! Encoding frames to H.264 with an ffmpeg child process.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::thread::JoinHandle;

use crate::ExportError;

/// The ffmpeg program to run: `KINETIC_FFMPEG` when set, otherwise `ffmpeg` from `PATH`.
pub fn ffmpeg_program() -> OsString {
    std::env::var_os("KINETIC_FFMPEG").unwrap_or_else(|| "ffmpeg".into())
}

/// What to encode: the size and rate of the incoming frames and the quality to keep.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VideoSettings {
    /// Width in pixels; must be even for 4:2:0 video.
    pub width: u32,
    /// Height in pixels; must be even for 4:2:0 video.
    pub height: u32,
    /// Frames per second.
    pub fps: f64,
    /// x264 constant rate factor, 0 (lossless) to 51 (worst). 16 is visually lossless for
    /// flat-coloured motion graphics.
    pub crf: u8,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self { width: 1280, height: 720, fps: 60.0, crf: 16 }
    }
}

impl VideoSettings {
    /// Checks the settings before anything is started.
    pub fn validate(&self) -> Result<(), ExportError> {
        let (w, h) = (self.width, self.height);
        if w == 0 || h == 0 || w % 2 == 1 || h % 2 == 1 {
            return Err(ExportError::Settings(format!(
                "video needs an even, non-zero width and height; got {w}x{h}. Try {}x{}, or \
                 write PNG frames instead",
                (w + 1) & !1,
                (h + 1) & !1
            )));
        }
        if !(self.fps.is_finite() && self.fps > 0.0) {
            return Err(ExportError::Settings(format!("frame rate {} must be above 0", self.fps)));
        }
        if self.crf > 51 {
            return Err(ExportError::Settings(format!(
                "quality (crf) {} must be from 0 to 51",
                self.crf
            )));
        }
        Ok(())
    }

    /// The ffmpeg arguments that read raw frames from stdin and write `output`.
    ///
    /// Frames come in as sRGB RGBA8 and leave as BT.709 limited-range 4:2:0 H.264, tagged
    /// as such, so players show the colours that were rendered.
    pub fn args(&self, output: &Path) -> Vec<OsString> {
        let size = format!("{}x{}", self.width, self.height);
        let mut args: Vec<OsString> = [
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-s",
            &size,
            "-framerate",
            &self.fps.to_string(),
            "-i",
            "pipe:0",
            "-vf",
            "scale=out_color_matrix=bt709:out_range=tv,format=yuv420p",
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            &self.crf.to_string(),
            "-colorspace",
            "bt709",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-color_range",
            "tv",
            "-movflags",
            "+faststart",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        args.push(output.as_os_str().to_owned());
        args
    }
}

/// A running ffmpeg that turns raw frames written to it into a video file.
///
/// Write every frame with [`Encoder::write_frame`], then call [`Encoder::finish`] to wait
/// for the file to be complete. Dropping an unfinished encoder stops ffmpeg.
#[derive(Debug)]
pub struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
    log: Option<JoinHandle<String>>,
    frame_bytes: usize,
    output: PathBuf,
}

impl Encoder {
    /// Starts `program` (usually [`ffmpeg_program`]) encoding to `output`.
    pub fn spawn(
        program: impl AsRef<std::ffi::OsStr>,
        output: &Path,
        settings: &VideoSettings,
    ) -> Result<Self, ExportError> {
        settings.validate()?;
        let program = program.as_ref();
        let mut child = Command::new(program)
            .args(settings.args(output))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => {
                    ExportError::FfmpegMissing { program: program.to_string_lossy().into_owned() }
                }
                _ => ExportError::Ffmpeg(format!(
                    "could not start {}: {error}",
                    program.to_string_lossy()
                )),
            })?;
        // Read the log as it comes, so a chatty ffmpeg never blocks on a full pipe.
        let log = child.stderr.take().map(|mut stderr| {
            std::thread::spawn(move || {
                let mut log = String::new();
                let mut bytes = Vec::new();
                if stderr.read_to_end(&mut bytes).is_ok() {
                    log = String::from_utf8_lossy(&bytes).into_owned();
                }
                log
            })
        });
        Ok(Self {
            stdin: child.stdin.take(),
            child,
            log,
            frame_bytes: settings.width as usize * settings.height as usize * 4,
            output: output.to_path_buf(),
        })
    }

    /// Sends one frame: tightly packed sRGB RGBA8 rows, top to bottom, of the size the
    /// encoder was started with.
    pub fn write_frame(&mut self, rgba: &[u8]) -> Result<(), ExportError> {
        if rgba.len() != self.frame_bytes {
            return Err(ExportError::Settings(format!(
                "frame has {} bytes; the video expects {}",
                rgba.len(),
                self.frame_bytes
            )));
        }
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(ExportError::Ffmpeg("ffmpeg's input is already closed".into()));
        };
        if let Err(error) = stdin.write_all(rgba) {
            // ffmpeg stopped reading, almost always because it failed; its log says why.
            return Err(self.failure(&format!("ffmpeg stopped taking frames ({error})")));
        }
        Ok(())
    }

    /// Closes ffmpeg's input and waits for it to write the end of the file.
    pub fn finish(mut self) -> Result<(), ExportError> {
        drop(self.stdin.take());
        let status = self
            .child
            .wait()
            .map_err(|e| ExportError::Ffmpeg(format!("could not wait for ffmpeg: {e}")))?;
        let log = self.take_log();
        if status.success() {
            Ok(())
        } else {
            Err(ExportError::Ffmpeg(failure_message(
                &format!("ffmpeg could not write {} ({status})", self.output.display()),
                &log,
            )))
        }
    }

    /// Stops ffmpeg after a failed write and collects what it said.
    fn failure(&mut self, what: &str) -> ExportError {
        drop(self.stdin.take());
        let status = self.child.wait().map(|s| format!(", {s}")).unwrap_or_default();
        let log = self.take_log();
        ExportError::Ffmpeg(failure_message(&format!("{what}{status}"), &log))
    }

    fn take_log(&mut self) -> String {
        self.log.take().and_then(|log| log.join().ok()).unwrap_or_default()
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        if self.stdin.is_some() {
            // Never finished: stop ffmpeg rather than leave it waiting for frames.
            drop(self.stdin.take());
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn failure_message(what: &str, log: &str) -> String {
    let log = log.trim();
    if log.is_empty() {
        what.to_owned()
    } else {
        format!("{what}:\n{log}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odd_sizes_are_refused_with_a_suggestion() {
        let settings = VideoSettings { width: 333, height: 200, ..Default::default() };
        let error = settings.validate().expect_err("odd width");
        assert!(error.to_string().contains("334x200"), "{error}");
        assert!(VideoSettings { width: 0, ..Default::default() }.validate().is_err());
    }

    #[test]
    fn quality_and_rate_are_checked() {
        assert!(VideoSettings { crf: 52, ..Default::default() }.validate().is_err());
        assert!(VideoSettings { fps: 0.0, ..Default::default() }.validate().is_err());
        assert!(VideoSettings::default().validate().is_ok());
    }

    #[test]
    fn arguments_describe_the_raw_input_and_tag_the_output() {
        let settings = VideoSettings { width: 640, height: 360, fps: 30.0, crf: 20 };
        let args: Vec<String> = settings
            .args(Path::new("out.mp4"))
            .into_iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        assert!(joined.contains("-f rawvideo -pix_fmt rgba -s 640x360 -framerate 30 -i pipe:0"));
        assert!(joined.contains("-crf 20"));
        assert!(joined.contains("-colorspace bt709"));
        assert_eq!(args.last().map(String::as_str), Some("out.mp4"));
    }

    #[test]
    fn a_missing_program_is_reported_as_missing() {
        let error = Encoder::spawn(
            "kinetic-no-such-ffmpeg",
            Path::new("never.mp4"),
            &VideoSettings::default(),
        )
        .expect_err("no such program");
        assert!(matches!(error, ExportError::FfmpegMissing { .. }), "{error}");
        assert!(error.to_string().contains("kinetic-no-such-ffmpeg"), "{error}");
    }
}
