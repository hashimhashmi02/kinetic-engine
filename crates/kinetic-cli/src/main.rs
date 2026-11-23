//! Command line entry point: preview, render, new.

use std::process::ExitCode;

const USAGE: &str = "usage: kinetic <command>

commands:
  preview    open the preview window
  version    print the version";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("preview") => match kinetic_editor::run() {
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
