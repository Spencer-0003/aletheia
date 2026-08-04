// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};
use serde::Deserialize;
use std::fs::{File, read_dir};
use std::io::BufReader;
use std::path::Path;

pub struct XboxScanner;

#[derive(Deserialize)]
struct GameInfo {
    #[serde(rename = "@configVersion")]
    config_version: String,
    #[serde(rename = "ShellVisuals")]
    shell_visuals: ShellVisuals
}

#[derive(Deserialize)]
struct ShellVisuals {
    #[serde(rename = "@DefaultDisplayName")]
    default_display_name: String
}

impl Scanner for XboxScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];
        let xbox_games_path = Path::new("C:/XboxGames");

        if !xbox_games_path.exists() {
            return games;
        }

        let Ok(entries) = read_dir(xbox_games_path) else {
            log::error!("Cannot read XboxGames directory");
            return games;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_dir() || path.file_name().unwrap() == "GameSave" {
                continue;
            }

            let content_dir = path.join("Content");
            if !content_dir.exists() {
                log::debug!("No Content folder found in {}", path.display());
                continue;
            }

            let config_path = content_dir.join("MicrosoftGame.Config");
            if !config_path.exists() {
                log::debug!("No MicrosoftGame.Config found in {}", content_dir.display());
                continue;
            }

            let config_file = File::open(config_path).unwrap();
            let config: GameInfo = quick_xml::de::from_reader(BufReader::new(config_file)).unwrap();

            if config.config_version != "1" {
                log::debug!("Unsupported MicrosoftGame config version: {}", config.config_version);
                continue;
            }

            games.push(Game {
                name: config.shell_visuals.default_display_name,
                id: None,
                installation_dir: Some(path),
                source: "Xbox".into()
            });
        }

        games
    }
}
