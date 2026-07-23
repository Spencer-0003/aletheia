// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::list_backups;
use crate::archive::{ArchiveReader, Error as ArchiveError};
use crate::config::Config;
use crate::dirs::expand_path;
use crate::gamedb::GameDbEntry;
use crate::scanner::Game;
use crate::utils::sanitize_game_name;
use glob::Pattern;
use std::fs::create_dir_all;
use std::path::{Component, Path};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] ArchiveError),
    #[error("No backups found")]
    NoBackupsFound
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn restore_game(game: &Game, db_entry: &GameDbEntry, config: &Config) -> Result<()> {
    let backup_folder = config.save_dir.join(sanitize_game_name(&game.name).as_ref());

    let Some(backup) = list_backups(&backup_folder).into_iter().next() else {
        log::error!("No backup found for game {}", game.name);
        return Err(Error::NoBackupsFound);
    };

    restore_archive(&ArchiveReader::open(&backup.path)?, game, db_entry, config)
}

pub fn restore_archive(reader: &ArchiveReader, game: &Game, db_entry: &GameDbEntry, config: &Config) -> Result<()> {
    let steam_id = config.steam_account_id.as_deref();

    for entry in &reader.files {
        let shrunk_path = Path::new(&entry.shrunk_path);
        if !matches_known_pattern(shrunk_path, db_entry) {
            log::error!("Refusing to restore '{}' for {}: not a known save path for this game", entry.shrunk_path, game.name);
            continue;
        }

        #[cfg(unix)]
        let expanded = expand_path(shrunk_path, game.installation_dir.as_deref(), game.prefix.as_deref(), steam_id);

        #[cfg(windows)]
        let expanded = expand_path(shrunk_path, game.installation_dir.as_deref(), steam_id);

        create_dir_all(expanded.parent().unwrap()).unwrap();
        reader.extract_file(&entry.shrunk_path, &expanded)?;

        log::info!("Restored: {}", expanded.display());
    }

    Ok(())
}

fn matches_known_pattern(shrunk: &Path, entry: &GameDbEntry) -> bool {
    if !shrunk.components().all(|c| matches!(c, Component::Normal(_))) {
        return false; // Path traversal
    }

    let mut patterns: Vec<&String> = vec![];

    if let Some(ref windows_paths) = entry.files.windows {
        patterns.extend(windows_paths);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    if let Some(ref linux_paths) = entry.files.linux {
        patterns.extend(linux_paths);
    }

    #[cfg(target_os = "macos")]
    if let Some(ref mac_paths) = entry.files.mac {
        patterns.extend(mac_paths);
    }

    patterns.iter().any(|pattern| Pattern::new(pattern).is_ok_and(|p| p.matches(&shrunk.to_string_lossy())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamedb::GameFiles;

    fn requiem() -> GameDbEntry {
        GameDbEntry {
            files: GameFiles {
                windows: Some(vec!["{SteamUserData}/3764200/remote/win64_save/*".to_owned()]),
                #[cfg(all(unix, not(target_os = "macos")))]
                linux: None,
                #[cfg(target_os = "macos")]
                mac: None
            }
        }
    }

    fn mobw() -> GameDbEntry {
        GameDbEntry {
            files: GameFiles {
                windows: Some(vec!["{Documents}/KoeiTecmo/FATAL FRAME MOBW/SAVEDATA/*".to_owned()]),
                #[cfg(all(unix, not(target_os = "macos")))]
                linux: None,
                #[cfg(target_os = "macos")]
                mac: None
            }
        }
    }

    #[test]
    fn reject_path_traversal() {
        assert!(!matches_known_pattern(Path::new("{SteamUserData}/../"), &requiem()));
    }

    #[test]
    fn reject_arbitrary_file_write() {
        assert!(!matches_known_pattern(Path::new("C:/Windows/System32/config/SAM"), &requiem()));
    }

    #[test]
    fn accept_declared_save_path() {
        assert!(matches_known_pattern(Path::new("{SteamUserData}/3764200/remote/win64_save/save1.sav"), &requiem()));
    }

    #[test]
    fn reject_mismatched_game_and_path() {
        assert!(!matches_known_pattern(Path::new("{Documents}/KoeiTecmo/FATAL FRAME MOBW/SAVEDATA/slot1.dat"), &requiem()));
        assert!(!matches_known_pattern(Path::new("{SteamUserData}/3764200/remote/win64_save/save1.sav"), &mobw()));
    }
}
