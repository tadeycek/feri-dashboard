<script>
  import { app } from "../lib/store.svelte.js";
  import { addDays, dayKey, hm, minutesOfDay, weekdayShort } from "../lib/time.js";
  import { colorOf, kindShort } from "../lib/subjects.js";

  /** `days`: Dates to show as columns. `events`: all classes (filtered per day here). `range`: {start, end} in minutes. */
  let { days, events, range, ppm = 0.95, header = true, onselect } = $props();

  const hours = $derived(Array.from({ length: (range.end - range.start) / 60 + 1 }, (_, i) => range.start / 60 + i));
  const height = $derived((range.end - range.start) * ppm);
  const todayKey = $derived(dayKey(app.now));

  /** Place overlapping classes side by side. */
  function layout(list) {
    const out = [];
    let cluster = [];
    let clusterEnd = 0;
    const flush = () => {
      const lanes = [];
      const placed = cluster.map((ev) => {
        let lane = lanes.findIndex((t) => t <= ev.s.getTime());
        if (lane < 0) { lane = lanes.length; lanes.push(ev.e.getTime()); } else lanes[lane] = ev.e.getTime();
        return { ev, lane };
      });
      placed.forEach((p) => out.push({ ...p, lanes: lanes.length }));
      cluster = [];
    };
    for (const ev of list) {
      if (cluster.length && ev.s.getTime() >= clusterEnd) flush();
      cluster.push(ev);
      clusterEnd = Math.max(clusterEnd, ev.e.getTime());
    }
    flush();
    return out;
  }

  const columns = $derived(
    days.map((d) => {
      const key = dayKey(d);
      return { d, key, today: key === todayKey, items: layout(events.filter((c) => c.day === key)) };
    }),
  );

  // The now-line belongs to today's column only.
  const nowTop = $derived.by(() => {
    const m = minutesOfDay(app.now);
    return m >= range.start && m <= range.end ? (m - range.start) * ppm : null;
  });
</script>

<div class="grid" style:--cols={days.length}>
  {#if header}
    <div class="days">
      <div class="gutter"></div>
      {#each columns as col (col.key)}
        <div class="dh" class:today={col.today}>
          <span>{weekdayShort(col.d)}</span><b class="num">{col.d.getDate()}</b>
        </div>
      {/each}
    </div>
  {/if}

  <div class="body" style:height="{height}px">
    <div class="gutter">
      {#each hours as h (h)}
        <span class="hl num" style:top="{(h * 60 - range.start) * ppm}px">{h}:00</span>
      {/each}
    </div>
    <div class="lanes">
      {#each hours as h (h)}
        <i class="hline" style:top="{(h * 60 - range.start) * ppm}px"></i>
      {/each}
      {#each columns as col (col.key)}
        <div class="col" class:today={col.today}>
          {#each col.items as { ev, lane, lanes } (ev.uid)}
            {@const h = ((ev.e - ev.s) / 60000) * ppm - 3}
            <button
              class="ev" class:one={h < 44} class:tall={h >= 90} class:past={ev.e <= app.now} class:live={ev.s <= app.now && app.now < ev.e}
              style:--c={colorOf(ev.key)}
              style:top="{(minutesOfDay(ev.s) - range.start) * ppm}px"
              style:height="{h}px"
              style:left="calc({(lane / lanes) * 100}% + 3px)"
              style:width="calc({100 / lanes}% - 6px)"
              title="{ev.subject}, {kindShort(ev.kind)}{ev.room ? ` · ${ev.room}` : ''}, {hm(ev.s)}–{hm(ev.e)}{ev.lecturers.length ? `, ${ev.lecturers.join(', ')}` : ''}"
              onclick={() => onselect?.(ev)}
            >
              <b>{ev.subject}</b>
              {#if h >= 62}
                <span>{kindShort(ev.kind)}{ev.room ? ` · ${ev.room}` : ""}</span>
                <span class="num t">{hm(ev.s)}–{hm(ev.e)}</span>
                {#if h >= 82 && ev.lecturers.length}<span class="who">{ev.lecturers.join(", ")}</span>{/if}
              {:else if h >= 44}
                <span>{kindShort(ev.kind)}{ev.room ? ` · ${ev.room}` : ""} · <span class="num">{hm(ev.s)}–{hm(ev.e)}</span></span>
              {/if}
            </button>
          {/each}
          {#if col.today && nowTop !== null}
            <div class="nowline" style:top="{nowTop}px"><i></i></div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .grid { min-width: 0; }
  .days, .body { display: grid; grid-template-columns: 52px minmax(0, 1fr); }
  .days { grid-template-columns: 52px repeat(var(--cols), minmax(0, 1fr)); border-bottom: 1px solid var(--line); }
  .dh { display: flex; align-items: baseline; gap: 8px; padding: 8px 12px; border-left: 1px solid var(--line-2); font-weight: 600; color: var(--ink-2); }
  .dh b { font-size: 24px; font-weight: 600; color: var(--ink); line-height: 1; }
  .dh.today b { background: var(--accent); color: var(--accent-ink); border-radius: var(--r-sm); padding: 2px 8px; }

  .body { position: relative; margin: 10px 0; }
  .gutter { position: relative; }
  .hl { position: absolute; right: 10px; transform: translateY(-50%); font-size: 13px; color: var(--ink-3); }
  .lanes { position: relative; display: grid; grid-template-columns: repeat(var(--cols), minmax(0, 1fr)); }
  .hline { position: absolute; left: 0; right: 0; height: 1px; background: var(--line-2); }
  .col { position: relative; border-left: 1px solid var(--line-2); }
  .col.today { background: color-mix(in oklab, var(--accent) 4%, transparent); }

  /* Classes: a tinted block, subject colour in the title. No stripe. */
  .ev {
    position: absolute; display: grid; align-content: start; gap: 1px; overflow: hidden; text-align: left;
    padding: 4px 9px; border: 0; border-radius: var(--r-sm);
    background: color-mix(in oklab, var(--c) 17%, var(--panel)); color: var(--ink);
    transition: background 0.12s;
  }
  .ev:hover { background: color-mix(in oklab, var(--c) 27%, var(--panel)); z-index: 2; }
  .ev b { font-size: 14px; line-height: 1.25; overflow: hidden; text-overflow: ellipsis; color: color-mix(in oklab, var(--c) 72%, var(--ink)); }
  .ev b { white-space: nowrap; }
  .ev.tall b { white-space: normal; }
  .ev span { font-size: 13px; color: var(--ink-2); line-height: 1.25; }
  .ev .who { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ev .t { font-size: 14px; font-weight: 600; color: var(--ink-2); }
  .ev.past { opacity: 0.5; }
  .ev.live { background: color-mix(in oklab, var(--c) 34%, var(--panel)); outline: 1.5px solid var(--c); outline-offset: -1.5px; }

  .nowline { position: absolute; left: 0; right: 0; height: 0; border-top: 2px solid var(--accent); z-index: 3; pointer-events: none; }
  .nowline i { position: absolute; left: -5px; top: -6px; width: 10px; height: 10px; border-radius: 50%; background: var(--accent); }
</style>
