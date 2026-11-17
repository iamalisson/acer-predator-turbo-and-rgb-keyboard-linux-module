//! Predator RGB: a settings window and boot helper for the facer RGB keyboard driver.
//!
//! - `predator-rgb`                 opens the window
//! - `predator-rgb boot [CONFIG]`   applies the saved settings (used by the systemd service)

mod config;
mod device;
mod ui;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

const USAGE: &str = "usage: predator-rgb [boot [CONFIG]]";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        None => ui::run(config::default_path()).map_err(|e| e.to_string()),
        Some("boot") => boot(&args.next().map_or_else(config::default_path, PathBuf::from)),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown argument '{other}'\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("predator-rgb: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Applies the saved settings, unless the user turned off restore-at-boot.
fn boot(config_path: &Path) -> Result<(), String> {
    let config = config::load(config_path);
    if !config.apply_on_boot {
        return Ok(());
    }
    device::apply(&config).map_err(|e| format!("cannot write to the keyboard: {e}"))
}
