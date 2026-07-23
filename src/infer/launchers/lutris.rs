// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::infer::Launcher;
use crate::scanner::Game;

pub struct Lutris;

impl Launcher for Lutris {
    fn get_game(installed_games: &[Game]) -> Option<&Game> {
        let Ok(game_name) = std::env::var("GAME_NAME") else {
            log::error!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
            return None;
        };

        installed_games.iter().find(|game| game.name == game_name && game.source == "Lutris")
    }
}
