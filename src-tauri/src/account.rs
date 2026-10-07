//! BeamMP accounts, done the way the official launcher does them: one call
//! to auth.beammp.com, and the returned session key saved as `key` in the
//! launcher's folder. The launcher then signs in on its own when the game
//! starts. The password is sent once, over HTTPS, and never stored.

use serde::Serialize;
use std::path::Path;

const AUTH: &str = "https://auth.beammp.com/userlogin";

#[derive(Debug, Clone, Serialize, Default)]
pub struct Account {
    pub signed_in: bool,
    pub username: Option<String>,
    pub role: Option<String>,
    pub id: Option<i64>,
    pub message: Option<String>,
}

fn key_file(launcher_dir: &Path) -> std::path::PathBuf {
    launcher_dir.join("key")
}

/// Same rule as the launcher: letters, digits and dashes only.
fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() < 100 && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn handle(launcher_dir: &Path, reply: &str) -> Result<Account, String> {
    let json: serde_json::Value = serde_json::from_str(reply)
        .map_err(|_| "the BeamMP auth service sent an unexpected reply".to_string())?;
    let message = json["message"].as_str().map(str::to_string);
    if json["success"].as_bool() != Some(true) {
        return Ok(Account {
            signed_in: false,
            message: message.or(Some("sign-in failed".into())),
            ..Default::default()
        });
    }
    if let Some(key) = json["private_key"].as_str().filter(|k| valid_key(k)) {
        std::fs::create_dir_all(launcher_dir).map_err(|e| e.to_string())?;
        std::fs::write(key_file(launcher_dir), key).map_err(|e| e.to_string())?;
    }
    Ok(Account {
        signed_in: true,
        username: json["username"].as_str().map(str::to_string),
        role: json["role"].as_str().map(str::to_string),
        id: json["id"].as_i64(),
        message,
    })
}

pub fn sign_in(launcher_dir: &Path, username: &str, password: &str) -> Result<Account, String> {
    if username.trim().is_empty() || password.is_empty() {
        return Err("enter your BeamMP username and password".into());
    }
    let reply = crate::net::post_json(
        AUTH,
        &serde_json::json!({ "username": username.trim(), "password": password }),
    )?;
    handle(launcher_dir, &reply)
}

/// Check the saved key. The service answers with a fresh key each time, so
/// it is written back, exactly as the launcher does on start.
pub fn resume(launcher_dir: &Path) -> Result<Account, String> {
    let Ok(key) = std::fs::read_to_string(key_file(launcher_dir)) else {
        return Ok(Account::default());
    };
    let key = key.trim();
    if !valid_key(key) {
        let _ = std::fs::remove_file(key_file(launcher_dir));
        return Ok(Account::default());
    }
    let reply = crate::net::post_json(AUTH, &serde_json::json!({ "pk": key }))?;
    let account = handle(launcher_dir, &reply)?;
    if !account.signed_in {
        let _ = std::fs::remove_file(key_file(launcher_dir));
    }
    Ok(account)
}

pub fn sign_out(launcher_dir: &Path) {
    let _ = std::fs::remove_file(key_file(launcher_dir));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_replies_store_the_key_and_failures_do_not() {
        let dir = std::env::temp_dir().join(format!("beamlink-acct-{}", std::process::id()));
        let ok = handle(
            &dir,
            r#"{"success":true,"username":"drifter","role":"USER","id":7,"private_key":"abc-123","public_key":"x"}"#,
        )
        .unwrap();
        assert!(ok.signed_in);
        assert_eq!(ok.username.as_deref(), Some("drifter"));
        assert_eq!(std::fs::read_to_string(dir.join("key")).unwrap(), "abc-123");

        sign_out(&dir);
        let bad = handle(&dir, r#"{"success":false,"message":"Invalid password"}"#).unwrap();
        assert!(!bad.signed_in);
        assert_eq!(bad.message.as_deref(), Some("Invalid password"));
        assert!(!dir.join("key").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn keys_with_odd_characters_are_rejected() {
        assert!(valid_key("abc-DEF-123"));
        assert!(!valid_key("../../etc"));
        assert!(!valid_key(""));
    }
}
