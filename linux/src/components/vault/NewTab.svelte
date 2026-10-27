<script>
  import { vault, stemOf } from "../../lib/vault.svelte.js";

  const recent = $derived([...vault.notes].sort((a, b) => b.modified - a.modified).slice(0, 5));
  const empty = $derived(!vault.active);
</script>

<div class="page">
  <div class="box">
    {#if empty}<p class="lead">No note is open.</p>{/if}
    <ul>
      <li><button onclick={() => vault.createNote()}>Create new note <kbd>Ctrl N</kbd></button></li>
      <li><button onclick={() => (vault.switcher = true)}>Go to file <kbd>Ctrl O</kbd></button></li>
      {#if vault.active && vault.tabs.length > 1}
        <li><button onclick={() => vault.closeTab(vault.active.id)}>Close <kbd>Ctrl W</kbd></button></li>
      {/if}
    </ul>
    {#if recent.length}
      <h3>Recently modified</h3>
      <ul class="recent">
        {#each recent as n (n.path)}
          <li><button onclick={() => vault.open(n.path)}>{stemOf(n.path)}</button></li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .page { flex: 1; display: grid; place-items: center; background: var(--paper); padding: 24px; }
  .box { display: grid; gap: 10px; text-align: center; justify-items: center; }
  .lead { color: var(--ink-3); margin-bottom: 4px; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; justify-items: center; }
  button { background: transparent; border: 0; color: var(--accent); font-size: 17px; padding: 4px 8px; border-radius: 6px; }
  button:hover { text-decoration: underline; }
  kbd { font: 500 12px var(--font-ui); color: var(--ink-3); margin-left: 8px; border: 1px solid var(--line); border-radius: 4px; padding: 1px 6px; }
  h3 { margin-top: 18px; font: 600 13px var(--font-ui); color: var(--ink-3); }
  .recent button { font-size: 15px; color: var(--ink-2); }
</style>
