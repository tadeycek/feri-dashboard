<script>
  import { app } from "../lib/store.svelte.js";
  import TimeGrid from "../components/TimeGrid.svelte";
  import EventDetail from "../components/EventDetail.svelte";
  import { addDays, count, minutesOfDay, startOfWeek, weekRange } from "../lib/time.js";

  let offset = $state(0);
  let selected = $state(null);

  const monday = $derived(addDays(startOfWeek(app.now), offset * 7));
  const weekEvents = $derived(app.classes.filter((c) => c.s >= monday && c.s < addDays(monday, 7)));
  const weekend = $derived(weekEvents.some((c) => c.s.getDay() === 0 || c.s.getDay() === 6));
  const days = $derived(Array.from({ length: weekend ? 7 : 5 }, (_, i) => addDays(monday, i)));

  // Always 07:00-21:00 (wider only if a class falls outside); the hour height follows the window height.
  let vh = $state(typeof window === "undefined" ? 860 : window.innerHeight);
  const range = $derived({
    start: Math.floor(Math.min(7 * 60, ...weekEvents.map((c) => minutesOfDay(c.s))) / 60) * 60,
    end: Math.ceil(Math.max(21 * 60, ...weekEvents.map((c) => minutesOfDay(c.e))) / 60) * 60,
  });
  // 235 px is the page chrome above and below the grid.
  const ppm = $derived(Math.min(0.9, Math.max(0.6, (vh - 235) / (range.end - range.start))));
</script>

<svelte:window bind:innerHeight={vh} />

<div class="page">
  <header class="head">
    <div>
      <h1 class="num">{weekRange(monday, addDays(monday, days.length - 1))}</h1>
      <p class="muted">{weekEvents.length ? `${count(weekEvents.length, "class", "classes")} this week.` : "No classes this week."}</p>
    </div>
    <div class="nav">
      <button class="btn ghost small" onclick={() => (offset -= 1)} aria-label="Previous week">Previous</button>
      <button class="btn ghost small" onclick={() => (offset = 0)} disabled={offset === 0}>This week</button>
      <button class="btn ghost small" onclick={() => (offset += 1)} aria-label="Next week">Next</button>
    </div>
  </header>

  <div class="cal panel">
    <TimeGrid {days} events={weekEvents} {range} {ppm} onselect={(ev) => (selected = ev)} />
  </div>
</div>

<EventDetail ev={selected} onclose={() => (selected = null)} />

<style>
  .page { display: grid; gap: 18px; }
  .head { display: flex; align-items: flex-end; justify-content: space-between; gap: 16px; }
  .head h1 { font-size: 44px; line-height: 0.95; font-weight: 700; }
  .head p { margin-top: 6px; }
  .nav { display: flex; gap: 8px; }
  .cal { overflow: hidden; }
</style>
