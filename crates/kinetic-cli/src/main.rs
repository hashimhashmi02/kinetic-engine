//! Command line entry point: preview, render, new.

mod new;
mod render;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: kinetic <command>

commands:
  new <scene-dir>       create a scene crate
  preview <scene-dir>   open a scene crate and reload it on every save
  render <scene-dir>    export a scene crate to a video; see kinetic render --help
  version               print the version";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["preview", dir] => match kinetic_editor::run_scene(Path::new(dir)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("kinetic: {error}");
                ExitCode::FAILURE
            }
        },
        ["new", dir] => match new::create(Path::new(dir)) {
            Ok(made) => {
                println!("created {}\n\nnext: kinetic preview {dir}", made.display());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("kinetic: {error}");
                ExitCode::FAILURE
            }
        },
        ["render", "--help" | "-h"] => {
            println!("{}", render::USAGE);
            ExitCode::SUCCESS
        }
        ["render", rest @ ..] => match render::parse(rest).and_then(|args| render::run(&args)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("kinetic: {error}");
                ExitCode::FAILURE
            }
        },
        ["preview"] => {
            eprintln!("kinetic: preview needs a scene directory, for example: kinetic preview scenes/hello");
            ExitCode::FAILURE
        }
        ["version" | "--version" | "-V"] => {
            println!("kinetic {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
