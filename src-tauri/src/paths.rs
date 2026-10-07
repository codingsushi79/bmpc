//! Finding BeamNG.drive and its user folder, the same way the official
//! BeamMP launcher does, so BeamLink installs mods exactly where the game and
//! the launcher will look for them.
//!
//! * Game directory: the BeamNG registry key, then every Steam library
//!   (`libraryfolders.vdf`) that holds app 284160.
//! * User folder: `startup.ini [filesystem] UserPath` in the game directory,
//!   then `userFolder` in `%LOCALAPPDATA%\BeamNG\BeamNG.Drive.ini`, then
//!   `%LOCALAPPDATA%\BeamNG\BeamNG.drive`. Mods go under its `current` folder.

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::settings::Settings;

pub const STEAM_APP: &str = "284160";

#[derive(Debug, Clone, Serialize, Default)]
pub struct GamePaths {
    /// Where BeamNG.drive is installed.
    pub game_dir: Option<PathBuf>,
    /// BeamNG's user folder root (what `--user-path` takes).
    pub user_root: Option<PathBuf>,
    /// `<user_root>/current`: mods, settings, everything versioned.
    pub user_dir: Option<PathBuf>,
    pub game_version: Option<String>,
    /// Where BeamMP-Launcher lives (and keeps its `key` and cache).
    pub launcher_dir: PathBuf,
    /// How each path was found, for the settings screen.
    pub notes: Vec<String>,
}

impl GamePaths {
    pub fn mods_dir(&self) -> Option<PathBuf> {
        self.user_dir.as_ref().map(|u| u.join("mods"))
    }

    pub fn multiplayer_dir(&self) -> Option<PathBuf> {
        self.mods_dir().map(|m| m.join("multiplayer"))
    }

    pub fn launcher_exe(&self) -> PathBuf {
        self.launcher_dir.join(launcher_name())
    }
}

pub fn launcher_name() -> &'static str {
    if cfg!(windows) {
        "BeamMP-Launcher.exe"
    } else {
        "BeamMP-Launcher"
    }
}

/// The BeamMP client mod's file name. The Linux game cannot load mods with
/// upper-case names, which is why the launcher uses lower case there.
pub fn client_mod_name() -> &'static str {
    if cfg!(windows) {
        "BeamMP.zip"
    } else {
        "beammp.zip"
    }
}

pub fn detect(settings: &Settings) -> GamePaths {
    let mut notes = Vec::new();
    let game_dir = override_path(&settings.game_dir)
        .inspect(|_| notes.push("game folder: set in settings".into()))
        .or_else(|| {
            let found = find_game_dir();
            if let Some((_, how)) = &found {
                notes.push(format!("game folder: {how}"));
            }
            found.map(|(p, _)| p)
        });
    let user_root = override_path(&settings.user_dir)
        .inspect(|_| notes.push("user folder: set in settings".into()))
        .or_else(|| {
            let found = find_user_root(game_dir.as_deref());
            if let Some((_, how)) = &found {
                notes.push(format!("user folder: {how}"));
            }
            found.map(|(p, _)| p)
        });
    let game_version = game_dir.as_deref().and_then(read_game_version);
    let launcher_dir = override_path(&settings.launcher_dir)
        .unwrap_or_else(|| crate::settings::app_dir().join("launcher"));
    GamePaths {
        user_dir: user_root.as_ref().map(|r| r.join("current")),
        game_dir,
        user_root,
        game_version,
        launcher_dir,
        notes,
    }
}

fn override_path(value: &str) -> Option<PathBuf> {
    let value = value.trim();
    (!value.is_empty()).then(|| PathBuf::from(value))
}

/// `integrity.json` in the game folder carries the build version.
fn read_game_version(game_dir: &Path) -> Option<String> {
    let text = std::fs::read(game_dir.join("integrity.json")).ok()?;
    let json: serde_json::Value = serde_json::from_slice(&text).ok()?;
    json["version"].as_str().map(str::to_string)
}

pub fn is_game_dir(dir: &Path) -> bool {
    dir.join("BeamNG.drive.exe").is_file()
        || dir.join("Bin64").join("BeamNG.drive.x64.exe").is_file()
        || dir.join("BinLinux").join("BeamNG.drive.x64").is_file()
        || dir.join("integrity.json").is_file()
}

fn find_game_dir() -> Option<(PathBuf, String)> {
    #[cfg(windows)]
    if let Some(path) = windows::beamng_rootpath().filter(|p| is_game_dir(p)) {
        return Some((path, "BeamNG registry key".into()));
    }
    for steamapps in steam_libraries() {
        let manifest = steamapps.join(format!("appmanifest_{STEAM_APP}.acf"));
        let dir = steamapps.join("common").join("BeamNG.drive");
        if manifest.is_file() && is_game_dir(&dir) {
            return Some((dir, "Steam library".into()));
        }
    }
    #[cfg(windows)]
    if let Some(path) = windows::uninstall_location().filter(|p| is_game_dir(p)) {
        return Some((path, "Steam uninstall entry".into()));
    }
    None
}

/// Every `steamapps` folder this machine's Steam knows about.
pub fn steam_libraries() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        if let Some(path) = windows::steam_path() {
            roots.push(path);
        }
        roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    }
    #[cfg(not(windows))]
    if let Some(home) = dirs::home_dir() {
        for rel in [
            ".steam/steam",
            ".local/share/Steam",
            ".var/app/com.valvesoftware.Steam/.steam/root",
            "snap/steam/common/.local/share/Steam",
            "Library/Application Support/Steam",
        ] {
            roots.push(home.join(rel));
        }
    }
    let mut libraries = Vec::new();
    for root in roots {
        let steamapps = root.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        push_unique(&mut libraries, steamapps.clone());
        if let Ok(vdf) = std::fs::read_to_string(steamapps.join("libraryfolders.vdf")) {
            for path in vdf_paths(&vdf) {
                push_unique(&mut libraries, PathBuf::from(path).join("steamapps"));
            }
        }
    }
    libraries
}

fn push_unique(list: &mut Vec<PathBuf>, path: PathBuf) {
    if !list.contains(&path) {
        list.push(path);
    }
}

/// The `"path"` values from a `libraryfolders.vdf`, unescaped.
pub fn vdf_paths(vdf: &str) -> Vec<String> {
    vdf.lines()
        .filter_map(|line| {
            let mut parts = line.split('"').filter(|p| !p.trim().is_empty());
            match (parts.next(), parts.next()) {
                (Some("path"), Some(value)) => Some(value.replace("\\\\", "\\")),
                _ => None,
            }
        })
        .collect()
}

fn find_user_root(game_dir: Option<&Path>) -> Option<(PathBuf, String)> {
    if let Some(game_dir) = game_dir
        && let Ok(text) = std::fs::read_to_string(game_dir.join("startup.ini"))
        && let Some(value) = ini_value(&text, Some("filesystem"), "UserPath")
    {
        let path = PathBuf::from(expand_env(&value));
        if path.is_dir() {
            return Some((path, "startup.ini UserPath".into()));
        }
    }
    #[cfg(windows)]
    {
        let base = dirs::data_local_dir()?.join("BeamNG");
        if let Ok(text) = std::fs::read_to_string(base.join("BeamNG.Drive.ini"))
            && let Some(value) = ini_value(&text, None, "userFolder")
        {
            let path = PathBuf::from(expand_env(&value));
            if path.is_dir() {
                return Some((path, "BeamNG.Drive.ini userFolder".into()));
            }
        }
        return Some((base.join("BeamNG.drive"), "default (%LOCALAPPDATA%)".into()));
    }
    #[cfg(not(windows))]
    {
        let path = dirs::home_dir()?.join(".local/share/BeamNG/BeamNG.drive");
        Some((path, "default (~/.local/share/BeamNG)".into()))
    }
}

/// A value from an ini file. `section = None` matches keys outside any
/// section (BeamNG.Drive.ini has no sections).
pub fn ini_value(text: &str, section: Option<&str>, key: &str) -> Option<String> {
    let mut current: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(';') || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            current = Some(name.trim().to_ascii_lowercase());
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let in_section = match section {
            Some(s) => current.as_deref() == Some(&s.to_ascii_lowercase()),
            None => true,
        };
        if in_section && k.trim().eq_ignore_ascii_case(key) {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// Expand `%VAR%` (Windows style) and a leading `~`.
pub fn expand_env(value: &str) -> String {
    let mut out = String::new();
    let mut rest = value;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name) {
                    Ok(v) => out.push_str(&v),
                    Err(_) => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    if let Some(stripped) = out.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(stripped).display().to_string();
    }
    out
}

#[cfg(windows)]
mod windows {
    use std::path::PathBuf;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

    pub fn beamng_rootpath() -> Option<PathBuf> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"Software\BeamNG\BeamNG.drive")
            .ok()?;
        let path: String = key.get_value("rootpath").ok()?;
        Some(PathBuf::from(path.trim_end_matches(['\\', '/'])))
    }

    pub fn steam_path() -> Option<PathBuf> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(r"Software\Valve\Steam")
            .ok()?;
        let path: String = key.get_value("SteamPath").ok()?;
        Some(PathBuf::from(path.replace('/', "\\")))
    }

    pub fn uninstall_location() -> Option<PathBuf> {
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 284160")
            .ok()?;
        let path: String = key.get_value("InstallLocation").ok()?;
        Some(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_library_paths_are_read_from_vdf() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"apps" { "228980" "1" }
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"apps" { "284160" "42" }
	}
}"#;
        assert_eq!(
            vdf_paths(vdf),
            vec![r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]
        );
    }

    #[test]
    fn ini_values_respect_sections() {
        let ini = "[general]\nUserPath = wrong\n[filesystem]\nUserPath = \"D:\\Beam\"\n";
        assert_eq!(
            ini_value(ini, Some("filesystem"), "userpath").as_deref(),
            Some("D:\\Beam")
        );
        assert_eq!(
            ini_value("userFolder = E:\\BeamUser\n", None, "userFolder").as_deref(),
            Some("E:\\BeamUser")
        );
    }

    #[test]
    fn env_vars_expand_and_unknown_ones_survive() {
        // SAFETY: tests in this module do not read this variable concurrently.
        unsafe { std::env::set_var("BEAMLINK_TEST_DIR", "/tmp/x") };
        assert_eq!(expand_env("%BEAMLINK_TEST_DIR%/beam"), "/tmp/x/beam");
        assert_eq!(expand_env("%NOPE_NOT_SET%/a"), "%NOPE_NOT_SET%/a");
        assert_eq!(expand_env("100% sure"), "100% sure");
    }
}
