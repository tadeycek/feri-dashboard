<script>
  import { app } from "../lib/store.svelte.js";
  import { ago, count } from "../lib/time.js";

  const items = [
    { id: "today", label: "Today", key: "1", d: "M12 3v9l5 3M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" },
    { id: "week", label: "Week", key: "2", d: "M4 6h16M4 12h16M4 18h16M9 4v16M15 4v16" },
    { id: "deadlines", label: "Deadlines", key: "3", d: "M5 4h14v16H5zM9 9l2 2 4-4M9 15h6" },
    { id: "notes", label: "Notes", key: "4", d: "M4 20h4L19 9l-4-4L4 16v4ZM13 7l4 4" },
  ];

  // On the notes page the rail shrinks to an icon strip, like Obsidian's ribbon.
  const compact = $derived(app.view === "notes");

  const status = $derived.by(() => {
    if (app.syncing) return "Refreshing …";
    if (app.offline) return "Offline";
    if (app.fetchedAt) return `Updated ${ago(new Date(app.fetchedAt), app.now)}`;
    return "Not updated yet";
  });
  const changes = $derived(count(app.unseenChanges.length, "timetable change"));
</script>

<nav class="rail" class:compact aria-label="Main navigation">
  <div class="brand" title="FERI">
    <svg viewBox="0 0 32 32" width="26" height="26" aria-hidden="true">
      <rect x="3" y="4" width="11" height="4" rx="1.5" fill="var(--c-MAT)" />
      <rect x="3" y="11" width="22" height="4" rx="1.5" fill="var(--c-UVR)" />
      <rect x="3" y="18" width="16" height="4" rx="1.5" fill="var(--c-PRO1)" />
      <rect x="3" y="25" width="8" height="4" rx="1.5" fill="var(--c-SPO)" />
    </svg>
    {#if !compact}<span class="num">FERI</span>{/if}
  </div>

  <ul>
    {#each items as it (it.id)}
      <li>
        <button class="nav" class:active={app.view === it.id} aria-current={app.view === it.id ? "page" : undefined} aria-label={it.label} title="{it.label} ({it.key})" onclick={() => app.go(it.id)}>
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={it.d} /></svg>
          {#if !compact}<span>{it.label}</span><kbd>{it.key}</kbd>{/if}
        </button>
      </li>
    {/each}
  </ul>

  <div class="foot">
    {#if app.unseenChanges.length}
      <button class="alert" onclick={() => app.go("today")} title={changes}>
        <span class="dot"></span>
        {#if !compact}{changes}{/if}
      </button>
    {/if}
    {#if !compact}
      <p class="status" class:off={app.offline} title={app.error}><span class="led"></span>{status}</p>
    {:else}
      <span class="led solo" class:off={app.offline} title={status}></span>
    {/if}
    <div class="actions">
      <button class="icon" onclick={() => app.refresh()} disabled={app.syncing} aria-label="Refresh timetable (r)" title="Refresh timetable (r)">
        <svg class:spin={app.syncing} viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.3-5.7M20 4v5h-5" /></svg>
      </button>
      <button class="icon" onclick={() => (app.settingsDialog = true)} aria-label="Settings" title="Settings">
        <svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 7h9M17 7h3M4 17h3M11 17h9M15 4v6M9 14v6" /></svg>
      </button>
    </div>
  </div>
</nav>

<style>
  .rail {
    --hover: color-mix(in oklab, var(--rail-ink) 8%, transparent);
    background: var(--rail); color: var(--rail-ink); border-right: 1px solid var(--line-2);
    display: flex; flex-direction: column; padding: 18px 12px 14px; min-height: 0; overflow: hidden;
  }
  .rail.compact { padding: 14px 8px; align-items: center; }
  .brand { display: flex; align-items: center; gap: 10px; padding: 0 8px 22px; }
  .compact .brand { padding: 0 0 18px; }
  .brand span { font-size: 25px; font-weight: 700; letter-spacing: 0.04em; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 2px; width: 100%; }
  .nav {
    width: 100%; display: flex; align-items: center; gap: 12px;
    padding: 8px 12px; border: 0; border-radius: var(--r-sm);
    background: transparent; color: var(--rail-dim); font-weight: 600; text-align: left;
    transition: background 0.12s, color 0.12s;
  }
  .compact .nav { justify-content: center; padding: 10px 0; }
  .nav:hover { color: var(--rail-ink); background: var(--hover); }
  .nav.active { color: var(--rail-ink); background: var(--accent-soft); }
  .nav.active svg { color: var(--accent); }
  .nav span { flex: 1; }
  kbd { font: 600 12px var(--font-ui); color: var(--rail-dim); opacity: 0; transition: opacity 0.15s; }
  .nav:hover kbd { opacity: 1; }

  .foot { margin-top: auto; display: grid; gap: 10px; width: 100%; justify-items: stretch; }
  .compact .foot { justify-items: center; }
  .alert {
    display: flex; align-items: center; gap: 9px; width: 100%; padding: 7px 10px;
    border: 1px solid color-mix(in oklab, var(--warn) 50%, transparent); border-radius: var(--r-sm);
    background: transparent; color: var(--rail-ink); font-size: 13.5px; font-weight: 600; text-align: left;
  }
  .alert:hover { background: color-mix(in oklab, var(--warn) 12%, transparent); }
  .compact .alert { width: 34px; height: 34px; justify-content: center; padding: 0; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--warn); flex: none; }
  .status { display: flex; align-items: center; gap: 8px; margin: 0 4px; font-size: 13.5px; color: var(--rail-dim); }
  .led { width: 7px; height: 7px; border-radius: 50%; background: var(--ok); flex: none; }
  .status.off .led, .led.off { background: var(--warn); }
  .led.solo { margin: 2px 0; }
  .actions { display: flex; gap: 6px; }
  .compact .actions { flex-direction: column; }
  .icon { width: 36px; height: 32px; display: grid; place-items: center; border: 1px solid var(--line); border-radius: var(--r-sm); background: transparent; color: var(--rail-dim); }
  .icon:hover:not(:disabled) { background: var(--hover); color: var(--rail-ink); }
  .icon:disabled { opacity: 0.6; cursor: default; }
  .spin { animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
