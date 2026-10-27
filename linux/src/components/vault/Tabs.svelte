<script>
  import { vault, stemOf } from "../../lib/vault.svelte.js";

  const title = (t) => (t.path ? stemOf(t.path) : "New tab");
  const dirty = (t) => !!(t.path && vault.docs[t.path]?.dirty);
</script>

<div class="bar">
  <button class="side" title={vault.leftOpen ? "Hide left sidebar" : "Show left sidebar"} aria-label="Toggle left sidebar" aria-pressed={vault.leftOpen} onclick={() => { vault.leftOpen = !vault.leftOpen; vault.persist(); }}>
    <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M9 4v16" /></svg>
  </button>

  <div class="tabs" role="tablist" aria-label="Open notes">
    {#each vault.tabs as t (t.id)}
      <div class="tab" class:on={t.id === vault.activeId} role="presentation">
        <button class="name" role="tab" aria-selected={t.id === vault.activeId} onclick={() => { vault.activeId = t.id; vault.selected = t.path ?? vault.selected; vault.persist(); }} onauxclick={(e) => e.button === 1 && vault.closeTab(t.id)} title={t.path ?? "New tab"}>
          {#if dirty(t)}<span class="dirty" title="Saving…"></span>{/if}
          <span class="t">{title(t)}</span>
        </button>
        <button class="x" aria-label="Close tab {title(t)}" onclick={() => vault.closeTab(t.id)}>
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18" /></svg>
        </button>
      </div>
    {/each}
    <button class="plus" title="New tab" aria-label="New tab" onclick={() => vault.newTab()}>
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14" /></svg>
    </button>
  </div>

  <button class="side" title={vault.rightOpen ? "Hide right sidebar" : "Show right sidebar"} aria-label="Toggle right sidebar" aria-pressed={vault.rightOpen} onclick={() => { vault.rightOpen = !vault.rightOpen; vault.persist(); }}>
    <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M15 4v16" /></svg>
  </button>
</div>

<style>
  .bar { display: flex; align-items: flex-end; gap: 6px; padding: 6px 8px 0; background: var(--rail); border-bottom: 1px solid var(--line-2); min-height: 40px; }
  .side { width: 30px; height: 30px; margin-bottom: 3px; display: grid; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--ink-3); flex: none; }
  .side:hover { background: var(--panel-3); color: var(--ink); }
  .side[aria-pressed="true"] { color: var(--ink-2); }
  .tabs { flex: 1; min-width: 0; display: flex; align-items: flex-end; gap: 2px; overflow-x: auto; scrollbar-width: none; }
  .tab { display: flex; align-items: center; min-width: 0; max-width: 220px; height: 32px; border-radius: 8px 8px 0 0; color: var(--ink-3); position: relative; flex: 0 1 auto; }
  .tab:hover { background: color-mix(in oklab, var(--ink) 6%, transparent); color: var(--ink-2); }
  .tab.on { background: var(--paper); color: var(--ink); box-shadow: 0 1px 0 var(--paper); }
  .tab.on::before, .tab.on::after { content: ""; position: absolute; bottom: 0; width: 8px; height: 8px; pointer-events: none; }
  .tab.on::before { left: -8px; background: radial-gradient(circle at 0 0, transparent 8px, var(--paper) 8.5px); }
  .tab.on::after { right: -8px; background: radial-gradient(circle at 100% 0, transparent 8px, var(--paper) 8.5px); }
  .name { display: flex; align-items: center; gap: 6px; min-width: 0; padding: 0 4px 0 12px; height: 100%; border: 0; background: transparent; color: inherit; font-size: 13.5px; }
  .t { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .dirty { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); flex: none; }
  .x { width: 22px; height: 22px; margin-right: 5px; display: grid; place-items: center; border: 0; border-radius: 5px; background: transparent; color: var(--ink-3); opacity: 0; flex: none; }
  .tab:hover .x, .tab.on .x, .x:focus-visible { opacity: 1; }
  .x:hover { background: var(--panel-3); color: var(--ink); }
  .plus { width: 28px; height: 28px; margin: 0 0 2px 4px; display: grid; place-items: center; border: 0; border-radius: 6px; background: transparent; color: var(--ink-3); flex: none; }
  .plus:hover { background: var(--panel-3); color: var(--ink); }
</style>
