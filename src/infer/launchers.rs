// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod heroic;
mod steam;

#[cfg(all(unix, not(target_os = "macos")))]
mod lutris;

pub use heroic::Heroic;
pub use steam::Steam;

#[cfg(all(unix, not(target_os = "macos")))]
pub use lutris::Lutris;
