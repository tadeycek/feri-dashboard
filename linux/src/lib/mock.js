// Demo backend for plain-browser development (vite dev only). Never used inside the Tauri app.
import events from "../../dev/mock-events.json";
import vaultSnapshot from "../../dev/mock-vault.json";

const SUBJECTS = [
  { key: "MAT", name: "Matematika 1" },
  { key: "UVR", name: "Uvod v računalništvo" },
  { key: "PRO1", name: "Programiranje 1" },
  { key: "SPO", name: "Sodobna programska orodja" },
  { key: "UVS", name: "Uvod v svetovni splet" },
];

const pad = (n) => String(n).padStart(2, "0");
const iso = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const inDays = (n) => iso(new Date(Date.now() + n * 86400000));

let roki = [
  { file: "1. kolokvij.md", title: "1. kolokvij", predmet: "Matematika 1", vrsta: "kolokvij", datum: inDays(9), ura: "10:00", prostor: "A-306", opravljeno: false, ocena: "" },
  { file: "Domača naloga 1.md", title: "Domača naloga 1", predmet: "Programiranje 1", vrsta: "oddaja", datum: inDays(2), ura: "23:59", prostor: "", opravljeno: false, ocena: "" },
  { file: "Projekt – spletna stran.md", title: "Projekt – spletna stran", predmet: "Uvod v svetovni splet", vrsta: "oddaja", datum: inDays(21), ura: "", prostor: "", opravljeno: false, ocena: "" },
  { file: "Prijava na izpit.md", title: "Prijava na izpit", predmet: "", vrsta: "pomembno", datum: inDays(-1), ura: "", prostor: "", opravljeno: false, ocena: "" },
  { file: "Seminar SPO.md", title: "Seminar SPO", predmet: "Sodobna programska orodja", vrsta: "oddaja", datum: inDays(-6), ura: "", prostor: "", opravljeno: true, ocena: "" },
];
let notes = [
  { predmet: "Matematika 1", kind: "Predavanja", title: "P01 – Uvod v zaporedja", path: "Predmeti/Matematika 1/Predavanja/P01 – Uvod v zaporedja.md", modified: Date.now() / 1000 - 3600 * 5 },
  { predmet: "Programiranje 1", kind: "Vaje", title: "V01 – Spremenljivke", path: "Predmeti/Programiranje 1/Vaje/V01 – Spremenljivke.md", modified: Date.now() / 1000 - 3600 * 30 },
];
let settings = {
  vault_path: "/home/tadej/Documents/FERI/Zapiski/Sola",
  close_to_tray: true, notify_classes: true, notify_minutes: 10, notify_deadlines: true,
  ics_url: "https://www.wise-tt.com/web/feri/reports?g=1211_1214_1220_1221&lang=sl&format=ics",
};

const stamp = () => {
  const d = new Date();
  return `${iso(d)}T${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
};
const lag = (v, ms = 250) => new Promise((r) => setTimeout(() => r(v), ms));
const demoChange = {
  kind: "spremenjeno", uid: "demo", subject: "Programiranje 1", key: "PRO1", ekind: "PR",
  start: events.find((e) => e.key === "PRO1" && e.start > stamp())?.start ?? stamp(),
  detail: "room E-105 → E-106", detected_at: stamp(),
};


// ---- in-memory vault (mirrors the Rust vault.rs behaviour closely enough for UI work) ----
const files = new Map(Object.entries(vaultSnapshot.files));
const dirSet = new Set(vaultSnapshot.dirs);
const stem = (p) => p.split("/").pop().replace(/\.md$/i, "");
const parent = (p) => (p.includes("/") ? p.slice(0, p.lastIndexOf("/")) : "");
const bad = (p) => !p || p.includes("\\") || p.split("/").some((x) => !x || x === "." || x === ".." || x.startsWith("."));
const exists = (p) => files.has(p) || dirSet.has(p);
function unique(p) {
  if (!exists(p)) return p;
  const m = /^(.*?)(\.[^./]+)?$/.exec(p);
  for (let n = 2; ; n++) { const c = `${m[1]} ${n}${m[2] ?? ""}`; if (!exists(c)) return c; }
}
function mkdirs(p) { for (let d = parent(p); d; d = parent(d)) dirSet.add(d); }
const now = () => Math.floor(Date.now() / 1000);
const mds = () => [...files.keys()].filter((k) => /\.md$/i.test(k));
const fold = (s) => s.normalize("NFD").replace(/[\u0300-\u036f]/g, "").replace(/đ/g, "d").toLowerCase();
const snip = (l) => (l.trim().length > 160 ? l.trim().slice(0, 160) + "…" : l.trim());

const vaultCall = {
  vault_tree: () => [...dirSet].map((path) => ({ path, is_dir: true, modified: now() })).concat([...files.keys()].map((path) => ({ path, is_dir: false, modified: now() - path.length * 977 }))),
  read_note: ({ path }) => { if (bad(path) || !/\.md$/i.test(path)) throw "Invalid path."; if (!files.has(path)) throw "Note does not exist."; return { content: files.get(path), modified: now() }; },
  write_note: ({ path, content }) => { if (!files.has(path)) throw "The note no longer exists. Create it again."; files.set(path, content); return now(); },
  create_note: ({ path, content }) => { let p = /\.md$/i.test(path) ? path : `${path}.md`; if (bad(p)) throw "Invalid path."; p = unique(p); mkdirs(p); files.set(p, content); return p; },
  create_folder: ({ path }) => { if (bad(path)) throw "Invalid path."; const p = unique(path); mkdirs(p); dirSet.add(p); return p; },
  rename_path: ({ from, to }) => {
    let dst = to;
    if (files.has(from) && /\.md$/i.test(from) && !/\.md$/i.test(dst)) dst += ".md";
    if (bad(dst)) throw "Invalid path.";
    if (exists(dst)) throw "A file with that name already exists.";
    let updated = 0;
    if (files.has(from)) {
      files.set(dst, files.get(from)); files.delete(from); mkdirs(dst);
      const [o, n] = [stem(from), stem(dst)];
      if (o !== n) for (const k of mds()) {
        let t = files.get(k), c = t;
        for (const tail of ["]]", "|", "#"]) c = c.split(`[[${o}${tail}`).join(`[[${n}${tail}`);
        if (c !== t) { files.set(k, c); updated++; }
      }
    } else if (dirSet.has(from)) {
      for (const k of [...files.keys()]) if (k.startsWith(from + "/")) { files.set(dst + k.slice(from.length), files.get(k)); files.delete(k); }
      for (const d of [...dirSet]) if (d === from || d.startsWith(from + "/")) { dirSet.delete(d); dirSet.add(dst + d.slice(from.length)); }
    } else throw "Source does not exist.";
    return [dst, updated];
  },
  trash_path: ({ path }) => {
    if (files.has(path)) files.delete(path);
    else for (const k of [...files.keys()]) if (k.startsWith(path + "/")) files.delete(k);
    for (const d of [...dirSet]) if (d === path || d.startsWith(path + "/")) dirSet.delete(d);
    return `.trash/${path.split("/").pop()}`;
  },
  search_vault: ({ query }) => {
    const q = fold(query.trim()); if (!q) return [];
    const hits = [];
    for (const k of mds()) {
      if (fold(stem(k)).includes(q)) hits.push({ path: k, line_no: 0, line: "" });
      files.get(k).split("\n").forEach((l, i) => { if (fold(l).includes(q)) hits.push({ path: k, line_no: i + 1, line: snip(l) }); });
    }
    return hits.slice(0, 200);
  },
  backlinks: ({ path }) => {
    const n = stem(path).toLowerCase(); const pats = [`[[${n}]]`, `[[${n}|`, `[[${n}#`]; const hits = [];
    for (const k of mds()) if (k !== path) files.get(k).split("\n").forEach((l, i) => { if (pats.some((p) => l.toLowerCase().includes(p))) hits.push({ path: k, line_no: i + 1, line: snip(l) }); });
    return hits;
  },
  open_external: ({ url }) => { console.info("open external:", url); return null; },
};

let autostart = false;
export async function call(cmd, args = {}) {
  if (vaultCall[cmd]) return vaultCall[cmd](args);
  switch (cmd) {
    case "get_autostart": return autostart;
    case "set_autostart": autostart = !!args.enabled; return null;
    case "get_subjects": return SUBJECTS;
    case "get_settings": return settings;
    case "save_settings": settings = args.settings; return null;
    case "load_cached": return { events, changes: [], fetched_at: null, offline: false, error: null };
    case "sync_timetable": return lag({ events, changes: [demoChange], fetched_at: stamp(), offline: false, error: null }, 600);
    case "list_roki": return roki;
    case "create_rok": {
      const r = args.rok;
      roki = [...roki, { file: `${r.title}.md`, title: r.title, predmet: r.predmet, vrsta: r.vrsta, datum: r.datum, ura: r.ura, prostor: r.prostor, opravljeno: false, ocena: "" }];
      return `${r.title}.md`;
    }
    case "toggle_rok": {
      let state = false;
      roki = roki.map((r) => (r.file === args.file ? ((state = !r.opravljeno), { ...r, opravljeno: state }) : r));
      return state;
    }
    case "save_note": {
      const n = args.note;
      const tip = n.kind === "Vaje" ? "vaje" : "predavanje";
      const path = unique(`Predmeti/${n.predmet}/${n.kind}/${n.title}.md`);
      mkdirs(path);
      files.set(path, `---\ntip: ${tip}\npredmet: "[[${n.predmet}]]"\ndatum: ${n.datum}\nst: \ntags:\n  - ${tip}\n---\n# ${n.title}\n\n## Zapiski\n\n${n.body}\n`);
      const info = { predmet: n.predmet, kind: n.kind, title: stem(path), path, modified: Date.now() / 1000 };
      notes = [info, ...notes];
      return info;
    }
    case "recent_notes": return notes;
    case "open_in_obsidian": console.info("open in Obsidian:", args.path); return null;
    default: throw new Error(`mock: unknown command ${cmd}`);
  }
}
