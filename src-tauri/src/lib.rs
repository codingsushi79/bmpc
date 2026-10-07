//! BeamLink: a desktop launcher for BeamMP.
//!
//! The UI (Svelte, in `../src`) talks to these commands. Multiplayer itself
//! is the official BeamMP stack — BeamMP-Launcher for auth and networking,
//! the BeamMP client mod in the game — which BeamLink installs, keeps up to
//! date and drives. The only thing BeamLink adds inside the game is a tiny
//! companion mod that joins the server you picked here.

mod account;
mod companion;
mod game;
mod install;
mod mods;
mod net;
mod paths;
mod servers;
mod settings;
mod zipw;

use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use paths::GamePaths;
use settings::{SavedServer, Settings};

/// The server list is ~2.7k entries; re-fetch at most this often.
const LIST_TTL: Duration = Duration::from_secs(20);

struct Inner {
    settings: Settings,
    paths: GamePaths,
    launcher: game::Launcher,
    list: Option<(Instant, Vec<servers::Server>)>,
    account: account::Account,
}

struct AppState(Mutex<Inner>);

impl AppState {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Serialize)]
struct View {
    settings: Settings,
    paths: GamePaths,
    install: install::InstallState,
    ready: bool,
    account: account::Account,
    launcher_running: bool,
    launcher_exit: Option<String>,
    game: Option<companion::GameStatus>,
    join_pending: bool,
    platform: &'static str,
    /// BeamNG.drive runs on Windows and Linux only.
    game_supported: bool,
    version: &'static str,
}

fn view(inner: &mut Inner) -> View {
    let install = install::state(&inner.paths);
    let user_dir = inner.paths.user_dir.clone();
    View {
        ready: install.ready(),
        install,
        settings: inner.settings.clone(),
        paths: inner.paths.clone(),
        account: inner.account.clone(),
        launcher_running: inner.launcher.running(),
        launcher_exit: inner.launcher.last_exit.clone(),
        game: user_dir.as_deref().and_then(companion::game_status),
        join_pending: user_dir.as_deref().is_some_and(companion::join_pending),
        platform: std::env::consts::OS,
        game_supported: cfg!(any(windows, target_os = "linux")),
        version: env!("CARGO_PKG_VERSION"),
    }
}

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

// ------------------------------------------------------------- commands ---

#[tauri::command]
fn get_state(state: State<AppState>) -> View {
    view(&mut state.lock())
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: Settings) -> Result<View, String> {
    let mut inner = state.lock();
    // The library lives in settings too; never let a stale UI copy drop
    // favorites added elsewhere.
    let mut next = settings;
    next.favorites = inner.settings.favorites.clone();
    next.recents = inner.settings.recents.clone();
    next.save()?;
    inner.paths = paths::detect(&next);
    inner.settings = next;
    Ok(view(&mut inner))
}

#[tauri::command]
async fn run_setup(app: AppHandle, state: State<'_, AppState>) -> Result<View, String> {
    let (paths, settings) = {
        let inner = state.lock();
        (inner.paths.clone(), inner.settings.clone())
    };
    let emitter = app.clone();
    let result = blocking(move || {
        install::run(&paths, &settings, &|progress| {
            let _ = emitter.emit("setup-progress", progress);
        })
    })
    .await;
    let mut inner = state.lock();
    if result.is_ok() {
        inner.settings.setup_done = true;
        let _ = inner.settings.save();
    }
    let view = view(&mut inner);
    result.map(|_| view)
}

#[tauri::command]
fn remove_mods(state: State<AppState>) -> Vec<String> {
    install::remove(&state.lock().paths)
}

#[tauri::command]
async fn fetch_servers(
    state: State<'_, AppState>,
    force: bool,
) -> Result<Vec<servers::Server>, String> {
    if !force
        && let Some((at, list)) = &state.lock().list
        && at.elapsed() < LIST_TTL
    {
        return Ok(list.clone());
    }
    let list = blocking(servers::fetch).await?;
    state.lock().list = Some((Instant::now(), list.clone()));
    Ok(list)
}

#[tauri::command]
async fn ping(targets: Vec<String>) -> Result<Vec<servers::Ping>, String> {
    blocking(move || Ok(servers::ping(targets))).await
}

/// Everything a launch needs, in order: companion present and re-enabled
/// (BeamMP turns it off whenever you join a server), then the launcher.
fn prepare_and_start(inner: &mut Inner) -> Result<(), String> {
    if let Some(mods) = inner.paths.mods_dir() {
        companion::install(&mods)?;
        companion::activate(&mods, &["multiplayerbeammp", companion::MOD_NAME])?;
    }
    let (paths, settings) = (inner.paths.clone(), inner.settings.clone());
    inner.launcher.start(&paths, &settings)
}

#[tauri::command]
fn play(state: State<AppState>) -> Result<String, String> {
    let mut inner = state.lock();
    if let Some(user) = inner.paths.user_dir.clone() {
        companion::cancel_join(&user);
    }
    if inner.launcher.running() {
        return Ok("BeamMP is already running".into());
    }
    prepare_and_start(&mut inner)?;
    Ok("Starting BeamMP — BeamNG.drive opens in a moment".into())
}

#[tauri::command]
fn join(state: State<AppState>, server: SavedServer) -> Result<String, String> {
    let mut inner = state.lock();
    let user = inner
        .paths
        .user_dir
        .clone()
        .ok_or("BeamNG.drive's user folder was not found — check Settings")?;
    companion::request_join(&user, &server.ip, server.port, &server.name)?;
    inner.settings.push_recent(server.clone());
    let _ = inner.settings.save();
    let in_game = companion::game_status(&user).is_some_and(|g| g.launcher);
    if in_game {
        return Ok(format!("Joining {} in game…", plain(&server.name)));
    }
    if inner.launcher.running() {
        // Launcher up but the game has not reported in yet: the request
        // waits for it.
        return Ok(format!(
            "Queued — BeamMP will join {} once the game is ready",
            plain(&server.name)
        ));
    }
    prepare_and_start(&mut inner)?;
    Ok(format!(
        "Starting BeamMP and joining {}",
        plain(&server.name)
    ))
}

#[tauri::command]
fn cancel_join(state: State<AppState>) {
    if let Some(user) = state.lock().paths.user_dir.clone() {
        companion::cancel_join(&user);
    }
}

#[tauri::command]
fn stop_launcher(state: State<AppState>) {
    state.lock().launcher.stop();
}

#[tauri::command]
fn launcher_log(state: State<AppState>, after: u64) -> Vec<game::LogLine> {
    state.lock().launcher.log_since(after)
}

#[tauri::command]
async fn sign_in(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<account::Account, String> {
    let dir = state.lock().paths.launcher_dir.clone();
    let account = blocking(move || account::sign_in(&dir, &username, &password)).await?;
    state.lock().account = account.clone();
    Ok(account)
}

#[tauri::command]
async fn resume_account(state: State<'_, AppState>) -> Result<account::Account, String> {
    let (dir, busy) = {
        let mut inner = state.lock();
        (inner.paths.launcher_dir.clone(), inner.launcher.running())
    };
    // The launcher rotates the key itself while it runs; two writers would
    // log each other out.
    if busy {
        return Ok(state.lock().account.clone());
    }
    let account = blocking(move || account::resume(&dir)).await?;
    state.lock().account = account.clone();
    Ok(account)
}

#[tauri::command]
fn sign_out(state: State<AppState>) {
    let mut inner = state.lock();
    account::sign_out(&inner.paths.launcher_dir);
    inner.account = account::Account::default();
}

#[tauri::command]
fn toggle_favorite(state: State<AppState>, server: SavedServer) -> Result<bool, String> {
    let mut inner = state.lock();
    let now = inner.settings.toggle_favorite(server.clone());
    inner.settings.save()?;
    if inner.settings.sync_favorites
        && let Some(user) = inner.paths.user_dir.clone()
    {
        // Best effort: the in-game list is a convenience, not the source.
        let _ = sync_game_favorite(&user, &server, now);
    }
    Ok(now)
}

/// Mirror a favorite into BeamMP's in-game list
/// (`settings/BeamMP/favorites.json`), adding or removing by address and
/// leaving everything else in that file alone.
fn sync_game_favorite(
    user_dir: &std::path::Path,
    server: &SavedServer,
    add: bool,
) -> Result<(), String> {
    let dir = user_dir.join("settings").join("BeamMP");
    let path = dir.join("favorites.json");
    let mut list: Vec<serde_json::Value> = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let same = |v: &serde_json::Value| {
        v["ip"].as_str() == Some(server.ip.as_str())
            && (v["port"].as_u64() == Some(server.port as u64)
                || v["port"].as_str() == Some(server.port.to_string().as_str()))
    };
    list.retain(|v| !same(v));
    if add {
        list.push(serde_json::json!({
            "ip": server.ip,
            "port": server.port,
            "sname": server.name,
            "map": format!("/levels/{}/info.json", server.map),
            "addTime": settings::now_ms(),
        }));
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&list).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn mods_report(state: State<AppState>) -> mods::ModsReport {
    let inner = state.lock();
    mods::report(inner.paths.mods_dir().as_deref(), &inner.paths.launcher_dir)
}

#[tauri::command]
fn set_mod_active(state: State<AppState>, name: String, active: bool) -> Result<(), String> {
    let inner = state.lock();
    if inner
        .paths
        .user_dir
        .as_deref()
        .and_then(companion::game_status)
        .is_some()
    {
        return Err(
            "close BeamNG.drive first — the game rewrites its mod list while running".into(),
        );
    }
    let mods = inner.paths.mods_dir().ok_or("no mods folder found")?;
    mods::set_active(&mods, &name, active)
}

#[tauri::command]
fn clear_cache(state: State<AppState>) -> Result<u64, String> {
    let mut inner = state.lock();
    if inner.launcher.running() {
        return Err("stop BeamMP first — the launcher is using its cache".into());
    }
    mods::clear_cache(&inner.paths.launcher_dir)
}

#[tauri::command]
fn open_folder(app: AppHandle, state: State<AppState>, which: String) -> Result<(), String> {
    let path = {
        let inner = state.lock();
        let p = &inner.paths;
        match which.as_str() {
            "game" => p.game_dir.clone(),
            "user" => p.user_dir.clone(),
            "mods" => p.mods_dir(),
            "launcher" => Some(p.launcher_dir.clone()),
            "cache" => Some(p.launcher_dir.join("Resources")),
            _ => None,
        }
    }
    .ok_or("that folder is not known yet")?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(path.display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    // Only the handful of BeamMP pages the UI links to.
    const ALLOWED: [&str; 4] = [
        "https://forum.beammp.com",
        "https://keymaster.beammp.com",
        "https://docs.beammp.com",
        "https://beammp.com",
    ];
    if !ALLOWED.iter().any(|a| url.starts_with(a)) {
        return Err("blocked link".into());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Strip `^x` colour codes for plain-text messages.
fn plain(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(|n| n.is_ascii_alphanumeric()) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

// ------------------------------------------------------------- headless ---

/// `--setup` / `--remove-mods`: what the installer runs after copying files
/// and before removing them. No window; exit code says how it went.
pub fn headless(arg: &str) -> Option<i32> {
    let settings = Settings::load();
    let paths = paths::detect(&settings);
    match arg {
        "--setup" => {
            let result = install::run(&paths, &settings, &|p| {
                if p.status != "running" {
                    println!("[{}] {}: {}", p.status, p.step, p.message);
                }
            });
            if result.is_ok() {
                let mut settings = settings;
                settings.setup_done = true;
                let _ = settings.save();
            }
            // The installer must not fail because the game is missing: the
            // app opens on its setup screen and walks the user through it.
            Some(if result.is_ok() { 0 } else { 2 })
        }
        "--remove-mods" => {
            for path in install::remove(&paths) {
                println!("removed {path}");
            }
            Some(0)
        }
        _ => None,
    }
}

pub fn run() {
    let settings = Settings::load();
    let paths = paths::detect(&settings);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(Inner {
            settings,
            paths,
            launcher: game::Launcher::default(),
            list: None,
            account: account::Account::default(),
        })))
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("BeamLink");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            save_settings,
            run_setup,
            remove_mods,
            fetch_servers,
            ping,
            play,
            join,
            cancel_join,
            stop_launcher,
            launcher_log,
            sign_in,
            resume_account,
            sign_out,
            toggle_favorite,
            mods_report,
            set_mod_active,
            clear_cache,
            open_folder,
            open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BeamLink");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_codes_do_not_leak_into_messages() {
        assert_eq!(plain("^1Red^r Server"), "Red Server");
    }

    #[test]
    fn game_favorites_are_added_and_removed_by_address() {
        let dir = std::env::temp_dir().join(format!("beamlink-fav-{}", std::process::id()));
        let fav_dir = dir.join("settings").join("BeamMP");
        std::fs::create_dir_all(&fav_dir).unwrap();
        std::fs::write(
            fav_dir.join("favorites.json"),
            r#"[{"ip":"9.9.9.9","port":"1","sname":"keep me","addTime":1}]"#,
        )
        .unwrap();
        let server = SavedServer {
            ip: "1.2.3.4".into(),
            port: 30814,
            name: "x".into(),
            map: "utah".into(),
            at: 0,
        };
        sync_game_favorite(&dir, &server, true).unwrap();
        sync_game_favorite(&dir, &server, true).unwrap();
        let list: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(fav_dir.join("favorites.json")).unwrap())
                .unwrap();
        assert_eq!(list.len(), 2);
        sync_game_favorite(&dir, &server, false).unwrap();
        let list: Vec<serde_json::Value> =
            serde_json::from_slice(&std::fs::read(fav_dir.join("favorites.json")).unwrap())
                .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["sname"], "keep me");
        let _ = std::fs::remove_dir_all(dir);
    }
}
