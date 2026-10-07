//! The companion BeamNG mod: packaging, installing, and the two small files
//! it shares with BeamLink (`autojoin.json` in, `status.json` out), both in
//! `<user>/current/settings/beamlink/`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::zipw;

pub const MOD_NAME: &str = "beamlink";
pub const MOD_FILE: &str = "beamlink.zip";

const EXTENSION: &str = include_str!("../companion/lua/ge/extensions/beamlink.lua");
const MOD_SCRIPT: &str = include_str!("../companion/scripts/beamlink/modScript.lua");
const INFO: &str = include_str!("../companion/info.json");

pub fn archive() -> Vec<u8> {
    zipw::build(&[
        ("lua/ge/extensions/beamlink.lua", EXTENSION.as_bytes()),
        ("scripts/beamlink/modScript.lua", MOD_SCRIPT.as_bytes()),
        ("info.json", INFO.as_bytes()),
    ])
}

fn dir(user_dir: &Path) -> PathBuf {
    user_dir.join("settings").join("beamlink")
}

/// Write the zip into `mods/` unless an identical one is already there.
/// Returns true when something was written.
pub fn install(mods_dir: &Path) -> Result<bool, String> {
    let bytes = archive();
    let path = mods_dir.join(MOD_FILE);
    if std::fs::read(&path).is_ok_and(|existing| existing == bytes) {
        return Ok(false);
    }
    std::fs::create_dir_all(mods_dir).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("part");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn installed(mods_dir: &Path) -> bool {
    std::fs::read(mods_dir.join(MOD_FILE)).is_ok_and(|b| b == archive())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRequest {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub created: u64,
}

pub fn request_join(user_dir: &Path, ip: &str, port: u16, name: &str) -> Result<(), String> {
    let dir = dir(user_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let request = JoinRequest {
        ip: ip.into(),
        port,
        name: name.into(),
        created: crate::settings::now_ms() / 1000,
    };
    let tmp = dir.join("autojoin.tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec(&request).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, dir.join("autojoin.json")).map_err(|e| e.to_string())
}

pub fn cancel_join(user_dir: &Path) {
    let _ = std::fs::remove_file(dir(user_dir).join("autojoin.json"));
}

pub fn join_pending(user_dir: &Path) -> bool {
    dir(user_dir).join("autojoin.json").is_file()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStatus {
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub beammp: bool,
    #[serde(default)]
    pub launcher: bool,
    #[serde(default)]
    pub session: bool,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub server: Option<CurrentServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrentServer {
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default, deserialize_with = "lenient_port")]
    pub port: Option<u16>,
    #[serde(default)]
    pub name: Option<String>,
}

fn lenient_port<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u16>, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    Ok(match value {
        serde_json::Value::Number(n) => n.as_u64().map(|n| n as u16),
        serde_json::Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

/// What the game last reported, if it reported in the last few seconds.
pub fn game_status(user_dir: &Path) -> Option<GameStatus> {
    let bytes = std::fs::read(dir(user_dir).join("status.json")).ok()?;
    let status: GameStatus = serde_json::from_slice(&bytes).ok()?;
    let now = crate::settings::now_ms() / 1000;
    (now.saturating_sub(status.time) <= 6).then_some(status)
}

/// Mark mods active in BeamNG's `mods/db.json`, the way the BeamMP
/// launcher re-enables itself. BeamMP deactivates every mod a server did not
/// send when you join one, which includes the companion; this undoes that
/// before each launch. A missing db.json is fine: new mods start active.
pub fn activate(mods_dir: &Path, names: &[&str]) -> Result<(), String> {
    let path = mods_dir.join("db.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(());
    };
    let Ok(mut db) = serde_json::from_str::<serde_json::Value>(&text) else {
        // Never rewrite a file we could not read; the game owns it.
        return Ok(());
    };
    let mut changed = false;
    if let Some(mods) = db.get_mut("mods").and_then(|m| m.as_object_mut()) {
        for name in names {
            if let Some(entry) = mods.get_mut(*name).and_then(|e| e.as_object_mut())
                && entry.get("active") != Some(&serde_json::Value::Bool(true))
            {
                entry.insert("active".into(), serde_json::Value::Bool(true));
                changed = true;
            }
        }
    }
    if changed {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec(&db).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> PathBuf {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "beamlink-test-{}-{}-{n}",
            std::process::id(),
            crate::settings::now_ms()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn install_is_idempotent() {
        let dir = temp();
        assert!(install(&dir).unwrap());
        assert!(!install(&dir).unwrap());
        assert!(installed(&dir));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn deactivated_mods_are_reactivated_and_others_left_alone() {
        let dir = temp();
        std::fs::write(
            dir.join("db.json"),
            r#"{"mods":{"beamlink":{"active":false,"x":1},"other":{"active":false}}}"#,
        )
        .unwrap();
        activate(&dir, &["beamlink", "missing"]).unwrap();
        let db: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("db.json")).unwrap()).unwrap();
        assert_eq!(db["mods"]["beamlink"]["active"], true);
        assert_eq!(db["mods"]["beamlink"]["x"], 1);
        assert_eq!(db["mods"]["other"]["active"], false);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn join_requests_round_trip_and_status_goes_stale() {
        let dir = temp();
        request_join(&dir, "1.2.3.4", 30814, "Test").unwrap();
        assert!(join_pending(&dir));
        cancel_join(&dir);
        assert!(!join_pending(&dir));

        let status_dir = dir.join("settings").join("beamlink");
        std::fs::write(
            status_dir.join("status.json"),
            format!(
                r#"{{"time":{},"beammp":true,"launcher":true,"session":true,"server":{{"ip":"1.2.3.4","port":"30814","name":"x"}}}}"#,
                crate::settings::now_ms() / 1000
            ),
        )
        .unwrap();
        let status = game_status(&dir).unwrap();
        assert_eq!(status.server.unwrap().port, Some(30814));
        std::fs::write(status_dir.join("status.json"), r#"{"time":5}"#).unwrap();
        assert!(game_status(&dir).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
