// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::app::{DropdownOption, FirstTimeSetup, SetupLogic};
use crate::config::Config;
use crate::scanner::SteamScanner;
use slint::{ComponentHandle, ModelRc, VecModel};

pub fn run_first_time_setup() {
    let default_config = Config::default();
    let first_time_setup = FirstTimeSetup::new().unwrap();
    let setup_logic = first_time_setup.global::<SetupLogic>();

    setup_logic.set_backup_path(default_config.save_dir.to_string_lossy().as_ref().into());
    setup_logic.set_max_backups(default_config.max_backups.into());

    setup_logic.on_browse({
        let weak = first_time_setup.as_weak();

        move || {
            let first_time_setup = weak.upgrade().unwrap();

            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new().set_directory(crate::dirs::home()).pick_folder().await {
                    first_time_setup.global::<SetupLogic>().set_backup_path(folder.path().to_string_lossy().as_ref().into());
                }
            })
            .unwrap();
        }
    });

    setup_logic.on_continue({
        let weak = first_time_setup.as_weak();

        move || {
            let first_time_setup = weak.upgrade().unwrap();
            let setup_logic = first_time_setup.global::<SetupLogic>();
            let steam_account_id = setup_logic.get_steam_account_id();

            Config::save(&Config {
                custom_databases: vec![],
                save_dir: (&setup_logic.get_backup_path()).into(),
                steam_account_id: (!steam_account_id.is_empty()).then(|| (&steam_account_id).into()),
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "UI is capped at 255 max backups")]
                max_backups: setup_logic.get_max_backups() as u8,
                #[cfg(feature = "updater")]
                check_for_updates: default_config.check_for_updates
            });

            first_time_setup.hide().unwrap();
        }
    });

    if let Some(users) = SteamScanner::get_users() {
        if users.len() > 1 {
            let mut options: Vec<DropdownOption> = users
                .into_iter()
                .map(|(steam_id, user)| DropdownOption {
                    label: user.persona_name.into(),
                    value: SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string().into()
                })
                .collect();

            options.sort_by(|a, b| a.label.cmp(&b.label));
            setup_logic.set_steam_account_options(ModelRc::new(VecModel::from(options)));
        } else {
            let (steam_id, user) = users.into_iter().next().unwrap();
            let steam_account_id = SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string();

            setup_logic.set_steam_account_id(steam_account_id.clone().into());
            setup_logic.set_steam_account_options(ModelRc::new(VecModel::from(vec![DropdownOption {
                label: user.persona_name.into(),
                value: steam_account_id.into()
            }])));
        }
    }

    slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();
    first_time_setup.run().unwrap();
}
