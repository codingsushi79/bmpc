<script lang="ts">
  import BeamText from "./BeamText.svelte";
  import Icon from "./Icon.svelte";
  import { store } from "../lib/store.svelte";
  import { bytes, mapName } from "../lib/beam";
  import type { Server } from "../lib/types";

  let { server }: { server: Server } = $props();
  const fill = $derived(server.max_players ? Math.min(1, server.players / server.max_players) : 0);
  const full = $derived(server.max_players > 0 && server.players >= server.max_players);
</script>

<div
  class="card"
  role="button"
  tabindex="0"
  onclick={() => (store.selected = server)}
  onkeydown={(e) => e.key === "Enter" && (store.selected = server)}
>
  <div class="top">
    <span class="map">{mapName(server.map)}</span>
    {#if server.official}<span class="tag official">Official</span>{/if}
    {#if server.featured}<span class="tag featured">Featured</span>{/if}
    {#if server.partner}<span class="tag partner">Partner</span>{/if}
    {#if server.password}<Icon name="lock" size={12} />{/if}
    <span class="region">{server.location || "--"}</span>
  </div>
  <div class="name"><BeamText text={server.name} /></div>
  <div class="meta">{server.mods.length ? `${server.mods.length} mods · ${bytes(server.mods_size)}` : "No mods"}</div>
  <div class="foot">
    <div class="players">
      <span class="count" class:full>{server.players}<span class="muted">/{server.max_players}</span></span>
      <div class="meter"><div class:hot={full} style:width={`${fill * 100}%`}></div></div>
    </div>
    <button
      class="btn btn-secondary btn-sm join"
      disabled={!store.canPlay}
      onclick={(e) => {
        e.stopPropagation();
        store.join(server);
      }}>Join</button
    >
  </div>
</div>

<style>
  .card {
    padding: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .card:hover {
    border-color: #3a3f4a;
    background: var(--surface-2);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    margin-bottom: 8px;
  }
  .map {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
    margin-right: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .region {
    margin-left: auto;
  }
  .name {
    font-weight: 600;
    font-size: 14px;
    line-height: 1.35;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    min-height: 2.7em;
  }
  .meta {
    font-size: 12px;
    color: var(--muted);
    margin-top: 6px;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 12px;
  }
  .players {
    flex: 1;
  }
  .count {
    display: block;
    font-size: 13px;
    font-weight: 650;
    font-variant-numeric: tabular-nums;
    margin-bottom: 5px;
  }
  .count.full {
    color: var(--warn);
  }
  .card:hover .join:not(:disabled) {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
</style>
