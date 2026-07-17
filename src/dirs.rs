// SPDX-FileCopyrightText: 2025-2026 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::scanner::SteamScanner;
use std::env::{home_dir, var_os};
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[cfg(all(unix, not(target_os = "macos")))]
use std::ffi::OsString;

#[cfg(target_os = "macos")]
pub fn cache() -> PathBuf {
    var_os("XDG_CACHE_HOME").map_or_else(|| home().join("Library/caches"), PathBuf::from).join("moe.spencer.aletheia")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn cache() -> PathBuf {
    var_os("XDG_CACHE_HOME").map_or_else(|| home().join(".cache"), PathBuf::from).join("aletheia")
}

#[cfg(windows)]
pub fn cache() -> PathBuf {
    app_data().join("aletheia/cache")
}

#[cfg(target_os = "macos")]
pub fn config() -> PathBuf {
    var_os("XDG_CONFIG_HOME").map_or_else(|| home().join("Library/Preferences"), PathBuf::from)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn config() -> PathBuf {
    var_os("XDG_CONFIG_HOME").map_or_else(|| home().join(".config"), PathBuf::from)
}

#[cfg(windows)]
pub fn config() -> PathBuf {
    var_os("APPDATA").map(PathBuf::from).unwrap()
}

#[cfg(target_os = "macos")]
pub fn app_data() -> PathBuf {
    var_os("XDG_DATA_HOME").map_or_else(|| home().join("Library/Application Support"), PathBuf::from)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn app_data() -> PathBuf {
    var_os("XDG_DATA_HOME").map_or_else(|| home().join(".local/share"), PathBuf::from)
}

#[cfg(windows)]
pub fn app_data() -> PathBuf {
    var_os("LOCALAPPDATA").map(PathBuf::from).unwrap()
}

pub fn home() -> PathBuf {
    home_dir().unwrap()
}

fn expand_path_components(path: &Path, replacements: &[(&str, PathBuf)]) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        let component_os = component.as_os_str();
        let mut replaced = false;

        for (pattern, replacement) in replacements {
            if component_os == *pattern {
                result.push(replacement);
                replaced = true;
                break;
            }
        }

        if !replaced {
            result.push(component);
        }
    }

    result
}

fn shrink_path_components(path: &Path, replacements: &[(&str, PathBuf)]) -> PathBuf {
    for (pattern, replacement) in replacements {
        if let Ok(stripped) = path.strip_prefix(replacement) {
            let mut new_path = PathBuf::from(pattern);
            new_path.push(stripped);

            for (steam_pattern, steam_replacement) in replacements.iter().filter(|(p, _)| *p == "{SteamID64}" || *p == "{SteamID3}") {
                let steam_str = steam_replacement.to_string_lossy();
                let path_str = new_path.to_string_lossy();
                if path_str.contains(steam_str.as_ref()) {
                    return PathBuf::from(path_str.replace(steam_str.as_ref(), steam_pattern));
                }
            }

            return new_path;
        }
    }

    path.to_path_buf()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn path_contains_subpath(haystack: &Path, needle: &str) -> bool {
    haystack.ancestors().any(|ancestor| ancestor.ends_with(needle))
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let linux_app_data = app_data();

    if let Some(wine_prefix) = prefix {
        let username = if path_contains_subpath(wine_prefix, "steamapps/compatdata") {
            OsString::from("steamuser")
        } else {
            var_os("USER").unwrap()
        };

        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(username);
        let windows_app_data = user.join("AppData");
        let documents = user.join("Documents");

        let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
        let steam_id_64 = steam_account_id
            .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
        let steam_user_data = steam_account_id
            .map_or_else(|| linux_app_data.join("Steam/userdata/[0-9]*"), |id| linux_app_data.join("Steam/userdata").join(id));

        replacements.extend([
            ("{AppData}", windows_app_data.join("Roaming")),
            ("{Documents}", documents),
            ("{Home}", user),
            ("{LocalAppData}", windows_app_data.join("Local")),
            ("{LocalLow}", windows_app_data.join("LocalLow")),
            ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
            ("{SteamID3}", steam_id_3),
            ("{SteamID64}", steam_id_64),
            ("{SteamUserData}", steam_user_data)
        ]);
    } else {
        replacements.push(("{Home}", home()));
    }

    replacements.extend([("{XDGConfig}", config()), ("{XDGData}", linux_app_data)]);

    expand_path_components(path, &replacements)
}

#[cfg(windows)]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let roaming_app_data = config();
    let local_app_data = app_data();
    let home_dir = home();

    let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
    let steam_id_64 = steam_account_id
        .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
    let steam_user_data = {
        let base_path = steamlocate::SteamDir::locate()
            .map_or_else(|_| PathBuf::from("C:/Program Files (x86)/Steam"), |dir| dir.path().to_path_buf());

        let userdata_path = base_path.join("userdata");
        steam_account_id.map_or_else(|| userdata_path.join("[0-9]*"), |id| userdata_path.join(id))
    };

    replacements.extend([
        ("{AppData}", roaming_app_data),
        ("{Documents}", home_dir.join("Documents")),
        ("{Home}", home_dir),
        ("{LocalAppData}", local_app_data.clone()),
        ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
        ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
        ("{SteamID3}", steam_id_3),
        ("{SteamID64}", steam_id_64),
        ("{SteamUserData}", steam_user_data)
    ]);

    expand_path_components(path, &replacements)
}

#[cfg(target_os = "macos")]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let home_dir = home();
    let application_support = home_dir.join("Library/Application Support"); // app_data is not used here as most games don't use the XDG spec on MacOS
    let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
    let steam_id_64 = steam_account_id
        .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
    let steam_user_data = steam_account_id
        .map_or_else(|| application_support.join("Steam/userdata/[0-9]*"), |id| application_support.join("Steam/userdata").join(id));

    replacements.extend([("{SteamID3}", steam_id_3), ("{SteamID64}", steam_id_64), ("{SteamUserData}", steam_user_data)]);

    if let Some(wine_prefix) = prefix {
        let username = var_os("USER").unwrap();

        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(username);
        let windows_app_data = user.join("AppData");
        let documents = user.join("Documents");

        replacements.extend([
            ("{AppData}", windows_app_data.join("Roaming")),
            ("{Documents}", documents),
            ("{Home}", user),
            ("{LocalAppData}", windows_app_data.join("Local")),
            ("{LocalLow}", windows_app_data.join("LocalLow")),
            ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications"))
        ]);
    } else {
        replacements.extend([
            ("{AppData}", application_support.clone()),
            ("{Documents}", home_dir.join("Documents")),
            ("{Home}", home_dir),
            ("{GOGAppData}", application_support.join("GOG.com/Galaxy/Applications"))
        ]);
    }

    expand_path_components(path, &replacements)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn shrink_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let linux_app_data = app_data();

    if let Some(wine_prefix) = prefix {
        let username = if path_contains_subpath(wine_prefix, "steamapps/compatdata") {
            OsString::from("steamuser")
        } else {
            var_os("USER").unwrap()
        };

        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(username);
        let windows_app_data = user.join("AppData");

        let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
        let steam_id_64 = steam_account_id
            .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
        let steam_user_data = steam_account_id
            .map_or_else(|| linux_app_data.join("Steam/userdata/[0-9]*"), |id| linux_app_data.join("Steam/userdata").join(id));

        replacements.extend([
            ("{LocalLow}", windows_app_data.join("LocalLow")),
            ("{LocalAppData}", windows_app_data.join("Local")),
            ("{AppData}", windows_app_data.join("Roaming")),
            ("{Documents}", user.join("Documents")),
            ("{Home}", user),
            ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
            ("{SteamID3}", steam_id_3),
            ("{SteamID64}", steam_id_64),
            ("{SteamUserData}", steam_user_data)
        ]);
    } else {
        replacements.push(("{Home}", home()));
    }

    replacements.extend([("{XDGConfig}", config()), ("{XDGData}", linux_app_data)]);

    shrink_path_components(path, &replacements)
}

#[cfg(windows)]
pub fn shrink_path(path: &Path, installation_dir: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let roaming_app_data = config();
    let local_app_data = app_data();
    let home_dir = home();

    let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
    let steam_id_64 = steam_account_id
        .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
    let steam_user_data = {
        let base_path = steamlocate::SteamDir::locate()
            .map_or_else(|_| PathBuf::from("C:/Program Files (x86)/Steam"), |dir| dir.path().to_path_buf());

        let userdata_path = base_path.join("userdata");
        steam_account_id.map_or_else(|| userdata_path.join("[0-9]*"), |id| userdata_path.join(id))
    };

    replacements.extend([
        ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
        ("{LocalAppData}", local_app_data.clone()),
        ("{AppData}", roaming_app_data),
        ("{Documents}", home_dir.join("Documents")),
        ("{Home}", home_dir),
        ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
        ("{SteamID3}", steam_id_3),
        ("{SteamID64}", steam_id_64),
        ("{SteamUserData}", steam_user_data)
    ]);

    shrink_path_components(path, &replacements)
}

#[cfg(target_os = "macos")]
pub fn shrink_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let home_dir = home();
    let application_support = home_dir.join("Library/Application Support");
    let steam_id_3 = steam_account_id.map_or_else(|| PathBuf::from("*"), PathBuf::from);
    let steam_id_64 = steam_account_id
        .map_or_else(|| PathBuf::from("*"), |id| PathBuf::from(SteamScanner::id3_to_id64(id.parse().unwrap()).to_string()));
    let steam_user_data = steam_account_id
        .map_or_else(|| application_support.join("Steam/userdata/[0-9]*"), |id| application_support.join("Steam/userdata").join(id));

    replacements.extend([("{SteamID3}", steam_id_3), ("{SteamID64}", steam_id_64), ("{SteamUserData}", steam_user_data)]);

    if let Some(wine_prefix) = prefix {
        let username = var_os("USER").unwrap();

        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(username);
        let windows_app_data = user.join("AppData");

        replacements.extend([
            ("{LocalLow}", windows_app_data.join("LocalLow")),
            ("{LocalAppData}", windows_app_data.join("Local")),
            ("{AppData}", windows_app_data.join("Roaming")),
            ("{Documents}", user.join("Documents")),
            ("{Home}", user),
            ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications"))
        ]);
    } else {
        replacements.extend([
            ("{AppData}", application_support.clone()),
            ("{Documents}", home_dir.join("Documents")),
            ("{Home}", home_dir),
            ("{GOGAppData}", application_support.join("GOG.com/Galaxy/Applications"))
        ]);
    }

    shrink_path_components(path, &replacements)
}

pub fn get_size(path: &Path) -> u64 {
    let mut size = 0;

    for entry in read_dir(path).unwrap() {
        let dir_entry = entry.unwrap();
        let entry_path = dir_entry.path();

        if entry_path.is_file() {
            size += entry_path.metadata().unwrap().len();
        } else if entry_path.is_dir() {
            size += get_size(&entry_path);
        }
    }

    size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_expansion() {
        #[cfg(unix)]
        let username = var_os("USER").unwrap();

        let home_dir = home();
        let root_dir = home_dir.join("Games/Unit Test");

        let save_file_1 = Path::new("{LocalLow}/AllianceArts/All in Abyss/SaveData/{SteamID64}/GameData/GameSaveData_0.sav");
        let save_file_2 = Path::new("{GameRoot}/SAVEDATA/SonicDX01.snc");
        let gabe_steam_id3 = "22202";
        let gabe_steam_id64 = "76561197960287930";

        #[cfg(unix)]
        {
            let prefix = home_dir.join("Games/UnitTest");

            assert_eq!(
                expand_path(save_file_1, None, Some(&prefix), Some(gabe_steam_id3)),
                prefix
                    .join("drive_c/users")
                    .join(username)
                    .join("AppData/LocalLow/AllianceArts/All in Abyss/SaveData")
                    .join(gabe_steam_id64)
                    .join("GameData/GameSaveData_0.sav")
            );
            assert_eq!(expand_path(save_file_2, Some(&root_dir), None, None), root_dir.join("SAVEDATA/SonicDX01.snc"));
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let save_file_3 = Path::new("{XDGData}/Terraria/Players/UnitTest.plr");
            let xdg_data = app_data();

            assert_eq!(expand_path(save_file_3, None, None, None), xdg_data.join("Terraria/Players/UnitTest.plr"));
        }

        #[cfg(target_os = "macos")]
        {
            let save_file_3 = Path::new("{AppData}/Terraria/Players/UnitTest.plr");
            let application_support = home_dir.join("Library/Application Support");

            assert_eq!(expand_path(save_file_3, None, None, None), application_support.join("Terraria/Players/UnitTest.plr"));
        }

        #[cfg(windows)]
        {
            let save_file_3 = Path::new("{Documents}/My Games/Terraria/Players/UnitTest.plr");

            assert_eq!(
                expand_path(save_file_1, None, Some(&gabe_steam_id3)),
                home_dir
                    .join("AppData/LocalLow/AllianceArts/All in Abyss/SaveData")
                    .join(gabe_steam_id64)
                    .join("GameData/GameSaveData_0.sav")
            );
            assert_eq!(expand_path(save_file_2, Some(&root_dir), None), root_dir.join("SAVEDATA/SonicDX01.snc"));
            assert_eq!(expand_path(save_file_3, None, None), home_dir.join("Documents/My Games/Terraria/Players/UnitTest.plr"));
        }
    }

    #[test]
    fn test_path_shrinkage() {
        let home_dir = home();

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let save_file_1 = home_dir.join(".local/share/Steam/steamapps/common/Sonic Adventure DX/SAVEDATA/SonicDX01.snc");
            let installation_dir = home_dir.join(".local/share/Steam/steamapps/common/Sonic Adventure DX");

            assert_eq!(shrink_path(&save_file_1, Some(&installation_dir), None, None), Path::new("{GameRoot}/SAVEDATA/SonicDX01.snc"));
        }

        #[cfg(target_os = "macos")]
        {
            let save_file_1 = home_dir.join("Games/Sonic Adventure DX/SAVEDATA/SonicDX01.snc");
            let installation_dir = home_dir.join("Games/Sonic Adventure DX");

            assert_eq!(shrink_path(&save_file_1, Some(&installation_dir), None, None), Path::new("{GameRoot}/SAVEDATA/SonicDX01.snc"));
        }

        #[cfg(windows)]
        {
            let save_file_1 = home_dir.join("Games/Sonic Adventure DX/SAVEDATA/SonicDX01.snc");
            let installation_dir = home_dir.join("Games/Sonic Adventure DX");

            assert_eq!(shrink_path(&save_file_1, Some(&installation_dir), None), Path::new("{GameRoot}/SAVEDATA/SonicDX01.snc"));
        }
    }
}
