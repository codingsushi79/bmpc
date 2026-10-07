// App-wide state. One object of runes, so every component reads the same
// live values and nothing has to be passed down five levels.

import { api, savedFrom } from "./api";
import { serverKey } from "./beam";
import type { Server, SavedServer, View } from "./types";

export type Page = "home" | "servers" | "library" | "mods" | "account" | "settings";

interface Toast {
  id: number;
  text: string;
  kind: "ok" | "err" | "info";
}

class Store {
  view = $state<View | null>(null);
  page = $state<Page>("home");
  servers = $state.raw<Server[]>([]);
  serversAt = $state(0);
  serversError = $state<string | null>(null);
  loadingServers = $state(false);
  pings = $state<Record<string, number | null>>({});
  selected = $state.raw<Server | null>(null);
  toasts = $state<Toast[]>([]);
  logOpen = $state(false);
  setupOpen = $state(false);
  directOpen = $state(false);

  favoriteKeys = $derived(new Set((this.view?.settings.favorites ?? []).map(serverKey)));
  totals = $derived.by(() => {
    let players = 0;
    for (const s of this.servers) players += s.players;
    return { servers: this.servers.length, players };
  });
  /** What the game says it is doing, if it is running with the companion. */
  inSession = $derived(this.view?.game?.session ? this.view.game.server : null);
  canPlay = $derived(!!this.view?.game_supported);

  private toastId = 0;

  toast(text: string, kind: Toast["kind"] = "ok") {
    const id = ++this.toastId;
    this.toasts = [...this.toasts, { id, text, kind }];
    setTimeout(() => (this.toasts = this.toasts.filter((t) => t.id !== id)), kind === "err" ? 7000 : 4000);
  }

  async refresh() {
    try {
      this.view = await api.state();
    } catch (e) {
      this.toast(String(e), "err");
    }
  }

  async loadServers(force = false) {
    if (this.loadingServers) return;
    this.loadingServers = true;
    try {
      this.servers = await api.servers(force);
      this.serversAt = Date.now();
      this.serversError = null;
    } catch (e) {
      this.serversError = String(e);
    } finally {
      this.loadingServers = false;
    }
  }

  async ping(targets: string[]) {
    const fresh = targets.filter((t) => !(t in this.pings));
    if (!fresh.length) return;
    // Mark as pending so the same rows are not pinged twice in flight.
    for (const t of fresh) this.pings[t] = -1;
    for (const p of await api.ping(fresh)) this.pings[p.key] = p.ms;
  }

  async play() {
    await this.run(() => api.play());
  }

  async join(server: Server | SavedServer) {
    if (!this.canPlay) {
      this.toast("BeamNG.drive does not run on this OS — join from Windows or Linux", "err");
      return;
    }
    if (!this.view?.ready) {
      this.setupOpen = true;
      this.toast("Finish setup first so BeamMP is installed", "info");
      return;
    }
    await this.run(() => api.join(savedFrom(server)));
  }

  async toggleFavorite(server: Server | SavedServer) {
    try {
      const now = await api.toggleFavorite(savedFrom(server));
      this.toast(now ? "Added to favorites" : "Removed from favorites", "info");
      await this.refresh();
    } catch (e) {
      this.toast(String(e), "err");
    }
  }

  isFavorite(server: { ip: string; port: number }) {
    return this.favoriteKeys.has(serverKey(server));
  }

  /** Run an action, toast its message, then refresh state. */
  async run(action: () => Promise<string | void>) {
    try {
      const message = await action();
      if (message) this.toast(message);
    } catch (e) {
      this.toast(String(e), "err");
    }
    await this.refresh();
  }
}

export const store = new Store();
