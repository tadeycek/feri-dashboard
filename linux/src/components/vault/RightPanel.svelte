<script>
  import { vault, stemOf } from "../../lib/vault.svelte.js";
  import { count } from "../../lib/time.js";

  let { onjump } = $props();

  const tabs = [
    { id: "backlinks", label: "Backlinks" },
    { id: "outgoing", label: "Outgoing" },
    { id: "outline", label: "Outline" },
  ];

  // Backlinks are looked up in the vault whenever the open note changes or the vault is rescanned.
  $effect(() => {
    vault.active?.path;
    vault.entries;
    vault.refreshBacklinks();
  });

  const grouped = $derived.by(() => {
    const m = new Map();
    for (const h of vault.backlinkHits) {
      if (!m.has(h.path)) m.set(h.path, []);
      m.get(h.path).push(h);
    }
    return [...m];
  });
</script>

<aside class="right" aria-label="Right sidebar">
  <div class="tabs" role="tablist">
    {#each tabs as t (t.id)}
      <button role="tab" class:on={vault.rightTab === t.id} aria-selected={vault.rightTab === t.id} onclick={() => { vault.rightTab = t.id; vault.persist(); }}>{t.label}</button>
    {/each}
  </div>

  <div class="body">
    {#if !vault.active?.path}
      <p class="muted msg">Open a note to see its links and outline.</p>
    {:else if vault.rightTab === "backlinks"}
      {#if grouped.length}
        <p class="count">{count(vault.backlinkHits.length, "link")} in {count(grouped.length, "note")}</p>
        {#each grouped as [path, lines] (path)}
          <div class="grp">
            <button class="file" onclick={() => vault.open(path)}>{stemOf(path)}</button>
            {#each lines as l (l.line_no)}<p class="ctx">{l.line}</p>{/each}
          </div>
        {/each}
      {:else}
        <p class="muted msg">No other note links here.</p>
      {/if}
    {:else if vault.rightTab === "outgoing"}
      {#if vault.outgoing.length}
        {#each vault.outgoing as l (l.name)}
          <button class="link" class:miss={!l.path} onclick={() => vault.openByName(l.name, vault.active.path)}>
            {l.name}{#if !l.path}<em>doesn't exist</em>{/if}
          </button>
        {/each}
      {:else}
        <p class="muted msg">This note has no [[…]] links.</p>
      {/if}
    {:else}
      {#if vault.outline.length}
        {@const min = Math.min(...vault.outline.map((h) => h.level))}
        {#each vault.outline as h (h.line)}
          <button class="head" style:padding-left="{8 + (h.level - min) * 14}px" onclick={() => onjump?.(h)}>{h.text}</button>
        {/each}
      {:else}
        <p class="muted msg">This note has no headings.</p>
      {/if}
    {/if}
  </div>
</aside>

<style>
  .right { display: flex; flex-direction: column; min-height: 0; width: 280px; min-width: 280px; background: var(--panel); border-left: 1px solid var(--line-2); }
  .tabs { display: flex; padding: 6px 8px 0; border-bottom: 1px solid var(--line-2); gap: 2px; overflow-x: auto; scrollbar-width: none; }
  .tabs button { padding: 6px 9px; border: 0; background: transparent; color: var(--ink-3); font-size: 13px; white-space: nowrap; border-radius: 6px 6px 0 0; position: relative; }
  .tabs button:hover { color: var(--ink); background: var(--panel-3); }
  .tabs button.on { color: var(--ink); font-weight: 600; }
  .tabs button.on::after { content: ""; position: absolute; left: 6px; right: 6px; bottom: -1px; height: 2px; background: var(--accent); border-radius: 2px; }
  .body { flex: 1; min-height: 0; overflow: auto; padding: 8px 8px 20px; }
  .msg { padding: 8px; font-size: 13.5px; }
  .count { padding: 2px 8px 8px; font-size: 13px; color: var(--ink-3); }
  .grp { margin-bottom: 10px; }
  .file { width: 100%; text-align: left; padding: 4px 8px; border: 0; background: transparent; font-weight: 600; font-size: 14px; border-radius: 6px; }
  .file:hover, .link:hover, .head:hover { background: var(--panel-3); }
  .ctx { padding: 2px 8px 2px 16px; font-size: 13px; color: var(--ink-2); line-height: 1.4; border-left: 2px solid var(--line); margin: 2px 0 2px 8px; }
  .link, .head { width: 100%; display: flex; justify-content: space-between; gap: 8px; text-align: left; padding: 4px 8px; border: 0; background: transparent; border-radius: 6px; font-size: 14px; color: var(--ink-2); }
  .link { color: var(--accent); }
  .link.miss { opacity: 0.65; }
  .link em { font-style: normal; font-size: 13px; color: var(--ink-3); }
</style>
