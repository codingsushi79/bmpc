// Self-update via Tauri's updater: GitHub Releases publish a signed
// latest.json; installed copies check it, download the signed installer for
// their platform, verify it against the public key baked into the app, and
// restart into the new version.

import { inTauri } from "./api";

type Status = "idle" | "checking" | "none" | "available" | "downloading" | "ready" | "error";

const SIX_HOURS = 6 * 60 * 60 * 1000;

class Updater {
  status = $state<Status>("idle");
  version = $state<string | null>(null);
  notes = $state<string | null>(null);
  done = $state(0);
  total = $state(0);
  error = $state<string | null>(null);
  checkedAt = $state<number | null>(null);

  // The plugin's Update handle; kept out of reactive state.
  private update: { downloadAndInstall: (cb: (e: any) => void) => Promise<void> } | null = null;

  start() {
    setTimeout(() => this.check(), 4000);
    setInterval(() => {
      if (this.status === "idle" || this.status === "none" || this.status === "error") this.check();
    }, SIX_HOURS);
  }

  async check(manual = false) {
    if (this.status === "checking" || this.status === "downloading") return;
    this.status = "checking";
    this.error = null;
    try {
      if (!inTauri) {
        // Browser preview: pretend a release is out so the UI can be seen.
        await new Promise((r) => setTimeout(r, 400));
        this.version = "0.2.1";
        this.notes = "Preview build";
        this.status = "available";
        return;
      }
      const { check } = await import("@tauri-apps/plugin-updater");
      const found = await check();
      this.checkedAt = Date.now();
      if (found) {
        this.update = found;
        this.version = found.version;
        this.notes = found.body ?? null;
        this.status = "available";
      } else {
        this.status = "none";
      }
    } catch (e) {
      this.error = String(e);
      // A failed background check is not worth an error badge; a manual
      // one is.
      this.status = manual ? "error" : "idle";
    }
  }

  async install() {
    if (this.status !== "available") return;
    this.status = "downloading";
    this.done = 0;
    this.total = 0;
    try {
      if (!inTauri || !this.update) {
        for (let i = 1; i <= 20; i++) {
          this.total = 20;
          this.done = i;
          await new Promise((r) => setTimeout(r, 60));
        }
        this.status = "ready";
        return;
      }
      await this.update.downloadAndInstall((event: any) => {
        if (event.event === "Started") this.total = event.data.contentLength ?? 0;
        else if (event.event === "Progress") this.done += event.data.chunkLength;
        else if (event.event === "Finished") this.status = "ready";
      });
      this.status = "ready";
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      this.error = String(e);
      this.status = "error";
    }
  }
}

export const updater = new Updater();
