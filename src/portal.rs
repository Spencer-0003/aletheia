// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    static GRANTS: RefCell<Vec<(PathBuf, PathBuf)>> = const { RefCell::new(Vec::new()) };
    static UNREADABLE: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
}

pub fn granted_for(host: &Path) -> Option<PathBuf> {
    todo!()
}
