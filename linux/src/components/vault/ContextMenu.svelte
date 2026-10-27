<script>
  import { vault } from "../../lib/vault.svelte.js";

  const m = $derived(vault.menu);
  // Keep the menu inside the window.
  const pos = $derived(m ? { x: Math.min(m.x, window.innerWidth - 230), y: Math.min(m.y, window.innerHeight - (m.items.length * 32 + 16)) } : null);

  const close = () => (vault.menu = null);
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && close()} onblur={close} />

{#if m}
  <div class="scrim" role="presentation" onclick={close} oncontextmenu={(e) => { e.preventDefault(); close(); }}></div>
  <ul class="menu" role="menu" style:left="{pos.x}px" style:top="{pos.y}px">
    {#each m.items as it, i (i)}
      {#if it.sep}
        <li class="sep" role="separator"></li>
      {:else}
        <li role="none"><button role="menuitem" class:danger={it.danger} onclick={() => { close(); it.run(); }}>{it.label}</button></li>
      {/if}
    {/each}
  </ul>
{/if}

<style>
  .scrim { position: fixed; inset: 0; z-index: 60; }
  .menu { position: fixed; z-index: 61; list-style: none; margin: 0; padding: 5px; min-width: 210px; background: var(--panel-2); border: 1px solid var(--line); border-radius: 8px; box-shadow: var(--shadow); }
  .menu button { width: 100%; text-align: left; padding: 6px 12px; border: 0; background: transparent; border-radius: 5px; font-size: 14px; }
  .menu button:hover { background: var(--accent); color: var(--accent-ink); }
  .menu button.danger { color: var(--danger); }
  .menu button.danger:hover { background: var(--danger); color: #fff; }
  .sep { height: 1px; margin: 5px 4px; background: var(--line); }
</style>
