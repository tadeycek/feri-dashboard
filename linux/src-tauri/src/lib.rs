mod background;
mod ics;
mod model;
mod starter;
mod store;
mod vault;

use model::{Change, Event, NewNote, NewRok, NoteInfo, Rok, Settings, SubjectInfo, SUBJECTS};
use vault::{Entry, NoteContent, SearchHit};
use serde::Serialize;
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct SyncResult {
    events: Vec<Event>,
    changes: Vec<Change>,
    fetched_at: Option<String>,
    /// True when the feed could not be reached and the cached copy is shown.
    offline: bool,
    error: Option<String>,
}

fn now_local() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn result_from(cache: store::Cache, error: Option<String>) -> SyncResult {
    SyncResult {
        events: cache.events,
        changes: cache.changes,
        fetched_at: cache.fetched_at,
        offline: error.is_some(),
        error,
    }
}

async fn fetch_feed(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent(concat!("feri-dashboard/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    let text = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Could not download the timetable ({e})."))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    if !text.contains("BEGIN:VCALENDAR") {
        return Err("The server did not return a calendar (iCal). Check the link in Settings.".into());
    }
    Ok(text)
}

#[tauri::command]
fn get_subjects() -> Vec<SubjectInfo> {
    SUBJECTS.to_vec()
}

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    store::load_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    store::save_settings(&app, &settings)
}

/// Instant paint at boot: whatever was fetched last time.
#[tauri::command]
fn load_cached(app: AppHandle) -> SyncResult {
    result_from(store::load_cache(&app), None)
}

/// Fetch the timetable feed, diff it against the cache, save, return.
/// Never fails hard: on a network error the cached copy comes back with `offline: true`.
#[tauri::command]
async fn sync_timetable(app: AppHandle) -> Result<SyncResult, String> {
    do_sync(&app).await
}

/// Shared by the `sync_timetable` command and the background refresh while the app sits in the tray.
pub(crate) async fn do_sync(app: &AppHandle) -> Result<SyncResult, String> {
    let settings = store::load_settings(app);
    let mut cache = store::load_cache(app);
    let text = match fetch_feed(&settings.ics_url).await {
        Ok(t) => t,
        Err(e) => return Ok(result_from(cache, Some(e))),
    };
    let events = ics::parse(&text);
    if events.is_empty() {
        return Ok(result_from(cache, Some("The timetable from the server is empty.".into())));
    }
    let now = now_local();
    if cache.fetched_at.is_some() {
        let mut fresh = store::diff(&cache.events, &events, &now);
        fresh.append(&mut cache.changes);
        cache.changes = fresh;
        store::trim_changes(&mut cache.changes);
    }
    cache.events = events;
    cache.fetched_at = Some(now);
    store::save_cache(app, &cache)?;
    Ok(result_from(cache, None))
}

#[tauri::command]
fn list_roki(app: AppHandle) -> Result<Vec<Rok>, String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    Ok(vault::list_roki(&root))
}

#[tauri::command]
fn create_rok(app: AppHandle, rok: NewRok) -> Result<String, String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    vault::create_rok(&root, &rok)
}

#[tauri::command]
fn toggle_rok(app: AppHandle, file: String) -> Result<bool, String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    vault::toggle_rok(&root, &file)
}

#[tauri::command]
fn save_note(app: AppHandle, note: NewNote) -> Result<NoteInfo, String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    vault::save_note(&root, &note)
}

#[tauri::command]
fn recent_notes(app: AppHandle) -> Result<Vec<NoteInfo>, String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    Ok(vault::recent_notes(&root, 12))
}

/// Hands a URL or URI to the system's default handler.
fn open_uri(uri: &str) -> std::io::Result<()> {
    #[cfg(windows)]
    let mut cmd = std::process::Command::new("rundll32");
    #[cfg(windows)]
    cmd.arg("url.dll,FileProtocolHandler");
    #[cfg(not(windows))]
    let mut cmd = std::process::Command::new("xdg-open");
    cmd.arg(uri).spawn().map(|_| ())
}

/// Fills a folder with the empty starter vault. Existing files are kept. Returns the number of files written.
#[tauri::command]
fn create_starter_vault(path: String) -> Result<usize, String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("Choose a folder first.".into());
    }
    starter::create(Path::new(path))
}

#[tauri::command]
fn open_in_obsidian(app: AppHandle, path: String) -> Result<(), String> {
    let root = vault::root(&store::load_settings(&app).vault_path)?;
    let uri = vault::obsidian_uri(&root, &path);
    open_uri(&uri).map_err(|e| format!("Could not open Obsidian ({e})."))
}

fn vault_root(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    vault::root(&store::load_settings(app).vault_path)
}

#[tauri::command]
fn vault_tree(app: AppHandle) -> Result<Vec<Entry>, String> {
    Ok(vault::tree(&vault_root(&app)?))
}

#[tauri::command]
fn read_note(app: AppHandle, path: String) -> Result<NoteContent, String> {
    vault::read_note(&vault_root(&app)?, &path)
}

#[tauri::command]
fn write_note(app: AppHandle, path: String, content: String) -> Result<u64, String> {
    vault::write_note(&vault_root(&app)?, &path, &content)
}

#[tauri::command]
fn create_note(app: AppHandle, path: String, content: String) -> Result<String, String> {
    vault::create_note(&vault_root(&app)?, &path, &content)
}

#[tauri::command]
fn create_folder(app: AppHandle, path: String) -> Result<String, String> {
    vault::create_folder(&vault_root(&app)?, &path)
}

/// Returns the new path and how many other notes had their links updated.
#[tauri::command]
fn rename_path(app: AppHandle, from: String, to: String) -> Result<(String, usize), String> {
    vault::rename(&vault_root(&app)?, &from, &to)
}

#[tauri::command]
fn trash_path(app: AppHandle, path: String) -> Result<String, String> {
    vault::trash(&vault_root(&app)?, &path)
}

#[tauri::command]
fn search_vault(app: AppHandle, query: String) -> Result<Vec<SearchHit>, String> {
    Ok(vault::search(&vault_root(&app)?, &query, 200))
}

#[tauri::command]
fn backlinks(app: AppHandle, path: String) -> Result<Vec<SearchHit>, String> {
    Ok(vault::backlinks(&vault_root(&app)?, &path))
}

/// Opens a web link in the default browser. Only http(s) and mailto are allowed.
#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let ok = ["http://", "https://", "mailto:"].iter().any(|p| url.starts_with(p));
    if !ok {
        return Err("This kind of link is not allowed.".into());
    }
    open_uri(&url).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // `feri-dashboard --test-notification` sends one notification and exits.
    if std::env::args().any(|a| a == "--test-notification") {
        background::test_notification();
        return;
    }

    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // A second launch (menu, autostart) brings the running window forward instead of starting another copy.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| background::show_main(app)));

    builder
        .setup(|app| {
            background::setup(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if background::hide_on_close(window.app_handle()) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_subjects,
            get_settings,
            save_settings,
            load_cached,
            sync_timetable,
            list_roki,
            create_rok,
            toggle_rok,
            save_note,
            recent_notes,
            open_in_obsidian,
            create_starter_vault,
            vault_tree,
            read_note,
            write_note,
            create_note,
            create_folder,
            rename_path,
            trash_path,
            search_vault,
            backlinks,
            open_external,
            background::get_autostart,
            background::set_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FERI Dashboard");
}
