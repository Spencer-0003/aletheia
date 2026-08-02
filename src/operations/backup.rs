// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::list_backups;
use crate::archive::{ArchiveReader, ArchiveWriter};
use crate::config::Config;
use crate::dirs::{expand_path, shrink_path};
use crate::file::hash_file;
use crate::gamedb::GameDbEntry;
use crate::scanner::Game;
use crate::utils;
use glob::glob;
use std::fs::{create_dir_all, remove_file};
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create backup directory: {0}")]
    DirectoryCreation(#[from] std::io::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn backup_game(game: &Game, config: &Config, entry: &GameDbEntry) -> Result<bool> {
    let steam_id = config.steam_account_id.as_deref();
    let backup_folder = config.save_dir.join(utils::sanitize_game_name(&game.name).as_ref());
    let existing_backups = list_backups(&backup_folder);
    let previous_files =
        existing_backups.first().and_then(|latest| ArchiveReader::read_index(&latest.path).ok()).map(|(_, _, files)| files);

    let mut changed = false;
    let mut paths = vec![];

    #[cfg(windows)]
    if let Some(ref windows_paths) = entry.files.windows {
        paths.extend(windows_paths);
    }

    #[cfg(unix)]
    if game.prefix.is_some()
        && let Some(ref windows_paths) = entry.files.windows
    {
        paths.extend(windows_paths);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    if let Some(ref linux_paths) = entry.files.linux {
        paths.extend(linux_paths);
    }

    #[cfg(target_os = "macos")]
    if let Some(ref mac_paths) = entry.files.mac {
        paths.extend(mac_paths);
    }

    let mut files = vec![];

    for path in paths {
        #[cfg(unix)]
        let expanded = expand_path(Path::new(path), game.installation_dir.as_deref(), game.prefix.as_deref(), steam_id);

        #[cfg(windows)]
        let expanded = expand_path(Path::new(path), game.installation_dir.as_deref(), steam_id);

        let found_paths = glob(&expanded.to_string_lossy()).unwrap();

        for file in found_paths {
            let file = file.unwrap();

            if file.is_dir() {
                log::warn!("Found {} while backing up {}. Glob patterns should match files only.", file.display(), game.name);
                continue;
            }

            if file.file_name().unwrap() == "steam_autocloud.vdf" {
                continue;
            }

            #[cfg(target_os = "macos")]
            if file.file_name().unwrap() == ".DS_Store" {
                continue;
            }

            files.push(file);
        }
    }

    if files.is_empty() {
        return Ok(false);
    }

    create_dir_all(&backup_folder)?;

    let archive_path =
        backup_folder.join(format!("{}.aletheia", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()));
    let mut writer = ArchiveWriter::new(game.name.clone(), &archive_path);

    for file in files {
        #[cfg(unix)]
        let shrunk_file = shrink_path(file.as_path(), game.installation_dir.as_deref(), game.prefix.as_deref(), steam_id);

        #[cfg(windows)]
        let shrunk_file = shrink_path(file.as_path(), game.installation_dir.as_deref(), steam_id);

        let shrunk_file_path = shrunk_file.to_string_lossy();
        let file_hash = hash_file(&file);

        changed |= previous_files
            .as_ref()
            .and_then(|entries| entries.iter().find(|e| e.shrunk_path == shrunk_file_path))
            .is_none_or(|existing| existing.checksum != file_hash);
        writer.add_file(&shrunk_file_path, &file, file_hash);
    }

    if !changed {
        return Ok(false);
    }

    writer.finalize().unwrap();
    prune_backups(&backup_folder, config.max_backups);

    Ok(true)
}

fn prune_backups(backup_folder: &Path, max_backups: u8) {
    if max_backups == 0 {
        return;
    }

    let backups = list_backups(backup_folder);
    for stale in backups.into_iter().skip(max_backups as usize) {
        if let Err(e) = remove_file(&stale.path) {
            log::warn!("Failed to prune old backup {}: {e}", stale.path.display());
        }
    }
}
