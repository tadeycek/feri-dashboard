<script>
  import { untrack } from "svelte";
  import { app } from "../lib/store.svelte.js";
  import Dialog from "./Dialog.svelte";
  import { dayKey } from "../lib/time.js";
  import { ROK_TYPES } from "../lib/subjects.js";

  let form = $state({ title: "", predmet: "", vrsta: "kolokvij", datum: "", ura: "", prostor: "", snov: "" });
  let saving = $state(false);
  let error = $state("");
  let titleEl = $state();

  $effect(() => {
    if (app.rokDialog) {
      error = "";
      form = untrack(() => ({ title: "", predmet: "", vrsta: "kolokvij", datum: toDisplay(dayKey(app.now)), ura: "", prostor: "", snov: "" }));
      queueMicrotask(() => titleEl?.focus());
    }
  });

  // The dialog shows dd-mm-yyyy and 24h HH:MM; the vault stores YYYY-MM-DD.
  const toDisplay = (iso) => iso.split("-").reverse().join("-");
  const maskDate = (v) => {
    const d = v.replace(/\D/g, "").slice(0, 8);
    return d.slice(0, 2) + (d.length > 2 ? "-" + d.slice(2, 4) : "") + (d.length > 4 ? "-" + d.slice(4) : "");
  };
  const maskTime = (v) => {
    const d = v.replace(/\D/g, "").slice(0, 4);
    return d.slice(0, 2) + (d.length > 2 ? ":" + d.slice(2) : "");
  };
  function parseDate(v) {
    const m = /^(\d{2})-(\d{2})-(\d{4})$/.exec(v);
    if (!m) return null;
    const [d, mo, y] = [+m[1], +m[2], +m[3]];
    const dt = new Date(y, mo - 1, d);
    return dt.getFullYear() === y && dt.getMonth() === mo - 1 && dt.getDate() === d ? `${m[3]}-${m[2]}-${m[1]}` : null;
  }
  const validTime = (v) => v === "" || /^([01]\d|2[0-3]):[0-5]\d$/.test(v);

  async function submit(e) {
    e.preventDefault();
    const datum = parseDate(form.datum);
    if (!datum) return (error = "Enter a valid date as dd-mm-yyyy.");
    if (!validTime(form.ura)) return (error = "Enter the time as HH:MM (24-hour).");
    error = "";
    saving = true;
    await app.addRok({ ...form, datum, title: form.title.trim() });
    saving = false;
  }
  const close = () => (app.rokDialog = false);
</script>

<Dialog open={app.rokDialog} title="New deadline" onclose={close} width={560}>
  <form class="form" onsubmit={submit}>
    <label class="field">
      <span>Name</span>
      <input type="text" bind:this={titleEl} bind:value={form.title} required placeholder="e.g. First midterm" />
    </label>
    <div class="two">
      <label class="field">
        <span>Type</span>
        <select bind:value={form.vrsta}>{#each ROK_TYPES as t (t.id)}<option value={t.id}>{t.label}</option>{/each}</select>
      </label>
      <label class="field">
        <span>Subject</span>
        <select bind:value={form.predmet}>
          <option value="">General</option>
          {#each app.subjects as s (s.key)}<option value={s.name}>{s.name}</option>{/each}
        </select>
      </label>
    </div>
    <div class="three">
      <label class="field"><span>Date</span><input type="text" inputmode="numeric" bind:value={form.datum} oninput={() => (form.datum = maskDate(form.datum))} required placeholder="dd-mm-yyyy" maxlength="10" /></label>
      <label class="field"><span>Time</span><input type="text" inputmode="numeric" bind:value={form.ura} oninput={() => (form.ura = maskTime(form.ura))} placeholder="HH:MM" maxlength="5" /></label>
      <label class="field"><span>Room</span><input type="text" bind:value={form.prostor} placeholder="optional" /></label>
    </div>
    <label class="field">
      <span>Topics (one per line)</span>
      <textarea rows="4" bind:value={form.snov} placeholder="Limits&#10;Derivatives"></textarea>
    </label>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="actions">
      <button type="button" class="btn ghost" onclick={close}>Cancel</button>
      <button class="btn" disabled={saving || !form.title.trim()}>Save deadline</button>
    </div>
  </form>
</Dialog>

<style>
  .form { display: grid; gap: 16px; }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .three { display: grid; grid-template-columns: 1.3fr 1fr 1fr; gap: 14px; }
  .error { margin: 0; color: var(--danger, #e5484d); font-size: 13px; }
  .actions { display: flex; justify-content: flex-end; gap: 10px; }
</style>
