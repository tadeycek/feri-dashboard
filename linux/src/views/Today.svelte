<script>
  import { onMount } from "svelte";
  import { app } from "../lib/store.svelte.js";
  import { vault, stemOf, isMd } from "../lib/vault.svelte.js";
  import TimeGrid from "../components/TimeGrid.svelte";
  import EventDetail from "../components/EventDetail.svelte";
  import VaultSetup from "../components/VaultSetup.svelte";
  import { addDays, ago, count, dayKey, daysUntil, dueLabel, hm, longDate, minutesOfDay, numericDate, startOfDay, startOfWeek, weekdayShort } from "../lib/time.js";
  import { colorOf, kindLong, kindShort, rokTypeLabel } from "../lib/subjects.js";

  // The vault tree gives note counts per subject and the recent-notes list.
  onMount(() => { if (!vault.entries.length) vault.loadTree(); });

  let selected = $state(null);
  const now = $derived(app.now);
  const todayKey = $derived(dayKey(now));
  const tomorrowKey = $derived(dayKey(addDays(now, 1)));

  const weekdayLong = (d) => longDate(d).split(" ")[0];
  const whenShort = (c) => `${c.day === todayKey ? "Today" : c.day === tomorrowKey ? "Tomorrow" : weekdayShort(c.s)} ${hm(c.s)}`;

  // One plain sentence instead of a hero: what is on now, or what comes next.
  const statusLine = $derived.by(() => {
    const c = app.current;
    if (c) return `${c.subject} ${kindLong(c.kind).toLowerCase()} is on until ${hm(c.e)}${c.room ? ` in ${c.room}` : ""}.`;
    const n = app.next;
    if (!n) return "No upcoming classes.";
    const room = n.room ? ` in ${n.room}` : "";
    if (n.day === todayKey) return `Next: ${n.subject} at ${hm(n.s)}${room}.`;
    const when = n.day === tomorrowKey ? "tomorrow" : `on ${weekdayLong(n.s)}`;
    return `${app.today.length ? "Done for today. " : "No classes today. "}Next: ${n.subject} ${when} at ${hm(n.s)}${room}.`;
  });

  // Today's grid always covers 07:00-21:00 (wider only if a class falls outside). The hour height follows the window
  // height, so the whole day fits on screen with compact hour rows.
  let vh = $state(typeof window === "undefined" ? 860 : window.innerHeight);
  const range = $derived.by(() => {
    const mins = app.today.flatMap((c) => [minutesOfDay(c.s), minutesOfDay(c.e)]);
    return { start: Math.floor(Math.min(7 * 60, ...mins) / 60) * 60, end: Math.ceil(Math.max(21 * 60, ...mins) / 60) * 60 };
  });
  // 252 px is the page chrome above and below the grid.
  const ppm = $derived(Math.min(0.9, Math.max(0.6, (vh - 252) / (range.end - range.start))));

  // Deadlines: overdue first, then the nearest upcoming.
  const deadlines = $derived([...app.overdueRoki, ...app.upcomingRoki].slice(0, 6));

  const comingUp = $derived.by(() => {
    const out = [];
    for (let i = 1; i <= 10 && out.length < 3; i++) {
      const d = addDays(now, i);
      const list = app.classes.filter((c) => c.day === dayKey(d));
      if (list.length) out.push({ key: dayKey(d), label: i === 1 ? "Tomorrow" : longDate(d), list });
    }
    return out;
  });

  // Class hours per day of the current week.
  const week = $derived.by(() => {
    const monday = startOfWeek(now);
    return Array.from({ length: 5 }, (_, i) => {
      const d = addDays(monday, i);
      const k = dayKey(d);
      const hours = app.classes.filter((c) => c.day === k).reduce((s, c) => s + (c.e - c.s) / 3600000, 0);
      return { d, k, hours, today: k === todayKey, past: k < todayKey };
    });
  });
  const maxHours = $derived(Math.max(6, ...week.map((w) => w.hours)));
  const weekHours = $derived(week.reduce((s, w) => s + w.hours, 0));
  const fmtH = (h) => (h ? `${Number.isInteger(h) ? h : h.toFixed(1)} h` : "–");

  const subjects = $derived(
    app.subjects.map((s) => ({
      ...s,
      next: app.classes.find((c) => c.key === s.key && c.e > now),
      notes: vault.entries.filter((e) => !e.is_dir && isMd(e.path) && e.path.startsWith(`Predmeti/${s.name}/`) && stemOf(e.path) !== s.name).length,
    })),
  );

  // Generated index notes, templates and subject hubs are not recent work.
  const INDEX_NOTES = new Set(["Urnik.md", "🏠 Domov.md", "Pojmi/Pojmi.md"]);
  const recent = $derived(
    [...vault.notes]
      .filter((e) => !e.path.startsWith("Predloge/") && !INDEX_NOTES.has(e.path) && !/^Predmeti\/[^/]+\/[^/]+\.md$/.test(e.path))
      .sort((a, b) => b.modified - a.modified)
      .slice(0, 6),
  );
  const keyForPath = (p) => app.subjects.find((s) => p.startsWith(`Predmeti/${s.name}/`))?.key ?? "";
  const changeLabel = (k) => ({ dodano: "Added", odstranjeno: "Removed", spremenjeno: "Changed" })[k] ?? k;
  const openNote = (path) => { app.go("notes"); vault.open(path); };
</script>

<svelte:window bind:innerHeight={vh} />

<div class="page">
  <header class="head">
    <h1 class="num">{longDate(now)}</h1>
    <p class="status">{statusLine}</p>
  </header>

  {#if app.error && app.offline}
    <p class="banner">Couldn't refresh the timetable, showing the last saved one. <span class="muted">{app.error}</span></p>
  {/if}
  {#if app.vaultError}<p class="banner">{app.vaultError}</p><div class="banner-setup"><VaultSetup /></div>{/if}

  <div class="grid">
    <!-- Column 1: today's timetable -->
    <section class="section today">
      <header>
        <h2 class="section-title">Today</h2>
        <span class="muted">{app.today.length ? count(app.today.length, "class", "classes") : "No classes"}</span>
      </header>
      {#if app.today.length}
        <div class="gridbox">
          <TimeGrid days={[startOfDay(now)]} events={app.today} {range} {ppm} header={false} onselect={(ev) => (selected = ev)} />
        </div>
      {:else}
        <p class="empty-line">Nothing scheduled today. The whole week is under Week.</p>
      {/if}
    </section>

    <!-- Column 2: deadlines and what is coming -->
    <div class="stack">
      <section class="section">
        <header>
          <h2 class="section-title">Deadlines</h2>
          <button class="link" onclick={() => (app.rokDialog = true)}>Add</button>
        </header>
        {#if deadlines.length}
          <ul class="rows">
            {#each deadlines as r (r.file)}
              {@const n = daysUntil(r.datum, app.now)}
              <li style:--c={colorOf(app.keyOf(r.predmet))}>
                <button class="tick" aria-label="Mark as done: {r.title}" onclick={() => app.toggleRok(r.file)}></button>
                <div class="what">
                  <p class="name">{r.title}</p>
                  <p class="sub">{r.predmet || rokTypeLabel(r.vrsta)}{r.predmet ? ` · ${rokTypeLabel(r.vrsta)}` : ""}</p>
                </div>
                <p class="due" class:late={n < 0} class:soon={n >= 0 && n <= 2}>{dueLabel(n)}<span>{numericDate(r.datum)}{r.ura ? `, ${r.ura}` : ""}</span></p>
              </li>
            {/each}
          </ul>
          <button class="link more" onclick={() => app.go("deadlines")}>All deadlines</button>
        {:else}
          <p class="empty-line">No open deadlines.</p>
        {/if}
      </section>

      <section class="section">
        <header><h2 class="section-title">Coming up</h2></header>
        {#if comingUp.length}
          {#each comingUp as g (g.key)}
            <div class="day">
              <p class="dayname">{g.label}</p>
              <ul class="plain">
                {#each g.list as c (c.uid)}
                  <li style:--c={colorOf(c.key)}>
                    <span class="num t">{hm(c.s)}</span>
                    <span class="n"><span class="dot"></span>{c.subject} <em>{kindShort(c.kind)}</em></span>
                    <span class="muted r">{c.room}</span>
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        {:else}
          <p class="empty-line">Nothing in the next few days.</p>
        {/if}
      </section>
    </div>

    <!-- Column 3: subjects, changes, notes -->
    <div class="stack">
      {#if app.unseenChanges.length}
        <section class="section changes">
          <header>
            <h2 class="section-title">Timetable changes</h2>
            <button class="link" onclick={() => app.markChangesSeen()}>Got it</button>
          </header>
          <ul class="plain chg">
            {#each app.unseenChanges as ch (ch.uid + ch.kind + ch.detected_at)}
              <li><b class="k {ch.kind}">{changeLabel(ch.kind)}</b> {ch.subject} {kindShort(ch.ekind)}, {ch.detail}</li>
            {/each}
          </ul>
        </section>
      {/if}

      <section class="section">
        <header><h2 class="section-title">Subjects</h2></header>
        <ul class="rows subj">
          {#each subjects as s (s.key)}
            <li style:--c={colorOf(s.key)}>
              <span class="dot"></span>
              <div class="what">
                <p class="name">{s.name}</p>
                <p class="sub">{s.next ? `${whenShort(s.next)}${s.next.room ? ` · ${s.next.room}` : ""}` : "No upcoming classes"}</p>
              </div>
              <button class="cnt" class:zero={!s.notes} title="Open subject in Notes" onclick={() => { app.go("notes"); vault.open(`Predmeti/${s.name}/${s.name}.md`, { mode: "read" }); }}>{count(s.notes, "note")}</button>
            </li>
          {/each}
        </ul>
      </section>

      <section class="section">
        <header>
          <h2 class="section-title">This week</h2>
          <span class="muted">{fmtH(weekHours)} of classes</span>
        </header>
        <div class="bars" role="img" aria-label="Class hours per day">
          {#each week as w (w.k)}
            <div class="bar" class:today={w.today} class:past={w.past}>
              <span class="num h">{fmtH(w.hours)}</span>
              <i style:height="{Math.max(2, (w.hours / maxHours) * 54)}px"></i>
              <span class="dn">{weekdayShort(w.d)}</span>
            </div>
          {/each}
        </div>
      </section>

      <section class="section">
        <header>
          <h2 class="section-title">Recent notes</h2>
          <button class="link" onclick={() => app.go("notes")}>All notes</button>
        </header>
        {#if recent.length}
          <ul class="rows recent">
            {#each recent as n (n.path)}
              <li style:--c={colorOf(keyForPath(n.path))}>
                <span class="dot"></span>
                <button class="rn" onclick={() => openNote(n.path)}>{stemOf(n.path)}</button>
                <span class="muted">{ago(new Date(n.modified * 1000), now)}</span>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="empty-line">No notes yet. Use "Write a note" on a class.</p>
        {/if}
      </section>
    </div>
  </div>
</div>

<EventDetail ev={selected} onclose={() => (selected = null)} />

<style>
  .page { display: grid; gap: 26px; container-type: inline-size; }
  .head h1 { font-size: 52px; line-height: 0.95; font-weight: 700; }
  .status { margin-top: 8px; color: var(--ink-2); font-size: 16px; }

  .grid { display: grid; grid-template-columns: minmax(300px, 1.1fr) minmax(280px, 1fr) minmax(260px, 0.9fr); gap: 48px; align-items: start; }
  .stack { display: grid; gap: 34px; min-width: 0; align-content: start; }
  .gridbox { background: var(--panel); border: 1px solid var(--line); border-radius: var(--r-lg); overflow: hidden; }

  .name { font-weight: 600; line-height: 1.3; }
  .sub { font-size: 13.5px; color: var(--ink-3); line-height: 1.35; }
  .rows, .plain { list-style: none; margin: 0; padding: 0; }
  .rows li { display: grid; grid-template-columns: 22px minmax(0, 1fr) auto; gap: 12px; align-items: center; padding: 9px 0; border-top: 1px solid var(--line-2); }
  .rows li:first-child { border-top: 0; }
  .more { justify-self: start; margin-top: 2px; }

  .tick { width: 18px; height: 18px; border-radius: var(--r-sm); border: 2px solid var(--c, var(--ink-3)); background: transparent; padding: 0; }
  .tick:hover { background: color-mix(in oklab, var(--c, var(--ink-3)) 28%, transparent); }
  .due { text-align: right; display: grid; font-weight: 600; line-height: 1.2; font-size: 15px; }
  .due span { font-weight: 400; font-size: 13px; color: var(--ink-3); }
  .due.soon { color: var(--warn); }
  .due.late { color: var(--danger); }

  .day + .day { margin-top: 12px; }
  .dayname { font-size: 13.5px; font-weight: 600; color: var(--ink-2); margin-bottom: 4px; }
  .plain li { display: grid; grid-template-columns: 46px minmax(0, 1fr) auto; gap: 10px; align-items: baseline; padding: 3px 0; }
  .plain .t { font-size: 18px; font-weight: 600; text-align: right; }
  .plain .n { font-weight: 600; font-size: 14.5px; display: flex; align-items: center; gap: 8px; min-width: 0; }
  .plain .n em { font-style: normal; font-weight: 400; color: var(--ink-3); }
  .plain .r { font-size: 13px; }
  .dot { width: 9px; height: 9px; border-radius: 2px; background: var(--c); flex: none; }

  .bars { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; align-items: end; }
  .bar { display: grid; justify-items: stretch; gap: 4px; }
  .bar .h { font-size: 14px; color: var(--ink-2); text-align: center; }
  .bar i { display: block; background: var(--line); border-radius: 2px 2px 0 0; }
  .bar.today i { background: var(--accent); }
  .bar.today .h, .bar.today .dn { color: var(--ink); font-weight: 600; }
  .bar.past { opacity: 0.55; }
  .bar .dn { font-size: 13px; color: var(--ink-3); text-align: center; }

  .subj .what { min-width: 0; }
  .cnt { border: 0; background: transparent; padding: 0; font-size: 13.5px; color: var(--ink-2); }
  .cnt.zero { color: var(--ink-3); }
  .cnt:hover { color: var(--accent); text-decoration: underline; text-underline-offset: 3px; }
  .subj li { grid-template-columns: 9px minmax(0, 1fr) auto; }
  .recent li { grid-template-columns: 9px minmax(0, 1fr) auto; padding: 6px 0; font-size: 13.5px; }
  .rn { text-align: left; border: 0; background: transparent; padding: 0; font-size: 14.5px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .rn:hover { color: var(--accent); }

  .chg { display: grid; gap: 8px; font-size: 14.5px; }
  .chg li { display: block; padding: 0; }
  .k { font-weight: 700; }
  .k.spremenjeno { color: var(--warn); }
  .k.dodano { color: var(--ok); }
  .k.odstranjeno { color: var(--danger); }

  /* The layout follows the width of the content area, not the window, so it also works with the narrow icon rail. */
  @container (max-width: 1040px) {
    .grid { grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr); gap: 40px; }
    .grid > .stack:last-child { grid-column: 1 / -1; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 30px 40px; }
  }
  @container (max-width: 640px) {
    .grid { grid-template-columns: minmax(0, 1fr); }
    .head h1 { font-size: 40px; }
  }
</style>
