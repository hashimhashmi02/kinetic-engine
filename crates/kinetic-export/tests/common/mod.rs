//! Shared helpers for export tests.

// Each test binary uses its own subset of these helpers.
#![allow(dead_code)]

use std::sync::{Mutex, OnceLock};

use kinetic_render::{AdapterChoice, Gpu};

/// Runs `test` with a device on the software adapter, so every machine renders the same
/// pixels. Tests take turns with one shared device, as in the render tests.
///
/// Skipped (with a notice) only when no adapter exists and `KINETIC_ALLOW_NO_GPU=1`.
pub fn with_gpu(test: impl FnOnce(&Gpu)) {
    static GPU: OnceLock<Result<Gpu, String>> = OnceLock::new();
    static TURN: Mutex<()> = Mutex::new(());
    let _turn = TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match GPU.get_or_init(|| Gpu::headless(AdapterChoice::Software).map_err(|e| e.to_string())) {
        Ok(gpu) => test(gpu),
        Err(error) if std::env::var("KINETIC_ALLOW_NO_GPU").as_deref() == Ok("1") => {
            eprintln!("SKIPPED: {error} (KINETIC_ALLOW_NO_GPU=1)");
        }
        Err(error) => panic!("{error}. Set KINETIC_ALLOW_NO_GPU=1 to skip render tests."),
    }
}
