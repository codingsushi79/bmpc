//! BeamLink's own settings and library (favorites, recents), one JSON file in
//! the OS config directory. Passwords are never stored; the BeamMP session
//! key lives where the official launcher keeps it (see `account`).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedServer {
    pub ip: String,
    pub port: u16,
    pub name: String,
    #[serde(default)]
    pub map: String,
    /// Unix milliseconds: when favorited, or when last joined for recents.
    #[serde(default)]
    pub at: u64,
}

impl SavedServer {
    pub fn key(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Overrides for when detection gets it wrong. Empty = detect.
    pub game_dir: String,
    pub user_dir: String,
    pub launcher_dir: String,
    /// Port the BeamMP launcher listens on for the game (BeamMP default 4444).
    pub launcher_port: u16,
    /// BeamMP release branch for the client mod: Default or Public.
    pub branch: String,
    /// Extra arguments passed to BeamNG.drive.
    pub game_args: String,
    /// Mirror BeamLink favorites into BeamMP's in-game favorites.
    pub sync_favorites: bool,
    pub favorites: Vec<SavedServer>,
    pub recents: Vec<SavedServer>,
    /// Servers added by address: private, LAN, or just not listed.
    pub direct: Vec<SavedServer>,
    /// The first-run setup finished at least once.
    pub setup_done: bool,
    /// Accent colour name for the UI.
    pub accent: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            game_dir: String::new(),
            user_dir: String::new(),
            launcher_dir: String::new(),
            launcher_port: 4444,
            branch: "Default".into(),
            game_args: String::new(),
            sync_favorites: true,
            favorites: Vec::new(),
            recents: Vec::new(),
            direct: Vec::new(),
            setup_done: false,
            accent: "hesi".into(),
        }
    }
}

pub fn app_dir() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("BeamLink")
}

fn file() -> PathBuf {
    app_dir().join("settings.json")
}

impl Settings {
    pub fn load() -> Settings {
        std::fs::read(file())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let tmp = path.with_extension("tmp");
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    /// Add or remove a favorite; returns whether it is now a favorite.
    pub fn toggle_favorite(&mut self, server: SavedServer) -> bool {
        let key = server.key();
        if let Some(index) = self.favorites.iter().position(|f| f.key() == key) {
            self.favorites.remove(index);
            false
        } else {
            self.favorites.insert(
                0,
                SavedServer {
                    at: now_ms(),
                    ..server
                },
            );
            true
        }
    }

    /// Add a direct server, or rename it if the address is already saved.
    pub fn save_direct(&mut self, server: SavedServer) {
        let key = server.key();
        match self.direct.iter_mut().find(|d| d.key() == key) {
            Some(existing) => existing.name = server.name,
            None => self.direct.push(SavedServer {
                at: now_ms(),
                ..server
            }),
        }
    }

    pub fn remove_direct(&mut self, key: &str) -> bool {
        let before = self.direct.len();
        self.direct.retain(|d| d.key() != key);
        self.direct.len() != before
    }

    /// Most recent first, de-duplicated, capped at 20.
    pub fn push_recent(&mut self, server: SavedServer) {
        let key = server.key();
        self.recents.retain(|r| r.key() != key);
        self.recents.insert(
            0,
            SavedServer {
                at: now_ms(),
                ..server
            },
        );
        self.recents.truncate(20);
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(port: u16) -> SavedServer {
        SavedServer {
            ip: "1.2.3.4".into(),
            port,
            name: format!("s{port}"),
            map: String::new(),
            at: 0,
        }
    }

    #[test]
    fn favorites_toggle() {
        let mut s = Settings::default();
        assert!(s.toggle_favorite(server(1)));
        assert_eq!(s.favorites.len(), 1);
        assert!(!s.toggle_favorite(server(1)));
        assert!(s.favorites.is_empty());
    }

    #[test]
    fn direct_servers_save_rename_and_remove() {
        let mut s = Settings::default();
        s.save_direct(server(1));
        s.save_direct(SavedServer {
            name: "renamed".into(),
            ..server(1)
        });
        assert_eq!(s.direct.len(), 1);
        assert_eq!(s.direct[0].name, "renamed");
        assert!(s.remove_direct("1.2.3.4:1"));
        assert!(!s.remove_direct("1.2.3.4:1"));
    }

    #[test]
    fn recents_dedupe_and_cap() {
        let mut s = Settings::default();
        for port in 0..30 {
            s.push_recent(server(port));
        }
        s.push_recent(server(5));
        assert_eq!(s.recents.len(), 20);
        assert_eq!(s.recents[0].port, 5);
        assert_eq!(s.recents.iter().filter(|r| r.port == 5).count(), 1);
    }
}
