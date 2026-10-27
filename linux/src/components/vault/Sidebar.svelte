<script>
  import { vault } from "../../lib/vault.svelte.js";
  import { app } from "../../lib/store.svelte.js";
  import FileExplorer from "./FileExplorer.svelte";
  import SearchPanel from "./SearchPanel.svelte";

  const tabs = [
    { id: "files", label: "Files", d: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" },
    { id: "search", label: "Search", d: "M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14ZM21 21l-5-5" },
  ];
  const pick = (id) => { vault.leftTab = id; vault.persist(); };
</script>

<aside class="left" aria-label="Sidebar">
  <div class="tabs" role="tablist">
    {#each tabs as t (t.id)}
      <button role="tab" aria-selected={vault.leftTab === t.id} class:on={vault.leftTab === t.id} title={t.label} aria-label={t.label} onclick={() => pick(t.id)}>
        <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={t.d} /></svg>
      </button>
    {/each}
  </div>

  {#if vault.error}
    <div class="err">
      <p>{vault.error}</p>
      <button class="btn small" onclick={() => (app.settingsDialog = true)}>Open settings</button>
    </div>
  {:else if vault.leftTab === "files"}
    <FileExplorer />
  {:else}
    <SearchPanel />
  {/if}

  <footer>
    <span class="vname" title={app.settings?.vault_path}>
      <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3ZM12 12l8-4.5M12 12v9M12 12L4 7.5" /></svg>
      {vault.vaultName}
    </span>
    <button class="gear" title="Settings" aria-label="Settings" onclick={() => (app.settingsDialog = true)}>
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h9M17 7h3M4 17h3M11 17h9M15 4v6M9 14v6" /></svg>
    </button>
  </footer>
</aside>

<style>
  .left { display: flex; flex-direction: column; min-height: 0; background: var(--panel); border-right: 1px solid var(--line-2); width: 288px; min-width: 288px; }
  .tabs { display: flex; gap: 2px; padding: 6px 8px 0; border-bottom: 1px solid var(--line-2); }
  .tabs button { width: 34px; height: 32px; display: grid; place-items: center; border: 0; border-radius: 6px 6px 0 0; background: transparent; color: var(--ink-3); position: relative; }
  .tabs button:hover { color: var(--ink); background: var(--panel-3); }
  .tabs button.on { color: var(--ink); }
  .tabs button.on::after { content: ""; position: absolute; left: 6px; right: 6px; bottom: -1px; height: 2px; background: var(--accent); border-radius: 2px; }
  .err { padding: 14px; display: grid; gap: 10px; font-size: 13.5px; color: var(--ink-2); }
  footer { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 6px 8px 6px 12px; border-top: 1px solid var(--line-2); }
  .vname { display: inline-flex; align-items: center; gap: 8px; font-size: 13.5px; font-weight: 600; color: var(--ink-2); min-width: 0; }
  .gear { width: 28px; height: 28px; display: grid; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--ink-3); }
  .gear:hover { background: var(--panel-3); color: var(--ink); }
</style>
