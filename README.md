# BeamLink

**A desktop launcher for BeamMP: a fast server browser, one-click join,
favorites, account and mod management, in a dark neon UI.**

BeamLink runs on the official BeamMP stack rather than replacing it.
BeamMP's multiplayer relies on its launcher for sign-in and for relaying the
game's traffic to servers, and on its client mod inside BeamNG.drive.
Rewriting either would cut you off from the real server network, so BeamLink
installs, updates and drives the official pieces, and puts its own UI in front
of them.

```
BeamLink (this app)                     BeamNG.drive
  server browser, library, mods           ├── BeamMP client mod  (mods/multiplayer/BeamMP.zip)
  account, settings, setup                └── BeamLink companion (mods/beamlink.zip)
        │                                              ▲
        │ starts, configures, reads its log            │ autojoin.json / status.json
        ▼                                              │ (settings/beamlink/)
  BeamMP-Launcher (official) ◀── localhost:4444 ──▶ game
        │
        └── auth.beammp.com, backend.beammp.com, game servers
```

## Installing

Download the latest installer from
[Releases](https://github.com/codingsushi79/bmpc/releases/latest) and run
`BeamLink_x.y.z_x64-setup.exe`. After copying files it
runs `BeamLink --setup`, which:

1. finds BeamNG.drive (BeamNG's registry key, then every Steam library that
   has app 284160) and its user folder (`startup.ini` → `BeamNG.Drive.ini` →
   `%LOCALAPPDATA%\BeamNG\BeamNG.drive`). This is the same lookup order the
   official launcher uses.
2. downloads the official **BeamMP-Launcher** from GitHub and checks it
   against the published `.sha256`.
3. downloads the **BeamMP client mod** from the BeamMP backend into
   `<user folder>\current\mods\multiplayer\BeamMP.zip`, checked against the
   backend's sha256. The launcher runs the same check.
4. writes the **BeamLink companion** to `<user folder>\current\mods\beamlink.zip`.
5. marks both active in `mods\db.json`.

If the game isn't found during install (for example it's on an unusual drive),
the app opens on its setup screen, where you can paste the game folder and
finish. **Settings → Run setup / repair** reruns the same steps. Uninstalling
runs `BeamLink --remove-mods` first, so no BeamLink files are left in the
game folders.

## Updates

Installed copies update themselves. On start and every 6 hours BeamLink
checks the latest release's `latest.json`. When a new version is out, an
**Update to x.y.z** button appears in the title bar, and Settings → Updates
has the same option. The installer is checked against the public key built
into the app before it runs, so only builds signed with the project's key
are accepted. Windows updates install silently and the app restarts into
the new version.

To publish an update, bump the version in `src-tauri/tauri.conf.json`,
`src-tauri/Cargo.toml` and `package.json`, then push a tag:

```bash
git tag v0.2.1 && git push origin v0.2.1
```

CI signs the build with the `TAURI_SIGNING_PRIVATE_KEY` repository secret.

## Features

- **Server browser** for the full public list (~2,700 servers), virtualised
  so only the visible rows are drawn. Search covers server name, map, owner,
  tags **and player names**, so you can find a friend and see which server
  they're on. Filters: has players, not full, no mods, official, featured and
  partner, favorites, locked servers, map, region. Sorting: players, name,
  map, mod size and ping.
- **Ping on demand.** Uses BeamMP's own `P` ping on each server's game port,
  only for the rows on screen or the server you open, at most 32 at a time.
- **Server details:** description with BeamMP `^` colour codes rendered,
  who's online, tags, every mod with its total download size (big packs are
  flagged), version, region and address.
- **One-click join.** Join writes a request to `settings/beamlink/autojoin.json`
  and starts BeamMP if needed. Once BeamMP is connected to its launcher, the
  companion mod calls BeamMP's own `MPCoreNetwork.connectToServer`, so mod
  downloads and the mod-security prompt work as normal. If the game is
  already running, the join happens in place.
- **Library:** favorites and recently played, with live player counts.
  Favorites are also copied into BeamMP's in-game favorites (can be turned off).
- **Direct connect** for any server, listed or not: private, LAN or
  invite-only. Type an address (`host`, `host:port`, `[v6]:port`) and BeamLink
  asks the server itself for its name, map, players, mods and version, using
  the same `I` information packet BeamMP's own browser relies on. Then join.
  Save servers to a list that shows each one's live status. This works for
  servers hosted with [beamhost](https://github.com/codingsushi79/bmps).
- **Account:** sign in with your BeamMP account (one HTTPS call to
  auth.beammp.com). The session key is saved as `key` in the launcher folder,
  where the official launcher expects it, so the game signs you in on its own.
  Your password is never stored. Guest play works too.
- **Mods:** your own mods with enable/disable (edits `db.json` while the game
  is closed), the BeamMP and companion mods, and the launcher's cache of
  downloaded server mods with its size and a Clear cache button.
- **Live status:** the status bar shows whether the launcher is running,
  whether the game is connected, and which server you're on. The game side
  of that comes from the companion's `status.json`. The log drawer streams
  the launcher's console.

## Platforms

| | Browse servers | Play |
|---|---|---|
| Windows | yes | yes (the official launcher is Windows-only) |
| Linux | yes | yes, with a self-built `BeamMP-Launcher` in the launcher folder (BeamMP doesn't publish Linux launcher builds) |
| macOS | yes | no. BeamNG.drive doesn't run on macOS, so BeamLink is browse-only there |

## Building

```bash
npm install
npm run tauri dev                         # run the desktop app
npx tauri build --bundles nsis            # Windows installer (on Windows)
npx tauri build --bundles deb,appimage    # Linux
npm run dev                               # UI only, in a browser, with a mock backend
```

The workflow in `.github/workflows/build.yml` runs the tests and builds the
Windows installer, Linux packages and a macOS dmg on every push. Pushing a
version tag also publishes them as a GitHub Release:

```bash
git tag v0.1.1 && git push origin v0.1.1
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # 24 tests
npm run check                                     # svelte-check / TypeScript
```

## Layout

```
src/                      Svelte 5 UI
  components/             pages, drawer, setup wizard, hero art
  lib/api.ts              Tauri commands (+ mock backend for the browser)
  lib/store.svelte.ts     app state (runes)
  lib/beam.ts             colour codes, map names, flags, sizes
src-tauri/
  src/paths.rs            game + user folder detection (registry, Steam libraries, ini files)
  src/install.rs          setup/repair/remove
  src/companion.rs        companion mod packaging, autojoin/status files, db.json activation
  src/servers.rs          server list normalisation, ping
  src/game.rs             launcher process + log capture
  src/account.rs          BeamMP sign-in via the official auth endpoint
  src/mods.rs             mods report, enable/disable, cache
  src/zipw.rs             tiny zip writer for the companion
  companion/              the companion mod's Lua source
  windows/hooks.nsh       installer hooks (--setup / --remove-mods)
```

BeamLink isn't affiliated with BeamMP or BeamNG GmbH. BeamMP and its launcher
are AGPL-3.0 projects by the BeamMP team. BeamLink downloads them from BeamMP's
own distribution points and does not bundle or modify them.
