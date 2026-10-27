<script>
  import { vault, stemOf } from "../../lib/vault.svelte.js";

  let input = $state();
  $effect(() => {
    if (vault.leftTab === "search") input?.focus();
  });

  // Group hits by file, like Obsidian's search results.
  const groups = $derived.by(() => {
    const m = new Map();
    for (const h of vault.results) {
      if (!m.has(h.path)) m.set(h.path, []);
      if (h.line_no > 0) m.get(h.path).push(h);
    }
    return [...m].map(([path, lines]) => ({ path, lines }));
  });
</script>

<div class="search">
  <div class="box">
    <input type="search" bind:this={input} value={vault.query} oninput={(e) => vault.setQuery(e.currentTarget.value)} placeholder="Search the vault…" spellcheck="false" aria-label="Search" />
  </div>
  <div class="results">
    {#if vault.query.trim() && !vault.searching && !groups.length}
      <p class="muted msg">No results for “{vault.query}”.</p>
    {/if}
    {#each groups as g (g.path)}
      <div class="group">
        <button class="file" onclick={() => vault.open(g.path)}>
          <span>{stemOf(g.path)}</span><em>{g.lines.length || ""}</em>
        </button>
        {#each g.lines.slice(0, 5) as l (l.line_no)}
          <button class="line" onclick={() => vault.open(g.path)}>{l.line}</button>
        {/each}
        {#if g.lines.length > 5}<p class="more muted">+ {g.lines.length - 5} more</p>{/if}
      </div>
    {/each}
    {#if !vault.query.trim()}
      <p class="muted msg">Type a word or phrase. Searches the names and contents of all notes.</p>
    {/if}
  </div>
</div>

<style>
  .search { display: flex; flex-direction: column; min-height: 0; flex: 1; }
  .box { padding: 8px; }
  .results { flex: 1; overflow: auto; padding: 0 6px 16px; }
  .msg { padding: 8px; font-size: 13.5px; }
  .group { margin-bottom: 10px; }
  .file { width: 100%; display: flex; justify-content: space-between; gap: 8px; padding: 4px 8px; border: 0; background: transparent; text-align: left; font-weight: 600; font-size: 14px; border-radius: 6px; }
  .file:hover, .line:hover { background: var(--panel-3); }
  .file em { font-style: normal; font-size: 13px; color: var(--ink-3); }
  .line { width: 100%; display: block; padding: 3px 8px 3px 20px; border: 0; background: transparent; text-align: left; color: var(--ink-2); font-size: 13px; line-height: 1.4; border-radius: 6px; overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }
  .more { padding: 0 8px 0 20px; font-size: 13px; }
</style>
