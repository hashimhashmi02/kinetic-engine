//! Project maintenance tasks, run as `cargo xtask <task>`.

use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo xtask <task>

tasks:
  bless    regenerate golden images from the current renderer output";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("bless") => bless(),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

/// Reruns the golden tests with `KINETIC_BLESS=1`, which makes them write their output as
/// the new reference images. Look at the images before committing them.
fn bless() -> ExitCode {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args(["test", "-p", "kinetic-render", "--test", "golden"])
        .env("KINETIC_BLESS", "1")
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("goldens rewritten under tests/golden/; review them before committing");
            ExitCode::SUCCESS
        }
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("could not run cargo: {e}");
            ExitCode::FAILURE
        }
    }
}
