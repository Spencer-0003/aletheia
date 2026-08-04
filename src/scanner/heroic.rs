// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(all(unix, not(target_os = "macos")))]
use crate::dirs::home;

#[cfg(target_os = "macos")]
use crate::dirs::app_data;

#[cfg(not(target_os = "macos"))]
use crate::dirs::config;

pub struct HeroicScanner;

#[derive(Deserialize)]
struct HeroicGOGGame {
    #[serde(rename = "appName")]
    app_id: String,
    install_path: PathBuf,
    #[cfg(unix)]
    platform: String
}

#[derive(Deserialize)]
struct HeroicGOGProduct {
    name: String
}

#[derive(Deserialize)]
struct HeroicGOGGameManifest {
    products: Vec<HeroicGOGProduct>
}

#[derive(Deserialize)]
struct HeroicGOGManifest {
    installed: Vec<HeroicGOGGame>
}

#[derive(Deserialize)]
struct HeroicGamesSideloadLibrary {
    games: Vec<HeroicSideloadGame>
}

#[derive(Deserialize)]
struct HeroicSideloadGame {
    #[cfg(unix)]
    app_name: String,
    title: String,
    #[cfg(unix)]
    install: HeroicSideloadInstall,
    folder_name: PathBuf
}

#[cfg(unix)]
#[derive(Deserialize)]
struct HeroicSideloadInstall {
    platform: String
}

impl HeroicScanner {
    fn get_game_name(heroic_path: &Path, game: &HeroicGOGGame) -> Option<String> {
        let manifest_path = heroic_path.join("gogdlConfig/heroic_gogdl/manifests").join(&game.app_id);

        if manifest_path.exists() {
            let Ok(manifest) = serde_json::from_reader::<File, HeroicGOGGameManifest>(File::open(manifest_path).unwrap()) else {
                return None;
            };

            return manifest.products.into_iter().next().map(|p| p.name);
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Heroic doesn't store manifests for Linux games
            game.install_path.file_name().and_then(|n| n.to_str()).map(ToOwned::to_owned)
        }

        #[cfg(any(windows, target_os = "macos"))]
        None
    }

    #[cfg(unix)]
    fn get_wine_prefix(heroic_path: &Path, app_name: &str, platform: &str) -> Option<PathBuf> {
        if platform != "Windows" && platform != "windows" {
            // GOG uses "windows" while custom games use "Windows" for some reason
            return None;
        }

        let game_config = heroic_path.join("GamesConfig").join(app_name).with_extension("json");
        if !game_config.exists() {
            return None;
        }

        let Ok(game_config) = serde_json::from_reader::<File, serde_json::Value>(File::open(game_config).unwrap()) else {
            return None;
        };

        game_config.get(app_name).and_then(|c| c.get("winePrefix")).and_then(|p| p.as_str()).map(Into::into)
    }
}

impl Scanner for HeroicScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        #[cfg(all(unix, not(target_os = "macos")))]
        let heroic_path = [config().join("heroic"), home().join(".var/app/com.heroicgameslauncher.hgl/config/heroic")]
            .into_iter()
            .find(|p| p.exists());

        #[cfg(target_os = "macos")]
        let heroic_path = {
            let path = app_data().join("heroic");
            path.exists().then_some(path)
        };

        #[cfg(windows)]
        let heroic_path = {
            let path = config().join("heroic");
            path.exists().then_some(path)
        };

        let Some(heroic_path) = heroic_path else {
            return games;
        };

        let gog_manifest = heroic_path.join("gog_store/installed.json");

        if !gog_manifest.exists() {
            return games;
        }

        let Ok(gog_manifest) = serde_json::from_reader::<File, HeroicGOGManifest>(File::open(gog_manifest).unwrap()) else {
            log::error!("Failed to parse GOG manifest.");
            return games;
        };

        for game in gog_manifest.installed {
            let Some(game_name) = Self::get_game_name(&heroic_path, &game) else {
                continue;
            };

            games.push(Game {
                name: game_name,
                id: game.app_id.parse().ok(),
                installation_dir: Some(game.install_path),
                #[cfg(unix)]
                prefix: Self::get_wine_prefix(&heroic_path, &game.app_id, &game.platform),
                source: "Heroic".into()
            });
        }

        let sideload_path = heroic_path.join("sideload_apps/library.json");
        if !sideload_path.exists() {
            return games;
        }

        let Ok(sideload_library) = serde_json::from_reader::<File, HeroicGamesSideloadLibrary>(File::open(&sideload_path).unwrap())
        else {
            log::error!("Failed to parse sideload library.");
            return games;
        };

        for game in sideload_library.games {
            games.push(Game {
                name: game.title,
                id: None,
                installation_dir: Some(game.folder_name),
                #[cfg(unix)]
                prefix: Self::get_wine_prefix(&heroic_path, &game.app_name, &game.install.platform),
                source: "Heroic".into()
            });
        }

        games
    }
}
