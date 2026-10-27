<script>
  import { app } from "../lib/store.svelte.js";
  import VaultSetup from "../components/VaultSetup.svelte";
  import { count, dayKey, daysUntil, dueLabel, numericDate } from "../lib/time.js";
  import { colorOf, rokTypeLabel } from "../lib/subjects.js";

  const open = $derived(app.roki.filter((r) => !r.opravljeno));
  const done = $derived(app.roki.filter((r) => r.opravljeno).reverse());
  const overdue = $derived(open.filter((r) => r.datum < dayKey(app.now)));
  const upcoming = $derived(open.filter((r) => r.datum >= dayKey(app.now)));
  const nextUp = $derived(upcoming[0]);
</script>

<div class="page">
  <header class="head">
    <div>
      <h1 class="num">Deadlines</h1>
      <p class="muted">Stored in your Obsidian vault, in the Roki folder.</p>
    </div>
    <button class="btn" onclick={() => (app.rokDialog = true)}>Add deadline</button>
  </header>

  {#if app.vaultError}<p class="banner">{app.vaultError}</p><div class="banner-setup"><VaultSetup /></div>{/if}

  {#snippet row(r)}
    {@const n = daysUntil(r.datum, app.now)}
    <li class="r" class:late={n < 0 && !r.opravljeno} class:done={r.opravljeno} style:--c={colorOf(app.keyOf(r.predmet))}>
      <button class="tick" class:on={r.opravljeno} aria-label={r.opravljeno ? `Mark as not done: ${r.title}` : `Mark as done: ${r.title}`} onclick={() => app.toggleRok(r.file)}>
        {#if r.opravljeno}<svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>{/if}
      </button>
      <div class="what">
        <p class="name">{r.title}</p>
        <p class="sub">
          {rokTypeLabel(r.vrsta)}{#if r.predmet}<span class="dot"></span>{r.predmet}{/if}{#if r.prostor} · {r.prostor}{/if}
        </p>
      </div>
      <div class="due">
        <b class="num">{r.opravljeno ? "done" : dueLabel(n)}</b>
        <span>{numericDate(r.datum)}{r.ura ? `, ${r.ura}` : ""}</span>
      </div>
      <button class="link" aria-label="Open in Obsidian: {r.title}" onclick={() => app.openNote(`Roki/${r.file}`)}>In Obsidian</button>
    </li>
  {/snippet}

  <div class="cols">
    <section class="section">
      <header><h2 class="section-title">Upcoming</h2></header>
      {#if upcoming.length}
        <ul class="list">{#each upcoming as r (r.file)}{@render row(r)}{/each}</ul>
      {:else}
        <p class="empty-line">No upcoming deadlines. Add a midterm, an exam or an assignment.</p>
      {/if}
    </section>

    <div class="stack">
      <section class="section">
        <header><h2 class="section-title">Overview</h2></header>
        <p class="sum">{open.length} open, <span class:bad={overdue.length}>{overdue.length} overdue</span>, {done.length} done.</p>
        {#if nextUp}
          <p class="next">Next: <b>{nextUp.title}</b> {dueLabel(daysUntil(nextUp.datum, app.now))}{nextUp.predmet ? `, ${nextUp.predmet}` : ""}.</p>
        {/if}
      </section>

      {#if overdue.length}
        <section class="section">
          <header><h2 class="section-title late-t">Overdue</h2><span class="muted">{count(overdue.length, "item")}</span></header>
          <ul class="list">{#each overdue as r (r.file)}{@render row(r)}{/each}</ul>
        </section>
      {/if}

      {#if done.length}
        <details class="section fold">
          <summary><h2 class="section-title">Done</h2><span class="muted num">{done.length}</span></summary>
          <ul class="list">{#each done as r (r.file)}{@render row(r)}{/each}</ul>
        </details>
      {/if}
    </div>
  </div>
</div>

<style>
  .page { display: grid; gap: 26px; }
  .head { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; }
  .head h1 { font-size: 46px; line-height: 0.95; font-weight: 700; }
  .head p { margin-top: 6px; }
  .cols { display: grid; grid-template-columns: minmax(0, 1.7fr) minmax(280px, 1fr); gap: 44px; align-items: start; }
  .stack { display: grid; gap: 30px; min-width: 0; }
  .late-t { color: var(--danger); }
  .list { list-style: none; margin: 0; padding: 0; }
  .r { display: grid; grid-template-columns: 22px minmax(0, 1fr) auto auto; gap: 16px; align-items: center; padding: 11px 0; border-top: 1px solid var(--line-2); }
  .r:first-child { border-top: 0; }
  .name { font-weight: 600; font-size: 16px; }
  .sub { display: flex; align-items: center; gap: 7px; font-size: 14px; color: var(--ink-3); margin-top: 1px; }
  .sub .dot { background: var(--c); border-radius: 2px; margin-left: 4px; }
  .due { text-align: right; display: grid; line-height: 1.15; }
  .due b { font-size: 20px; font-weight: 600; }
  .due span { font-size: 13.5px; color: var(--ink-3); }
  .late .due b { color: var(--danger); }
  .done .name { text-decoration: line-through; text-decoration-color: var(--ink-3); color: var(--ink-3); }
  .done .due b { font-size: 16px; color: var(--ok); }

  .tick { width: 20px; height: 20px; display: grid; place-items: center; border-radius: var(--r-sm); border: 2px solid var(--c, var(--ink-3)); background: transparent; color: var(--paper); padding: 0; }
  .tick:hover { background: color-mix(in oklab, var(--c, var(--ink-3)) 25%, transparent); }
  .tick.on { background: var(--c, var(--ink-3)); }

  .sum { color: var(--ink-2); }
  .sum .bad { color: var(--danger); font-weight: 600; }
  .next { color: var(--ink-2); line-height: 1.5; }
  .next b { color: var(--ink); }

  .fold summary { display: flex; align-items: baseline; justify-content: space-between; gap: 12px; cursor: pointer; padding-bottom: 7px; border-bottom: 1px solid var(--line); list-style: none; }
  .fold summary::-webkit-details-marker { display: none; }
  .fold[open] summary { margin-bottom: 4px; }

  @media (max-width: 980px) { .cols { grid-template-columns: minmax(0, 1fr); gap: 30px; } }
</style>
