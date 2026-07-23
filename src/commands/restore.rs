// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Args, Command};
use crate::archive::ArchiveReader;
use crate::cli_helpers::ensure_steam_account_selected;
use crate::config::Config;
use crate::gamedb;
use crate::infer;
use crate::operations::{restore_archive, restore_game};
use std::path::Path;

pub struct Restore;

impl Command for Restore {
    fn run(args: Args, config: &Config) {
        let (game_db, installed_games) = gamedb::get_installed_games_with_db();

        if config.steam_account_id.is_none() && installed_games.iter().any(|g| g.source == "Steam") {
            ensure_steam_account_selected(config);
        }

        if let Some(path_str) = args.positional.first() {
            let archive_path = Path::new(&path_str);
            if !archive_path.exists() {
                eprintln!("Archive file not found: {}", archive_path.display());
                return;
            }

            let reader = match ArchiveReader::open(archive_path) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to open archive: {e}");
                    return;
                }
            };

            let Some(game) = installed_games.iter().find(|g| g.name == reader.game) else {
                eprintln!("{} is not installed.", reader.game);
                return;
            };

            println!("Restoring {}", reader.game);

            if let Err(e) = restore_archive(&reader, game, &game_db[&game.name], config) {
                eprintln!("Failed to restore {}: {e}", reader.game);
            } else {
                println!("Restored {}.", reader.game);
            }

            return;
        }

        if !config.save_dir.exists() {
            eprintln!("Backup directory doesn't exist.");
            return;
        }

        if let Some(launcher) = args.get_flag_value("infer") {
            infer::restore(launcher, config);
            return;
        }

        for game in &installed_games {
            if !args.positional.is_empty() && !args.positional.contains(&game.name) {
                continue;
            }

            if let Err(e) = restore_game(game, &game_db[&game.name], config) {
                eprintln!("Failed to restore {}: {e}", game.name);
            } else {
                println!("Restored {}.", game.name);
            }
        }
    }
}
