// State of the notes workspace: file tree, tabs, open documents, search. Mirrors Obsidian's model.
import * as api from "./api.js";
import { app } from "./store.svelte.js";
import { links as mdLinks, outline as mdOutline, wordStats } from "./md.js";

const WS_KEY = "feri.workspace";
const collator = new Intl.Collator("sl", { numeric: true, sensitivity: "base" });
const SAVE_DELAY = 700;

export const isMd = (p) => /\.md$/i.test(p);
export const stemOf = (p) => p.split(/[\\/]/).pop().replace(/\.md$/i, "");
export const nameOf = (p) => p.split("/").pop();
export const dirOf = (p) => (p.includes("/") ? p.slice(0, p.lastIndexOf("/")) : "");
const labelOf = (e) => (e.is_dir || !isMd(e.path) ? nameOf(e.path) : stemOf(e.path));
const uid = () => Math.random().toString(36).slice(2, 9);
const errText = (e) => (typeof e === "string" ? e : e?.message ?? String(e));
const store = {
  get: (k) => { try { return localStorage.getItem(k); } catch { return null; } },
  set: (k, v) => { try { localStorage.setItem(k, v); } catch { /* storage unavailable */ } },
};

class VaultState {
  entries = $state.raw([]);
  loaded = $state(false);
  error = $state("");

  expanded = $state({});
  selected = $state(null);
  renaming = $state(null);
  titleFocus = $state(null);

  leftOpen = $state(true);
  rightOpen = $state(true);
  leftTab = $state("files");
  rightTab = $state("backlinks");

  tabs = $state([]);
  activeId = $state(null);
  docs = $state({});

  query = $state("");
  results = $state.raw([]);
  searching = $state(false);
  backlinkHits = $state.raw([]);
  switcher = $state(false);
  menu = $state(null);
  confirm = $state(null);

  active = $derived(this.tabs.find((t) => t.id === this.activeId) ?? null);
  activeDoc = $derived(this.active?.path ? (this.docs[this.active.path] ?? null) : null);
  notes = $derived(this.entries.filter((e) => !e.is_dir && isMd(e.path)));

  /** lower-case note name → shortest path, for resolving [[links]]. */
  byStem = $derived.by(() => {
    const m = new Map();
    for (const e of this.notes) {
      const k = stemOf(e.path).toLowerCase();
      const cur = m.get(k);
      if (!cur || e.path.length < cur.length) m.set(k, e.path);
    }
    return m;
  });

  childMap = $derived.by(() => {
    const m = new Map();
    for (const e of this.entries) {
      const d = dirOf(e.path);
      if (!m.has(d)) m.set(d, []);
      m.get(d).push(e);
    }
    for (const list of m.values()) {
      list.sort((a, b) => (a.is_dir !== b.is_dir ? (a.is_dir ? -1 : 1) : collator.compare(labelOf(a), labelOf(b))));
    }
    return m;
  });

  rows = $derived.by(() => {
    const out = [];
    const walk = (dir, depth) => {
      for (const e of this.childMap.get(dir) ?? []) {
        out.push({ ...e, depth, label: labelOf(e) });
        if (e.is_dir && this.expanded[e.path]) walk(e.path, depth + 1);
      }
    };
    walk("", 0);
    return out;
  });

  outline = $derived(this.activeDoc ? mdOutline(this.activeDoc.content) : []);
  outgoing = $derived(this.activeDoc ? mdLinks(this.activeDoc.content).map((name) => ({ name, path: this.resolve(name) })) : []);
  stats = $derived(this.activeDoc ? wordStats(this.activeDoc.content) : { words: 0, chars: 0 });
  vaultName = $derived((app.settings?.vault_path ?? "").split("/").filter(Boolean).pop() ?? "Vault");

  #timers = new Map();
  #started = false;

  resolve = (name) => {
    const n = name.trim().replace(/\.md$/i, "");
    if (n.includes("/")) {
      const hit = this.notes.find((e) => e.path.toLowerCase() === `${n}.md`.toLowerCase() || e.path.toLowerCase().endsWith(`/${n}.md`.toLowerCase()));
      return hit?.path ?? null;
    }
    return this.byStem.get(n.toLowerCase()) ?? null;
  };

  async init() {
    if (this.#started) return;
    this.#started = true;
    await this.loadTree();
    this.#restore();
    if (!this.tabs.length) {
      const home = this.notes.find((e) => /domov/i.test(e.path));
      if (home) await this.open(home.path, { mode: "read" });
      for (const p of ["Predmeti"]) if (this.entries.some((e) => e.path === p)) this.expanded[p] = true;
      this.#persist();
    }
    await Promise.all(this.tabs.filter((t) => t.path).map((t) => this.#ensureDoc(t.path)));
    this.loaded = true;
    window.addEventListener("focus", () => this.refreshFromDisk());
    window.addEventListener("beforeunload", () => this.flushAll());
    // Hidden to the tray, or quitting from the tray menu: write any pending edit first.
    document.addEventListener("visibilitychange", () => document.hidden && this.flushAll());
    api.onEvent("before-quit", () => this.flushAll());
  }

  async loadTree() {
    try {
      this.entries = await api.vaultTree();
      this.error = "";
    } catch (e) {
      this.error = errText(e);
    }
  }

  #persist() {
    store.set(
      WS_KEY,
      JSON.stringify({
        tabs: this.tabs.map((t) => ({ path: t.path, mode: t.mode })),
        active: this.tabs.findIndex((t) => t.id === this.activeId),
        leftOpen: this.leftOpen,
        rightOpen: this.rightOpen,
        leftTab: this.leftTab,
        rightTab: this.rightTab,
        expanded: Object.keys(this.expanded).filter((k) => this.expanded[k]),
      }),
    );
  }
  persist = () => this.#persist();

  #restore() {
    try {
      const s = JSON.parse(store.get(WS_KEY) ?? "null");
      if (!s) return;
      const have = new Set(this.entries.map((e) => e.path));
      this.tabs = (s.tabs ?? []).filter((t) => !t.path || have.has(t.path)).map((t) => ({ id: uid(), path: t.path, mode: t.mode === "read" ? "read" : "edit" }));
      this.activeId = this.tabs[Math.min(Math.max(s.active ?? 0, 0), this.tabs.length - 1)]?.id ?? null;
      this.leftOpen = s.leftOpen ?? true;
      this.rightOpen = s.rightOpen ?? true;
      this.leftTab = s.leftTab ?? "files";
      this.rightTab = s.rightTab ?? "backlinks";
      this.expanded = Object.fromEntries((s.expanded ?? []).map((k) => [k, true]));
    } catch { /* ignore a corrupt saved workspace */ }
  }

  // ----- documents -----

  async #ensureDoc(path) {
    if (this.docs[path] && !this.docs[path].error) return;
    this.docs[path] = { content: "", modified: 0, dirty: false, loading: true, error: "" };
    try {
      const r = await api.readNote(path);
      this.docs[path] = { content: r.content, modified: r.modified, dirty: false, loading: false, error: "" };
    } catch (e) {
      this.docs[path] = { content: "", modified: 0, dirty: false, loading: false, error: errText(e) };
    }
  }

  setContent(path, content) {
    const d = this.docs[path];
    if (!d || d.content === content) return;
    this.docs[path] = { ...d, content, dirty: true };
    clearTimeout(this.#timers.get(path));
    this.#timers.set(path, setTimeout(() => this.flush(path), SAVE_DELAY));
  }

  async flush(path) {
    clearTimeout(this.#timers.get(path));
    const d = this.docs[path];
    if (!d?.dirty) return;
    try {
      const modified = await api.writeNote(path, d.content);
      const cur = this.docs[path];
      if (!cur) return;
      this.docs[path] = { ...cur, modified, dirty: cur.content !== d.content };
      if (this.docs[path].dirty) this.#timers.set(path, setTimeout(() => this.flush(path), SAVE_DELAY));
    } catch (e) {
      app.notify(errText(e));
    }
  }

  async flushAll() {
    await Promise.all(Object.keys(this.docs).map((p) => this.flush(p)));
  }

  /** Picks up edits made outside the app (e.g. in Obsidian) when the window regains focus. */
  async refreshFromDisk() {
    await this.loadTree();
    for (const t of this.tabs) {
      const d = t.path && this.docs[t.path];
      if (!d || d.dirty) continue;
      try {
        const r = await api.readNote(t.path);
        if (r.content !== d.content) this.docs[t.path] = { ...d, content: r.content, modified: r.modified };
      } catch { /* file vanished; leave the tab alone */ }
    }
  }

  // ----- tabs -----

  async open(path, { newTab = false, mode } = {}) {
    if (!isMd(path)) {
      app.notify("This file type can't be opened here.", { label: "Open in Obsidian", run: () => app.openNote(path) });
      return;
    }
    const existing = this.tabs.find((t) => t.path === path);
    if (existing) {
      this.activeId = existing.id;
      if (mode) existing.mode = mode;
    } else if (!newTab && this.active) {
      this.active.path = path;
      this.active.mode = mode ?? this.active.mode ?? "edit";
    } else {
      const tab = { id: uid(), path, mode: mode ?? "edit" };
      this.tabs = [...this.tabs, tab];
      this.activeId = tab.id;
    }
    this.selected = path;
    this.reveal(path);
    this.#persist();
    await this.#ensureDoc(path);
  }

  openByName(name, fromPath = null) {
    const path = this.resolve(name);
    if (path) return this.open(path);
    return this.#createFromLink(name, fromPath);
  }

  async #createFromLink(name, fromPath) {
    const dir = fromPath ? dirOf(fromPath) : "";
    try {
      const created = await api.createNote(dir ? `${dir}/${name}` : name, "");
      await this.loadTree();
      await this.open(created);
      app.notify(`Created note ${stemOf(created)}.`);
    } catch (e) {
      app.notify(errText(e));
    }
  }

  newTab() {
    const tab = { id: uid(), path: null, mode: "edit" };
    this.tabs = [...this.tabs, tab];
    this.activeId = tab.id;
    this.#persist();
  }

  async closeTab(id) {
    const i = this.tabs.findIndex((t) => t.id === id);
    if (i < 0) return;
    const path = this.tabs[i].path;
    if (path) await this.flush(path);
    this.tabs = this.tabs.filter((t) => t.id !== id);
    if (this.activeId === id) this.activeId = (this.tabs[i] ?? this.tabs[i - 1])?.id ?? null;
    if (path && !this.tabs.some((t) => t.path === path)) delete this.docs[path];
    this.#persist();
  }

  cycleTab(step) {
    if (this.tabs.length < 2) return;
    const i = this.tabs.findIndex((t) => t.id === this.activeId);
    this.activeId = this.tabs[(i + step + this.tabs.length) % this.tabs.length].id;
    this.selected = this.active?.path ?? this.selected;
    this.#persist();
  }

  toggleMode() {
    if (!this.active?.path) return;
    this.active.mode = this.active.mode === "read" ? "edit" : "read";
    this.#persist();
  }

  // ----- tree -----

  reveal(path) {
    let dir = dirOf(path);
    while (dir) {
      this.expanded[dir] = true;
      dir = dirOf(dir);
    }
  }

  toggleDir(path) {
    this.expanded[path] = !this.expanded[path];
    this.#persist();
  }

  collapseAll() {
    this.expanded = {};
    this.#persist();
  }

  /** Where "new note" goes: the selected folder, else next to the selected/open note. */
  #targetDir() {
    const sel = this.selected;
    if (sel) return this.entries.find((e) => e.path === sel)?.is_dir ? sel : dirOf(sel);
    return this.active?.path ? dirOf(this.active.path) : "";
  }

  async createNote(dir = this.#targetDir()) {
    try {
      const path = await api.createNote(dir ? `${dir}/Untitled` : "Untitled", "");
      await this.loadTree();
      this.reveal(path);
      await this.open(path, { mode: "edit" });
      this.titleFocus = path;
    } catch (e) {
      app.notify(errText(e));
    }
  }

  async createNoteFromTemplate({ predmet, kind, title, datum }) {
    try {
      const info = await api.saveNote({ predmet, kind, title, body: "", datum });
      await this.loadTree();
      await this.open(info.path, { mode: "edit" });
      return info;
    } catch (e) {
      app.notify(errText(e));
      return null;
    }
  }

  async createFolder(parent = this.#targetDir()) {
    try {
      const path = await api.createFolder(parent ? `${parent}/New folder` : "New folder");
      await this.loadTree();
      this.reveal(path);
      this.expanded[path] = true;
      this.renaming = path;
    } catch (e) {
      app.notify(errText(e));
    }
  }

  async rename(path, newName) {
    const name = newName.trim().replace(/\.md$/i, "");
    this.renaming = null;
    const entry = this.entries.find((e) => e.path === path);
    const base = entry && !entry.is_dir && !isMd(path) ? newName.trim() : name;
    if (!base || base === (entry?.is_dir ? nameOf(path) : stemOf(path)) || /[\\/:*?"<>|#^[\]]/.test(base)) return path;
    const dir = dirOf(path);
    const to = dir ? `${dir}/${base}${isMd(path) ? ".md" : ""}` : `${base}${isMd(path) ? ".md" : ""}`;
    try {
      await this.flushAll();
      const [newPath, updated] = await api.renamePath(path, to);
      this.#remap(path, newPath);
      await this.refreshFromDisk();
      this.#persist();
      if (updated) app.notify(`Renamed. Links were updated in ${updated} other ${updated === 1 ? "note" : "notes"}.`);
      return newPath;
    } catch (e) {
      app.notify(errText(e));
      return path;
    }
  }

  /** After a rename, point tabs/docs/selection at the new path (also for files inside a renamed folder). */
  #remap(from, to) {
    const map = (p) => (p === from ? to : p?.startsWith(`${from}/`) ? to + p.slice(from.length) : p);
    for (const t of this.tabs) t.path = map(t.path);
    for (const p of Object.keys(this.docs)) {
      const np = map(p);
      if (np !== p) {
        this.docs[np] = this.docs[p];
        delete this.docs[p];
      }
    }
    this.selected = map(this.selected);
    if (this.expanded[from]) {
      this.expanded[to] = true;
      delete this.expanded[from];
    }
  }

  askTrash(path) {
    this.confirm = { path, label: nameOf(path) };
  }

  async trash(path) {
    this.confirm = null;
    try {
      await this.flushAll();
      await api.trashPath(path);
      for (const t of [...this.tabs]) if (t.path === path || t.path?.startsWith(`${path}/`)) await this.closeTab(t.id);
      if (this.selected === path) this.selected = null;
      await this.loadTree();
      app.notify(`${nameOf(path)} moved to .trash in the vault.`);
    } catch (e) {
      app.notify(errText(e));
    }
  }

  // ----- search & backlinks -----

  #searchTimer = null;
  setQuery(q) {
    this.query = q;
    clearTimeout(this.#searchTimer);
    if (!q.trim()) {
      this.results = [];
      this.searching = false;
      return;
    }
    this.searching = true;
    this.#searchTimer = setTimeout(async () => {
      try {
        const hits = await api.searchVault(q);
        if (q === this.query) this.results = hits;
      } catch (e) {
        app.notify(errText(e));
      } finally {
        if (q === this.query) this.searching = false;
      }
    }, 250);
  }

  async refreshBacklinks() {
    const path = this.active?.path;
    if (!path) {
      this.backlinkHits = [];
      return;
    }
    try {
      const hits = await api.backlinks(path);
      if (this.active?.path === path) this.backlinkHits = hits;
    } catch {
      this.backlinkHits = [];
    }
  }
}

export const vault = new VaultState();
