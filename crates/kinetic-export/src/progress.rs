//! A render's progress as one line of text.

use std::time::Duration;

/// Width of the bar, in characters, between the brackets.
const BAR: usize = 24;

/// `[#########---------------]  120/600  20%  45.2 fps  ETA 0:10`
///
/// `done` frames of `total` are finished after `elapsed`. The rate and the time left come
/// from the average so far; before the first frame there is no estimate. The caller reads
/// the clock, so the line itself is a pure function of its arguments.
pub fn progress_line(done: u64, total: u64, elapsed: Duration) -> String {
    let total = total.max(1);
    let done = done.min(total);
    let filled = (done as f64 / total as f64 * BAR as f64).floor() as usize;
    let bar = format!("{}{}", "#".repeat(filled), "-".repeat(BAR - filled));
    let percent = done * 100 / total;
    let width = total.to_string().len();
    let seconds = elapsed.as_secs_f64();
    let (rate, eta) = if done == 0 || seconds <= 0.0 {
        ("-- fps".to_owned(), "--:--".to_owned())
    } else {
        let rate = done as f64 / seconds;
        let left = (total - done) as f64 / rate;
        (format!("{rate:.1} fps"), clock(left))
    };
    format!("[{bar}] {done:>width$}/{total}  {percent:>3}%  {rate}  ETA {eta}")
}

/// `m:ss`, or `h:mm:ss` from an hour up, rounded up to whole seconds.
fn clock(seconds: f64) -> String {
    let s = seconds.max(0.0).ceil() as u64;
    let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn before_the_first_frame_there_is_no_estimate() {
        assert_eq!(
            progress_line(0, 600, Duration::ZERO),
            "[------------------------]   0/600    0%  -- fps  ETA --:--"
        );
    }

    #[test]
    fn the_estimate_comes_from_the_average_rate() {
        // 120 frames in 4 s is 30 fps; 480 frames left take 16 s.
        assert_eq!(
            progress_line(120, 600, Duration::from_secs(4)),
            "[####--------------------] 120/600   20%  30.0 fps  ETA 0:16"
        );
    }

    #[test]
    fn a_finished_render_has_a_full_bar() {
        let line = progress_line(600, 600, Duration::from_secs(20));
        assert!(line.starts_with(&format!("[{}] 600/600  100%", "#".repeat(24))), "{line}");
        assert!(line.ends_with("ETA 0:00"), "{line}");
    }

    #[test]
    fn long_renders_show_hours() {
        assert_eq!(clock(3725.2), "1:02:06");
        assert_eq!(clock(59.01), "1:00");
        assert_eq!(clock(0.0), "0:00");
    }

    #[test]
    fn the_line_keeps_its_width_as_it_counts() {
        let lines =
            [1, 9, 10, 99, 100, 999].map(|d| progress_line(d, 1000, Duration::from_secs(1)));
        let width = |l: &String| l.find("  ").unwrap_or(0);
        assert!(lines.iter().all(|l| width(l) == width(&lines[0])), "{lines:#?}");
    }

    #[test]
    fn out_of_range_counts_are_clamped() {
        assert!(progress_line(7, 5, Duration::from_secs(1)).contains("5/5  100%"));
        assert!(progress_line(0, 0, Duration::ZERO).contains("0/1"));
    }
}
