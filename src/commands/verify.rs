// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Args, Command};
use crate::archive::ArchiveReader;
use crate::config::Config;
use std::fs::read_dir;
use std::path::Path;

pub struct Verify;

fn verify_archive(path: &Path) {
    match ArchiveReader::open(path) {
        Ok(r) => println!("{} ({}) is valid.", r.game, path.display()),
        Err(e) => eprintln!("Archive is invalid: {e}")
    }
}

impl Command for Verify {
    fn run(args: Args, config: &Config) {
        if let Some(file) = args.positional.first() {
            let path = Path::new(&file);
            if !path.exists() {
                eprintln!("Archive file not found: {}", path.display());
                return;
            }

            verify_archive(path);
            return;
        }

        for entry in read_dir(&config.save_dir).unwrap() {
            let path = entry.unwrap().path().join("backup.aletheia");
            if path.exists() {
                verify_archive(&path);
            }
        }
    }
}
