//! Shared helpers for render tests: a software GPU and golden image comparison.

// Each test binary uses its own subset of these helpers.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use kinetic_render::{AdapterChoice, Gpu};

/// Largest per-channel difference that still counts as the same pixel.
const CHANNEL_TOLERANCE: u8 = 2;
/// Share of pixels allowed to differ by more than the tolerance.
const MAX_DIFFERENT: f64 = 0.001;

/// Runs `test` with a device on the software adapter, so every machine renders the same
/// pixels.
///
/// One device is shared by every test in the binary and tests take turns with it:
/// creating several devices at once from parallel test threads crashed the process on
/// Windows.
///
/// The test is skipped (with a notice) only when no adapter exists and
/// `KINETIC_ALLOW_NO_GPU=1` is set. Otherwise a missing adapter fails the test, so a
/// misconfigured CI machine cannot pass by silently skipping.
pub fn with_gpu(test: impl FnOnce(&Gpu)) {
    static GPU: OnceLock<Result<Gpu, String>> = OnceLock::new();
    static TURN: Mutex<()> = Mutex::new(());
    // A test that panicked while holding the lock should not fail the others.
    let _turn = TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match GPU.get_or_init(|| Gpu::headless(AdapterChoice::Software).map_err(|e| e.to_string())) {
        Ok(gpu) => test(gpu),
        Err(error) if std::env::var("KINETIC_ALLOW_NO_GPU").as_deref() == Ok("1") => {
            eprintln!("SKIPPED: {error} (KINETIC_ALLOW_NO_GPU=1)");
        }
        Err(error) => panic!("{error}. Set KINETIC_ALLOW_NO_GPU=1 to skip render tests."),
    }
}

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/render")
}

fn write_png(path: &PathBuf, width: u32, height: u32, rgba: &[u8]) {
    let file = std::fs::File::create(path).unwrap_or_else(|e| panic!("create {path:?}: {e}"));
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("png header");
    writer.write_image_data(rgba).expect("png data");
}

fn read_png(path: &PathBuf) -> (u32, u32, Vec<u8>) {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("open {path:?}: {e}"));
    let mut reader =
        png::Decoder::new(std::io::BufReader::new(file)).read_info().expect("png info");
    let mut data = vec![0; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut data).expect("png frame");
    data.truncate(info.buffer_size());
    (info.width, info.height, data)
}

/// Compares `rgba` with the golden image `name`. With `KINETIC_BLESS=1` (set by
/// `cargo xtask bless`) the golden is rewritten instead.
///
/// On a mismatch, `<name>.actual.png` and `<name>.diff.png` are written beside the golden.
pub fn assert_golden(name: &str, width: u32, height: u32, rgba: &[u8]) {
    let dir = golden_dir();
    let golden = dir.join(format!("{name}.png"));
    if std::env::var("KINETIC_BLESS").as_deref() == Ok("1") {
        std::fs::create_dir_all(&dir).expect("create golden dir");
        write_png(&golden, width, height, rgba);
        eprintln!("blessed {}", golden.display());
        return;
    }
    assert!(golden.exists(), "missing golden {}; run `cargo xtask bless`", golden.display());
    let (gw, gh, expected) = read_png(&golden);
    assert_eq!((gw, gh), (width, height), "golden {name} has a different size");

    let mut different = 0usize;
    let mut diff = Vec::with_capacity(rgba.len());
    for (a, b) in rgba.as_chunks::<4>().0.iter().zip(expected.as_chunks::<4>().0) {
        let worst = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0);
        if worst > CHANNEL_TOLERANCE {
            different += 1;
            diff.extend_from_slice(&[255, 0, 80, 255]);
        } else {
            diff.extend_from_slice(&[b[0] / 4, b[1] / 4, b[2] / 4, 255]);
        }
    }
    let share = different as f64 / (width * height) as f64;
    if share > MAX_DIFFERENT {
        let actual = dir.join(format!("{name}.actual.png"));
        let diff_path = dir.join(format!("{name}.diff.png"));
        write_png(&actual, width, height, rgba);
        write_png(&diff_path, width, height, &diff);
        panic!(
            "{name}: {different} pixels ({:.2}%) differ from the golden.\n  actual: {}\n  diff:   {}",
            share * 100.0,
            actual.display(),
            diff_path.display()
        );
    }
}
