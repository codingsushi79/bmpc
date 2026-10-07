//! HTTP with one shared agent (connection reuse, gzip, sane timeouts).

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

pub const USER_AGENT: &str = concat!("BeamLink/", env!("CARGO_PKG_VERSION"));

fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(8)))
            .timeout_recv_response(Some(Duration::from_secs(20)))
            .user_agent(USER_AGENT)
            .build()
            .into()
    })
}

pub fn get_text(url: &str, limit: u64) -> Result<String, String> {
    let mut response = agent().get(url).call().map_err(|e| format!("{url}: {e}"))?;
    response
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_string()
        .map_err(|e| format!("{url}: {e}"))
}

pub fn post_json(url: &str, body: &serde_json::Value) -> Result<String, String> {
    let mut response = agent()
        .post(url)
        .header("Content-Type", "application/json")
        .send(body.to_string())
        .map_err(|e| format!("{url}: {e}"))?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("{url}: {e}"))
}

/// Download to `dest` (via a temp file), verifying `sha256` when given.
/// `progress(done, total)` is called as bytes arrive; total may be 0.
pub fn download(
    url: &str,
    dest: &Path,
    sha256: Option<&str>,
    mut progress: impl FnMut(u64, u64),
) -> Result<(), String> {
    let response = agent()
        .get(url)
        .config()
        .timeout_recv_body(Some(Duration::from_secs(600)))
        .build()
        .call()
        .map_err(|e| format!("{url}: {e}"))?;
    let total = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0u64);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = dest.with_extension("part");
    let mut file = std::fs::File::create(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let mut reader = response.into_body().into_reader();
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        progress(done, total);
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if let Some(expected) = sha256 {
        let actual = hex(&hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected.trim()) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!(
                "checksum mismatch for {url}: expected {expected}, got {actual}"
            ));
        }
    }
    std::fs::rename(&tmp, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

pub fn sha256_file(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Keep only hex digits: the backend's hash endpoints answer in plain text
/// and sometimes with stray whitespace or quotes.
pub fn clean_hash(text: &str) -> Option<String> {
    let hash: String = text
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect::<String>()
        .to_ascii_lowercase();
    (hash.len() == 64).then_some(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_cleaned_and_validated() {
        let h = "a".repeat(64);
        assert_eq!(clean_hash(&format!("  \"{h}\"\n")), Some(h));
        assert_eq!(clean_hash("error: no such branch"), None);
    }
}
