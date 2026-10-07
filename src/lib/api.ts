// Calls into the Rust side. Outside Tauri (plain `vite` in a browser) a
// mock backend answers instead, so the UI can be designed and previewed
// without the desktop shell or the game.

import type {
  Account,
  LogLine,
  ModsReport,
  Ping,
  Progress,
  SavedServer,
  Server,
  Settings,
  View,
} from "./types";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  return mock(cmd, args) as Promise<T>;
}

export const api = {
  state: () => call<View>("get_state"),
  saveSettings: (settings: Settings) => call<View>("save_settings", { settings }),
  runSetup: () => call<View>("run_setup"),
  removeMods: () => call<string[]>("remove_mods"),
  servers: (force = false) => call<Server[]>("fetch_servers", { force }),
  ping: (targets: string[]) => call<Ping[]>("ping", { targets }),
  play: () => call<string>("play"),
  join: (server: SavedServer) => call<string>("join", { server }),
  cancelJoin: () => call<void>("cancel_join"),
  stopLauncher: () => call<void>("stop_launcher"),
  launcherLog: (after: number) => call<LogLine[]>("launcher_log", { after }),
  signIn: (username: string, password: string) => call<Account>("sign_in", { username, password }),
  resumeAccount: () => call<Account>("resume_account"),
  signOut: () => call<void>("sign_out"),
  toggleFavorite: (server: SavedServer) => call<boolean>("toggle_favorite", { server }),
  mods: () => call<ModsReport>("mods_report"),
  setModActive: (name: string, active: boolean) => call<void>("set_mod_active", { name, active }),
  clearCache: () => call<number>("clear_cache"),
  openFolder: (which: string) => call<void>("open_folder", { which }),
  openUrl: (url: string) => call<void>("open_url", { url }),
};

export async function onSetupProgress(handler: (p: Progress) => void): Promise<() => void> {
  if (inTauri) {
    const { listen } = await import("@tauri-apps/api/event");
    return listen<Progress>("setup-progress", (e) => handler(e.payload));
  }
  mockProgress = handler;
  return () => (mockProgress = null);
}

export function savedFrom(s: Server | SavedServer): SavedServer {
  return { ip: s.ip, port: s.port, name: s.name, map: s.map, at: "at" in s ? s.at : 0 };
}

// ---------------------------------------------------------------- mock ---

let mockProgress: ((p: Progress) => void) | null = null;

const mockSettings: Settings = {
  game_dir: "",
  user_dir: "",
  launcher_dir: "",
  launcher_port: 4444,
  branch: "Default",
  game_args: "",
  sync_favorites: true,
  favorites: [],
  recents: [],
  setup_done: false,
  accent: "hesi",
};

let mockView: View = {
  settings: mockSettings,
  paths: {
    game_dir: "C:\\Program Files (x86)\\Steam\\steamapps\\common\\BeamNG.drive",
    user_root: "C:\\Users\\you\\AppData\\Local\\BeamNG\\BeamNG.drive",
    user_dir: "C:\\Users\\you\\AppData\\Local\\BeamNG\\BeamNG.drive\\current",
    game_version: "0.39.1.0",
    launcher_dir: "C:\\Users\\you\\AppData\\Local\\BeamLink\\launcher",
    notes: ["game folder: Steam library", "user folder: default (%LOCALAPPDATA%)"],
  },
  install: {
    game_found: true,
    user_dir_exists: true,
    launcher: false,
    client_mod: false,
    companion: false,
    launcher_supported: true,
  },
  ready: false,
  account: { signed_in: false, username: null, role: null, id: null, message: null },
  launcher_running: false,
  launcher_exit: null,
  game: null,
  join_pending: false,
  platform: "windows",
  game_supported: true,
  version: "0.1.0",
};

const NAMES = [
  "^l^cNO HESI ^r^7| ^fHighway Traffic ^8#1",
  "^6Official BeamMP Server ^7| West Coast ^8(02)",
  "^bDrift Kings ^7— ^dTouge Nights",
  "^aChill Cruise ^7| No Rules, Just Vibes",
  "^4[RP] ^fLos Santos County ^7| Police / EMS",
  "^eDerby Destruction ^7| 24/7",
  "^9Offroad Expedition ^7| Utah Trails",
  "^5Rally Stage ^7| Italy Gravel",
  "^fFreeroam ^8| ^7Grid Map Playground",
  "^cDrag Strip ^7| 1/4 Mile Shootout",
  "^3Truckers Union ^7| Industrial Logistics",
  "^dJDM Meet ^7| Hirochi Raceway",
];
const MAPS = [
  "west_coast_usa", "east_coast_usa", "utah", "italy", "gridmap_v2",
  "jungle_rock_island", "industrial", "hirochi_raceway", "johnson_valley", "derby",
];
const LOCS = ["US", "DE", "GB", "FR", "NL", "PL", "BR", "AU", "CA", "SE", "JP", "FI"];

function rand(seed: number) {
  let s = seed;
  return () => ((s = (s * 1103515245 + 12345) % 2147483648) / 2147483648);
}

const mockServers: Server[] = (() => {
  const r = rand(7);
  const list: Server[] = [];
  for (let i = 0; i < 420; i++) {
    const max = [8, 10, 16, 20, 24, 32, 50][Math.floor(r() * 7)];
    const players = r() < 0.35 ? 0 : Math.min(max, Math.floor(r() ** 1.6 * max + 1));
    const mods = r() < 0.5 ? [] : Array.from({ length: Math.ceil(r() * 12) }, (_, m) => `pack_${i}_${m}.zip`);
    list.push({
      ip: `51.${(i * 7) % 255}.${(i * 13) % 255}.${(i * 29) % 255}`,
      port: 30814 + (i % 40),
      name: NAMES[i % NAMES.length] + (i >= NAMES.length ? ` ^8#${i}` : ""),
      description: "^7Welcome! ^fRespect others^7, no ramming outside events.^p^bDiscord: ^fdiscord.gg/example",
      owner: `host${i}`,
      map: MAPS[Math.floor(r() * MAPS.length)],
      tags: ["Freeroam", "Drift", "Racing", "Roleplay", "Traffic", "Offroad"].filter(() => r() < 0.3),
      players,
      max_players: max,
      player_names: Array.from({ length: players }, (_, p) => (p === 0 && i === 4 ? "you_friend" : `driver${(i * 17 + p) % 9999}`)),
      mods,
      mods_size: mods.length * Math.floor(r() * 180_000_000),
      version: "3.9.3",
      client_version: "2.7.0",
      location: LOCS[Math.floor(r() * LOCS.length)],
      official: i % NAMES.length === 1,
      featured: i < 3,
      partner: i === 0,
      password: r() < 0.04,
      guests: r() < 0.85,
    });
  }
  return list.sort((a, b) => b.players - a.players);
})();

const mockLog: LogLine[] = [];

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function mock(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  await sleep(120);
  switch (cmd) {
    case "get_state":
      return structuredClone(mockView);
    case "save_settings":
      mockView = { ...mockView, settings: { ...(args!.settings as Settings), favorites: mockView.settings.favorites, recents: mockView.settings.recents } };
      return structuredClone(mockView);
    case "run_setup": {
      for (const step of ["game", "launcher", "client", "companion", "activate"]) {
        mockProgress?.({ step, status: "running", message: `Working on ${step}`, done: 0, total: 0 });
        for (let p = 0; p <= 10 && (step === "launcher" || step === "client"); p++) {
          mockProgress?.({ step, status: "running", message: "Downloading", done: p * 1e6, total: 10e6 });
          await sleep(60);
        }
        await sleep(200);
        mockProgress?.({ step, status: "done", message: "done", done: 0, total: 0 });
      }
      mockView.install = { ...mockView.install, launcher: true, client_mod: true, companion: true };
      mockView.ready = true;
      mockView.settings.setup_done = true;
      return structuredClone(mockView);
    }
    case "fetch_servers":
      return mockServers;
    case "ping":
      return (args!.targets as string[]).map((key, i) => ({ key, ms: i % 9 === 8 ? null : 18 + ((i * 37) % 160) }));
    case "play":
      mockView.launcher_running = true;
      mockLog.push({ seq: mockLog.length + 1, text: "[INFO] Game Launched!" });
      return "Starting BeamMP — BeamNG.drive opens in a moment";
    case "join": {
      const s = args!.server as SavedServer;
      mockView.launcher_running = true;
      mockView.settings.recents = [{ ...s, at: Date.now() }, ...mockView.settings.recents.filter((r) => r.ip !== s.ip || r.port !== s.port)].slice(0, 20);
      return `Starting BeamMP and joining ${s.name.replace(/\^./g, "")}`;
    }
    case "toggle_favorite": {
      const s = args!.server as SavedServer;
      const favs = mockView.settings.favorites;
      const idx = favs.findIndex((f) => f.ip === s.ip && f.port === s.port);
      if (idx >= 0) favs.splice(idx, 1);
      else favs.unshift({ ...s, at: Date.now() });
      return idx < 0;
    }
    case "stop_launcher":
      mockView.launcher_running = false;
      return null;
    case "launcher_log":
      return mockLog.filter((l) => l.seq > (args!.after as number));
    case "sign_in":
      if (!(args!.password as string)) throw "enter your BeamMP username and password";
      mockView.account = { signed_in: true, username: args!.username as string, role: "USER", id: 1, message: null };
      return mockView.account;
    case "resume_account":
      return mockView.account;
    case "sign_out":
      mockView.account = { signed_in: false, username: null, role: null, id: null, message: null };
      return null;
    case "mods_report":
      return {
        mods: [
          { file: "beamlink.zip", name: "beamlink", bytes: 4096, active: true, managed: true, location: "mods" },
          { file: "BeamMP.zip", name: "multiplayerbeammp", bytes: 9_800_000, active: true, managed: true, location: "multiplayer" },
          { file: "custom_drift_pack.zip", name: "custom_drift_pack", bytes: 312_000_000, active: true, managed: false, location: "mods" },
          { file: "old_map.zip", name: "old_map", bytes: 1_200_000_000, active: false, managed: false, location: "mods" },
        ],
        cache_bytes: 4_300_000_000,
        cache_files: 213,
        mods_dir: mockView.paths.user_dir + "\\mods",
        cache_dir: mockView.paths.launcher_dir + "\\Resources",
      } satisfies ModsReport;
    case "clear_cache":
      return 4_300_000_000;
    default:
      return null;
  }
}
