<script>
  import { app } from "../lib/store.svelte.js";
  import { vault } from "../lib/vault.svelte.js";
  import * as api from "../lib/api.js";

  // `path` is the folder to work on (defaults to the saved vault folder). With `onpick`, choosing a folder only
  // reports it (the Settings form saves it later); without it, the folder is saved and loaded straight away.
  let { path = undefined, onpick = undefined, oncreated = undefined } = $props();
  const target = $derived((path ?? app.settings?.vault_path ?? "").trim());
  let busy = $state(false);

  async function use(folder) {
    if (folder !== app.settings?.vault_path) {
      const next = { ...app.settings, vault_path: folder };
      await api.saveSettings(next);
      app.settings = next;
    }
    await app.refresh();
    await vault.loadTree();
  }

  async function choose() {
    const folder = await api.pickFolder(target);
    if (!folder) return;
    if (onpick) return onpick(folder);
    busy = true;
    try {
      await use(folder);
    } catch (e) {
      app.notify(String(e?.message ?? e));
    } finally {
      busy = false;
    }
  }

  async function create() {
    busy = true;
    try {
      const n = await api.createStarterVault(target);
      app.notify(n ? `Starter vault created: ${n} files in ${target}.` : "Nothing to add, those files already exist.");
      await use(target);
      oncreated?.();
    } catch (e) {
      app.notify(String(e?.message ?? e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="setup">
  <button type="button" class="btn small ghost" disabled={busy} onclick={choose}>Choose folder…</button>
  <button type="button" class="btn small" disabled={busy || !target} onclick={create} title="Adds the folders, templates and index notes. Existing files are kept.">Create starter vault</button>
</div>

<style>
  .setup { display: flex; flex-wrap: wrap; gap: 8px; }
</style>
