// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs;
use std::path::PathBuf;

#[cfg(not(target_os = "macos"))]
use std::fs::{File, create_dir_all};

#[cfg(target_os = "macos")]
use std::fs::File;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    pub custom_databases: Vec<String>,
    pub save_dir: PathBuf,
    pub steam_account_id: Option<String>,
    pub max_backups: u8,
    #[cfg(feature = "updater")]
    pub check_for_updates: bool
}

impl Config {
    #[cfg(target_os = "macos")]
    fn get_dir() -> PathBuf {
        dirs::config()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn get_dir() -> PathBuf {
        dirs::config().join("aletheia")
    }

    #[cfg(windows)]
    fn get_dir() -> PathBuf {
        dirs::app_data().join("aletheia")
    }

    #[cfg(target_os = "macos")]
    fn get_save_dir() -> PathBuf {
        dirs::app_data().join("moe.spencer.aletheia")
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn get_save_dir() -> PathBuf {
        dirs::app_data().join("aletheia")
    }

    #[cfg(windows)]
    fn get_save_dir() -> PathBuf {
        Self::get_dir().join("saves")
    }

    #[cfg(not(target_os = "macos"))]
    pub fn load() -> Option<Self> {
        let dir = Self::get_dir();
        let config_path = dir.join("config.json");

        if config_path.exists() {
            let config_file = File::open(&config_path).expect("Failed to read config file.");
            let mut cfg: Self = serde_json::from_reader(&config_file).expect("Failed to parse config file.");

            if !cfg.save_dir.exists() {
                log::warn!("Save directory does not exist, resetting.");

                cfg.save_dir = Self::get_save_dir();
                Self::save(&cfg);
            }

            return Some(cfg);
        }

        None
    }

    #[cfg(target_os = "macos")]
    pub fn load() -> Option<Self> {
        let config_path = Self::get_dir().join("moe.spencer.aletheia.plist");

        if config_path.exists() {
            let config_file = File::open(&config_path).expect("Failed to read config file.");
            let mut cfg: Self = plist::from_reader(&config_file).expect("Failed to parse config file.");

            if !cfg.save_dir.exists() {
                log::warn!("Save directory does not exist, resetting.");

                cfg.save_dir = Self::get_save_dir();
                Self::save(&cfg);
            }

            return Some(cfg);
        }

        None
    }

    #[cfg(not(target_os = "macos"))]
    pub fn save(cfg: &Self) {
        let dir = Self::get_dir();
        let config_path = dir.join("config.json");

        create_dir_all(&dir).unwrap();
        serde_json::to_writer_pretty(File::create(&config_path).unwrap(), &cfg).unwrap();
    }

    #[cfg(target_os = "macos")]
    pub fn save(cfg: &Self) {
        let config_path = Self::get_dir().join("moe.spencer.aletheia.plist");

        plist::to_writer_xml(File::create(&config_path).unwrap(), &cfg).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            custom_databases: vec![],
            save_dir: Self::get_save_dir(),
            steam_account_id: None,
            max_backups: 3,
            #[cfg(feature = "updater")]
            check_for_updates: true
        }
    }
}
