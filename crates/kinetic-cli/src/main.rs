//! Command line entry point: preview, render, new.

use std::process::ExitCode;

const USAGE: &str = "usage: kinetic <command>

commands:
  preview    open the preview window on the built-in hello scene
  version    print the version";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        // Until hot reload (M3) loads scene libraries, preview plays the built-in hello scene.
        Some("preview") => match kinetic_editor::run(hello::build()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("kinetic: {error}");
                ExitCode::FAILURE
            }
        },
        Some("version" | "--version" | "-V") => {
            println!("kinetic {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("help" | "--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
