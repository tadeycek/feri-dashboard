import * as api from "./api.js";
import { count, dayKey } from "./time.js";
import { noteKindFor } from "./subjects.js";

const SEEN_KEY = "feri.seenChanges";
const STALE_MS = 5 * 60 * 1000; // refetch the timetable when the window regains focus after this long

const store = {
  get: (k, d) => { try { return localStorage.getItem(k) ?? d; } catch { return d; } },
  set: (k, v) => { try { localStorage.setItem(k, v); } catch { /* storage unavailable */ } },
};

const localIso = (d = new Date()) => {
  const p = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
};
const errText = (e) => (typeof e === "string" ? e : e?.message ?? String(e));

class AppState {
  ready = $state(false);
  view = $state("today");
  now = $state(new Date());

  rawEvents = $state.raw([]);
  changes = $state.raw([]);
  fetchedAt = $state(null);
  syncing = $state(false);
  offline = $state(false);
  error = $state("");

  subjects = $state.raw([]);
  settings = $state(null);
  roki = $state.raw([]);
  notes = $state.raw([]);
  vaultError = $state("");

  toast = $state(null);
  pendingNote = $state(null);
  rokDialog = $state(false);
  settingsDialog = $state(false);
  seen = $state(store.get(SEEN_KEY, ""));

  classes = $derived.by(() =>
    this.rawEvents
      .filter((e) => e.kind !== "REZ")
      .map((e) => ({ ...e, s: new Date(e.start), e: new Date(e.end), day: e.start.slice(0, 10) })),
  );
  today = $derived(this.classes.filter((c) => c.day === dayKey(this.now)));
  current = $derived(this.classes.find((c) => c.s <= this.now && this.now < c.e) ?? null);
  next = $derived(this.classes.find((c) => c.s > this.now) ?? null);
  upcomingRoki = $derived.by(() => {
    const t = dayKey(this.now);
    return this.roki.filter((r) => !r.opravljeno && r.datum >= t);
  });
  overdueRoki = $derived.by(() => {
    const t = dayKey(this.now);
    return this.roki.filter((r) => !r.opravljeno && r.datum && r.datum < t);
  });
  unseenChanges = $derived(this.changes.filter((c) => c.detected_at > this.seen));

  keyOf = (name) => this.subjects.find((s) => s.name === name)?.key ?? "";

  #lastSync = 0;
  #toastTimer = null;
  #booted = false;

  /** Paint from cache immediately, then fetch the live timetable and read the vault. */
  async boot() {
    if (this.#booted) return;
    this.#booted = true;
    try {
      const [subjects, settings, cached] = await Promise.all([api.subjects(), api.getSettings(), api.loadCached()]);
      this.subjects = subjects;
      this.settings = settings;
      this.#apply(cached);
    } catch (e) {
      this.error = errText(e);
    }
    this.ready = true;

    // The tray's "Refresh" and the 30-minute background refresh update the cache while the window may be hidden.
    api.onEvent("timetable-updated", async () => {
      try { this.#apply(await api.loadCached()); await this.loadVault(); } catch { /* keep what is shown */ }
    });

    setInterval(() => (this.now = new Date()), 30_000);
    window.addEventListener("focus", () => {
      this.now = new Date();
      if (Date.now() - this.#lastSync > STALE_MS) this.refresh();
    });

    await this.refresh();
  }

  /** Timetable + vault. Called at boot, by the refresh button and by `r`. */
  async refresh() {
    await Promise.all([this.sync(), this.loadVault()]);
  }

  async sync() {
    if (this.syncing) return;
    this.syncing = true;
    const before = this.unseenChanges.length;
    try {
      const res = await api.syncTimetable();
      this.#apply(res);
      this.#lastSync = Date.now();
      const fresh = this.unseenChanges.length - before;
      if (fresh > 0) this.notify(`Timetable changed: ${count(fresh, "change")}.`);
    } catch (e) {
      this.offline = true;
      this.error = errText(e);
    } finally {
      this.syncing = false;
    }
  }

  #apply(res) {
    this.rawEvents = res.events;
    this.changes = res.changes;
    this.fetchedAt = res.fetched_at;
    this.offline = res.offline;
    this.error = res.error ?? "";
  }

  async loadVault() {
    try {
      const [roki, notes] = await Promise.all([api.listRoki(), api.recentNotes()]);
      this.roki = roki;
      this.notes = notes;
      this.vaultError = "";
    } catch (e) {
      this.vaultError = errText(e);
    }
  }

  markChangesSeen() {
    this.seen = localIso();
    store.set(SEEN_KEY, this.seen);
  }

  notify(message, action = null) {
    clearTimeout(this.#toastTimer);
    this.toast = { message, action };
    this.#toastTimer = setTimeout(() => (this.toast = null), 6000);
  }

  go(view) {
    this.view = view;
  }

  async addRok(rok) {
    try {
      await api.createRok(rok);
      await this.loadVault();
      this.rokDialog = false;
      this.notify("Deadline saved to Obsidian.");
      return true;
    } catch (e) {
      this.notify(errText(e));
      return false;
    }
  }

  async toggleRok(file) {
    try {
      await api.toggleRok(file);
      await this.loadVault();
    } catch (e) {
      this.notify(errText(e));
    }
  }

  async openNote(path) {
    try {
      await api.openInObsidian(path);
    } catch (e) {
      this.notify(errText(e));
    }
  }

  async saveSettings(settings) {
    try {
      await api.saveSettings(settings);
      this.settings = settings;
      this.settingsDialog = false;
      this.notify("Settings saved.");
      await this.refresh();
    } catch (e) {
      this.notify(errText(e));
    }
  }

  /** "Write a note" on a class: the notes workspace creates the note from the vault template and opens it. */
  startNote(ev) {
    this.view = "notes";
    if (!ev.key) return;
    const kind = noteKindFor(ev.kind);
    const d = ev.s ?? new Date(ev.start);
    this.pendingNote = { predmet: ev.subject, kind, title: `${kind === "Vaje" ? "Exercises" : "Lecture"} ${d.getDate()} ${d.toLocaleString("en-GB", { month: "short" })}` };
  }
}

export const app = new AppState();
