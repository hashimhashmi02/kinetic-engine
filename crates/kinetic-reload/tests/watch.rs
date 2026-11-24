//! The reloader rebuilds a real scene crate when its source changes.

mod common;

use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use kinetic_reload::{ReloadEvent, Reloader, SceneCrate};

const FIRST_BUILD: Duration = Duration::from_secs(300);
const REBUILD: Duration = Duration::from_secs(120);

fn scene(duration: f64) -> String {
    format!(
        "fn scene(s: &mut Scene) {{ let c = s.add(Circle::new(1.0)); s.play(c.move_to((1.0, 0.0)), {duration:?}); }}"
    )
}

/// The next event that is not `Building`.
fn outcome(events: &Receiver<ReloadEvent>, within: Duration) -> ReloadEvent {
    loop {
        match events.recv_timeout(within).expect("reloader went quiet") {
            ReloadEvent::Building => continue,
            other => return other,
        }
    }
}

#[test]
fn rebuilds_on_save_and_reports_errors_without_stopping() {
    let dir = common::fixture("watched", &scene(1.0));
    let (send, events) = channel();
    let _reloader = Reloader::start(
        SceneCrate::open(&dir).expect("open"),
        common::target_dir(),
        common::target_dir().join("scratch").join("watched"),
        move |event| {
            let _ = send.send(event);
        },
    )
    .expect("start");

    match outcome(&events, FIRST_BUILD) {
        ReloadEvent::Loaded { timeline, .. } => assert_eq!(timeline.duration, 1.0),
        other => panic!("first build: {other:?}"),
    }

    common::fixture("watched", &scene(3.0));
    match outcome(&events, REBUILD) {
        ReloadEvent::Loaded { timeline, took } => {
            assert_eq!(timeline.duration, 3.0);
            eprintln!("rebuild and reload took {took:?}");
        }
        other => panic!("after edit: {other:?}"),
    }

    common::fixture("watched", "fn scene(s: &mut Scene) { s.play(); }");
    match outcome(&events, REBUILD) {
        ReloadEvent::Failed(text) => assert!(text.contains("error"), "{text}"),
        other => panic!("after a broken edit: {other:?}"),
    }

    // A fix after a failure is picked up like any other save.
    common::fixture("watched", &scene(2.0));
    match outcome(&events, REBUILD) {
        ReloadEvent::Loaded { timeline, .. } => assert_eq!(timeline.duration, 2.0),
        other => panic!("after the fix: {other:?}"),
    }
}
