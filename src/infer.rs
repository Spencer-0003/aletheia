// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod backup;
mod launchers;
mod restore;

use crate::scanner::Game;

pub use backup::backup;
pub use restore::restore;

pub(super) trait Launcher {
    fn get_game(installed_games: &[Game]) -> Option<&Game>;
}
