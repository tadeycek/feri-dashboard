use serde::{Deserialize, Serialize};

/// One timetable occurrence. `start`/`end` are local Ljubljana time, `YYYY-MM-DDTHH:MM:SS`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Event {
    pub uid: String,
    pub start: String,
    pub end: String,
    /// Vault name for known subjects ("Matematika 1"), otherwise the title-cased feed name.
    pub subject: String,
    /// Short code of a known subject (MAT, UVR, PRO1, SPO, UVS), empty otherwise.
    pub key: String,
    /// PR, RV, SV, LV, or REZ for a faculty-wide reservation.
    pub kind: String,
    /// Extra tags from the title, e.g. "RU, LA".
    pub tags: Vec<String>,
    pub room: String,
    pub lecturers: Vec<String>,
    pub groups: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Change {
    /// "dodano" | "odstranjeno" | "spremenjeno"
    pub kind: String,
    pub uid: String,
    pub subject: String,
    pub key: String,
    pub ekind: String,
    /// Start of the affected class (new start if it still exists).
    pub start: String,
    pub detail: String,
    pub detected_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Rok {
    /// File name inside `Roki/`, e.g. "1. kolokvij.md".
    pub file: String,
    pub title: String,
    pub predmet: String,
    pub vrsta: String,
    pub datum: String,
    pub ura: String,
    pub prostor: String,
    pub opravljeno: bool,
    pub ocena: String,
}

#[derive(Deserialize, Debug)]
pub struct NewRok {
    pub title: String,
    pub predmet: String,
    pub vrsta: String,
    pub datum: String,
    pub ura: String,
    pub prostor: String,
    pub snov: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct NoteInfo {
    pub predmet: String,
    pub kind: String,
    pub title: String,
    /// Path relative to the vault root.
    pub path: String,
    /// Seconds since the Unix epoch.
    pub modified: u64,
}

#[derive(Deserialize, Debug)]
pub struct NewNote {
    pub predmet: String,
    /// "Predavanja" or "Vaje"
    pub kind: String,
    pub title: String,
    pub body: String,
    pub datum: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub vault_path: String,
    pub ics_url: String,
    /// Closing the window hides it to the tray instead of quitting.
    pub close_to_tray: bool,
    /// Desktop notification shortly before each class, and when the timetable changes.
    pub notify_classes: bool,
    pub notify_minutes: u32,
    /// Notification the day before a deadline and on the day.
    pub notify_deadlines: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap_or_default();
        let sep = if cfg!(windows) { '\\' } else { '/' };
        Settings {
            vault_path: ["Documents", "FERI", "Zapiski", "Sola"].iter().fold(home, |p, d| format!("{p}{sep}{d}")),
            ics_url: "https://www.wise-tt.com/web/feri/reports?g=1211_1214_1220_1221&lang=sl&format=ics".into(),
            close_to_tray: true,
            notify_classes: true,
            notify_minutes: 10,
            notify_deadlines: true,
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct SubjectInfo {
    pub key: &'static str,
    pub name: &'static str,
    pub feed_name: &'static str,
}

/// Known subjects: feed title → vault name → short code.
pub const SUBJECTS: [SubjectInfo; 5] = [
    SubjectInfo { key: "MAT", name: "Matematika 1", feed_name: "MATEMATIKA I" },
    SubjectInfo { key: "UVR", name: "Uvod v računalništvo", feed_name: "UVOD V RAČUNALNIŠTVO" },
    SubjectInfo { key: "PRO1", name: "Programiranje 1", feed_name: "PROGRAMIRANJE I" },
    SubjectInfo { key: "SPO", name: "Sodobna programska orodja", feed_name: "SODOBNA PROGRAMSKA ORODJA" },
    SubjectInfo { key: "UVS", name: "Uvod v svetovni splet", feed_name: "UVOD V SVETOVNI SPLET" },
];
