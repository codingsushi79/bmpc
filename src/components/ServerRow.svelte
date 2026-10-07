<script lang="ts">
  // One line per server, used by every list outside the main browser so
  // they all read the same: name and details, players, ping, actions.
  import type { Snippet } from "svelte";
  import BeamText from "./BeamText.svelte";
  import { mapName } from "../lib/beam";
  import type { Server } from "../lib/types";

  let {
    name,
    ownName = false,
    server = null,
    detail = "",
    ping = undefined,
    offline = false,
    onopen,
    actions,
  }: {
    name: string;
    /** Show `name` even when live info has the server's own name. */
    ownName?: boolean;
    server?: Server | null;
    detail?: string;
    ping?: number | null;
    offline?: boolean;
    onopen?: () => void;
    actions?: Snippet;
  } = $props();

  const sub = $derived(
    [server ? mapName(server.map) : null, detail || null, server?.location || null].filter(Boolean).join(" · "),
  );
</script>

<!-- Only focusable (and a button) when it opens something. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div class="row" class:clickable={!!onopen} role={onopen ? "button" : undefined} tabindex={onopen ? 0 : undefined}
  onclick={() => onopen?.()} onkeydown={(e) => e.key === "Enter" && onopen?.()}>
  <span class="dot" class:good={!!server && server.players > 0} class:bad={offline}></span>
  <div class="main">
    <div class="name"><BeamText text={ownName ? name : server?.name || name} /></div>
    {#if sub}<div class="sub">{sub}</div>{/if}
  </div>
  <div class="players">
    {#if server}{server.players}<span class="muted">/{server.max_players}</span>{:else if offline}<span class="muted">Offline</span>{:else}<span class="muted">—</span>{/if}
  </div>
  <div class="ping">
    {#if ping == null || ping < 0}<span class="muted">{ping === -1 ? "…" : ""}</span>{:else}{ping} ms{/if}
  </div>
  <div class="actions">{@render actions?.()}</div>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: 10px minmax(0, 1fr) 64px 56px auto;
    align-items: center;
    gap: 12px;
    min-height: 52px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child {
    border-bottom: none;
  }
  .clickable {
    cursor: pointer;
  }
  .clickable:hover {
    background: var(--surface-2);
  }
  .main {
    min-width: 0;
  }
  .name {
    font-weight: 550;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub {
    margin-top: 2px;
    font-size: 12px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .players,
  .ping {
    font-size: 12.5px;
    font-variant-numeric: tabular-nums;
    text-align: right;
  }
  .ping {
    color: var(--text-2);
  }
  .actions {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
  }
</style>
