<script>
  import { onMount } from "svelte";
  import { vault } from "../../lib/vault.svelte.js";
  import { app } from "../../lib/store.svelte.js";
  import { count, dayKey } from "../../lib/time.js";
  import Dialog from "../Dialog.svelte";
  import Sidebar from "./Sidebar.svelte";
  import Tabs from "./Tabs.svelte";
  import NoteView from "./NoteView.svelte";
  import NewTab from "./NewTab.svelte";
  import RightPanel from "./RightPanel.svelte";
  import QuickSwitcher from "./QuickSwitcher.svelte";
  import ContextMenu from "./ContextMenu.svelte";

  let note = $state();

  onMount(() => { vault.init(); });

  // "Write a note" on a class creates the note from the vault template and opens it here.
  $effect(() => {
    if (app.pendingNote && vault.loaded) {
      const p = app.pendingNote;
      app.pendingNote = null;
      vault.createNoteFromTemplate({ ...p, datum: dayKey(app.now) });
    }
  });

  function onKey(e) {
    const mod = e.ctrlKey || e.metaKey;
    if (!mod) return;
    const k = e.key.toLowerCase();
    if (k === "o") { e.preventDefault(); vault.switcher = true; }
    else if (k === "n") { e.preventDefault(); vault.createNote(); }
    else if (k === "e") { e.preventDefault(); vault.toggleMode(); }
    else if (k === "w") { e.preventDefault(); if (vault.active) vault.closeTab(vault.active.id); }
    else if (k === "s") { e.preventDefault(); if (vault.active?.path) vault.flush(vault.active.path); }
    else if (k === "tab") { e.preventDefault(); vault.cycleTab(e.shiftKey ? -1 : 1); }
    else if (k === "f" && e.shiftKey) { e.preventDefault(); vault.leftOpen = true; vault.leftTab = "search"; }
  }

  const saving = $derived(!!vault.activeDoc?.dirty);
  const links = $derived(vault.backlinkHits.length);
</script>

<svelte:window onkeydown={onKey} />

<div class="ws">
  {#if vault.leftOpen}<Sidebar />{/if}

  <div class="center">
    <Tabs />
    {#if vault.active?.path}
      {#key vault.active.id}
        <NoteView bind:this={note} tab={vault.active} />
      {/key}
    {:else}
      <NewTab />
    {/if}
    <footer class="status">
      {#if vault.active?.path && vault.activeDoc && !vault.activeDoc.error}
        <span>{count(links, "backlink")}</span>
        <span>{count(vault.stats.words, "word")}</span>
        <span>{count(vault.stats.chars, "character")}</span>
        <span class="save" class:on={saving}>{saving ? "Saving…" : "Saved"}</span>
      {/if}
    </footer>
  </div>

  {#if vault.rightOpen}<RightPanel onjump={(h) => note?.jumpTo(h)} />{/if}
</div>

<QuickSwitcher />
<ContextMenu />

<Dialog open={!!vault.confirm} title="Delete" onclose={() => (vault.confirm = null)} width={440}>
  {#if vault.confirm}
    <p class="ask">Move <b>{vault.confirm.label}</b> to trash? The file stays in the vault's <code>.trash</code> folder, where you can restore it from.</p>
    <div class="row">
      <button class="btn ghost" onclick={() => (vault.confirm = null)}>Cancel</button>
      <button class="btn danger" onclick={() => vault.trash(vault.confirm.path)}>Move to trash</button>
    </div>
  {/if}
</Dialog>

<style>
  .ws { display: flex; height: 100%; min-height: 0; background: var(--paper); }
  .center { flex: 1; min-width: 0; display: flex; flex-direction: column; min-height: 0; }
  .status { display: flex; justify-content: flex-end; gap: 14px; padding: 3px 14px; font-size: 13px; color: var(--ink-3); border-top: 1px solid var(--line-2); background: var(--paper); min-height: 24px; }
  .save.on { color: var(--accent); }
  .ask { line-height: 1.55; }
  code { font-family: ui-monospace, monospace; background: var(--panel-3); padding: 0 5px; border-radius: 4px; }
  .row { display: flex; justify-content: flex-end; gap: 10px; }
</style>
