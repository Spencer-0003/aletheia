// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::archive::Error as ArchiveError;
use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use crate::operations::{RestoreError, backup_game, restore_game};
use crate::ui::app::{App, GameLogic, GamesScreenLogic, NotificationLogic, UiGame};
use crate::utils;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

#[expect(clippy::too_many_lines, reason = "This is as simple as it's going to get")]
pub fn setup(app: &slint::Weak<App>, config: &Rc<RefCell<AletheiaConfig>>) {
    let app = app.upgrade().unwrap();
    let game_logic = app.global::<GameLogic>();
    let games_screen_logic = app.global::<GamesScreenLogic>();
    let save_dir = config.borrow().save_dir.clone();

    game_logic.on_refresh_games({
        let app_weak = app.as_weak().unwrap();

        move || {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let selected_games = games_screen_logic.get_selected_games();
            let select_all = selected_games.row_count() == 0;

            let mut games = gamedb::get_installed_games();
            games.sort_by_key(|a| a.name.to_lowercase());

            let ui_games: Vec<UiGame> = games
                .into_iter()
                .map(|g| {
                    let backup_path = save_dir.join(utils::sanitize_game_name(&g.name).as_ref());
                    let selected = select_all || selected_games.iter().any(|selected| selected.name.as_str() == g.name);

                    UiGame {
                        name: g.name.into(),
                        backup_size: if backup_path.exists() {
                            format_size(crate::dirs::get_size(&backup_path)).into()
                        } else {
                            "0B".into()
                        },
                        source: g.source.into(),
                        selected
                    }
                })
                .collect();

            let ui_games_model = ModelRc::new(VecModel::from(ui_games));
            let ui_selected_games: Vec<UiGame> = ui_games_model.iter().filter(|game| game.selected).collect();

            // In a perfect world, Slint would have a way to filter in their markdown language so I could avoid this
            app_weak.global::<GameLogic>().set_games(ui_games_model.clone());
            games_screen_logic.set_filtered_games(ui_games_model.clone());
            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(ui_selected_games)));
            games_screen_logic
                .set_all_filtered_selected(ui_games_model.row_count() > 0 && ui_games_model.iter().all(|game| game.selected));
        }
    });

    games_screen_logic.on_filter({
        let app_weak = app.as_weak().unwrap();

        move |query| {
            let query_lower = query.to_lowercase();
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let games = app_weak.global::<GameLogic>().get_games();
            let selected_games = games_screen_logic.get_selected_games();
            let filtered_games: Vec<UiGame> = games
                .iter()
                .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query_lower))
                .map(|mut g| {
                    g.selected = selected_games.iter().any(|selected| selected.name == g.name);
                    g
                })
                .collect();

            let all_filtered_selected = !filtered_games.is_empty() && filtered_games.iter().all(|g| g.selected);

            games_screen_logic.set_filtered_games(ModelRc::new(VecModel::from(filtered_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_select_all({
        let app_weak = app.as_weak().unwrap();

        move |enabled| {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let filtered_games_model = games_screen_logic.get_filtered_games();
            let updated_games: Vec<UiGame> = filtered_games_model
                .iter()
                .map(|mut g| {
                    g.selected = enabled;
                    g
                })
                .collect();
            let mut selected_games: Vec<UiGame> = games_screen_logic.get_selected_games().iter().collect();

            selected_games.retain(|g| !updated_games.iter().any(|updated| updated.name == g.name));
            if enabled {
                selected_games.extend(updated_games.iter().cloned());
            }

            let all_filtered_selected = enabled && !updated_games.is_empty();
            games_screen_logic.set_filtered_games(ModelRc::new(VecModel::from(updated_games)));
            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(selected_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_select_game({
        let app_weak = app.as_weak().unwrap();

        move |game| {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let selected_games_model = games_screen_logic.get_selected_games();
            let mut selected_games: Vec<UiGame> = selected_games_model.iter().collect();

            if let Some(index) = selected_games.iter().position(|g| g.name == game.name) {
                selected_games.swap_remove(index);
            } else {
                selected_games.push(game);
            }

            let filtered_games = games_screen_logic.get_filtered_games();

            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(selected_games)));
            games_screen_logic.set_all_filtered_selected(filtered_games.iter().all(|g| g.selected));
        }
    });

    games_screen_logic.on_perform_operation({
        let app_weak = app.as_weak().unwrap();
        let cfg = Rc::clone(config);

        move |action| {
            let cfg = cfg.as_ref().borrow();
            let notification_logic = app_weak.global::<NotificationLogic>();
            let selected_games = app_weak.global::<GamesScreenLogic>().get_selected_games();
            let installed_games = gamedb::get_installed_games();

            if cfg.steam_account_id.is_none() && selected_games.iter().any(|g| g.source == "Steam") {
                notification_logic.invoke_show_warning("STEAM_ACCOUNT_MISSING".into());
                return;
            }

            if action == "backup" {
                let game_db = gamedb::parse();
                let mut backed_up = 0;

                for ui_game in selected_games.iter() {
                    let game = installed_games.iter().find(|g| *g.name == *ui_game.name).unwrap();
                    if let Err(e) = backup_game(game, &cfg, &game_db[&game.name]) {
                        log::error!("Failed to backup {}.\n{e}", game.name);
                    } else {
                        log::info!("Successfully backed up {}.", game.name);
                        backed_up += 1;
                    }
                }

                app_weak.global::<GameLogic>().invoke_refresh_games();
                notification_logic.invoke_show_success(format!("Backed up {backed_up} games").into());
            } else {
                if !cfg.save_dir.exists() {
                    notification_logic.invoke_show_error("BACKUP_DIRECTORY_MISSING".into());
                    return;
                }

                let mut restored = 0;
                for ui_game in selected_games.iter() {
                    let Some(game) = installed_games.iter().find(|g| *g.name == *ui_game.name) else {
                        log::error!("Failed to restore {}: Uninstalled while Aletheia was still open", ui_game.name);
                        notification_logic.invoke_show_error("GAME_NOT_INSTALLED".into());
                        continue;
                    };

                    if let Err(e) = restore_game(game, &cfg) {
                        log::error!("Failed to restore {}: {e}", game.name);

                        let error_message = if let RestoreError::Archive(ae) = &e {
                            match ae {
                                ArchiveError::ChecksumMismatch(..) | ArchiveError::FileNotFound(_) => "ARCHIVE_CORRUPTED",
                                ArchiveError::InvalidArchive | ArchiveError::Serialization(_) => "INVALID_ARCHIVE",
                                ArchiveError::Io(_) => "IO_ERROR",
                                ArchiveError::UnsupportedVersion(_) => "UNSUPPORTED_ARCHIVE_VERSION"
                            }
                        } else {
                            "NO_BACKUPS_FOUND"
                        };

                        notification_logic.invoke_show_error(error_message.into());
                    } else {
                        log::info!("Successfully restored {}", game.name);
                        restored += 1;
                    }
                }

                if restored > 0 {
                    notification_logic.invoke_show_success(format!("Restored {restored} games").into());
                }
            }
        }
    });

    game_logic.invoke_refresh_games();
}

#[expect(clippy::cast_precision_loss, reason = "Only used for UI")]
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1048576;
    const GB: u64 = 1073741824;

    if size < KB {
        format!("{size}B")
    } else if size < MB {
        format!("{:.1}KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.1}MB", size as f64 / MB as f64)
    } else {
        format!("{:.2}GB", size as f64 / GB as f64)
    }
}
