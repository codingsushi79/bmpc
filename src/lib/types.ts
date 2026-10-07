export interface Server {
  ip: string;
  port: number;
  name: string;
  description: string;
  owner: string;
  map: string;
  tags: string[];
  players: number;
  max_players: number;
  player_names: string[];
  mods: string[];
  mods_size: number;
  version: string;
  client_version: string;
  location: string;
  official: boolean;
  featured: boolean;
  partner: boolean;
  password: boolean;
  guests: boolean;
}

export interface SavedServer {
  ip: string;
  port: number;
  name: string;
  map: string;
  at: number;
}

export interface Settings {
  game_dir: string;
  user_dir: string;
  launcher_dir: string;
  launcher_port: number;
  branch: string;
  game_args: string;
  sync_favorites: boolean;
  favorites: SavedServer[];
  recents: SavedServer[];
  setup_done: boolean;
  accent: string;
}

export interface GamePaths {
  game_dir: string | null;
  user_root: string | null;
  user_dir: string | null;
  game_version: string | null;
  launcher_dir: string;
  notes: string[];
}

export interface InstallState {
  game_found: boolean;
  user_dir_exists: boolean;
  launcher: boolean;
  client_mod: boolean;
  companion: boolean;
  launcher_supported: boolean;
}

export interface Account {
  signed_in: boolean;
  username: string | null;
  role: string | null;
  id: number | null;
  message: string | null;
}

export interface GameStatus {
  time: number;
  beammp: boolean;
  launcher: boolean;
  session: boolean;
  version: string | null;
  server: { ip: string | null; port: number | null; name: string | null } | null;
}

export interface View {
  settings: Settings;
  paths: GamePaths;
  install: InstallState;
  ready: boolean;
  account: Account;
  launcher_running: boolean;
  launcher_exit: string | null;
  game: GameStatus | null;
  join_pending: boolean;
  platform: string;
  game_supported: boolean;
  version: string;
}

export interface Progress {
  step: string;
  status: "running" | "done" | "skipped" | "error";
  message: string;
  done: number;
  total: number;
}

export interface Ping {
  key: string;
  ms: number | null;
}

export interface LogLine {
  seq: number;
  text: string;
}

export interface LocalMod {
  file: string;
  name: string;
  bytes: number;
  active: boolean | null;
  managed: boolean;
  location: string;
}

export interface ModsReport {
  mods: LocalMod[];
  cache_bytes: number;
  cache_files: number;
  mods_dir: string | null;
  cache_dir: string;
}
