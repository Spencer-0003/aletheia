// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use crate::scanner::SteamScanner;
use crate::ui::app::{App, Config, DropdownOption, GameLogic, NotificationLogic, SettingsScreenLogic};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

#[expect(clippy::too_many_lines, reason = "Only 4 more than allowed")]
pub fn setup(app: &slint::Weak<App>, config: &Rc<RefCell<AletheiaConfig>>) {
    let app = app.upgrade().unwrap();
    let settings_screen_logic = app.global::<SettingsScreenLogic>();

    settings_screen_logic.on_browse({
        let app_weak = app.as_weak();

        move || {
            let app = app_weak.upgrade().unwrap();

            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().set_directory(crate::dirs::home()).pick_folder().await {
                    let settings_screen_logic = app.global::<SettingsScreenLogic>();
                    let mut cfg = settings_screen_logic.get_config();

                    cfg.save_dir = SharedString::from(folder.path().to_string_lossy().as_ref());

                    settings_screen_logic.set_config(cfg);
                }
            })
            .unwrap();
        }
    });

    settings_screen_logic.on_save_config({
        let app_weak = app.as_weak().unwrap();
        let cfg = Rc::clone(config);

        move |ui_cfg| {
            let notification_logic = app_weak.global::<NotificationLogic>();
            let settings_logic = app_weak.global::<SettingsScreenLogic>();

            let new_config = AletheiaConfig {
                custom_databases: ui_cfg.custom_databases.iter().map(Into::into).collect(),
                save_dir: (&ui_cfg.save_dir).into(),
                steam_account_id: (!ui_cfg.steam_account_id.is_empty()).then(|| (&ui_cfg.steam_account_id).into()),
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "UI is capped at 255 max backups")]
                max_backups: ui_cfg.max_backups as u8,
                #[cfg(feature = "updater")]
                check_for_updates: ui_cfg.check_for_updates
            };

            settings_logic.set_previous_save_dir(ui_cfg.save_dir);
            settings_logic.set_previous_steam_account_id(ui_cfg.steam_account_id);
            settings_logic.set_previous_max_backups(ui_cfg.max_backups);
            settings_logic.set_previous_check_for_updates(ui_cfg.check_for_updates);

            AletheiaConfig::save(&new_config);
            *cfg.borrow_mut() = new_config;

            notification_logic.invoke_show_success("SAVED_SETTINGS".into());
        }
    });

    settings_screen_logic.on_get_steam_users({
        let app_weak = app.as_weak().unwrap();

        move || {
            let settings_logic = app_weak.global::<SettingsScreenLogic>();

            if let Some(users) = SteamScanner::get_users() {
                let mut options: Vec<DropdownOption> = users
                    .into_iter()
                    .map(|(steam_id, user)| DropdownOption {
                        label: user.persona_name.into(),
                        value: SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string().into()
                    })
                    .collect();

                options.sort_by(|a, b| a.label.cmp(&b.label));
                settings_logic.set_steam_account_options(ModelRc::new(VecModel::from(options)));
            }
        }
    });

    settings_screen_logic.on_update_gamedb({
        let app_weak = app.as_weak().unwrap();

        move || {
            let notification_logic = app_weak.global::<NotificationLogic>();

            match gamedb::update() {
                Ok(true) => {
                    app_weak.global::<GameLogic>().invoke_refresh_games();
                    notification_logic.invoke_show_success("GAMEDB_UPDATED".into());
                }
                Ok(false) => notification_logic.invoke_show_info("GAMEDB_UP_TO_DATE".into()),
                Err(e) => {
                    notification_logic.invoke_show_error("GAMEDB_UPDATE_FAILED".into());
                    log::error!("Error updating GameDB: {e}");
                }
            }
        }
    });

    let config_ref = config.borrow();
    let steam_account_id = get_steam_id(&config_ref);
    let steam_account_id_str = steam_account_id.as_deref().unwrap_or_default();

    if let Some(ref id3) = steam_account_id
        && config_ref.steam_account_id.as_ref() != Some(id3)
    {
        let new_config = AletheiaConfig { steam_account_id: Some(id3.clone()), ..config_ref.clone() };

        AletheiaConfig::save(&new_config);
        *config.borrow_mut() = new_config;
    }

    settings_screen_logic.set_config(Config {
        custom_databases: ModelRc::new(config_ref.custom_databases.iter().map(Into::into).collect::<VecModel<_>>()),
        save_dir: config_ref.save_dir.to_string_lossy().as_ref().into(),
        steam_account_id: steam_account_id_str.into(),
        max_backups: config_ref.max_backups.into(),
        #[cfg(feature = "updater")]
        check_for_updates: config_ref.check_for_updates,
        #[cfg(not(feature = "updater"))]
        check_for_updates: false
    });

    settings_screen_logic.set_previous_save_dir(config_ref.save_dir.to_string_lossy().as_ref().into());
    settings_screen_logic.set_previous_steam_account_id(steam_account_id_str.into());
    settings_screen_logic.set_previous_max_backups(config_ref.max_backups.into());

    #[cfg(feature = "updater")]
    {
        settings_screen_logic.set_previous_check_for_updates(config_ref.check_for_updates);
        settings_screen_logic.set_show_update_settings(true);
    }

    settings_screen_logic.invoke_get_steam_users();
}

fn get_steam_id(config: &AletheiaConfig) -> Option<String> {
    let users = SteamScanner::get_users()?;

    if users.is_empty() {
        return None;
    }

    if let Some(ref id3) = config.steam_account_id {
        let config_user_exists = users
            .keys()
            .filter_map(|id64_str| id64_str.parse::<u64>().ok())
            .any(|id64| SteamScanner::id64_to_id3(id64).to_string() == *id3);

        if config_user_exists {
            return Some(id3.to_owned());
        }
    }

    if users.len() == 1 {
        let steam_id64 = users.keys().next()?.parse::<u64>().ok()?;
        return Some(SteamScanner::id64_to_id3(steam_id64).to_string());
    }

    None
}
