<script>
  import { app } from "../lib/store.svelte.js";
  import Dialog from "./Dialog.svelte";
  import { hm, longDate } from "../lib/time.js";
  import { colorOf, kindLong } from "../lib/subjects.js";

  /** `ev` is the selected class (or null). `onclose` clears it. */
  let { ev, onclose } = $props();
</script>

<Dialog open={ev !== null} title={ev?.subject ?? ""} onclose={onclose} width={460}>
  {#if ev}
    <div class="detail" style:--c={colorOf(ev.key)}>
      <p class="kind"><span class="dot"></span>{kindLong(ev.kind)}{ev.tags.length ? ` (${ev.tags.join(", ")})` : ""}</p>
      <dl>
        <dt>When</dt><dd>{longDate(ev.s)}, <span class="num">{hm(ev.s)}–{hm(ev.e)}</span></dd>
        {#if ev.room}<dt>Room</dt><dd>{ev.room}</dd>{/if}
        {#if ev.lecturers.length}<dt>Taught by</dt><dd>{ev.lecturers.join(", ")}</dd>{/if}
        {#if ev.groups}<dt>Groups</dt><dd>{ev.groups}</dd>{/if}
      </dl>
      <div class="row">
        <button class="btn ghost" onclick={onclose}>Close</button>
        <button class="btn" onclick={() => { const e = ev; onclose(); app.startNote(e); }}>Write a note</button>
      </div>
    </div>
  {/if}
</Dialog>

<style>
  .detail { display: grid; gap: 16px; }
  .kind { display: flex; align-items: center; gap: 9px; font-weight: 600; }
  .kind .dot { background: var(--c); border-radius: 2px; }
  dl { margin: 0; display: grid; grid-template-columns: 90px 1fr; gap: 8px 12px; }
  dt { color: var(--ink-3); font-size: 14px; }
  dd { margin: 0; font-weight: 500; }
  .row { display: flex; justify-content: flex-end; gap: 10px; margin-top: 4px; }
</style>
