// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::infer::Launcher;
use crate::infer::launchers::{Heroic, Steam};
use crate::operations::restore_game;

#[cfg(all(unix, not(target_os = "macos")))]
use crate::infer::launchers::Lutris;

pub fn restore(launcher: &str, config: &Config) {
    let game = match launcher.to_lowercase().as_str() {
        "heroic" => Heroic::get_game(),
        #[cfg(all(unix, not(target_os = "macos")))]
        "lutris" => Lutris::get_game(),
        "steam" => Steam::get_game(),
        _ => {
            log::warn!("Restore was ran with infer using an unsupported launcher.");
            return;
        }
    };

    if let Some(game) = game {
        if let Err(e) = restore_game(&game, config) {
            log::error!("Failed to restore {}: {}", game.name, e);
        } else {
            log::info!("Restored {}.", game.name);
        }
    }
}
