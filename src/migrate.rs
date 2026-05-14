// SPDX-FileCopyrightText: 2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only
// TODO: Remove module after releasing 2.0

use crate::archive::ArchiveWriter;
use crate::config::Config;
use crate::gamedb::Manifest;
use std::fs::{File, read_dir, remove_file};
use std::path::Path;

pub fn run(config: &Config) {
    if !config.save_dir.exists() {
        return;
    }

    for entry in read_dir(&config.save_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("aletheia_manifest.yaml");
        let archive_path = path.join("backup.aletheia");

        if archive_path.exists() || !manifest_path.exists() {
            continue;
        }

        let manifest_file = File::open(&manifest_path).unwrap();
        let manifest: Manifest = serde_yaml::from_reader(manifest_file).unwrap();

        log::info!("Migrating {} to archive format...", manifest.name);

        let mut writer = ArchiveWriter::new(manifest.name.clone(), &archive_path);

        let mut missing = false;
        for file in &manifest.files {
            let file_path = path.join(Path::new(&file.path).file_name().unwrap());

            if !file_path.exists() {
                missing = true;
                log::warn!("Failed to migrate {}, file {} missing.", manifest.name, file.path);
                continue;
            }

            writer.add_file(&file.path, &file_path, file.hash.clone());
        }

        if missing {
            continue;
        }

        if let Err(e) = writer.finalize() {
            log::error!("Failed to create archive for {}: {e}", manifest.name);
            continue;
        }

        remove_file(&manifest_path).unwrap();
        for file in &manifest.files {
            let file_path = path.join(Path::new(&file.path).file_name().unwrap());
            if file_path.exists() {
                remove_file(&file_path).unwrap();
            }
        }

        log::info!("Successfully migrated {}.", manifest.name);
    }
}
