// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use chrono::{DateTime, Local, Utc};
use std::borrow::Cow;
use std::time::SystemTime;

const INVALID_CHARS: &[char] = &[':', '/', '\\'];

pub fn sanitize_game_name(name: &str) -> Cow<'_, str> {
    if name.contains(INVALID_CHARS) {
        Cow::Owned(name.replace(INVALID_CHARS, ""))
    } else {
        Cow::Borrowed(name)
    }
}

pub fn format_timestamp(time: SystemTime) -> String {
    let secs = time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().cast_signed();
    let dt = DateTime::<Utc>::from_timestamp(secs, 0).unwrap().with_timezone(&Local);

    dt.format("%B %-d %Y %H:%M").to_string()
}
