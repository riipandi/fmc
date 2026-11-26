mod app;
mod args;
mod services;
mod ui;

use std::io::{IsTerminal, Write};
use std::process::ExitCode;

use args::Command;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let line = format!("fmc: {}", fmc::text::display_label(&message));
            let mut stderr = std::io::stderr();
            // Bun's console.error colors the line red on a terminal; match it.
            let _ = if stderr.is_terminal() {
                writeln!(stderr, "\x1b[0m\x1b[31m{line}\x1b[0m")
            } else {
                writeln!(stderr, "{line}")
            };
            ExitCode::FAILURE
        }
    }
}

/// Removes the index copies when `run` returns, on an error and on a panic too (`services::git::remove_copies`).
struct RemoveCopies;

impl Drop for RemoveCopies {
    fn drop(&mut self) {
        services::git::remove_copies();
    }
}

fn run() -> Result<(), String> {
    let _copies = RemoveCopies;
    match args::parse(std::env::args_os().skip(1))? {
        Command::Help => println!("{}", args::help_text()),
        Command::Version => println!("{}", args::version_text()),
        Command::Open(path) => {
            if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
                return Err("Run fmc in an interactive terminal.".into());
            }
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            let target = args::open_target(path, &cwd)?;
            // Read before the first frame, so a bad value's notice is the first thing the footer shows.
            let settings = services::settings::path(std::env::var_os("XDG_CONFIG_HOME"), std::env::var_os("HOME"))
                .map(|path| services::settings::load(&path))
                .unwrap_or_default();
            app::run(&target, settings.notices).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
