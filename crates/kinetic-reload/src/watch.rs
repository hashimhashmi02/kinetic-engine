//! Watching a scene crate and rebuilding it in the background.

use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};

use kinetic_core::Timeline;
use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, Debouncer};

use crate::build::SceneCrate;
use crate::load::{clean_scratch, load};

/// How long the files must stay quiet before a rebuild starts. Editors often write a file
/// in several steps; this turns one save into one build.
pub const DEBOUNCE: Duration = Duration::from_millis(150);

/// What the reloader is doing, reported from its background thread.
#[derive(Debug, Clone, PartialEq)]
pub enum ReloadEvent {
    /// A build has started.
    Building,
    /// A new timeline is ready. `took` covers the build and the load.
    Loaded { timeline: Box<Timeline>, took: Duration },
    /// The build or load failed. The text is ready to show to the person editing.
    Failed(String),
}

/// The scene could not be watched.
#[derive(Debug)]
pub struct WatchError(String);

impl fmt::Display for WatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not watch the scene for changes: {}", self.0)
    }
}

impl std::error::Error for WatchError {}

/// Builds a scene once, then again whenever its sources or manifest change. Dropping the
/// reloader stops the watching; a build already running finishes on its own.
pub struct Reloader {
    _debouncer: Debouncer<RecommendedWatcher>,
    rebuild: Sender<()>,
}

impl fmt::Debug for Reloader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reloader").finish_non_exhaustive()
    }
}

impl Reloader {
    /// Starts watching `scene` and queues the first build. `on_event` is called from a
    /// background thread for every [`ReloadEvent`].
    pub fn start(
        scene: SceneCrate,
        target_dir: PathBuf,
        scratch: PathBuf,
        on_event: impl Fn(ReloadEvent) + Send + 'static,
    ) -> Result<Self, WatchError> {
        let (rebuild, requests) = channel::<()>();
        clean_scratch(&scratch);

        let dir = scene.dir().to_path_buf();
        std::thread::Builder::new()
            .name("kinetic-reload".into())
            .spawn(move || {
                while requests.recv().is_ok() {
                    // Saves that arrived during the last build are covered by this one.
                    while requests.try_recv().is_ok() {}
                    on_event(ReloadEvent::Building);
                    let started = Instant::now();
                    let event = match scene.build(&target_dir) {
                        Ok(library) => match load(&library, &scratch) {
                            Ok(timeline) => ReloadEvent::Loaded {
                                timeline: Box::new(timeline),
                                took: started.elapsed(),
                            },
                            Err(error) => ReloadEvent::Failed(error.to_string()),
                        },
                        Err(error) => ReloadEvent::Failed(error.to_string()),
                    };
                    on_event(event);
                }
            })
            .map_err(|e| WatchError(e.to_string()))?;

        let on_change = rebuild.clone();
        let mut debouncer =
            new_debouncer(DEBOUNCE, move |result: notify_debouncer_mini::DebounceEventResult| {
                // A batch only arrives when something changed; watcher errors are ignored, and
                // the next save triggers a build as usual.
                if result.is_ok() {
                    let _ = on_change.send(());
                }
            })
            .map_err(|e| WatchError(e.to_string()))?;
        debouncer
            .watcher()
            .watch(&dir.join("src"), RecursiveMode::Recursive)
            .map_err(|e| WatchError(e.to_string()))?;
        debouncer
            .watcher()
            .watch(&dir.join("Cargo.toml"), RecursiveMode::NonRecursive)
            .map_err(|e| WatchError(e.to_string()))?;

        let reloader = Self { _debouncer: debouncer, rebuild };
        reloader.request();
        Ok(reloader)
    }

    /// Queues a rebuild now, as if a file had been saved.
    pub fn request(&self) {
        let _ = self.rebuild.send(());
    }
}
