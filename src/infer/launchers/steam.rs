// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::infer::Launcher;
use crate::scanner::Game;

pub struct Steam;

impl Launcher for Steam {
    fn get_game(installed_games: &[Game]) -> Option<&Game> {
        let Some(app_id) = std::env::var("SteamAppId").ok().and_then(|v| v.parse::<u32>().ok()) else {
            log::error!("SteamAppId environment variable not found, is the game being launched by Steam?");
            return None;
        };

        let Ok(steam_dir) = steamlocate::SteamDir::locate() else {
            log::error!("Failed to locate Steam directory.");
            return None;
        };

        let Ok(Some((game, _))) = steam_dir.find_app(app_id) else {
            log::error!("Failed to find Steam app {app_id}.");
            return None;
        };

        let game_name = game.name?;
        installed_games.iter().find(|g| g.name == game_name && g.source == "Steam")
    }
}
