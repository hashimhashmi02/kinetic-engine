//! Playback state: where the playhead is and whether it is moving.
//!
//! The wall clock only ever advances `time`; what is drawn depends on `time` alone.

use std::time::Instant;

#[derive(Debug)]
pub(crate) struct Transport {
    pub time: f64,
    pub duration: f64,
    pub playing: bool,
    last_tick: Instant,
}

impl Transport {
    /// At the start of a timeline `duration` seconds long, playing.
    pub fn new(duration: f64) -> Self {
        Self { time: 0.0, duration: duration.max(0.0), playing: true, last_tick: Instant::now() }
    }

    /// Advances by the wall-clock time since the last tick, if playing.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.advance(now.duration_since(self.last_tick).as_secs_f64());
        self.last_tick = now;
    }

    /// Advances by `seconds` if playing, looping at the end. A zero-length timeline stays on
    /// its only frame.
    pub fn advance(&mut self, seconds: f64) {
        if self.playing {
            self.time += seconds;
            self.time = if self.duration > 0.0 { self.time % self.duration } else { 0.0 };
        }
    }

    /// A new version of the timeline arrived. The playhead stays where it was, so editing
    /// a scene keeps you looking at the same moment; it is clamped if the timeline got
    /// shorter. Playing or paused stays as it was.
    pub fn retime(&mut self, duration: f64) {
        self.duration = duration.max(0.0);
        self.time = self.time.clamp(0.0, self.duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_and_loops() {
        let mut t = Transport::new(2.0);
        t.advance(1.5);
        assert_eq!(t.time, 1.5);
        t.advance(1.0);
        assert_eq!(t.time, 0.5);
    }

    #[test]
    fn paused_does_not_move() {
        let mut t = Transport::new(2.0);
        t.playing = false;
        t.advance(1.0);
        assert_eq!(t.time, 0.0);
    }

    #[test]
    fn retime_keeps_the_playhead_and_play_state() {
        let mut t = Transport::new(4.0);
        t.advance(1.25);
        t.playing = false;
        t.retime(6.0);
        assert_eq!((t.time, t.duration, t.playing), (1.25, 6.0, false));
    }

    #[test]
    fn retime_clamps_when_the_timeline_gets_shorter() {
        let mut t = Transport::new(4.0);
        t.advance(3.5);
        t.retime(2.0);
        assert_eq!(t.time, 2.0);
        assert!(t.playing);
    }

    #[test]
    fn zero_length_timeline_holds_still() {
        let mut t = Transport::new(0.0);
        t.advance(1.0);
        assert_eq!(t.time, 0.0);
        t.retime(-3.0);
        assert_eq!(t.duration, 0.0);
    }
}
