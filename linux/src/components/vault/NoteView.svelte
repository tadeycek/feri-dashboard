<script>
  import { tick } from "svelte";
  import { vault, stemOf, dirOf } from "../../lib/vault.svelte.js";
  import { app } from "../../lib/store.svelte.js";
  import Editor from "./Editor.svelte";
  import Reading from "./Reading.svelte";

  let { tab } = $props();

  const path = $derived(tab.path);
  const doc = $derived(vault.docs[path]);
  const crumbs = $derived(dirOf(path) ? dirOf(path).split("/") : []);

  let titleValue = $state("");
  let titleEl = $state();
  let editor = $state();
  let scroller = $state();

  // Keep the inline title in step with the file name (e.g. after a rename).
  $effect(() => { titleValue = stemOf(path); });

  // A freshly created note focuses its title so it can be named straight away.
  $effect(() => {
    if (vault.titleFocus === path) {
      vault.titleFocus = null;
      tick().then(() => titleEl?.select());
    }
  });

  async function commitTitle() {
    const next = titleValue.trim();
    if (!next || next === stemOf(path)) { titleValue = stemOf(path); return; }
    const newPath = await vault.rename(path, next);
    if (newPath === path) titleValue = stemOf(path);
  }

  const names = () => vault.notes.map((n) => stemOf(n.path));

  /** Outline clicks arrive through this: jump in the editor, or scroll the reading view. */
  export function jumpTo(h) {
    if (tab.mode === "read") scroller?.querySelector(`#${CSS.escape(h.id)}`)?.scrollIntoView({ behavior: "smooth", block: "start" });
    else editor?.goToLine(h.line);
  }
</script>

<section class="note">
  <header class="head">
    <nav class="crumbs" aria-label="Path to note">
      {#each crumbs as c, i (i)}<span>{c}</span><i>/</i>{/each}
      <b>{stemOf(path)}</b>
    </nav>
    <div class="tools">
      <button class="tool" title={tab.mode === "read" ? "Switch to editing (Ctrl+E)" : "Switch to reading (Ctrl+E)"} aria-label={tab.mode === "read" ? "Switch to editing" : "Switch to reading"} onclick={() => vault.toggleMode()}>
        {#if tab.mode === "read"}
          <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 20h4L19 9l-4-4L4 16v4ZM13 7l4 4" /></svg>
        {:else}
          <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z" /><circle cx="12" cy="12" r="3" /></svg>
        {/if}
      </button>
      <button class="tool" title="Open in Obsidian" aria-label="Open in Obsidian" onclick={() => app.openNote(path)}>
        <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5" /></svg>
      </button>
    </div>
  </header>

  <div class="scroll" bind:this={scroller}>
    {#if !doc || doc.loading}
      <p class="msg muted">Loading…</p>
    {:else if doc.error}
      <div class="msg">
        <p>{doc.error}</p>
      </div>
    {:else}
      <div class="inner">
        <input
          class="title" bind:this={titleEl} bind:value={titleValue} spellcheck="false" aria-label="Note title"
          onblur={commitTitle}
          onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); e.currentTarget.blur(); editor?.focus(); } else if (e.key === "Escape") { titleValue = stemOf(path); e.currentTarget.blur(); } }}
        />
      </div>
      {#if tab.mode === "read"}
        <Reading {path} content={doc.content} />
      {:else}
        {#key path}
          <div class="ed">
            <Editor bind:this={editor} content={doc.content} {names} onchange={(text) => vault.setContent(path, text)} onlink={(n) => vault.openByName(n, path)} autofocus={!!doc && !doc.content} />
          </div>
        {/key}
      {/if}
    {/if}
  </div>
</section>

<style>
  .note { display: flex; flex-direction: column; min-height: 0; flex: 1; background: var(--paper); }
  .head { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 4px 12px 4px 16px; min-height: 36px; }
  .crumbs { display: flex; align-items: center; gap: 6px; font-size: 13.5px; color: var(--ink-3); min-width: 0; overflow: hidden; white-space: nowrap; }
  .crumbs i { font-style: normal; opacity: 0.6; }
  .crumbs b { font-weight: 500; color: var(--ink-2); overflow: hidden; text-overflow: ellipsis; }
  .tools { display: flex; gap: 2px; flex: none; }
  .tool { width: 30px; height: 28px; display: grid; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--ink-3); }
  .tool:hover { background: var(--panel-3); color: var(--ink); }
  .scroll { flex: 1; min-height: 0; overflow: auto; display: flex; flex-direction: column; }
  .inner { width: min(760px, 100%); margin: 0 auto; padding: 18px 24px 6px; flex: none; }
  .title { width: 100%; border: 0; background: transparent; padding: 0; font: 700 2.2rem/1.25 var(--font-ui); color: var(--ink); box-shadow: none; border-radius: 4px; }
  .title:hover { border: 0; }
  .title:focus { border: 0; box-shadow: none; outline: none; }
  .ed { flex: 1; min-height: 360px; }
  .msg { width: min(760px, 100%); margin: 40px auto; padding: 0 24px; color: var(--ink-2); }
</style>
