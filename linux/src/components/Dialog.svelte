<script>
  let { open = false, title, onclose, children, width = 520 } = $props();
  let el = $state();

  $effect(() => {
    if (!el) return;
    if (open && !el.open) el.showModal();
    else if (!open && el.open) el.close();
  });
</script>

<dialog bind:this={el} {onclose} onclick={(e) => e.target === el && onclose()} style:--w="{width}px">
  {#if open}
    <div class="box">
      <header>
        <h2 class="section-title">{title}</h2>
        <button class="x" onclick={onclose} aria-label="Close">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
        </button>
      </header>
      {@render children()}
    </div>
  {/if}
</dialog>

<style>
  dialog { padding: 0; border: 1px solid var(--line); border-radius: var(--r-lg); background: var(--panel); color: var(--ink); width: min(var(--w), calc(100vw - 48px)); max-height: calc(100vh - 48px); box-shadow: var(--shadow); }
  dialog::backdrop { background: rgb(0 0 0 / 0.55); }
  dialog[open] { animation: fade 0.12s ease-out; }
  @keyframes fade { from { opacity: 0; } }
  .box { padding: 22px 24px 24px; display: grid; gap: 18px; }
  header { display: flex; align-items: center; justify-content: space-between; }
  .x { width: 32px; height: 32px; display: grid; place-items: center; border: 0; border-radius: var(--r-sm); background: transparent; color: var(--ink-3); }
  .x:hover { background: var(--panel-2); color: var(--ink); }
</style>
