<script>
  import { tick } from "svelte";
  import { vault, isMd, nameOf } from "../../lib/vault.svelte.js";
  import { app } from "../../lib/store.svelte.js";

  let renameValue = $state("");
  let renameEl = $state();

  $effect(() => {
    if (vault.renaming) {
      const e = vault.entries.find((x) => x.path === vault.renaming);
      renameValue = e ? (e.is_dir || !isMd(e.path) ? nameOf(e.path) : nameOf(e.path).replace(/\.md$/i, "")) : "";
      tick().then(() => renameEl?.select());
    }
  });

  function click(e, row) {
    vault.selected = row.path;
    if (row.is_dir) vault.toggleDir(row.path);
    else vault.open(row.path, { newTab: e.ctrlKey || e.metaKey || e.button === 1 });
  }

  function menu(e, row) {
    e.preventDefault();
    vault.selected = row?.path ?? vault.selected;
    const dir = row ? (row.is_dir ? row.path : row.path.includes("/") ? row.path.slice(0, row.path.lastIndexOf("/")) : "") : "";
    const items = [
      { label: "New note", run: () => vault.createNote(dir) },
      { label: "New folder", run: () => vault.createFolder(dir) },
    ];
    if (row) {
      items.push({ sep: true });
      if (!row.is_dir && isMd(row.path)) items.push({ label: "Open in new tab", run: () => vault.open(row.path, { newTab: true }) });
      items.push({ label: "Rename…", run: () => (vault.renaming = row.path) });
      items.push({ label: "Open in Obsidian", run: () => app.openNote(row.path) });
      items.push({ sep: true });
      items.push({ label: "Delete (to trash)", danger: true, run: () => vault.askTrash(row.path) });
    }
    vault.menu = { x: e.clientX, y: e.clientY, items };
  }

  async function commit() {
    const p = vault.renaming;
    if (p) await vault.rename(p, renameValue);
  }
</script>

<div class="actions" role="toolbar" aria-label="File actions">
  <button title="New note (Ctrl+N)" aria-label="New note" onclick={() => vault.createNote()}>
    <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 20h4L19 9l-4-4L4 16v4ZM13 7l4 4" /></svg>
  </button>
  <button title="New folder" aria-label="New folder" onclick={() => vault.createFolder()}>
    <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7ZM12 11v5M9.5 13.5h5" /></svg>
  </button>
  <button title="Collapse all" aria-label="Collapse all folders" onclick={() => vault.collapseAll()}>
    <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M7 9l5-5 5 5M7 15l5 5 5-5" /></svg>
  </button>
</div>

<div class="tree" role="tree" aria-label="Files in vault" oncontextmenu={(e) => menu(e, null)}>
  {#each vault.rows as row (row.path)}
    <div
      class="row" class:sel={vault.selected === row.path} class:active={vault.active?.path === row.path} class:dim={!row.is_dir && !isMd(row.path)}
      style:--d={row.depth} role="treeitem" aria-selected={vault.active?.path === row.path} aria-expanded={row.is_dir ? !!vault.expanded[row.path] : undefined}
      oncontextmenu={(e) => { e.stopPropagation(); menu(e, row); }}
    >
      {#if vault.renaming === row.path}
        <span class="chev"></span>
        <input
          class="rename" bind:this={renameEl} bind:value={renameValue} spellcheck="false"
          onblur={commit}
          onkeydown={(e) => { if (e.key === "Enter") { e.preventDefault(); commit(); } else if (e.key === "Escape") { vault.renaming = null; } }}
        />
      {:else}
        <button class="hit" tabindex="0" onclick={(e) => click(e, row)} onauxclick={(e) => e.button === 1 && click(e, row)} ondblclick={() => !row.is_dir && (vault.renaming = null)}>
          <span class="chev" class:open={row.is_dir && vault.expanded[row.path]}>
            {#if row.is_dir}<svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6" /></svg>{/if}
          </span>
          <span class="label" class:folder={row.is_dir}>{row.label}</span>
          {#if !row.is_dir && !isMd(row.path)}<span class="ext">{nameOf(row.path).split(".").pop()}</span>{/if}
        </button>
      {/if}
    </div>
  {/each}
  {#if vault.loaded && !vault.rows.length && !vault.error}
    <p class="empty muted">The vault is empty. Create your first note.</p>
  {/if}
</div>

<style>
  .actions { display: flex; justify-content: center; gap: 2px; padding: 4px 8px 6px; border-bottom: 1px solid var(--line-2); }
  .actions button { width: 30px; height: 28px; display: grid; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--ink-3); }
  .actions button:hover { background: var(--panel-3); color: var(--ink); }

  .tree { flex: 1; min-height: 0; overflow: auto; padding: 6px 6px 16px; }
  .row { position: relative; border-radius: 6px; }
  .row:hover { background: var(--panel-3); }
  .row.sel { background: color-mix(in oklab, var(--ink) 7%, transparent); }
  .row.active { background: var(--accent-soft); }
  .row.active .label { color: var(--ink); font-weight: 600; }
  .hit { width: 100%; display: flex; align-items: center; gap: 2px; padding: 3px 8px 3px calc(6px + var(--d) * 16px); border: 0; background: transparent; text-align: left; font-size: 14px; color: var(--ink-2); min-height: 26px; }
  .chev { width: 16px; height: 16px; display: inline-grid; place-items: center; flex: none; color: var(--ink-3); transition: transform 0.12s; }
  .chev.open { transform: rotate(90deg); }
  .label { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .label.folder { color: var(--ink); }
  .dim .label { color: var(--ink-3); }
  .ext { font-size: 13px; padding: 0 5px; border-radius: 4px; border: 1px solid var(--line); color: var(--ink-3); text-transform: lowercase; }
  .rename { margin: 2px 6px 2px calc(22px + var(--d) * 16px); width: calc(100% - 28px - var(--d) * 16px); padding: 2px 6px; font-size: 14px; border-radius: 4px; }
  .empty { padding: 12px 10px; font-size: 13.5px; }
</style>
