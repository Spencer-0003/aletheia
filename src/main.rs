// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#![warn(clippy::pedantic)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::option_if_let_else)]
#![deny(clippy::allow_attributes_without_reason)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::get_unwrap)]
#![deny(clippy::str_to_string)]
#![allow(clippy::unreadable_literal, reason = "'Readable' literals are ugly")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod archive;
mod cli_helpers;
mod commands;
mod config;
mod dirs;
mod file;
mod gamedb;
mod infer;
mod operations;
mod scanner;
mod ui;
mod utils;

#[cfg(all(feature = "updater", not(debug_assertions)))]
mod updater;

use commands::{Args, Command};

fn main() {
    env_logger::init();

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let flatpak = cfg!(flatpak_build);
        log::info!(
            "Aletheia v{} (Linux) (Flatpak: {}, AppImage: {})",
            env!("CARGO_PKG_VERSION"),
            flatpak,
            !flatpak && std::env::var("APPIMAGE").is_ok()
        );
    }

    #[cfg(target_os = "macos")]
    log::info!("Aletheia v{} (MacOS)", env!("CARGO_PKG_VERSION"));

    #[cfg(windows)]
    log::info!("Aletheia v{} (Windows)", env!("CARGO_PKG_VERSION"));

    let config = config::Config::load();
    let mut args = std::env::args().skip(1);

    if let Some(cmd) = args.next() {
        let cfg = config.unwrap_or_else(|| {
            let default = config::Config::default();
            config::Config::save(&default);
            default
        });

        if cmd.ends_with(".aletheia") {
            ui::run_restore_dialog(&cfg, &cmd);
            return;
        }

        let args = Args::parse(args);
        match cmd.as_str() {
            "backup" => commands::Backup::run(args, &cfg),
            "restore" => commands::Restore::run(args, &cfg),
            #[cfg(all(feature = "updater", not(debug_assertions)))]
            "update" => commands::Update::run(args, &cfg),
            "update_gamedb" => commands::UpdateGameDb::run(args, &cfg),
            "update_custom_gamedbs" => commands::UpdateCustom::run(args, &cfg),
            "verify" => commands::Verify::run(args, &cfg),
            _ => eprintln!("Command not found.")
        }
    } else if let Some(ref cfg) = config {
        ui::run(cfg);
    } else {
        ui::run_first_time_setup();
    }
}
