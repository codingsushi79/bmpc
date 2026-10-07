<script lang="ts">
  import Icon from "./Icon.svelte";
  import HeroArt from "./HeroArt.svelte";
  import { api, onSetupProgress } from "../lib/api";
  import { store } from "../lib/store.svelte";
  import { bytes } from "../lib/beam";
  import type { Progress } from "../lib/types";

  const STEPS: { id: string; label: string; detail: string }[] = [
    { id: "game", label: "Find BeamNG.drive", detail: "Locates the game and its user folder" },
    { id: "launcher", label: "BeamMP launcher", detail: "Official build, checksum-verified" },
    { id: "client", label: "BeamMP client mod", detail: "Goes in mods/multiplayer" },
    { id: "companion", label: "BeamLink companion", detail: "Tiny mod for one-click joining" },
    { id: "activate", label: "Enable mods", detail: "Turns both on in BeamNG's mod list" },
  ];

  let progress = $state<Record<string, Progress>>({});
  let running = $state(false);
  let finished = $state<"ok" | "err" | null>(null);
  let error = $state<string | null>(null);
  let gameDir = $state("");

  const view = $derived(store.view);

  $effect(() => {
    let off: (() => void) | undefined;
    onSetupProgress((p) => (progress[p.step] = p)).then((f) => (off = f));
    return () => off?.();
  });

  async function start() {
    running = true;
    finished = null;
    error = null;
    progress = {};
    try {
      if (gameDir.trim() && view) {
        store.view = await api.saveSettings({ ...$state.snapshot(view.settings), game_dir: gameDir.trim() });
      }
      store.view = await api.runSetup();
      finished = "ok";
    } catch (e) {
      error = String(e);
      finished = "err";
      await store.refresh();
    } finally {
      running = false;
    }
  }

  function close() {
    store.setupOpen = false;
  }
</script>

<div class="setup">
  <HeroArt />
  <div class="shade"></div>
  <div class="card panel">
    <div class="label">Setup</div>
    <h1 class="page-title">Set up BeamMP</h1>
    <p class="muted lead">
      BeamLink installs the official BeamMP launcher and client mod into BeamNG.drive, plus a tiny companion mod that lets you join a server here with one click.
    </p>

    {#if view && !view.install.game_found}
      <label class="field">
        <span class="label">BeamNG.drive folder</span>
        <input bind:value={gameDir} placeholder="e.g. D:\SteamLibrary\steamapps\common\BeamNG.drive" spellcheck="false" />
        <span class="hint">We couldn't find the game in Steam's libraries. Paste the folder that holds BeamNG.drive.exe.</span>
      </label>
    {:else if view}
      <div class="found"><Icon name="check" size={16} /> Found {view.paths.game_dir}{view.paths.game_version ? ` (v${view.paths.game_version})` : ""}</div>
    {/if}

    <ol class="steps">
      {#each STEPS as step}
        {@const p = progress[step.id]}
        <li class={p?.status ?? "pending"}>
          <span class="mark">
            {#if p?.status === "done"}<Icon name="check" size={15} />
            {:else if p?.status === "error"}<Icon name="close" size={15} />
            {:else if p?.status === "skipped"}–
            {:else if p?.status === "running"}<span class="spin"><Icon name="refresh" size={15} /></span>
            {/if}
          </span>
          <div class="text">
            <div class="step-name">{step.label}</div>
            <div class="detail">{p && p.status !== "running" ? p.message : p?.message || step.detail}</div>
            {#if p?.status === "running" && p.total > 0}
              <div class="bar"><div style:width={`${(p.done / p.total) * 100}%`}></div></div>
              <div class="detail">{bytes(p.done)} of {bytes(p.total)}</div>
            {/if}
          </div>
        </li>
      {/each}
    </ol>

    {#if error}<div class="error"><Icon name="alert" size={15} /> {error}</div>{/if}

    <div class="actions">
      {#if finished === "ok"}
        <button class="btn btn-primary" onclick={close}><Icon name="play" size={16} /> Let's drive</button>
      {:else}
        <button class="btn btn-primary" disabled={running || !store.canPlay} onclick={start}>
          {running ? "Installing…" : finished === "err" ? "Try again" : "Install"}
        </button>
        <button class="btn btn-secondary" disabled={running} onclick={close}>{view?.ready ? "Close" : "Skip for now"}</button>
      {/if}
    </div>
    {#if !store.canPlay}
      <p class="hint">BeamNG.drive doesn't run on this OS, so there is nothing to install here. You can still browse servers.</p>
    {/if}
  </div>
</div>

<style>
  .setup {
    position: absolute;
    inset: 0;
    z-index: 40;
    display: grid;
    place-items: center;
    background: var(--bg);
    overflow: hidden;
  }
  .shade {
    position: absolute;
    inset: 0;
    background: rgba(11, 12, 15, 0.82);
  }
  .card {
    position: relative;
    width: 620px;
    max-width: calc(100% - 40px);
    max-height: calc(100% - 40px);
    overflow-y: auto;
    padding: 28px 30px;
    background: var(--surface);
  }
  h1 {
    margin: 6px 0 8px;
  }
  .lead {
    line-height: 1.55;
    margin: 0 0 18px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  .hint {
    font-size: 12.5px;
    color: var(--muted);
    line-height: 1.5;
  }
  .found {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--good);
    font-size: 13px;
    margin-bottom: 14px;
    word-break: break-all;
  }
  .steps {
    list-style: none;
    margin: 0 0 16px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .steps li {
    display: flex;
    gap: 14px;
    padding: 11px 12px;
    border-radius: var(--radius);
    border: 1px solid transparent;
  }
  .steps li.running {
    background: var(--surface-2);
    border-color: var(--line-hi);
  }
  .mark {
    width: 26px;
    height: 26px;
    border-radius: var(--radius);
    flex: none;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.06);
    color: var(--muted);
    font-weight: 800;
  }
  .done .mark {
    background: rgba(63, 185, 80, 0.15);
    color: var(--good);
  }
  .error .mark {
    background: rgba(255, 77, 109, 0.15);
    color: var(--bad);
  }
  .running .mark {
    color: var(--accent);
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .step-name {
    font-weight: 600;
  }
  .detail {
    font-size: 12.5px;
    color: var(--muted);
    margin-top: 2px;
    word-break: break-word;
  }
  li.error .detail {
    color: var(--bad);
  }
  .bar {
    height: 4px;
    margin-top: 8px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    background: var(--accent);
    transition: width 0.15s;
  }
  div.error {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    color: var(--bad);
    font-size: 13px;
    margin-bottom: 14px;
  }
  .actions {
    display: flex;
    gap: 10px;
  }
  .actions .btn {
    height: 46px;
    padding: 0 24px;
    font-size: 17px;
  }
</style>
