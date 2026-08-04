// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};
use crate::dirs::{app_data, config, home};
use std::path::PathBuf;

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let lutris_config_dir_deprecated = config().join("lutris/games");
        let lutris_config_dir_new = app_data().join("lutris/games");
        let lutris_config_dir_flatpak = home().join(".var/app/net.lutris.Lutris/data/lutris/games");

        let lutris_db_path = app_data().join("lutris/pga.db");
        let lutris_db_path_flatpak = home().join(".var/app/net.lutris.Lutris/data/lutris/pga.db");

        let lutris_config_dir = if lutris_config_dir_deprecated.exists() {
            lutris_config_dir_deprecated
        } else if lutris_config_dir_new.exists() {
            lutris_config_dir_new
        } else if lutris_config_dir_flatpak.exists() {
            lutris_config_dir_flatpak
        } else {
            return games;
        };

        let con = if lutris_db_path.exists() {
            rusqlite::Connection::open(lutris_db_path).unwrap()
        } else if lutris_db_path_flatpak.exists() {
            rusqlite::Connection::open(lutris_db_path_flatpak).unwrap()
        } else {
            return games;
        };

        let mut stmt = con.prepare("SELECT name, directory, platform, configpath FROM games").unwrap();
        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let path: String = row.get(1)?;
                let platform: String = row.get(2)?;
                let config_path: String = row.get(3)?;
                Ok((name, PathBuf::from(path), platform, lutris_config_dir.join(&config_path).with_extension("yml")))
            })
            .unwrap();

        for row in rows {
            let (name, dir, platform, config_file) = row.unwrap();

            if !dir.as_os_str().is_empty() && !dir.exists() || !config_file.exists() {
                // Lutris leaves some directories empty (not sure why) and doesn't seem to remove the directory from the database after the game is uninstalled
                continue;
            }

            if platform == "Windows" {
                let game_config: serde_yaml::Value = serde_yaml::from_reader(std::fs::File::open(config_file).unwrap()).unwrap();
                let installation_dir = game_config
                    .get("game")
                    .and_then(|g| g.get("exe"))
                    .and_then(|e| e.as_str())
                    .map(PathBuf::from)
                    .map(|p| p.parent().unwrap().to_path_buf());

                games.push(Game { name, id: None, installation_dir, prefix: Some(dir), source: "Lutris".into() });
            } else {
                let installation_dir = (!dir.as_os_str().is_empty()).then_some(dir);
                // Flatpak games have their installation directory set as "" instead of None
                games.push(Game { name, id: None, installation_dir, prefix: None, source: "Lutris".into() });
            }
        }

        games
    }
}
