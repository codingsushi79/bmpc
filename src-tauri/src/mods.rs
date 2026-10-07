//! The Mods page: what is in BeamNG's mods folder, which are enabled, and
//! the launcher's cache of server mods (which grows with every server you
//! visit and is safe to clear).

use serde::Serialize;
use std::path::Path;

use crate::companion;

#[derive(Debug, Clone, Serialize)]
pub struct LocalMod {
    pub file: String,
    /// The name BeamNG keys it by in db.json (lower case, no extension).
    pub name: String,
    pub bytes: u64,
    pub active: Option<bool>,
    /// BeamLink or BeamMP's own: shown, but not toggleable.
    pub managed: bool,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ModsReport {
    pub mods: Vec<LocalMod>,
    pub cache_bytes: u64,
    pub cache_files: u64,
    pub mods_dir: Option<String>,
    pub cache_dir: String,
}

fn db(mods_dir: &Path) -> serde_json::Value {
    std::fs::read(mods_dir.join("db.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(serde_json::Value::Null)
}

pub fn report(mods_dir: Option<&Path>, launcher_dir: &Path) -> ModsReport {
    let mut report = ModsReport {
        cache_dir: launcher_dir.join("Resources").display().to_string(),
        ..Default::default()
    };
    let (bytes, files) = dir_size(&launcher_dir.join("Resources"));
    report.cache_bytes = bytes;
    report.cache_files = files;
    let Some(mods_dir) = mods_dir else {
        return report;
    };
    report.mods_dir = Some(mods_dir.display().to_string());
    let db = db(mods_dir);
    for (folder, location) in [
        (mods_dir.to_path_buf(), "mods"),
        (mods_dir.join("multiplayer"), "multiplayer"),
        (mods_dir.join("repo"), "repository"),
    ] {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let file = entry.file_name().to_string_lossy().into_owned();
            if !file.to_ascii_lowercase().ends_with(".zip") {
                continue;
            }
            let name = file[..file.len() - 4].to_ascii_lowercase();
            let key = if location == "multiplayer" {
                format!("multiplayer{name}")
            } else {
                name.clone()
            };
            let active = db["mods"][&key]["active"]
                .as_bool()
                .or_else(|| db["mods"][&name]["active"].as_bool());
            report.mods.push(LocalMod {
                managed: name == companion::MOD_NAME || location == "multiplayer",
                bytes: entry.metadata().map(|m| m.len()).unwrap_or(0),
                file,
                name: key,
                active,
                location: location.into(),
            });
        }
    }
    report.mods.sort_by(|a, b| {
        a.managed
            .cmp(&b.managed)
            .then(a.file.to_lowercase().cmp(&b.file.to_lowercase()))
    });
    report
}

/// Flip a mod's `active` flag in db.json. Only while the game is closed —
/// the game rewrites db.json itself while running.
pub fn set_active(mods_dir: &Path, name: &str, active: bool) -> Result<(), String> {
    let path = mods_dir.join("db.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|_| "BeamNG has not created its mod list yet — start the game once".to_string())?;
    let mut db: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("mods/db.json: {e}"))?;
    let entry = db["mods"]
        .get_mut(name)
        .and_then(|e| e.as_object_mut())
        .ok_or_else(|| format!("`{name}` is not in BeamNG's mod list yet — start the game once"))?;
    entry.insert("active".into(), active.into());
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec(&db).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// Delete the launcher's downloaded server mods. They are fetched again
/// from a server when you next join it.
pub fn clear_cache(launcher_dir: &Path) -> Result<u64, String> {
    let dir = launcher_dir.join("Resources");
    let (bytes, _) = dir_size(&dir);
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    Ok(bytes)
}

fn dir_size(dir: &Path) -> (u64, u64) {
    let mut bytes = 0;
    let mut files = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            match entry.file_type() {
                Ok(t) if t.is_dir() => stack.push(entry.path()),
                Ok(t) if t.is_file() => {
                    files += 1;
                    bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    (bytes, files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_reads_activity_from_db_json() {
        let root = std::env::temp_dir().join(format!("beamlink-mods-{}", std::process::id()));
        let mods = root.join("mods");
        std::fs::create_dir_all(mods.join("multiplayer")).unwrap();
        std::fs::write(mods.join("Cool_Car.zip"), b"PK").unwrap();
        std::fs::write(mods.join("multiplayer").join("BeamMP.zip"), b"PK").unwrap();
        std::fs::write(
            mods.join("db.json"),
            r#"{"mods":{"cool_car":{"active":false},"multiplayerbeammp":{"active":true}}}"#,
        )
        .unwrap();
        let report = report(Some(&mods), &root.join("launcher"));
        let car = report
            .mods
            .iter()
            .find(|m| m.file == "Cool_Car.zip")
            .unwrap();
        assert_eq!(car.active, Some(false));
        assert!(!car.managed);
        let beammp = report
            .mods
            .iter()
            .find(|m| m.location == "multiplayer")
            .unwrap();
        assert_eq!(beammp.active, Some(true));
        assert!(beammp.managed);

        set_active(&mods, "cool_car", true).unwrap();
        let again = super::report(Some(&mods), &root.join("launcher"));
        assert_eq!(
            again
                .mods
                .iter()
                .find(|m| m.name == "cool_car")
                .unwrap()
                .active,
            Some(true)
        );
        assert!(set_active(&mods, "nope", true).is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
