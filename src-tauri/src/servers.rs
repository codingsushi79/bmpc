//! The public server list, straight from the BeamMP backend (the same list
//! the in-game browser shows), normalized once here so the UI gets clean,
//! typed rows: numbers as numbers, player and mod lists split, colour codes
//! left in place for the UI to render.

use serde::Serialize;
use std::io::{Read, Write};
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

/// Round trip of BeamMP's own ping: the server answers a `P` with `P`
/// before any handshake, so this measures the game port, not just TCP.
/// Servers too old to answer fall back to the connect time.
fn connect_time(target: &str, timeout: Duration) -> Option<u32> {
    let (host, port) = parse_address(target).ok()?;
    let addr = resolve(&host, port).ok()?;
    let start = Instant::now();
    let mut stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
    let connected = start.elapsed();
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_nodelay(true);
    let sent = Instant::now();
    let mut reply = [0u8; 1];
    let ms = if stream.write_all(b"P").is_ok()
        && stream.read_exact(&mut reply).is_ok()
        && reply[0] == b'P'
    {
        sent.elapsed()
    } else {
        connected
    };
    Some((ms.as_millis() as u32).max(1))
}

pub const DEFAULT_PORT: u16 = 30814;

/// `host`, `host:port`, `[v6]` or `[v6]:port`. A bare IPv6 address (more
/// than one colon, no brackets) takes the default port.
pub fn parse_address(input: &str) -> Result<(String, u16), String> {
    let t = input.trim().trim_end_matches('/');
    let t = t.strip_prefix("beammp://").unwrap_or(t);
    if t.is_empty() {
        return Err("enter an address, like 192.168.1.20 or play.example.com:30814".into());
    }
    let port_of = |p: &str| -> Result<u16, String> {
        match p.parse::<u16>() {
            Ok(0) | Err(_) => Err(format!("`{p}` is not a valid port")),
            Ok(n) => Ok(n),
        }
    };
    let (host, port) = if let Some(rest) = t.strip_prefix('[') {
        let (host, after) = rest.split_once(']').ok_or("missing `]` in IPv6 address")?;
        let port = match after.strip_prefix(':') {
            Some(p) => port_of(p)?,
            None if after.is_empty() => DEFAULT_PORT,
            None => return Err("unexpected text after `]`".into()),
        };
        (host.to_string(), port)
    } else if t.matches(':').count() == 1 {
        let (host, p) = t.split_once(':').expect("one colon");
        (host.to_string(), port_of(p)?)
    } else {
        (t.to_string(), DEFAULT_PORT)
    };
    if host.is_empty()
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'))
    {
        return Err(format!("`{host}` is not a valid host name or IP address"));
    }
    Ok((host, port))
}

fn resolve(host: &str, port: u16) -> Result<SocketAddr, String> {
    (host, port)
        .to_socket_addrs()
        .map_err(|_| format!("couldn't find `{host}` — check the address"))?
        .next()
        .ok_or_else(|| format!("`{host}` has no address"))
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectInfo {
    pub server: Server,
    pub ping_ms: Option<u32>,
    /// False when the server answered but has its information packet
    /// switched off, so only the address is known.
    pub details: bool,
}

/// Ask a server about itself over its game port: BeamMP answers `I` with a
/// length-prefixed JSON summary (name, map, players, mods...). Works for
/// private and LAN servers that never appear on the public list.
pub fn query(address: &str) -> Result<DirectInfo, String> {
    const TIMEOUT: Duration = Duration::from_secs(4);
    let (host, port) = parse_address(address)?;
    let addr = resolve(&host, port)?;
    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT).map_err(|e| {
        format!(
            "no BeamMP server answered at {host}:{port} ({}). Is it running, and is the port open?",
            e.kind()
        )
    })?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream.write_all(b"I").map_err(|e| e.to_string())?;
    let mut size = [0u8; 4];
    stream
        .read_exact(&mut size)
        .map_err(|_| format!("{host}:{port} answered, but not like a BeamMP server"))?;
    let size = i32::from_le_bytes(size);
    if !(0..=4 * 1024 * 1024).contains(&size) {
        return Err(format!("{host}:{port} sent a malformed reply"));
    }
    let mut body = vec![0u8; size as usize];
    stream
        .read_exact(&mut body)
        .map_err(|_| format!("{host}:{port} closed the connection mid-reply"))?;
    drop(stream);
    let ping_ms = connect_time(&format_address(&host, port), Duration::from_millis(2000));
    if body.is_empty() {
        let mut server =
            parse(&serde_json::json!({"ip": host, "port": port})).expect("ip and port set");
        server.name = format_address(&host, port);
        return Ok(DirectInfo {
            server,
            ping_ms,
            details: false,
        });
    }
    let info: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|_| format!("{host}:{port} sent unreadable server info"))?;
    // The information packet uses the heartbeat's field names; map them onto
    // the public list's so one parser serves both.
    let entry = serde_json::json!({
        "ip": host,
        "port": port,
        "sname": info["name"],
        "sdesc": info["desc"],
        "map": info["map"],
        "tags": info["tags"],
        "players": info["players"],
        "maxplayers": info["maxplayers"],
        "playerslist": info["playerslist"],
        "modlist": info["modlist"],
        "modstotalsize": info["modstotalsize"],
        "version": info["version"],
        "cversion": info["clientversion"],
        "guests": info["guests"],
    });
    let server =
        parse(&entry).ok_or_else(|| format!("{host}:{port} sent incomplete server info"))?;
    Ok(DirectInfo {
        server,
        ping_ms,
        details: true,
    })
}

/// `host:port`, bracketing IPv6 so it parses back.
pub fn format_address(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
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
    fn addresses_parse_in_every_common_shape() {
        assert_eq!(parse_address("1.2.3.4").unwrap(), ("1.2.3.4".into(), 30814));
        assert_eq!(
            parse_address(" play.example.com:30900 ").unwrap(),
            ("play.example.com".into(), 30900)
        );
        assert_eq!(parse_address("[::1]:30815").unwrap(), ("::1".into(), 30815));
        assert_eq!(parse_address("fe80::1").unwrap(), ("fe80::1".into(), 30814));
        assert_eq!(
            parse_address("beammp://1.2.3.4:5").unwrap(),
            ("1.2.3.4".into(), 5)
        );
        assert!(parse_address("").is_err());
        assert!(parse_address("host:0").is_err());
        assert!(parse_address("host:abc").is_err());
        assert!(parse_address("bad host").is_err());
        assert_eq!(format_address("::1", 1), "[::1]:1");
    }

    /// A stand-in BeamMP server that speaks the `I` and `P` codes the way
    /// TNetwork::Identify does.
    fn fake_server(info: &'static str) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(2) {
                let mut stream = stream.unwrap();
                let mut code = [0u8; 1];
                stream.read_exact(&mut code).unwrap();
                match code[0] {
                    b'I' => {
                        let mut out = (info.len() as i32).to_le_bytes().to_vec();
                        out.extend_from_slice(info.as_bytes());
                        stream.write_all(&out).unwrap();
                    }
                    b'P' => stream.write_all(b"P\0").unwrap(),
                    _ => {}
                }
            }
        });
        addr
    }

    #[test]
    fn direct_query_reads_the_information_packet() {
        let addr = fake_server(
            r#"{"players":"2","maxplayers":"8","port":"30814","map":"/levels/utah/info.json","private":"true","version":"3.9.3","clientversion":"2.7.0","name":"^1LAN party","tags":"Freeroam","guests":"true","modlist":"/a.zip;","modstotalsize":"1000","modstotal":"1","playerslist":"x;y;","desc":"hi"}"#,
        );
        let info = query(&addr).unwrap();
        assert!(info.details);
        assert_eq!(info.server.name, "^1LAN party");
        assert_eq!(info.server.map, "utah");
        assert_eq!(info.server.players, 2);
        assert_eq!(info.server.player_names, vec!["x", "y"]);
        assert_eq!(info.server.mods, vec!["a.zip"]);
        assert!(info.ping_ms.is_some());
    }

    #[test]
    fn a_server_with_info_disabled_still_answers() {
        let addr = fake_server("");
        let info = query(&addr).unwrap();
        assert!(!info.details);
        assert_eq!(info.server.name, addr);
    }

    #[test]
    fn nothing_listening_is_a_clear_error() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        drop(listener);
        assert!(
            query(&addr)
                .unwrap_err()
                .contains("no BeamMP server answered")
        );
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
