//! The public server list, straight from the BeamMP backend (the same list
//! the in-game browser shows), normalized once here so the UI gets clean,
//! typed rows: numbers as numbers, player and mod lists split, colour codes
//! left in place for the UI to render.

use serde::Serialize;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const LIST_URL: &str = "https://backend.beammp.com/servers-info";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Server {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub map: String,
    pub tags: Vec<String>,
    pub players: u32,
    pub max_players: u32,
    pub player_names: Vec<String>,
    pub mods: Vec<String>,
    pub mods_size: u64,
    pub version: String,
    pub client_version: String,
    pub location: String,
    pub official: bool,
    pub featured: bool,
    pub partner: bool,
    pub password: bool,
    pub guests: bool,
}

fn num(value: &serde_json::Value) -> u64 {
    match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0),
        serde_json::Value::String(s) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

fn flag(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::String(s) => s == "true" || s == "1",
        serde_json::Value::Number(n) => n.as_u64() == Some(1),
        _ => false,
    }
}

fn split(value: &str, sep: char) -> Vec<String> {
    value
        .split(sep)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// `/levels/east_coast_usa/info.json` → `east_coast_usa`.
pub fn map_name(path: &str) -> String {
    let trimmed = path.trim().trim_start_matches('/');
    let trimmed = trimmed.strip_prefix("levels/").unwrap_or(trimmed);
    trimmed.split('/').next().unwrap_or(trimmed).to_string()
}

pub fn parse(entry: &serde_json::Value) -> Option<Server> {
    let ip = text(&entry["ip"]);
    let port = num(&entry["port"]);
    if ip.is_empty() || port == 0 || port > u16::MAX as u64 {
        return None;
    }
    Some(Server {
        ip,
        port: port as u16,
        name: text(&entry["sname"]),
        description: text(&entry["sdesc"]),
        owner: text(&entry["owner"]),
        map: map_name(&text(&entry["map"])),
        tags: split(&text(&entry["tags"]), ','),
        players: num(&entry["players"]) as u32,
        max_players: num(&entry["maxplayers"]) as u32,
        player_names: split(&text(&entry["playerslist"]), ';'),
        mods: split(&text(&entry["modlist"]), ';')
            .into_iter()
            .map(|m| m.trim_start_matches('/').to_string())
            .collect(),
        mods_size: num(&entry["modstotalsize"]),
        version: text(&entry["version"]),
        client_version: text(&entry["cversion"]),
        location: text(&entry["location"]),
        official: flag(&entry["official"]),
        featured: flag(&entry["featured"]),
        partner: flag(&entry["partner"]),
        password: flag(&entry["password"]),
        guests: flag(&entry["guests"]),
    })
}

pub fn fetch() -> Result<Vec<Server>, String> {
    let body = crate::net::get_text(LIST_URL, 64 * 1024 * 1024)?;
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&body).map_err(|e| format!("server list: {e}"))?;
    let mut servers: Vec<Server> = entries.iter().filter_map(parse).collect();
    // Busiest first is what everyone wants on open; the UI re-sorts.
    servers.sort_by(|a, b| b.players.cmp(&a.players).then(a.name.cmp(&b.name)));
    Ok(servers)
}

#[derive(Debug, Clone, Serialize)]
pub struct Ping {
    pub key: String,
    pub ms: Option<u32>,
}

/// Round-trip estimate from a TCP connect to each server's game port. Runs
/// only when asked (a visible page, or one server's details), at most 32 at
/// a time, so it never sprays the whole list.
pub fn ping(targets: Vec<String>) -> Vec<Ping> {
    const PARALLEL: usize = 32;
    const TIMEOUT: Duration = Duration::from_millis(1500);
    let targets: Vec<String> = targets.into_iter().take(200).collect();
    let mut results = Vec::with_capacity(targets.len());
    for chunk in targets.chunks(PARALLEL) {
        let pings: Vec<Ping> = std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|key| {
                    scope.spawn(move || Ping {
                        key: key.clone(),
                        ms: connect_time(key, TIMEOUT),
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join().unwrap_or(Ping {
                        key: String::new(),
                        ms: None,
                    })
                })
                .collect()
        });
        results.extend(pings);
    }
    results
}

fn connect_time(target: &str, timeout: Duration) -> Option<u32> {
    let addr: SocketAddr = target.to_socket_addrs().ok()?.next()?;
    let start = Instant::now();
    let stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
    let ms = start.elapsed().as_millis() as u32;
    drop(stream);
    Some(ms.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_entries_normalize() {
        let entry = serde_json::json!({
            "owner": "me", "ip": "51.81.220.181", "port": "30882",
            "sname": "^1Red ^rServer", "sdesc": "desc",
            "map": "/levels/east_coast_usa/info.json", "tags": "Freeroam, Drift",
            "players": 14, "maxplayers": "16",
            "playerslist": "a;b;c;", "modlist": "/a.zip;/b.zip;",
            "modstotal": 2, "modstotalsize": "52000000",
            "version": "3.9.3", "cversion": "2.7.0", "location": "US",
            "official": true, "featured": "false", "password": false, "guests": true
        });
        let s = parse(&entry).unwrap();
        assert_eq!(s.port, 30882);
        assert_eq!(s.map, "east_coast_usa");
        assert_eq!(s.tags, vec!["Freeroam", "Drift"]);
        assert_eq!(s.max_players, 16);
        assert_eq!(s.player_names.len(), 3);
        assert_eq!(s.mods, vec!["a.zip", "b.zip"]);
        assert_eq!(s.mods_size, 52_000_000);
        assert!(s.official && !s.featured);
    }

    #[test]
    fn entries_without_an_address_are_dropped() {
        assert!(parse(&serde_json::json!({"sname": "x"})).is_none());
        assert!(parse(&serde_json::json!({"ip": "1.1.1.1", "port": 99999})).is_none());
    }

    #[test]
    fn map_names_from_any_shape() {
        assert_eq!(map_name("/levels/utah/info.json"), "utah");
        assert_eq!(map_name("levels/italy/"), "italy");
        assert_eq!(map_name("gridmap_v2"), "gridmap_v2");
    }

    #[test]
    fn ping_reports_unreachable_as_none() {
        // A local port nobody listens on fails fast.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let open = listener.local_addr().unwrap().to_string();
        drop(listener);
        let result = ping(vec![open]);
        assert_eq!(result.len(), 1);
        assert!(result[0].ms.is_none());
    }
}
