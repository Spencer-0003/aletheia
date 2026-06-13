// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod backup;
mod launchers;
mod restore;

pub use backup::backup;
pub use restore::restore;

pub(super) trait Launcher {
    fn get_game() -> Option<crate::scanner::Game>;
}
