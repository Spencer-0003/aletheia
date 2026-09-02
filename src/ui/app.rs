// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use super::handlers::{games, settings};
use crate::config::Config as AletheiaConfig;
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(all(feature = "updater", not(debug_assertions)))]
use crate::updater;

pub fn run(config: &AletheiaConfig) {
    #[cfg(all(feature = "updater", not(debug_assertions)))]
    if config.check_for_updates
        && let Ok(updater::UpdateStatus::Available(release)) = updater::check()
    {
        let updater_window = Updater::new().unwrap();
        let updater_window_weak = updater_window.as_weak();
        let updater_logic = updater_window.global::<UpdaterLogic>();

        slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

        updater_logic.set_current_version(env!("CARGO_PKG_VERSION").into());
        updater_logic.set_new_version(release.tag_name.into());
        updater_logic.set_changelog(release.body.into());
        updater_logic.set_release_url(release.url.into());

        updater_window.window().on_close_requested(move || {
            updater_window_weak.upgrade().unwrap().global::<UpdaterLogic>().set_exit_requested(true);
            slint::CloseRequestResponse::HideWindow
        });

        updater_window.run().unwrap();

        if updater_logic.get_downloading() || updater_logic.get_exit_requested() {
            return;
        }
    }

    let app = App::new().unwrap();
    let app_weak = app.as_weak();
    let cfg = Rc::new(RefCell::new(config.clone()));

    slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

    setup_app_handlers(&app);
    games::setup(&app_weak, &cfg);
    settings::setup(&app_weak, &cfg);

    #[cfg(all(unix, not(target_os = "macos")))]
    if std::env::var("SteamDeck").as_deref() == Ok("1") {
        // Without this, the UI on the Steam Deck is extremely blurry
        app.window().set_fullscreen(true);
    }

    app.run().unwrap();
}

fn setup_app_handlers(app: &App) {
    let app_logic = app.global::<AppLogic>();

    app_logic.set_version(env!("CARGO_PKG_VERSION").into());
}
