// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::archive::ArchiveReader;
use std::fs::{metadata, read_dir};
use std::path::{Path, PathBuf};

mod backup;
mod restore;

pub use backup::backup_game;
pub use restore::Error as RestoreError;
pub use restore::{restore_archive, restore_game};

pub struct Backup {
    pub path: PathBuf,
    pub created: u64,
    pub size: u64
}

pub fn list_backups(dir: &Path) -> Vec<Backup> {
    let Ok(entries) = read_dir(dir) else {
        return vec![];
    };

    let mut backups: Vec<Backup> = entries
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "aletheia"))
        .filter_map(|path| {
            let (_, created, _) = ArchiveReader::read_index(&path).ok()?;
            let size = metadata(&path).ok()?.len();
            Some(Backup { path, created, size })
        })
        .collect();

    backups.sort_unstable_by_key(|b| std::cmp::Reverse(b.created));
    backups
}
