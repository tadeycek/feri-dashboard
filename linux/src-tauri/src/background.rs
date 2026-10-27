//! Everything that runs while the window is hidden: the tray icon, autostart, desktop
//! notifications, and a periodic timetable refresh.

use crate::model::Event;
use crate::{store, vault};
use chrono::{Duration as Days, Local, NaiveDate, NaiveDateTime, NaiveTime};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};

const TICK: Duration = Duration::from_secs(30);
/// Background refresh every 60 ticks = 30 minutes.
const SYNC_EVERY_TICKS: u64 = 60;
const MAX_REMEMBERED: usize = 400;

/// Kept in managed state so the scheduler can update the tray, and the window can ask whether hiding is safe.
pub struct TrayState {
    pub ok: bool,
    info: Option<MenuItem<Wry>>,
}

// ---------- autostart ----------

#[cfg(not(windows))]
fn autostart_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("autostart").join("feri-dashboard.desktop"))
}

/// Quotes a path for the `Exec=` key of a desktop entry.
#[cfg(not(windows))]
fn exec_quote(path: &str) -> String {
    let mut out = String::from("\"");
    for c in path.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('"');
    out
}

#[cfg(not(windows))]
fn autostart_entry(exe: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=FERI Dashboard\nComment=Timetable, deadlines and notes for FERI\nExec={} --minimized\nIcon=feri-dashboard\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        exec_quote(exe)
    )
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_autostart() -> bool {
    autostart_file().map_or(false, |p| p.is_file())
}

/// Turns autostart on (writes the entry) or off (removes it). Starting at login opens the app in the tray, without a window.
#[cfg(not(windows))]
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let path = autostart_file().ok_or("Could not find your config folder.")?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        fs::write(&path, autostart_entry(&exe.to_string_lossy())).map_err(|e| e.to_string())
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

// Windows: the per-user Run key, so no admin rights and no files to clean up.
#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const RUN_VALUE: &str = "FERI Dashboard";

#[cfg(windows)]
#[tauri::command]
pub fn get_autostart() -> bool {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>(RUN_VALUE))
        .is_ok()
}

#[cfg(windows)]
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    use winreg::{enums::{HKEY_CURRENT_USER, KEY_SET_VALUE}, RegKey};
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey_with_flags(RUN_KEY, KEY_SET_VALUE).map_err(|e| e.to_string())?;
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        key.set_value(RUN_VALUE, &format!("\"{}\" --minimized", exe.display())).map_err(|e| e.to_string())
    } else {
        match key.delete_value(RUN_VALUE) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
            _ => Ok(()),
        }
    }
}

// ---------- window and tray ----------

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// True when closing the window should hide it to the tray instead of quitting.
pub fn hide_on_close(app: &AppHandle) -> bool {
    app.try_state::<TrayState>().map_or(false, |s| s.ok) && store::load_settings(app).close_to_tray
}

fn build_tray(app: &AppHandle) -> tauri::Result<MenuItem<Wry>> {
    let info = MenuItem::with_id(app, "info", "FERI Dashboard", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh timetable", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&info, &sep1, &show, &refresh, &sep2, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("FERI Dashboard")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main(app),
            "refresh" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::do_sync(&app).await;
                    let _ = app.emit("timetable-updated", ());
                });
            }
            "quit" => {
                // Let the page flush any unsaved note first, then exit.
                let _ = app.emit("before-quit", ());
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(1000));
                    app.exit(0);
                });
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(info)
}

/// Creates the tray, registers its state, and shows the window unless the app was started minimized.
pub fn setup(app: &tauri::App) {
    let handle = app.handle().clone();
    let tray = build_tray(&handle);
    if let Err(e) = &tray {
        eprintln!("tray icon unavailable: {e}");
    }
    let ok = tray.is_ok();
    app.manage(TrayState { ok, info: tray.ok() });

    // Without a working tray a hidden window could never be reopened, so never start hidden then.
    let minimized = std::env::args().any(|a| a == "--minimized");
    if !(minimized && ok) {
        show_main(&handle);
    }
    start_scheduler(handle);
}

// ---------- timetable helpers ----------

fn parse(iso: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S").ok()
}

fn kind_label(kind: &str) -> &str {
    match kind {
        "PR" => "Lecture",
        "RV" => "Exercises",
        "SV" => "Seminar",
        "LV" => "Lab",
        other => other,
    }
}

/// The line shown at the top of the tray menu.
fn tray_text(events: &[Event], now: NaiveDateTime) -> String {
    let classes = || events.iter().filter(|e| e.kind != "REZ").filter_map(|e| Some((parse(&e.start)?, parse(&e.end)?, e)));
    if let Some((_, end, e)) = classes().find(|(s, end, _)| *s <= now && now < *end) {
        return format!("Now: {} until {}", e.subject, end.format("%H:%M"));
    }
    match classes().find(|(s, _, _)| *s > now) {
        Some((start, _, e)) => {
            let when = if start.date() == now.date() {
                format!("today {}", start.format("%H:%M"))
            } else if start.date() == now.date() + Days::days(1) {
                format!("tomorrow {}", start.format("%H:%M"))
            } else {
                start.format("%a %H:%M").to_string()
            };
            format!("Next: {} {}", e.subject, when)
        }
        None => "No upcoming classes".into(),
    }
}

// ---------- notifications ----------

fn notify(summary: &str, body: &str) {
    let shown = notify_rust::Notification::new()
        .appname("FERI Dashboard")
        .summary(summary)
        .body(body)
        .icon("feri-dashboard")
        .timeout(notify_rust::Timeout::Milliseconds(12_000))
        .show();
    if let Err(e) = shown {
        eprintln!("notification failed: {e}");
    }
}

/// `feri-dashboard --test-notification` sends one notification and exits, to check that they work.
pub fn test_notification() {
    notify("FERI Dashboard", "Notifications work. You will get one before each class.");
}

/// Keys of notifications already shown, so a restart does not repeat them.
struct Notified {
    keys: Vec<String>,
    path: Option<PathBuf>,
}

impl Notified {
    fn load(app: &AppHandle) -> Self {
        let path = app.path().app_data_dir().ok().map(|d| d.join("notified.json"));
        let keys = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Notified { keys, path }
    }

    fn seen(&self, key: &str) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    fn mark(&mut self, key: String) {
        self.keys.push(key);
        if self.keys.len() > MAX_REMEMBERED {
            self.keys.drain(..self.keys.len() - MAX_REMEMBERED);
        }
        if let Some(p) = &self.path {
            if let Some(dir) = p.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = serde_json::to_string(&self.keys).map(|t| fs::write(p, t));
        }
    }
}

/// Classes that start within the next `minutes` and have not been announced yet. They are marked as announced.
fn classes_to_announce<'a>(events: &'a [Event], minutes: u32, now: NaiveDateTime, done: &mut Notified) -> Vec<(&'a Event, NaiveDateTime, i64)> {
    let window = i64::from(minutes.clamp(1, 120)) * 60;
    let mut out = Vec::new();
    for e in events.iter().filter(|e| e.kind != "REZ") {
        let Some(start) = parse(&e.start) else { continue };
        let secs = (start - now).num_seconds();
        let key = format!("class|{}", e.uid);
        if secs > 0 && secs <= window && !done.seen(&key) {
            done.mark(key);
            out.push((e, start, (secs + 59) / 60));
        }
    }
    out
}

fn notify_classes(events: &[Event], minutes: u32, now: NaiveDateTime, done: &mut Notified) {
    for (e, start, left) in classes_to_announce(events, minutes, now, done) {
        let room = if e.room.is_empty() { String::new() } else { format!(" · {}", e.room) };
        notify(&format!("{} in {} min", e.subject, left), &format!("{} at {}{}", kind_label(&e.kind), start.format("%H:%M"), room));
    }
}

fn notify_deadlines(app: &AppHandle, now: NaiveDateTime, done: &mut Notified) {
    let settings = store::load_settings(app);
    let Ok(root) = vault::root(&settings.vault_path) else { return };
    let today = now.date();
    let (from_tomorrow, from_today) = (NaiveTime::from_hms_opt(9, 0, 0).unwrap(), NaiveTime::from_hms_opt(8, 0, 0).unwrap());
    for r in vault::list_roki(&root).into_iter().filter(|r| !r.opravljeno) {
        let Ok(date) = NaiveDate::parse_from_str(&r.datum, "%Y-%m-%d") else { continue };
        let label = if date == today + Days::days(1) && now.time() >= from_tomorrow {
            "Due tomorrow"
        } else if date == today && now.time() >= from_today {
            "Due today"
        } else {
            continue;
        };
        let key = format!("due|{}|{}|{}", r.file, r.datum, label);
        if done.seen(&key) {
            continue;
        }
        done.mark(key);
        let mut body = r.title.clone();
        if !r.predmet.is_empty() {
            body.push_str(&format!(" · {}", r.predmet));
        }
        if !r.ura.is_empty() {
            body.push_str(&format!(" · {}", r.ura));
        }
        notify(label, &body);
    }
}

// ---------- scheduler ----------

fn start_scheduler(app: AppHandle) {
    std::thread::spawn(move || {
        let mut done = Notified::load(&app);
        // Give the page's own refresh at startup a head start.
        std::thread::sleep(Duration::from_secs(20));
        let mut tick: u64 = 0;
        loop {
            let settings = store::load_settings(&app);
            let cache = store::load_cache(&app);
            let now = Local::now().naive_local();

            if let Some(item) = app.try_state::<TrayState>().and_then(|s| s.info.clone()) {
                let _ = item.set_text(tray_text(&cache.events, now));
            }
            if settings.notify_classes {
                notify_classes(&cache.events, settings.notify_minutes, now, &mut done);
            }
            if settings.notify_deadlines {
                notify_deadlines(&app, now, &mut done);
            }

            if tick > 0 && tick % SYNC_EVERY_TICKS == 0 {
                if let Ok(res) = tauri::async_runtime::block_on(crate::do_sync(&app)) {
                    if !res.offline {
                        if settings.notify_classes {
                            let fresh: Vec<_> = res.changes.iter().filter(|c| Some(&c.detected_at) == res.fetched_at.as_ref()).collect();
                            if !fresh.is_empty() {
                                let lines: Vec<String> = fresh.iter().take(3).map(|c| format!("{} {}: {}", c.subject, kind_label(&c.ekind), c.detail)).collect();
                                notify("Timetable changed", &lines.join("\n"));
                            }
                        }
                        let _ = app.emit("timetable-updated", ());
                    }
                }
            }
            tick += 1;
            std::thread::sleep(TICK);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(uid: &str, start: &str, end: &str, kind: &str) -> Event {
        Event { uid: uid.into(), start: start.into(), end: end.into(), subject: "Matematika 1".into(), key: "MAT".into(), kind: kind.into(), room: "A-306".into(), ..Default::default() }
    }
    fn at(s: &str) -> NaiveDateTime {
        parse(s).unwrap()
    }

    #[cfg(not(windows))]
    #[test]
    fn desktop_entry_quotes_the_path() {
        let e = autostart_entry("/home/me/My Apps/feri-dashboard");
        assert!(e.contains("Exec=\"/home/me/My Apps/feri-dashboard\" --minimized"));
        assert!(e.contains("X-GNOME-Autostart-enabled=true"));
        assert_eq!(exec_quote("a\"b$c"), "\"a\\\"b\\$c\"");
    }

    #[test]
    fn tray_line_says_now_next_and_when() {
        let events = vec![ev("a", "2026-10-13T07:00:00", "2026-10-13T10:00:00", "PR"), ev("b", "2026-10-14T14:00:00", "2026-10-14T16:00:00", "RV"), ev("r", "2026-10-13T11:00:00", "2026-10-13T12:00:00", "REZ")];
        assert_eq!(tray_text(&events, at("2026-10-13T08:12:00")), "Now: Matematika 1 until 10:00");
        assert_eq!(tray_text(&events, at("2026-10-13T10:30:00")), "Next: Matematika 1 tomorrow 14:00");
        assert_eq!(tray_text(&events, at("2026-10-12T20:00:00")), "Next: Matematika 1 tomorrow 07:00");
        assert_eq!(tray_text(&events, at("2026-10-20T20:00:00")), "No upcoming classes");
    }

    #[test]
    fn class_notice_fires_once_inside_the_window() {
        let events = vec![ev("a", "2026-10-13T10:00:00", "2026-10-13T12:00:00", "PR")];
        let mut done = Notified { keys: vec![], path: None };
        // 15 minutes before with a 10 minute window: nothing. 8 minutes before: one. Again: none.
        assert!(classes_to_announce(&events, 10, at("2026-10-13T09:45:00"), &mut done).is_empty());
        let hit = classes_to_announce(&events, 10, at("2026-10-13T09:52:00"), &mut done);
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].2, 8, "minutes left");
        assert!(classes_to_announce(&events, 10, at("2026-10-13T09:53:00"), &mut done).is_empty(), "only once");
        // already started: never
        let mut late = Notified { keys: vec![], path: None };
        assert!(classes_to_announce(&events, 10, at("2026-10-13T10:01:00"), &mut late).is_empty());
        // reservations are not classes
        let res = vec![ev("r", "2026-10-13T10:00:00", "2026-10-13T11:00:00", "REZ")];
        assert!(classes_to_announce(&res, 10, at("2026-10-13T09:55:00"), &mut late).is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn autostart_can_be_added_and_removed() {
        let dir = std::env::temp_dir().join(format!("feri-autostart-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        std::env::set_var("XDG_CONFIG_HOME", &dir);
        assert!(!get_autostart());
        set_autostart(true).unwrap();
        assert!(get_autostart());
        let text = fs::read_to_string(dir.join("autostart/feri-dashboard.desktop")).unwrap();
        assert!(text.contains("--minimized"));
        set_autostart(false).unwrap();
        assert!(!get_autostart());
        set_autostart(false).unwrap(); // removing twice is fine
        let _ = fs::remove_dir_all(&dir);
    }
}
