<script>
  import { app } from "../lib/store.svelte.js";
</script>

{#if app.toast}
  <div class="toast" role="status">
    <span>{app.toast.message}</span>
    {#if app.toast.action}
      <button onclick={() => { app.toast.action.run(); app.toast = null; }}>{app.toast.action.label}</button>
    {/if}
  </div>
{/if}

<style>
  .toast {
    position: fixed; right: 24px; bottom: 24px; z-index: 50;
    display: flex; align-items: center; gap: 16px; max-width: 420px;
    padding: 11px 16px; border-radius: var(--r-lg); background: var(--panel-3); color: var(--ink); border: 1px solid var(--line);
    box-shadow: var(--shadow); font-weight: 600; font-size: 14px;
    animation: in 0.18s ease-out;
  }
  button { border: 0; background: transparent; color: inherit; font-weight: 700; text-decoration: underline; text-underline-offset: 3px; padding: 0; }
  @keyframes in { from { opacity: 0; transform: translateY(8px); } }
</style>
