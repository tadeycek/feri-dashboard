# FERI Dashboard (Linux)

Tauri 2 (Rust) + Svelte 5. Shows the FERI timetable from Wise Timetable and your deadlines (roki), and lets you write notes straight into the Obsidian vault.
Plan and scope for both platforms: [`../PLAN.md`](../PLAN.md). Design rules: [`../DESIGN.md`](../DESIGN.md).

## Run

```sh
./run.sh                     # after a build
```

## Build

```sh
npm install
npx tauri build --no-bundle  # → src-tauri/target/release/feri-dashboard
npx tauri build              # also makes .deb and .AppImage (src-tauri/target/release/bundle)
```

## Develop

```sh
npm run dev                  # UI in a browser with demo data (http://127.0.0.1:1420), no Rust needed
npx tauri dev                # real app with hot reload
cd src-tauri && cargo test   # parser, diff and vault tests
FERI_VAULT=/path/to/vault cargo test real_vault -- --nocapture   # read-only check against a real vault
FERI_ICS_FIXTURE=/path/to/feed.ics cargo test real_feed -- --nocapture   # check against a downloaded feed
```

`dev/screenshots/` has reference screenshots (light and dark) of each view.

## How it works

- **Timetable:** the public iCal feed from Wise Timetable (no login). It is fetched on every start, on refresh (button or `r`) and when the window regains focus after 5+ minutes. The last result is cached in `~/.local/share/si.tadej.feri-dashboard/cache.json`, so the app opens instantly and works offline.
- **Changes:** each fetch is compared with the previous one by event UID. Added, removed and moved classes (time or room) show up under *Timetable changes* on the Today page.
- **Vault:** `~/Documents/FERI/Zapiski/Sola` (change it in Nastavitve). The Zapiski page is an Obsidian-style workspace on the same folder, so changes show up in Obsidian and the other way round (the app re-reads notes when its window gets focus). Roki live in `Roki/`. New notes from a class use the vault's own template format.
- **Safety:** paths are checked so nothing outside the vault or in hidden folders (`.obsidian`) is touched. New files never overwrite existing ones (` 2`, ` 3` is appended). Delete moves to `.trash` in the vault. Renaming a note updates `[[links]]` to it in other notes, like Obsidian.
- **Settings:** `~/.config/si.tadej.feri-dashboard/settings.json`.

## Tray, notifications, autostart and the launcher

- **Application launcher:** `./install.sh` adds *FERI Dashboard* to the application menu (Education) with its icon, for your user only. `./install.sh --uninstall` removes the entry again.
- **Tray icon:** the top line of its menu shows what is on now or next. Other entries: Show dashboard, Refresh timetable, Quit. Closing the window hides it to the tray (Settings → turn this off to quit on close). A second launch brings the running window forward instead of starting another copy.
- **Notifications:** a desktop notification N minutes before each class (5, 10, 15 or 30), when the timetable changes, and for deadlines the day before (from 09:00) and on the day (from 08:00). Each one is shown once. They only fire while the app is running, which is why the tray and autostart exist. While it sits in the tray it refreshes the timetable every 30 minutes.
- **Autostart:** Settings → *Start with my computer* writes `~/.config/autostart/feri-dashboard.desktop` (starts hidden in the tray). Untick it and the file is removed again. The state shown is read from that file every time the dialog opens.
- `feri-dashboard --test-notification` sends one notification and exits. `feri-dashboard --minimized` starts in the tray without a window.

## Keys

`1` Today · `2` Week · `3` Deadlines · `4` Notes · `r` refresh

In Notes: `Ctrl+O` quick switcher · `Ctrl+N` new note · `Ctrl+E` reading/editing · `Ctrl+W` close tab · `Ctrl+Tab` next tab · `Ctrl+Shift+F` search · `Ctrl+click` on a `[[link]]` opens it · `Ctrl+B` / `Ctrl+I` bold / italic

## Menu entry (optional)

Copy `dev/feri-dashboard.desktop` to `~/.local/share/applications/`.
