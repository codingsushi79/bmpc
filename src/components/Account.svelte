<script lang="ts">
  import Icon from "./Icon.svelte";
  import { api } from "../lib/api";
  import { store } from "../lib/store.svelte";

  let username = $state("");
  let password = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  const account = $derived(store.view?.account);

  async function signIn(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = null;
    try {
      const result = await api.signIn(username, password);
      password = "";
      if (result.signed_in) {
        store.toast(`Signed in as ${result.username}`);
        await store.refresh();
      } else {
        error = result.message ?? "Sign-in failed";
      }
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  async function signOut() {
    await api.signOut();
    store.toast("Signed out", "info");
    await store.refresh();
  }
</script>

<div class="account fade-in">
  <h1 class="display">Account</h1>

  {#if account?.signed_in}
    <div class="card panel">
      <div class="avatar display">{(account.username ?? "?").slice(0, 1)}</div>
      <div class="who">
        <div class="name display">{account.username}</div>
        <div class="muted">{account.role ?? "BeamMP member"}{account.id ? ` · #${account.id}` : ""}</div>
      </div>
      <button class="btn btn-ghost" onclick={signOut}><Icon name="logout" size={16} /> Sign out</button>
    </div>
    <p class="muted info">
      BeamMP signs you in automatically when the game starts. Your session key is stored where the official launcher keeps it; BeamLink never saves your password.
    </p>
  {:else}
    <div class="grid">
      <form class="panel form" onsubmit={signIn}>
        <h2 class="display">Sign in to BeamMP</h2>
        <p class="muted">Use your BeamMP forum account. Some servers don't allow guests.</p>
        <label>
          <span class="eyebrow">Username</span>
          <input bind:value={username} autocomplete="username" spellcheck="false" />
        </label>
        <label>
          <span class="eyebrow">Password</span>
          <input type="password" bind:value={password} autocomplete="current-password" />
        </label>
        {#if error}<div class="error"><Icon name="alert" size={15} /> {error}</div>{/if}
        <button class="btn btn-primary" disabled={busy || !username || !password}>
          {#if busy}<span class="spin"><Icon name="refresh" size={16} /></span>{/if} Sign in
        </button>
        <button type="button" class="link" onclick={() => api.openUrl("https://forum.beammp.com/signup")}>No account? Create one on forum.beammp.com</button>
      </form>
      <div class="panel guest">
        <h2 class="display">Play as guest</h2>
        <p class="muted">
          No account needed. BeamMP gives you a guest name, and you can join any server that allows guests (most do — the browser shows the ones that don't).
        </p>
        <button class="btn btn-ghost" disabled={!store.canPlay} onclick={() => store.play()}><Icon name="play" size={16} /> Play as guest</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .account {
    height: 100%;
    overflow-y: auto;
    padding: 26px 32px 30px;
    max-width: 980px;
  }
  h1 {
    margin: 0 0 18px;
    font-size: 40px;
  }
  .card {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 22px 24px;
  }
  .avatar {
    width: 64px;
    height: 64px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    font-size: 36px;
    background: var(--grad);
  }
  .who {
    flex: 1;
  }
  .name {
    font-size: 30px;
    line-height: 1;
  }
  .info {
    margin-top: 14px;
    line-height: 1.55;
  }
  .grid {
    display: grid;
    grid-template-columns: 1.2fr 1fr;
    gap: 16px;
  }
  .form,
  .guest {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  h2 {
    margin: 0;
    font-size: 26px;
  }
  p {
    margin: 0;
    line-height: 1.5;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #ffb3c1;
    font-size: 13px;
  }
  .link {
    color: var(--muted);
    font-size: 12.5px;
    text-align: left;
  }
  .link:hover {
    color: var(--text);
    text-decoration: underline;
  }
  .guest .btn {
    align-self: flex-start;
  }
</style>
