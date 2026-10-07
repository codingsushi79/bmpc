<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { bytes, flag, mapHue, mapName } from "../lib/beam";
  import type { Server } from "../lib/types";

  let { server }: { server: Server } = $props();
  const hue = $derived(mapHue(server.map));
  const fill = $derived(server.max_players ? Math.min(1, server.players / server.max_players) : 0);
  const full = $derived(server.players >= server.max_players && server.max_players > 0);
</script>

<div class="card panel" role="button" tabindex="0" onclick={() => (store.selected = server)} onkeydown={(e) => e.key === "Enter" && (store.selected = server)}>
  <div class="banner" style:--h={hue}>
    <div class="map display">{mapName(server.map)}</div>
    <div class="badges">
      {#if server.partner}<span class="badge partner">Partner</span>{/if}
      {#if server.featured}<span class="badge featured">Featured</span>{/if}
      {#if server.official}<span class="badge official">Official</span>{/if}
      {#if server.password}<span class="badge"><Icon name="lock" size={11} /></span>{/if}
    </div>
    <span class="loc" title={server.location}>{flag(server.location)}</span>
  </div>
  <div class="body">
    <div class="name"><BeamText text={server.name} /></div>
    <div class="meta muted">
      <span><Icon name="box" size={13} /> {server.mods.length ? `${server.mods.length} mods · ${bytes(server.mods_size)}` : "No mods"}</span>
    </div>
    <div class="foot">
      <div class="players">
        <div class="count"><Icon name="users" size={14} /> <b class:full>{server.players}</b><span class="muted">/{server.max_players}</span></div>
        <div class="bar"><div style:width={`${fill * 100}%`} class:full></div></div>
      </div>
      <button
        class="btn btn-primary btn-sm"
        disabled={!store.canPlay}
        onclick={(e) => {
          e.stopPropagation();
          store.join(server);
        }}
      >
        Join
      </button>
    </div>
  </div>
</div>

<style>
  .card {
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.15s, border-color 0.15s, box-shadow 0.2s;
    outline: none;
  }
  .card:hover,
  .card:focus-visible {
    transform: translateY(-3px);
    border-color: rgba(255, 46, 99, 0.4);
    box-shadow: 0 18px 40px -20px rgba(255, 46, 99, 0.45);
  }
  .banner {
    position: relative;
    height: 78px;
    padding: 12px 14px;
    background:
      radial-gradient(120% 140% at 100% 0%, hsla(var(--h), 90%, 55%, 0.55), transparent 60%),
      linear-gradient(120deg, hsla(var(--h), 60%, 18%, 1), hsla(calc(var(--h) + 40), 70%, 10%, 1));
    display: flex;
    align-items: flex-end;
  }
  .banner::after {
    content: "";
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(115deg, rgba(255, 255, 255, 0.04) 0 2px, transparent 2px 14px);
  }
  .map {
    font-size: 22px;
    line-height: 1;
    text-shadow: 0 3px 14px rgba(0, 0, 0, 0.5);
    position: relative;
    z-index: 1;
  }
  .badges {
    position: absolute;
    top: 10px;
    left: 12px;
    display: flex;
    gap: 5px;
    z-index: 1;
  }
  .loc {
    position: absolute;
    top: 8px;
    right: 12px;
    font-size: 18px;
    z-index: 1;
  }
  .body {
    padding: 12px 14px 14px;
  }
  .name {
    font-weight: 700;
    font-size: 14.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 6px;
  }
  .meta {
    font-size: 12px;
    display: flex;
    gap: 10px;
  }
  .meta span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }
  .players {
    flex: 1;
  }
  .count {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 13px;
    margin-bottom: 6px;
  }
  .count b {
    color: var(--good);
  }
  .count b.full {
    color: var(--warn);
  }
  .bar {
    height: 4px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.07);
    overflow: hidden;
  }
  .bar div {
    height: 100%;
    border-radius: 4px;
    background: var(--grad);
  }
  .bar div.full {
    background: var(--warn);
  }
</style>
