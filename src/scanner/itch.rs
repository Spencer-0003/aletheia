// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};

#[cfg(not(target_os = "macos"))]
use crate::dirs::config;

#[cfg(all(unix, not(target_os = "macos")))]
use crate::dirs::home;

#[cfg(target_os = "macos")]
use crate::dirs::app_data;

pub struct ItchScanner;

impl Scanner for ItchScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        #[cfg(all(unix, not(target_os = "macos")))]
        let butler_db_path = {
            let native = config().join("itch/db/butler.db");
            let flatpak = home().join(".var/app/io.itch.itch/config/itch/db/butler.db");
            if native.exists() {
                native
            } else if flatpak.exists() {
                flatpak
            } else {
                return games;
            }
        };

        #[cfg(windows)]
        let butler_db_path = config().join("itch/db/butler.db");

        #[cfg(target_os = "macos")]
        let butler_db_path = app_data().join("itch/db/butler.db");

        #[cfg(any(windows, target_os = "macos"))]
        if !butler_db_path.exists() {
            return games;
        }

        let conn = rusqlite::Connection::open(butler_db_path).unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT g.title, d.install_folder
                FROM downloads d
                JOIN games g ON d.game_id = g.id"
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                let title: String = row.get(0)?;
                let install_folder: String = row.get(1)?;
                Ok((title, install_folder))
            })
            .unwrap();

        for row in rows {
            let (title, install_folder) = row.unwrap();
            games.push(Game {
                name: title,
                id: None,
                installation_dir: Some(install_folder.into()),
                source: "itch.io".to_owned(),
                #[cfg(unix)]
                prefix: None // Despite being able to install Windows games on Unix, itch.io does not offer any Wine/Proton integration
            });
        }

        games
    }
}
