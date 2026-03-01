// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::cache;
use crate::scanner::{Game, Scanner};
use crate::scanner::{HeroicScanner, SteamScanner};
use reqwest::{StatusCode, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, create_dir_all, read_to_string, write};

#[cfg(all(unix, not(target_os = "macos")))]
use crate::scanner::LutrisScanner;

#[cfg(any(windows, target_os = "macos"))]
use crate::scanner::GOGScanner;

#[cfg(windows)]
use crate::scanner::XboxScanner;

const GAMEDB_YAML: &str = include_str!("../resources/gamedb.yaml");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Deserialize, Serialize)]
pub struct GameDbEntry {
    pub files: GameFiles
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GameFiles {
    pub windows: Option<Vec<String>>,
    #[cfg(all(unix, not(target_os = "macos")))]
    pub linux: Option<Vec<String>>,
    #[cfg(target_os = "macos")]
    pub mac: Option<Vec<String>>
}

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub name: String,
    pub files: Vec<FileMetadata>
}

#[derive(Deserialize, Serialize)]
pub struct FileMetadata {
    pub hash: String,
    pub modified: std::time::SystemTime,
    pub path: String,
    pub size: u64
}

#[derive(Deserialize, Serialize)]
struct CustomDbMetadata {
    etag: Option<String>,
    data: HashMap<String, GameDbEntry>
}

#[derive(Default, Deserialize, Serialize)]
struct CustomDbCache {
    databases: HashMap<String, CustomDbMetadata>
}

pub fn parse() -> HashMap<String, GameDbEntry> {
    let gamedb_path = cache().join("gamedb.yaml");

    let mut db: HashMap<String, GameDbEntry> = if gamedb_path.exists() {
        serde_yaml::from_reader(File::open(gamedb_path).unwrap()).unwrap_or_else(|_| {
            log::error!("Failed to parse cached GameDB, falling back to built-in.");
            serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
        })
    } else {
        serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
    };

    db.extend(load_custom_db_cache().databases.into_values().flat_map(|custom_db| custom_db.data));
    db
}

pub fn get_installed_games() -> Vec<Game> {
    let db = parse();
    let mut games = vec![];

    #[cfg(all(unix, not(target_os = "macos")))]
    games.extend(LutrisScanner::get_games());

    #[cfg(any(windows, target_os = "macos"))]
    games.extend(GOGScanner::get_games());

    #[cfg(windows)]
    games.extend(XboxScanner::get_games());

    games.extend(HeroicScanner::get_games());
    games.extend(SteamScanner::get_games());

    games
        .into_iter()
        .filter_map(|mut game| {
            if db.contains_key(&game.name) {
                return Some(game);
            }

            let clean_name = game.name.replace(['™', '®'], "").trim().to_owned();
            db.contains_key(&clean_name).then(|| {
                game.name = clean_name;
                game
            })
        })
        .collect()
}

pub fn update() -> Result<bool> {
    let cache_dir = cache();

    let gamedb_path = cache_dir.join("gamedb.yaml");
    let etag_path = cache_dir.join("gamedb.etag");

    create_dir_all(cache_dir)?;

    let previous_etag = etag_path.exists().then(|| read_to_string(&etag_path).ok()).flatten();

    let client = reqwest::blocking::Client::new();
    let mut request = client
        .get("https://raw.githubusercontent.com/Spencer-0003/aletheia/refs/heads/master/resources/gamedb.yaml")
        .header(reqwest::header::USER_AGENT, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));

    if let Some(ref etag) = previous_etag {
        request = request.header(header::IF_NONE_MATCH, etag);
    }

    let response = request.send()?.error_for_status()?;
    let status = response.status();

    if status == StatusCode::NOT_MODIFIED {
        return Ok(false);
    }

    let current_etag = response.headers().get(header::ETAG).unwrap();

    write(&etag_path, current_etag.as_bytes())?;
    write(&gamedb_path, response.bytes()?)?;

    Ok(true)
}

fn load_custom_db_cache() -> CustomDbCache {
    let cache_path = cache().join("custom_gamedb.yaml");

    if cache_path.exists() {
        serde_yaml::from_reader(File::open(&cache_path).unwrap()).unwrap_or_else(|_| {
            log::error!("Failed to parse custom GameDB cache.");
            CustomDbCache::default()
        })
    } else {
        CustomDbCache::default()
    }
}

pub fn update_custom(cfg: &Config) -> Result<bool> {
    if cfg.custom_databases.is_empty() {
        return Ok(false);
    }

    let cache_dir = cache();

    let client = reqwest::blocking::Client::new();
    let mut db_cache = load_custom_db_cache();
    let mut updated = false;

    create_dir_all(&cache_dir)?;

    for db in &cfg.custom_databases {
        let mut request = client.get(db).header(header::USER_AGENT, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")));
        let cached_etag = db_cache.databases.get(db).and_then(|meta| meta.etag.as_ref());

        if let Some(etag) = cached_etag {
            request = request.header(header::IF_NONE_MATCH, etag);
        }

        let response = request.send()?.error_for_status()?;

        if response.status() == StatusCode::NOT_MODIFIED {
            continue;
        }

        let etag = response.headers().get(header::ETAG).and_then(|etag| etag.to_str().ok()).map(ToOwned::to_owned);

        db_cache.databases.insert(db.clone(), CustomDbMetadata { etag, data: serde_yaml::from_reader(response).unwrap() });
        updated = true;
    }

    if updated {
        db_cache.databases.retain(|url, _| cfg.custom_databases.contains(url));
        serde_yaml::to_writer(File::create(cache_dir.join("custom_gamedb.yaml"))?, &db_cache).unwrap();
    }

    Ok(updated)
}
