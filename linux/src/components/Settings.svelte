<script>
  import { app } from "../lib/store.svelte.js";
  import * as api from "../lib/api.js";
  import Dialog from "./Dialog.svelte";

  let vault = $state("");
  let url = $state("");
  let closeToTray = $state(true);
  let notifyClasses = $state(true);
  let minutes = $state(10);
  let notifyDeadlines = $state(true);
  let autostart = $state(false);
  let autostartBusy = $state(false);

  $effect(() => {
    if (app.settingsDialog && app.settings) {
      vault = app.settings.vault_path;
      url = app.settings.ics_url;
      closeToTray = app.settings.close_to_tray ?? true;
      notifyClasses = app.settings.notify_classes ?? true;
      minutes = app.settings.notify_minutes ?? 10;
      notifyDeadlines = app.settings.notify_deadlines ?? true;
      // The autostart entry is a file on disk, so read the real state every time the dialog opens.
      api.getAutostart().then((v) => (autostart = !!v)).catch(() => {});
    }
  });

  // Autostart applies immediately: switching it off removes the entry again.
  async function toggleAutostart(e) {
    const want = e.currentTarget.checked;
    autostartBusy = true;
    try {
      await api.setAutostart(want);
      autostart = want;
      app.notify(want ? "FERI Dashboard will start with your computer, in the tray." : "Removed from autostart.");
    } catch (err) {
      autostart = !want;
      e.currentTarget.checked = !want;
      app.notify(String(err?.message ?? err));
    } finally {
      autostartBusy = false;
    }
  }

  const close = () => (app.settingsDialog = false);
  const save = (e) => {
    e.preventDefault();
    app.saveSettings({
      vault_path: vault.trim(),
      ics_url: url.trim(),
      close_to_tray: closeToTray,
      notify_classes: notifyClasses,
      notify_minutes: Number(minutes),
      notify_deadlines: notifyDeadlines,
    });
  };
</script>

<Dialog open={app.settingsDialog} title="Settings" onclose={close} width={580}>
  <form onsubmit={save} class="form">
    <label class="field">
      <span>Obsidian vault folder</span>
      <input type="text" bind:value={vault} required spellcheck="false" />
      <small class="muted">Notes and deadlines are read from and saved to this folder.</small>
    </label>
    <label class="field">
      <span>Timetable link (iCal)</span>
      <input type="url" bind:value={url} required spellcheck="false" />
      <small class="muted">From Wise Timetable, under Subscribe. Downloaded on every start and refresh.</small>
    </label>

    <fieldset>
      <legend>Running in the background</legend>
      <label class="check">
        <input type="checkbox" checked={autostart} disabled={autostartBusy} onchange={toggleAutostart} />
        <span>Start with my computer<small class="muted">Opens in the tray without a window. Untick this to remove it from autostart.</small></span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={closeToTray} />
        <span>Keep running in the tray when I close the window<small class="muted">Needed for notifications. Quit from the tray menu.</small></span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={notifyClasses} />
        <span>
          Notify me before classes
          <select bind:value={minutes} disabled={!notifyClasses} aria-label="Minutes before a class">
            {#each [5, 10, 15, 30] as m (m)}<option value={m}>{m} minutes before</option>{/each}
          </select>
          <small class="muted">Also tells you when the timetable changes.</small>
        </span>
      </label>
      <label class="check">
        <input type="checkbox" bind:checked={notifyDeadlines} />
        <span>Notify me about deadlines<small class="muted">The day before (from 09:00) and on the day (from 08:00).</small></span>
      </label>
    </fieldset>

    <div class="row">
      <button type="button" class="btn ghost" onclick={close}>Cancel</button>
      <button class="btn">Save settings</button>
    </div>
  </form>
</Dialog>

<style>
  .form { display: grid; gap: 16px; }
  small { display: block; font-size: 13px; margin-top: 2px; }
  fieldset { border: 0; border-top: 1px solid var(--line); margin: 4px 0 0; padding: 12px 0 0; display: grid; gap: 12px; }
  legend { padding: 0 8px 0 0; font: 600 18px/1.2 var(--font-num); }
  .check { display: grid; grid-template-columns: 18px 1fr; gap: 12px; align-items: start; cursor: pointer; }
  .check input { width: 17px; height: 17px; margin: 3px 0 0; accent-color: var(--accent); }
  .check select { width: auto; margin: 4px 0 0 10px; padding: 3px 8px; }
  .row { display: flex; justify-content: flex-end; gap: 10px; margin-top: 4px; }
</style>
