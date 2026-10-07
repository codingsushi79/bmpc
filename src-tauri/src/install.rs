//! First-run setup and repair: everything BeamMP needs, put where the game
//! and the official launcher look for it.
//!
//! 1. BeamMP-Launcher (official build from GitHub, sha256-checked)
//! 2. BeamMP client mod → `<user>/current/mods/multiplayer/BeamMP.zip`
//!    (sha256 from the BeamMP backend, same check the launcher does)
//! 3. BeamLink companion → `<user>/current/mods/beamlink.zip`
//! 4. Both marked active in `mods/db.json`
//!
//! Each step reports progress through a callback so the installer, the setup
//! wizard and "repair" in settings all share this one code path.

use serde::Serialize;
use std::path::Path;

use crate::companion;
use crate::net;
use crate::paths::{self, GamePaths};
use crate::settings::Settings;

const LAUNCHER_RELEASE: &str =
    "https://api.github.com/repos/BeamMP/BeamMP-Launcher/releases/latest";
const BACKEND: &str = "https://backend.beammp.com";

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub step: &'static str,
    /// running | done | skipped | error
    pub status: &'static str,
    pub message: String,
    pub done: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct InstallState {
    pub game_found: bool,
    pub user_dir_exists: bool,
    pub launcher: bool,
    pub client_mod: bool,
    pub companion: bool,
    pub launcher_supported: bool,
}

impl InstallState {
    pub fn ready(&self) -> bool {
        self.game_found && self.launcher && self.client_mod && self.companion
    }
}

pub fn launcher_supported() -> bool {
    // BeamMP publishes Windows launcher builds only; on Linux it has to be
    // built from source and pointed at in settings.
    cfg!(windows)
}

pub fn state(paths: &GamePaths) -> InstallState {
    let mods = paths.mods_dir();
    InstallState {
        game_found: paths.game_dir.is_some(),
        user_dir_exists: paths.user_dir.as_ref().is_some_and(|d| d.is_dir()),
        launcher: paths.launcher_exe().is_file(),
        client_mod: paths
            .multiplayer_dir()
            .is_some_and(|d| d.join(paths::client_mod_name()).is_file()),
        companion: mods.as_deref().is_some_and(companion::installed),
        launcher_supported: launcher_supported(),
    }
}

/// Run every step. Errors in one step are reported and the rest still run,
/// so a flaky download does not leave the companion uninstalled.
pub fn run(paths: &GamePaths, settings: &Settings, emit: &dyn Fn(Progress)) -> Result<(), String> {
    let mut failures = Vec::new();
    let step = |name: &'static str, result: Result<String, StepError>| {
        let (status, message) = match result {
            Ok(message) => ("done", message),
            Err(StepError::Skip(message)) => ("skipped", message),
            Err(StepError::Fail(message)) => ("error", message),
        };
        emit(Progress {
            step: name,
            status,
            message,
            done: 0,
            total: 0,
        });
        status == "error"
    };
    let start = |name: &'static str, message: &str| {
        emit(Progress {
            step: name,
            status: "running",
            message: message.into(),
            done: 0,
            total: 0,
        })
    };

    start("game", "Looking for BeamNG.drive");
    if step("game", check_game(paths)) {
        failures.push("game");
        return Err("BeamNG.drive was not found — set its folder in Settings".into());
    }
    let user_dir = paths.user_dir.clone().expect("checked by check_game");
    let mods_dir = user_dir.join("mods");

    start("launcher", "Fetching the official BeamMP launcher");
    if step("launcher", install_launcher(paths, settings, emit)) {
        failures.push("launcher");
    }
    start("client", "Fetching the BeamMP client mod");
    if step("client", install_client_mod(paths, settings, emit)) {
        failures.push("client");
    }
    start("companion", "Installing the BeamLink companion");
    if step(
        "companion",
        companion::install(&mods_dir)
            .map(|written| {
                if written {
                    format!(
                        "installed to {}",
                        mods_dir.join(companion::MOD_FILE).display()
                    )
                } else {
                    "already up to date".into()
                }
            })
            .map_err(StepError::Fail),
    ) {
        failures.push("companion");
    }
    start("activate", "Enabling the mods in BeamNG");
    if step(
        "activate",
        companion::activate(&mods_dir, &["multiplayerbeammp", companion::MOD_NAME])
            .map(|_| "enabled in mods/db.json".into())
            .map_err(StepError::Fail),
    ) {
        failures.push("activate");
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "setup finished with problems in: {}",
            failures.join(", ")
        ))
    }
}

enum StepError {
    Skip(String),
    Fail(String),
}

fn check_game(paths: &GamePaths) -> Result<String, StepError> {
    let Some(game) = &paths.game_dir else {
        return Err(StepError::Fail(
            "BeamNG.drive is not installed here, or is somewhere unusual. Set the game folder in Settings."
                .into(),
        ));
    };
    let Some(user) = &paths.user_dir else {
        return Err(StepError::Fail(
            "could not work out BeamNG's user folder".into(),
        ));
    };
    // The game creates this on first launch; creating it early is harmless
    // and lets the mods land before the first run.
    std::fs::create_dir_all(user.join("mods").join("multiplayer"))
        .map_err(|e| StepError::Fail(format!("{}: {e}", user.display())))?;
    Ok(format!(
        "{} {}",
        game.display(),
        paths
            .game_version
            .as_deref()
            .map(|v| format!("(v{v})"))
            .unwrap_or_default()
    ))
}

fn install_launcher(
    paths: &GamePaths,
    settings: &Settings,
    emit: &dyn Fn(Progress),
) -> Result<String, StepError> {
    let exe = paths.launcher_exe();
    write_launcher_config(&paths.launcher_dir, settings).map_err(StepError::Fail)?;
    if !launcher_supported() {
        return if exe.is_file() {
            Ok(format!("using {}", exe.display()))
        } else {
            Err(StepError::Skip(format!(
                "BeamMP only publishes the launcher for Windows. Build BeamMP-Launcher from source \
                 and put it in {} (or set the folder in Settings).",
                paths.launcher_dir.display()
            )))
        };
    }
    let release: serde_json::Value =
        serde_json::from_str(&net::get_text(LAUNCHER_RELEASE, 1 << 20).map_err(StepError::Fail)?)
            .map_err(|e| StepError::Fail(e.to_string()))?;
    let tag = release["tag_name"].as_str().unwrap_or("latest").to_string();
    let assets = release["assets"].as_array().cloned().unwrap_or_default();
    let url_of = |name: &str| {
        assets
            .iter()
            .find(|a| a["name"].as_str() == Some(name))
            .and_then(|a| a["browser_download_url"].as_str())
            .map(str::to_string)
    };
    let exe_name = paths::launcher_name();
    let url = url_of(exe_name)
        .ok_or_else(|| StepError::Fail(format!("release {tag} has no {exe_name}")))?;
    let expected = match url_of(&format!("{exe_name}.sha256")) {
        Some(sha_url) => net::get_text(&sha_url, 4096)
            .ok()
            .and_then(|t| t.split_whitespace().next().and_then(net::clean_hash)),
        None => None,
    };
    if let (Some(expected), Some(actual)) = (&expected, net::sha256_file(&exe))
        && &actual == expected
    {
        return Ok(format!("{tag} already installed"));
    }
    if exe.is_file() && expected.is_none() {
        // No checksum to compare against: keep the working launcher, it
        // updates itself from the BeamMP backend anyway.
        return Ok(format!("using existing {}", exe.display()));
    }
    net::download(&url, &exe, expected.as_deref(), |done, total| {
        emit(Progress {
            step: "launcher",
            status: "running",
            message: format!("Downloading BeamMP-Launcher {tag}"),
            done,
            total,
        })
    })
    .map_err(StepError::Fail)?;
    Ok(format!(
        "{tag} installed to {}",
        paths.launcher_dir.display()
    ))
}

/// `Launcher.cfg` is read from the launcher's working directory.
pub fn write_launcher_config(dir: &Path, settings: &Settings) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join("Launcher.cfg");
    let mut config: serde_json::Value = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    config["Port"] = settings.launcher_port.into();
    config["Build"] = settings.branch.clone().into();
    if config.get("CachingDirectory").is_none() {
        config["CachingDirectory"] = "./Resources".into();
    }
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn install_client_mod(
    paths: &GamePaths,
    settings: &Settings,
    emit: &dyn Fn(Progress),
) -> Result<String, StepError> {
    let dir = paths
        .multiplayer_dir()
        .ok_or_else(|| StepError::Fail("no user folder".into()))?;
    let dest = dir.join(paths::client_mod_name());
    let branch = urlencode(&settings.branch);
    let expected = net::get_text(&format!("{BACKEND}/sha/mod?branch={branch}&pk="), 4096)
        .ok()
        .and_then(|t| net::clean_hash(&t));
    match (&expected, net::sha256_file(&dest)) {
        (Some(expected), Some(actual)) if &actual == expected => {
            return Ok("already up to date".into());
        }
        (None, Some(_)) => {
            return Ok(
                "installed (the BeamMP backend gave no checksum; the launcher re-checks on start)"
                    .into(),
            );
        }
        _ => {}
    }
    net::download(
        &format!("{BACKEND}/builds/client?download=true&pk=&branch={branch}"),
        &dest,
        expected.as_deref(),
        |done, total| {
            emit(Progress {
                step: "client",
                status: "running",
                message: "Downloading the BeamMP client mod".into(),
                done,
                total,
            })
        },
    )
    .map_err(StepError::Fail)?;
    // A stale unpacked copy would shadow the new zip.
    if let Some(mods) = paths.mods_dir() {
        let unpacked = mods.join("unpacked").join("beammp");
        if unpacked.is_dir() && !unpacked.join(".git").exists() {
            let _ = std::fs::remove_dir_all(unpacked);
        }
    }
    Ok(format!("installed to {}", dest.display()))
}

/// Undo setup: the uninstaller calls this so no BeamLink files stay behind
/// in the game's folders. BeamMP.zip goes too; the official launcher puts it
/// back on its next start if BeamMP is still wanted.
pub fn remove(paths: &GamePaths) -> Vec<String> {
    let mut removed = Vec::new();
    let mut remove_file = |path: std::path::PathBuf| {
        if std::fs::remove_file(&path).is_ok() {
            removed.push(path.display().to_string());
        }
    };
    if let Some(mods) = paths.mods_dir() {
        remove_file(mods.join(companion::MOD_FILE));
    }
    if let Some(mp) = paths.multiplayer_dir() {
        remove_file(mp.join(paths::client_mod_name()));
    }
    if let Some(user) = &paths.user_dir {
        let dir = user.join("settings").join("beamlink");
        if std::fs::remove_dir_all(&dir).is_ok() {
            removed.push(dir.display().to_string());
        }
    }
    removed
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_config_keeps_unknown_keys() {
        let dir = std::env::temp_dir().join(format!("beamlink-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Launcher.cfg"), r#"{"Port":1,"Custom":true}"#).unwrap();
        let settings = Settings {
            launcher_port: 4555,
            ..Default::default()
        };
        write_launcher_config(&dir, &settings).unwrap();
        let cfg: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("Launcher.cfg")).unwrap()).unwrap();
        assert_eq!(cfg["Port"], 4555);
        assert_eq!(cfg["Custom"], true);
        assert_eq!(cfg["Build"], "Default");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn branch_names_are_url_safe() {
        assert_eq!(urlencode("Default"), "Default");
        assert_eq!(urlencode("a b&c"), "a%20b%26c");
    }
}
