// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only
use std::path::PathBuf;

#[cfg(all(unix, not(target_os = "macos")))]
mod lutris;
mod steam;

#[cfg(any(windows, target_os = "macos"))]
mod gog;

#[cfg(windows)]
mod xbox;

mod heroic;
mod itch;

#[cfg(all(unix, not(target_os = "macos")))]
pub use lutris::LutrisScanner;
pub use steam::SteamScanner;

#[cfg(any(windows, target_os = "macos"))]
pub use gog::GOGScanner;

#[cfg(windows)]
pub use xbox::XboxScanner;

pub use heroic::HeroicScanner;
pub use itch::ItchScanner;

#[derive(Clone)]
pub struct Game {
    pub name: String,
    pub installation_dir: Option<PathBuf>,
    #[cfg(unix)]
    pub prefix: Option<PathBuf>,
    pub source: String
}

pub trait Scanner {
    fn get_games() -> Vec<Game>;
}
