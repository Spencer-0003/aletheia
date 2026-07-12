// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::app::{RestoreDialog, RestoreLogic};
use crate::archive::{ArchiveReader, Error as ArchiveError};
use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use crate::operations::{RestoreError, restore_archive};
use crate::utils::format_timestamp;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

pub fn run_restore_dialog(config: &AletheiaConfig, archive_path: &str) {
    let archive_path = Path::new(archive_path);
    if !archive_path.exists() {
        return;
    }

    let Ok(reader) = ArchiveReader::open(archive_path) else {
        return;
    };

    let cfg = Rc::new(RefCell::new(config.clone()));
    let restore_dialog = RestoreDialog::new().unwrap();
    let restore_logic = restore_dialog.global::<RestoreLogic>();

    restore_logic.on_cancel({
        let restore_weak = restore_dialog.as_weak().unwrap();

        move || {
            restore_weak.hide().unwrap();
        }
    });

    restore_logic.on_restore({
        let restore_weak = restore_dialog.as_weak().unwrap();
        let archive_path = archive_path.to_path_buf();

        move || {
            let restore_logic = restore_weak.global::<RestoreLogic>();
            let installed_games = gamedb::get_installed_games();
            let game_name = restore_logic.get_game_name();
            let game_name = game_name.as_str();

            let Some(game) = installed_games.iter().find(|g| g.name == game_name) else {
                restore_logic.set_error("GAME_NOT_INSTALLED".into());
                return;
            };

            if let Err(RestoreError::Archive(e)) = restore_archive(&archive_path, game, &cfg.borrow()) {
                let error_message = match e {
                    ArchiveError::ChecksumMismatch(..) | ArchiveError::FileNotFound(_) => "ARCHIVE_CORRUPTED",
                    ArchiveError::InvalidArchive | ArchiveError::Serialization(_) => "INVALID_ARCHIVE",
                    ArchiveError::Io(_) => "IO_ERROR",
                    ArchiveError::UnsupportedVersion(_) => "UNSUPPORTED_ARCHIVE_VERSION"
                };

                restore_logic.set_error(error_message.into());
            } else {
                restore_weak.hide().unwrap();
            }
        }
    });

    restore_logic.set_game_name(reader.game.into());
    restore_logic.set_timestamp(format_timestamp(SystemTime::UNIX_EPOCH + Duration::from_secs(reader.created)).into());
    slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

    restore_dialog.run().unwrap();
}
