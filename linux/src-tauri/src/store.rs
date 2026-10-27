use crate::model::{Change, Event, Settings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const MAX_CHANGES: usize = 40;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Cache {
    pub fetched_at: Option<String>,
    pub events: Vec<Event>,
    pub changes: Vec<Change>,
}

fn file_in(dir: Result<PathBuf, tauri::Error>, name: &str) -> Result<PathBuf, String> {
    let dir = dir.map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(name))
}

pub fn load_settings(app: &AppHandle) -> Settings {
    let defaults = Settings::default();
    let Ok(path) = file_in(app.path().app_config_dir(), "settings.json") else { return defaults };
    let Ok(text) = fs::read_to_string(path) else { return defaults };
    let mut s: Settings = serde_json::from_str(&text).unwrap_or_else(|_| defaults.clone());
    if s.vault_path.trim().is_empty() {
        s.vault_path = defaults.vault_path;
    }
    if s.ics_url.trim().is_empty() {
        s.ics_url = defaults.ics_url;
    }
    s
}

pub fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let path = file_in(app.path().app_config_dir(), "settings.json")?;
    fs::write(path, serde_json::to_string_pretty(s).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

pub fn load_cache(app: &AppHandle) -> Cache {
    file_in(app.path().app_data_dir(), "cache.json")
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_cache(app: &AppHandle, c: &Cache) -> Result<(), String> {
    let path = file_in(app.path().app_data_dir(), "cache.json")?;
    // Write to a temp file first so a crash never leaves a half-written cache.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string(c).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    fs::rename(tmp, path).map_err(|e| e.to_string())
}

pub fn trim_changes(changes: &mut Vec<Change>) {
    changes.truncate(MAX_CHANGES);
}

fn hm(iso: &str) -> &str {
    iso.get(11..16).unwrap_or(iso)
}

fn dm(iso: &str) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month = iso.get(5..7).and_then(|m| m.parse::<usize>().ok()).and_then(|m| MONTHS.get(m.wrapping_sub(1)));
    match (iso.get(8..10), month) {
        (Some(d), Some(m)) => format!("{} {}", d.trim_start_matches('0'), m),
        _ => iso.to_string(),
    }
}

/// Compare two fetches. Only classes that are today or later are reported,
/// and faculty-wide reservations are ignored.
pub fn diff(old: &[Event], new: &[Event], now: &str) -> Vec<Change> {
    let today = now.get(..10).unwrap_or(now);
    let old_map: HashMap<&str, &Event> = old.iter().map(|e| (e.uid.as_str(), e)).collect();
    let new_map: HashMap<&str, &Event> = new.iter().map(|e| (e.uid.as_str(), e)).collect();
    let mut out = Vec::new();

    let mk = |kind: &str, e: &Event, detail: String| Change {
        kind: kind.into(),
        uid: e.uid.clone(),
        subject: e.subject.clone(),
        key: e.key.clone(),
        ekind: e.kind.clone(),
        start: e.start.clone(),
        detail,
        detected_at: now.to_string(),
    };
    let relevant = |e: &Event| e.kind != "REZ" && e.start.get(..10).map_or(false, |d| d >= today);

    for e in new.iter().filter(|e| relevant(e)) {
        match old_map.get(e.uid.as_str()) {
            None => out.push(mk("dodano", e, format!("{} at {}", dm(&e.start), hm(&e.start)))),
            Some(o) => {
                let mut parts = Vec::new();
                if o.start != e.start || o.end != e.end {
                    if o.start.get(..10) != e.start.get(..10) {
                        parts.push(format!("{} {} → {} {}", dm(&o.start), hm(&o.start), dm(&e.start), hm(&e.start)));
                    } else {
                        parts.push(format!("time {}–{} → {}–{}", hm(&o.start), hm(&o.end), hm(&e.start), hm(&e.end)));
                    }
                }
                if o.room != e.room {
                    parts.push(format!("room {} → {}", if o.room.is_empty() { "?" } else { &o.room }, if e.room.is_empty() { "?" } else { &e.room }));
                }
                if !parts.is_empty() {
                    out.push(mk("spremenjeno", e, parts.join(", ")));
                }
            }
        }
    }
    for o in old.iter().filter(|e| relevant(e)) {
        if !new_map.contains_key(o.uid.as_str()) {
            out.push(mk("odstranjeno", o, format!("{} at {}", dm(&o.start), hm(&o.start))));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(uid: &str, start: &str, end: &str, room: &str) -> Event {
        Event { uid: uid.into(), start: start.into(), end: end.into(), subject: "Matematika 1".into(), key: "MAT".into(), kind: "PR".into(), room: room.into(), ..Default::default() }
    }

    #[test]
    fn detects_changes() {
        let old = vec![
            ev("a", "2026-10-13T07:00:00", "2026-10-13T10:00:00", "A-306"),
            ev("b", "2026-10-14T08:00:00", "2026-10-14T09:00:00", "A-1"),
            ev("gone", "2026-10-15T08:00:00", "2026-10-15T09:00:00", "A-1"),
            ev("past", "2026-10-01T08:00:00", "2026-10-01T09:00:00", "A-1"),
        ];
        let new = vec![
            ev("a", "2026-10-13T08:00:00", "2026-10-13T11:00:00", "A-306"),
            ev("b", "2026-10-14T08:00:00", "2026-10-14T09:00:00", "B-2"),
            ev("new", "2026-10-16T08:00:00", "2026-10-16T09:00:00", "C-3"),
        ];
        let d = diff(&old, &new, "2026-10-06T12:00:00");
        let kinds: Vec<_> = d.iter().map(|c| (c.kind.as_str(), c.uid.as_str())).collect();
        assert!(kinds.contains(&("spremenjeno", "a")));
        assert!(kinds.contains(&("spremenjeno", "b")));
        assert!(kinds.contains(&("dodano", "new")));
        assert!(kinds.contains(&("odstranjeno", "gone")));
        assert!(!kinds.iter().any(|(_, u)| *u == "past"));
        assert_eq!(d.iter().find(|c| c.uid == "a").unwrap().detail, "time 07:00–10:00 → 08:00–11:00");
    }
}
