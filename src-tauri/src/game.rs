//! Running the official BeamMP launcher (which in turn starts BeamNG.drive
//! and proxies the game's multiplayer traffic), and keeping its output.
//!
//! BeamLink starts the launcher hidden, in its own folder (it reads
//! `Launcher.cfg` and `key` from the working directory), captures its
//! console into a ring buffer for the log drawer, and notices when it exits.

use serde::Serialize;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::paths::GamePaths;
use crate::settings::Settings;

const LOG_CAP: usize = 2000;

#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub seq: u64,
    pub text: String,
}

#[derive(Default)]
pub struct Launcher {
    child: Option<Child>,
    log: Arc<Mutex<(VecDeque<LogLine>, u64)>>,
    pub last_exit: Option<String>,
}

impl Launcher {
    pub fn running(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };
        match child.try_wait() {
            Ok(None) => true,
            Ok(Some(status)) => {
                self.last_exit = Some(match status.code() {
                    Some(0) => "the launcher closed (game exited)".into(),
                    Some(code) => format!("the launcher exited with code {code}"),
                    None => "the launcher was stopped".into(),
                });
                self.child = None;
                false
            }
            Err(_) => false,
        }
    }

    pub fn start(&mut self, paths: &GamePaths, settings: &Settings) -> Result<(), String> {
        if self.running() {
            return Ok(());
        }
        let exe = paths.launcher_exe();
        if !exe.is_file() {
            return Err(format!(
                "BeamMP-Launcher is not installed ({}). Run setup from Settings.",
                exe.display()
            ));
        }
        crate::install::write_launcher_config(&paths.launcher_dir, settings)?;
        let mut command = Command::new(&exe);
        command
            .current_dir(&paths.launcher_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if settings.launcher_port != 4444 {
            command.args(["--port", &settings.launcher_port.to_string()]);
        }
        if !settings.user_dir.trim().is_empty()
            && let Some(root) = &paths.user_root
        {
            command.arg("--user-path").arg(root);
        }
        let game_args: Vec<&str> = settings.game_args.split_whitespace().collect();
        if !game_args.is_empty() {
            command.arg("--").args(game_args);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = command
            .spawn()
            .map_err(|e| format!("starting {}: {e}", exe.display()))?;
        for pipe in [
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let log = self.log.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(pipe);
                let mut buf = Vec::new();
                loop {
                    buf.clear();
                    match reader.read_until(b'\n', &mut buf) {
                        Ok(0) | Err(_) => return,
                        Ok(_) => {}
                    }
                    let text = strip_ansi(String::from_utf8_lossy(&buf).trim_end());
                    if text.is_empty() {
                        continue;
                    }
                    let mut guard = log.lock().unwrap_or_else(|e| e.into_inner());
                    let (lines, seq) = &mut *guard;
                    *seq += 1;
                    if lines.len() == LOG_CAP {
                        lines.pop_front();
                    }
                    lines.push_back(LogLine { seq: *seq, text });
                }
            });
        }
        self.push_note("── BeamLink started BeamMP-Launcher ──");
        self.child = Some(child);
        self.last_exit = None;
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
            self.push_note("── BeamLink stopped BeamMP-Launcher ──");
        }
    }

    fn push_note(&self, text: &str) {
        let mut guard = self.log.lock().unwrap_or_else(|e| e.into_inner());
        let (lines, seq) = &mut *guard;
        *seq += 1;
        if lines.len() == LOG_CAP {
            lines.pop_front();
        }
        lines.push_back(LogLine {
            seq: *seq,
            text: text.into(),
        });
    }

    /// Lines newer than `after`.
    pub fn log_since(&self, after: u64) -> Vec<LogLine> {
        let guard = self.log.lock().unwrap_or_else(|e| e.into_inner());
        guard.0.iter().filter(|l| l.seq > after).cloned().collect()
    }
}

impl Drop for Launcher {
    /// Closing BeamLink does not take the game down with it: the launcher
    /// is left running so an ongoing session continues.
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            std::mem::forget(child);
        }
    }
}

pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.clone().next() == Some('[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_codes_are_removed_from_launcher_output() {
        assert_eq!(
            strip_ansi("\u{1b}[32m[INFO]\u{1b}[0m Game Launched!"),
            "[INFO] Game Launched!"
        );
    }

    #[test]
    fn a_missing_launcher_is_a_clear_error() {
        let mut launcher = Launcher::default();
        let paths = GamePaths {
            launcher_dir: std::env::temp_dir().join("beamlink-no-launcher-here"),
            ..Default::default()
        };
        let err = launcher.start(&paths, &Settings::default()).unwrap_err();
        assert!(err.contains("not installed"), "{err}");
        assert!(!launcher.running());
    }
}
