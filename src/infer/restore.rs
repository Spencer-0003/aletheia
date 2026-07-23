// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::Launcher;
use super::launchers::{Heroic, Steam};
use crate::config::Config;
use crate::gamedb;
use crate::operations::restore_game;
use crate::scanner::Game;

#[cfg(all(unix, not(target_os = "macos")))]
use super::launchers::Lutris;

pub fn restore(launcher: &str, config: &Config) {
    let get_game: fn(&[Game]) -> Option<&Game> = match launcher.to_lowercase().as_str() {
        "heroic" => Heroic::get_game,
        #[cfg(all(unix, not(target_os = "macos")))]
        "lutris" => Lutris::get_game,
        "steam" => Steam::get_game,
        _ => {
            log::warn!("Restore was ran with infer using an unsupported launcher.");
            return;
        }
    };

    let (game_db, installed_games) = gamedb::get_installed_games_with_db();

    if let Some(game) = get_game(&installed_games) {
        if let Err(e) = restore_game(game, &game_db[&game.name], config) {
            log::error!("Failed to restore {}: {}", game.name, e);
        } else {
            log::info!("Restored {}.", game.name);
        }
    }
}
