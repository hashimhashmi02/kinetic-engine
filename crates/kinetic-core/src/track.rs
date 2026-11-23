//! Tracks: how one property of one node changes over time.

use serde::{Deserialize, Serialize};

use crate::bezier::CubicBezier;
use crate::ease::Ease;
use crate::lerp::Lerp;
use crate::node::{NodeId, PropertyId};
use crate::spring::Spring;
use crate::value::Value;

/// How a segment moves from its start value to its end value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Curve {
    /// A named easing over the segment's duration.
    Ease(Ease),
    /// A cubic bezier easing over the segment's duration.
    Bezier(CubicBezier),
    /// A spring; the segment's duration is the spring's settle time.
    Spring(Spring),
}

impl Curve {
    /// Progress at `local` seconds into a segment lasting `duration` seconds. Exactly 0
    /// before the segment starts and exactly 1 once it has ended.
    pub fn progress(&self, local: f64, duration: f64) -> f64 {
        // End first, so a zero-length segment is already complete at its start.
        if local >= duration {
            return 1.0;
        }
        if local <= 0.0 {
            return 0.0;
        }
        match self {
            Self::Ease(ease) => ease.apply(local / duration),
            Self::Bezier(bezier) => bezier.apply(local / duration),
            Self::Spring(spring) => spring.progress(local),
        }
    }
}

/// One change: from `from` at `start` to `to` at `start + duration`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start: f64,
    pub duration: f64,
    /// The track's value at `start`, resolved when the segment was created, so evaluating
    /// a segment never depends on evaluating another.
    pub from: Value,
    pub to: Value,
    pub curve: Curve,
}

impl Segment {
    /// Time at which the segment reaches `to`.
    pub fn end(&self) -> f64 {
        self.start + self.duration
    }

    /// The segment's value at absolute time `t`.
    pub fn value_at(&self, t: f64) -> Value {
        // Compare against the end in absolute time: `t - start` can round to a hair under
        // the duration exactly at the end, which would leave the value short of `to`.
        if t >= self.end() {
            return self.to;
        }
        self.from.lerp(&self.to, self.curve.progress(t - self.start, self.duration))
    }
}

/// Every change to one property of one node.
///
/// Segments are kept sorted by start time. At any moment the segment that started most
/// recently is in charge, so a new segment that begins while an older one is still running
/// takes over from wherever the older one had got to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub node: NodeId,
    pub property: PropertyId,
    pub initial: Value,
    pub segments: Vec<Segment>,
}

impl Track {
    /// An empty track holding `initial`.
    pub fn new(node: NodeId, property: PropertyId, initial: Value) -> Self {
        Self { node, property, initial, segments: Vec::new() }
    }

    /// The value at time `t`.
    pub fn value_at(&self, t: f64) -> Value {
        // Segments are sorted, so the active one is the last that has started.
        let active = self.segments.partition_point(|s| s.start <= t);
        match active.checked_sub(1) {
            Some(i) => self.segments[i].value_at(t),
            None => self.initial,
        }
    }

    /// Adds a change towards `to`, starting at `start`. Its starting value is whatever the
    /// track holds at that moment.
    pub fn push(&mut self, start: f64, duration: f64, to: Value, curve: Curve) {
        let from = self.value_at(start);
        let segment = Segment { start, duration: duration.max(0.0), from, to, curve };
        // Keep the order stable: among equal start times, the newest wins.
        let at = self.segments.partition_point(|s| s.start <= start);
        self.segments.insert(at, segment);
    }

    /// Time at which the last change finishes, or 0 for a track that never changes.
    pub fn end(&self) -> f64 {
        self.segments.iter().map(Segment::end).fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        Track::new(NodeId(0), PropertyId::Opacity, Value::Scalar(0.0))
    }

    const LINEAR: Curve = Curve::Ease(Ease::Linear);

    #[test]
    fn holds_initial_before_any_change() {
        let mut t = track();
        t.push(1.0, 2.0, Value::Scalar(1.0), LINEAR);
        assert_eq!(t.value_at(-5.0), Value::Scalar(0.0));
        assert_eq!(t.value_at(0.999), Value::Scalar(0.0));
    }

    #[test]
    fn interpolates_during_and_holds_after() {
        let mut t = track();
        t.push(1.0, 2.0, Value::Scalar(1.0), LINEAR);
        assert_eq!(t.value_at(2.0), Value::Scalar(0.5));
        assert_eq!(t.value_at(3.0), Value::Scalar(1.0));
        assert_eq!(t.value_at(99.0), Value::Scalar(1.0));
        assert_eq!(t.end(), 3.0);
    }

    #[test]
    fn later_change_starts_from_where_the_earlier_one_ended() {
        let mut t = track();
        t.push(0.0, 1.0, Value::Scalar(1.0), LINEAR);
        t.push(2.0, 1.0, Value::Scalar(0.5), LINEAR);
        assert_eq!(t.segments[1].from, Value::Scalar(1.0));
        assert_eq!(t.value_at(2.5), Value::Scalar(0.75));
    }

    #[test]
    fn interrupting_change_takes_over_without_a_jump() {
        let mut t = track();
        t.push(0.0, 2.0, Value::Scalar(1.0), LINEAR);
        // Starts half-way through the first change.
        t.push(1.0, 1.0, Value::Scalar(0.0), LINEAR);
        assert_eq!(t.segments[1].from, Value::Scalar(0.5));
        let before = t.value_at(1.0 - 1e-9);
        let after = t.value_at(1.0);
        match (before, after) {
            (Value::Scalar(a), Value::Scalar(b)) => assert!((a - b).abs() < 1e-6),
            _ => unreachable!(),
        }
    }

    #[test]
    fn zero_duration_change_jumps() {
        let mut t = track();
        t.push(1.0, 0.0, Value::Scalar(1.0), LINEAR);
        assert_eq!(t.value_at(0.5), Value::Scalar(0.0));
        assert_eq!(t.value_at(1.0), Value::Scalar(1.0));
    }

    #[test]
    fn spring_segment_snaps_to_its_target_at_its_end() {
        let spring = crate::spring::spring();
        let mut t = track();
        t.push(0.0, spring.settle_time(), Value::Scalar(1.0), Curve::Spring(spring));
        assert_eq!(t.value_at(spring.settle_time()), Value::Scalar(1.0));
    }

    #[test]
    fn reaches_its_target_exactly_at_its_end_despite_rounding() {
        // 0.1 + 0.2 is not 0.3 in floating point; the end must still be exact.
        let mut tr = Track::new(NodeId(0), PropertyId::Rotation, Value::Scalar(0.0));
        tr.push(0.1, 0.2, Value::Scalar(-65.129_279_724_760_64), Curve::Ease(Ease::InOutCubic));
        let end = tr.segments[0].end();
        assert_eq!(tr.value_at(end), Value::Scalar(-65.129_279_724_760_64));
    }

    #[test]
    fn segments_stay_sorted_when_pushed_out_of_order() {
        let mut t = track();
        t.push(3.0, 1.0, Value::Scalar(1.0), LINEAR);
        t.push(1.0, 1.0, Value::Scalar(0.5), LINEAR);
        let starts: Vec<f64> = t.segments.iter().map(|s| s.start).collect();
        assert_eq!(starts, [1.0, 3.0]);
    }
}
