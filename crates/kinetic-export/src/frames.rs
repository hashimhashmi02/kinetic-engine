//! Which frames to render, and when each one is.

use crate::ExportError;

/// Slack for floating-point products such as `duration * fps`, in frames: a product within
/// this distance of a whole number counts as that number.
const SNAP: f64 = 1e-6;

/// One frame to render: its index and the scene time it shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTime {
    /// Position from the start of the timeline at this frame rate, starting at 0.
    pub index: u64,
    /// Seconds, always `index as f64 / fps`.
    pub time: f64,
}

/// A run of frames at a fixed rate.
///
/// Frame `i` shows the scene at `i / fps` seconds, computed from the index every time and
/// never by adding up frame durations, so frame 100 000 is as exact as frame 1, and a
/// frame rendered as part of a range is the same frame a full render produces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frames {
    fps: f64,
    first: u64,
    last: u64,
}

impl Frames {
    /// Every frame of a timeline `duration` seconds long at `fps` frames per second: from
    /// `t = 0` up to and including the first frame at or after the end, so the final pose
    /// is always in the output. A zero duration gives one frame.
    pub fn new(duration: f64, fps: f64) -> Result<Self, ExportError> {
        if !(fps.is_finite() && fps > 0.0 && fps <= 1000.0) {
            return Err(ExportError::Settings(format!(
                "frame rate {fps} must be above 0 and at most 1000"
            )));
        }
        if !(duration.is_finite() && duration >= 0.0) {
            return Err(ExportError::Settings(format!("duration {duration} s is not usable")));
        }
        let last = (duration * fps - SNAP).ceil().max(0.0) as u64;
        Ok(Self { fps, first: 0, last })
    }

    /// Only the frames that show a time from `start` to `end` seconds, both included. The
    /// indices and times stay those of the full run.
    pub fn range(self, start: f64, end: f64) -> Result<Self, ExportError> {
        if start.is_nan() || end.is_nan() || start > end {
            return Err(ExportError::Settings(format!(
                "range {start}..{end} s is empty; the start must not be after the end"
            )));
        }
        let first = (start * self.fps - SNAP).ceil().max(0.0);
        let last = (end * self.fps + SNAP).floor();
        let (first, last) = (first.min(u64::MAX as f64) as u64, last.max(0.0) as u64);
        let (first, last) = (first.max(self.first), last.min(self.last));
        if first > last || end < 0.0 {
            return Err(ExportError::Settings(format!(
                "no frame falls in {start}..{end} s; at {} fps the frames run from 0 to {} s",
                self.fps,
                self.time(self.last)
            )));
        }
        Ok(Self { first, last, ..self })
    }

    /// Frames per second.
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Index of the first frame.
    pub fn first(&self) -> u64 {
        self.first
    }

    /// Index of the last frame, included.
    pub fn last(&self) -> u64 {
        self.last
    }

    /// Number of frames; never zero.
    pub fn len(&self) -> u64 {
        self.last - self.first + 1
    }

    /// Always false: a run holds at least one frame. Present for symmetry with [`Self::len`].
    pub fn is_empty(&self) -> bool {
        false
    }

    /// The scene time frame `index` shows, in seconds.
    pub fn time(&self, index: u64) -> f64 {
        index as f64 / self.fps
    }

    /// The frames in order.
    pub fn iter(&self) -> impl Iterator<Item = FrameTime> + '_ {
        (self.first..=self.last).map(|index| FrameTime { index, time: self.time(index) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn two_seconds_at_sixty_includes_the_end() {
        let frames = Frames::new(2.0, 60.0).expect("frames");
        assert_eq!(frames.len(), 121);
        let all: Vec<_> = frames.iter().collect();
        assert_eq!(all[0], FrameTime { index: 0, time: 0.0 });
        assert_eq!(all[120], FrameTime { index: 120, time: 2.0 });
    }

    #[test]
    fn a_product_just_above_a_whole_number_does_not_add_a_frame() {
        // 1/3 * 60 is 20.000000000000004 in floating point.
        assert_eq!(Frames::new(1.0 / 3.0, 60.0).expect("frames").last(), 20);
        assert_eq!(Frames::new(0.1, 30.0).expect("frames").last(), 3);
    }

    #[test]
    fn a_duration_between_frames_rounds_up() {
        let frames = Frames::new(1.01, 10.0).expect("frames");
        assert_eq!(frames.last(), 11);
        assert!(frames.time(frames.last()) >= 1.01);
    }

    #[test]
    fn an_empty_timeline_is_one_frame() {
        let frames = Frames::new(0.0, 24.0).expect("frames");
        assert_eq!(frames.iter().collect::<Vec<_>>(), [FrameTime { index: 0, time: 0.0 }]);
    }

    #[test]
    fn far_frames_are_exact() {
        let frames = Frames::new(4000.0, 60.0).expect("frames");
        assert_eq!(frames.time(239_999), 239_999.0 / 60.0);
        assert_eq!(frames.time(frames.last()), 4000.0);
    }

    #[test]
    fn bad_rates_and_durations_are_refused() {
        for fps in [0.0, -1.0, f64::NAN, f64::INFINITY, 1001.0] {
            assert!(Frames::new(1.0, fps).is_err(), "{fps}");
        }
        for duration in [-0.5, f64::NAN, f64::INFINITY] {
            assert!(Frames::new(duration, 30.0).is_err(), "{duration}");
        }
    }

    #[test]
    fn a_range_keeps_the_frames_of_the_full_run() {
        let full = Frames::new(3.0, 30.0).expect("frames");
        let part = full.range(1.0, 1.5).expect("range");
        assert_eq!((part.first(), part.last(), part.len()), (30, 45, 16));
        let times: Vec<_> = part.iter().collect();
        assert!(times.iter().all(|f| f.time == full.time(f.index)));
    }

    #[test]
    fn a_range_between_frames_takes_the_frames_inside_it() {
        let part = Frames::new(1.0, 10.0).expect("frames").range(0.15, 0.42).expect("range");
        assert_eq!((part.first(), part.last()), (2, 4));
    }

    #[test]
    fn a_range_past_either_end_is_clamped() {
        let full = Frames::new(2.0, 10.0).expect("frames");
        let part = full.range(-5.0, 100.0).expect("range");
        assert_eq!(part, full);
    }

    #[test]
    fn a_range_without_frames_is_refused() {
        let full = Frames::new(2.0, 10.0).expect("frames");
        for (start, end) in [(0.11, 0.19), (3.0, 4.0), (1.0, 0.5), (-2.0, -1.0), (f64::NAN, 1.0)] {
            let error = full.range(start, end).expect_err("empty");
            assert!(error.to_string().contains("..") || error.to_string().contains("range"));
        }
    }

    proptest! {
        #[test]
        fn frames_cover_the_timeline_and_no_more(duration in 0.0..120.0f64, fps in 1.0..240.0f64) {
            let frames = Frames::new(duration, fps).expect("frames");
            let times: Vec<f64> = frames.iter().map(|f| f.time).collect();
            prop_assert_eq!(times[0], 0.0);
            prop_assert!(times.windows(2).all(|w| w[0] < w[1]));
            let last = frames.time(frames.last());
            prop_assert!(last >= duration - SNAP / fps);
            if frames.last() > 0 {
                prop_assert!(frames.time(frames.last() - 1) < duration);
            }
        }

        #[test]
        fn every_time_comes_from_its_index(duration in 0.0..60.0f64, fps in 1.0..240.0f64) {
            let frames = Frames::new(duration, fps).expect("frames");
            for f in frames.iter() {
                prop_assert_eq!(f.time, f.index as f64 / fps);
            }
        }

        #[test]
        fn a_range_is_a_slice_of_the_full_run(
            duration in 0.1..30.0f64,
            fps in 1.0..120.0f64,
            a in 0.0..1.0f64,
            b in 0.0..1.0f64,
        ) {
            let full = Frames::new(duration, fps).expect("frames");
            let (start, end) = (a.min(b) * duration, a.max(b) * duration);
            if let Ok(part) = full.range(start, end) {
                prop_assert!(part.first() >= full.first() && part.last() <= full.last());
                for f in part.iter() {
                    prop_assert!(f.time >= start - SNAP / fps && f.time <= end + SNAP / fps);
                    prop_assert_eq!(f.time, full.time(f.index));
                }
                // No frame inside the range was left out.
                if part.first() > 0 {
                    prop_assert!(full.time(part.first() - 1) < start);
                }
                if part.last() < full.last() {
                    prop_assert!(full.time(part.last() + 1) > end);
                }
            }
        }
    }
}
