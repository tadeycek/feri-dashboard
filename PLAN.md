# FERI Dashboard: scope and plan

One place for everything about studying at FERI: timetable, upcoming deadlines, quick notes into the Obsidian vault.
Folder layout: `linux/` (built first, has everything) and `android/` (planned, built later).

## 1. Data sources (verified 2026-10-06)

| Source | Auth | Use |
|---|---|---|
| Wise Timetable iCal feed `https://www.wise-tt.com/web/feri/reports?g=1211_1214_1220_1221&lang=sl&format=ics` | none | The timetable. Whole year, live, no cache. Every class has a stable UID (`S<id>-<YYYYMMDD>@feri.wise-tt.com`), so changes can be diffed. |
| Wise REST API v2 (`wise-tt.com/api/v2`) | API account issued by Wise | Later upgrade: JSON, notifications, holidays, event types. Needs an email to info_wise@wise-t.com. |
| Obsidian vault `FERI/Zapiski/Sola` | local files | Roki (deadlines), notes, subject hubs. |

Rule: the feed is fetched **on every app start and on every refresh** (button, `r`, window regains focus after 5+ minutes). If the fetch fails the cached copy is shown with an "offline" marker.

## 2. Linux app (v1: implemented)

Stack: **Tauri 2** (Rust backend, system WebKitGTK webview) + **Svelte 5** frontend. Measured on 2026-10-06: 5.6 MB release binary, starts and fetches the timetable within a couple of seconds, about 650 MB RSS including the WebKit helper processes (mostly shared libraries, so the real cost is lower). Chosen because it writes plain files without any sandbox and the same frontend can be reused for Android through Tauri mobile. A native Rust UI would use less memory but costs far more design time.

### Features
1. **Today**: today's timetable as a time grid (same look and the same 07:00–21:00 range as the week grid, one column, now-line), deadlines, coming days, subjects (next class + note count), this week's class hours, recent notes, unseen timetable changes. Three columns that follow the width of the window, down to one.
2. **Week**: Mon–Fri time grid from 07:00 to 21:00 (the hour height is chosen from the window height so the whole day fits; short classes reflow their text), coloured by subject, a now-line in today's column only, week navigation, click a class for details and "Write a note".
3. **Deadlines**: list read from `Roki/*.md` in the vault, plus an overview. Add (midterm, exam, assignment, important), tick off, open in Obsidian. New ones use the vault's own *Rok* template format, so the Bases on the dashboard note pick them up.
4. **Notes**: an Obsidian-style workspace over the vault folder. File explorer (tree, new note/folder, rename with link updating, delete → `.trash`, right-click menu), search, tabs, a CodeMirror 6 editor (markdown highlighting, `[[` autocomplete, autosave) and a reading view (properties, callouts, tasks you can tick, `[[links]]`), right sidebar with backlinks / outgoing links / outline, status bar with word count, quick switcher (Ctrl+O). "Zapiši zapisek" on a class creates the note from the vault template and opens it here. Not included: graph, canvas, Bases rendering (shown as a card with "open in Obsidian"), plugins, live-preview editing.
5. **Timetable changes**: diff of each fetch against the last one (added, removed, moved class / room change), kept for the last 40.
6. Settings: vault path, feed URL, background options. The interface is English; vault content is not translated.
7. **Background:** tray icon with a next-class line, close-to-tray, autostart (add and remove from Settings), desktop notifications before classes / on timetable changes / for deadlines, a 30-minute refresh while in the tray, single instance, application-launcher entry (`install.sh`).
7. Offline cache, light and dark theme following the system.

### Architecture
```
src-tauri/src/
  ics.rs     parse the feed into Event structs (subject, kind PR/RV/SV/LV, room, lecturers, times)
  model.rs   Event, Change, Rok, Note, subject table (name, short code, colour)
  store.rs   settings + cache in the app config/data dir, diff of old vs new events
  vault.rs   vault access: roki, note templates, and the workspace API (tree, read, write, create, rename + relink,
             trash, search, backlinks). All paths are checked: no `..`, no hidden folders.
  background.rs  tray, autostart, notifications, 30-minute background refresh
  lib.rs     Tauri commands: load_cached, sync_timetable, list_roki, create_rok, toggle_rok, save_note,
             recent_notes, open_in_obsidian, vault_tree, read_note, write_note, create_note, create_folder,
             rename_path, trash_path, search_vault, backlinks, open_external, get/save_settings
src/
  App.svelte, lib/store.svelte.js (state, auto sync), lib/api.js (invoke + browser mock),
  views: Today, Week, Roki (deadlines); components: Rail, TimeGrid (shared by Today and Week), EventDetail, RokDialog,
         Dialog, vault/* (the notes workspace). Design rules: see DESIGN.md
  lib/vault.svelte.js (tabs, tree, autosave), lib/md.js (markdown rendering, outline, links)
```
The Rust logic has no UI dependencies, so Android can reuse it as is.

## 3. Android app (planned, not started)

Approach: **Tauri 2 mobile** with the same Svelte frontend and Rust core, plus a small Kotlin layer for the things web views cannot do.

| Feature | Android approach |
|---|---|
| Timetable | Same feed fetch. Refresh on open + WorkManager every ~30 min. |
| Notifications | Local notification 15 min before each class and for roki due tomorrow; alert when the diff finds a changed class. |
| Home-screen widget | Kotlin (Glance): next class + next deadline. Reads the cached JSON. |
| Obsidian notes | Vault is not reachable directly. Plan: sync the vault folder with **Syncthing** (phone ↔ PC), and read/write through a SAF folder the user picks once (`ACTION_OPEN_DOCUMENT_TREE`). Fallback: quick notes into an `Inbox/` folder that the PC side merges. |
| UI | Bottom tab bar instead of the rail; Today and Roki first, week view as a day-by-day swipe. |

Milestones: (A1) Tauri mobile shell + reuse Today/Week, (A2) SAF vault access for roki/notes, (A3) notifications + background refresh, (A4) widget, (A5) Syncthing setup guide.
Open decision for A2: SAF through a Tauri plugin vs. a Kotlin bridge (decide when starting).

## 4. Later / ideas (not in v1)
- Switch the timetable to the Wise REST API if credentials are granted (notifications, holidays, exam event types).
- Auto-create Roki from timetable reservations that look like exams.
- Grades and ECTS tracker, study-time timer, per-subject progress from vault notes.
- Global shortcut for a quick note into an Inbox folder. Grades and ECTS tracker.
- Single synced settings file shared by Linux and Android.

## 5. Risks
- The iCal feed is the student-facing export. Its format could change; the parser is isolated in `ics.rs` with tests.
- No documented rate limit: the app fetches on start/refresh/focus only, never on a tight timer.
- WebKitGTK on Wayland/NVIDIA can render blank; `run.sh` documents the `WEBKIT_DISABLE_DMABUF_RENDERER=1` fallback.
