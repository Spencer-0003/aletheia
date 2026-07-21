// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::list_backups;
use crate::archive::{ArchiveReader, Error as ArchiveError};
use crate::config::Config;
use crate::dirs::expand_path;
use crate::scanner::Game;
use crate::utils::sanitize_game_name;
use std::fs::create_dir_all;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Archive error: {0}")]
    Archive(#[from] ArchiveError),
    #[error("No backups found")]
    NoBackupsFound
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn restore_game(game: &Game, config: &Config) -> Result<()> {
    let backup_folder = config.save_dir.join(sanitize_game_name(&game.name).as_ref());

    let Some(backup) = list_backups(&backup_folder).into_iter().next() else {
        log::error!("No backup found for game {}", game.name);
        return Err(Error::NoBackupsFound);
    };

    restore_archive(&ArchiveReader::open(&backup.path)?, game, config)
}

pub fn restore_archive(reader: &ArchiveReader, game: &Game, config: &Config) -> Result<()> {
    let steam_id = config.steam_account_id.as_deref();

    for entry in &reader.files {
        #[cfg(unix)]
        let expanded = expand_path(Path::new(&entry.shrunk_path), game.installation_dir.as_deref(), game.prefix.as_deref(), steam_id);

        #[cfg(windows)]
        let expanded = expand_path(Path::new(&entry.shrunk_path), game.installation_dir.as_deref(), steam_id);

        create_dir_all(expanded.parent().unwrap()).unwrap();
        reader.extract_file(&entry.shrunk_path, &expanded)?;

        log::info!("Restored: {}", expanded.display());
    }

    Ok(())
}
