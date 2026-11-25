// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use steamlocate::{Library, SteamDir};

#[cfg(flatpak_build)]
use crate::portal;

const STEAM_ID64_BASE: u64 = 76561197960265728;

#[derive(Deserialize)]
struct LoginUsersFile {
    #[serde(flatten)]
    users: HashMap<String, LoginUser>
}

#[derive(Deserialize)]
pub struct LoginUser {
    #[serde(rename = "PersonaName")]
    pub persona_name: String
}

pub struct SteamScanner;

impl SteamScanner {
    pub const fn id3_to_id64(id3: u64) -> u64 {
        id3 + STEAM_ID64_BASE
    }

    pub const fn id64_to_id3(id64: u64) -> u64 {
        id64 - STEAM_ID64_BASE
    }

    pub fn get_users() -> Option<HashMap<String, LoginUser>> {
        let Ok(steam_directory) = SteamDir::locate() else {
            return None;
        };

        let Ok(login_users_file) = File::open(steam_directory.path().join("config/loginusers.vdf")) else {
            return None; // Installed Steam but never signed in
        };

        let login_users: LoginUsersFile = keyvalues_serde::from_reader(login_users_file).unwrap();

        Some(login_users.users)
    }

    #[cfg(flatpak_build)]
    fn libraries(steam: &SteamDir) -> Vec<Library> {
        let Ok(roots) = steam.library_paths() else {
            log::warn!("Failed to detect Steam libraries.");
            return vec![];
        };

        roots
            .into_iter()
            .filter_map(|root| {
                let resolved = Library::from_dir(&root)
                    .ok()
                    .or_else(|| portal::granted_for(&root).and_then(|granted| Library::from_dir(&granted).ok()));

                if resolved.is_none() {
                    log::warn!("Failed to access Steam library, possibly blocked by Flatpak.");
                }

                resolved
            })
            .collect()
    }

    #[cfg(not(flatpak_build))]
    fn libraries(steam: &SteamDir) -> Vec<Library> {
        let Ok(iter) = steam.libraries() else {
            log::warn!("Failed to detect Steam libraries.");
            return vec![];
        };

        iter.filter_map(|lib| lib.map_err(|_| log::warn!("Failed to access Steam library.")).ok()).collect()
    }
}

impl Scanner for SteamScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let Ok(steam_directory) = SteamDir::locate() else {
            return games;
        };

        for lib in SteamScanner::libraries(&steam_directory) {
            #[cfg(all(unix, not(target_os = "macos")))]
            let lib_path = lib.path();
            for app in lib.apps() {
                let game = app.unwrap();
                let install_dir = lib.resolve_app_dir(&game);
                let game_name = game.name.unwrap();

                games.push(Game {
                    name: game_name,
                    id: Some(game.app_id),
                    installation_dir: Some(install_dir),
                    #[cfg(all(unix, not(target_os = "macos")))]
                    prefix: {
                        let prefix_directory = lib_path.join("steamapps/compatdata").join(game.app_id.to_string()).join("pfx");
                        prefix_directory.exists().then_some(prefix_directory)
                    },
                    #[cfg(target_os = "macos")]
                    prefix: None,
                    source: "Steam".into()
                });
            }
        }

        for shortcut in steam_directory.shortcuts().unwrap() {
            let Ok(shortcut) = shortcut else {
                // This works around https://github.com/WilliamVenner/steamlocate-rs/issues/103
                log::warn!("Failed to parse Steam shortcut.");
                continue;
            };

            games.push(Game {
                name: shortcut.app_name,
                id: None,
                installation_dir: Some(shortcut.start_dir.into()),
                #[cfg(all(unix, not(target_os = "macos")))]
                prefix: {
                    let prefix_directory =
                        steam_directory.path().join("steamapps/compatdata").join(shortcut.app_id.to_string()).join("pfx");

                    prefix_directory.exists().then_some(prefix_directory)
                },
                #[cfg(target_os = "macos")]
                prefix: None,
                source: "Steam".into()
            });
        }

        games
    }
}
