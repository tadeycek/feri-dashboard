use crate::model::{Event, SUBJECTS};
use std::collections::HashMap;

fn unfold(s: &str) -> String {
    s.replace("\r\n ", "").replace("\r\n\t", "").replace("\n ", "").replace("\r\n", "\n")
}

fn unescape(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut it = v.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(x) => out.push(x),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `20261001T110000` → `2026-10-01T11:00:00`
fn fmt_dt(v: &str) -> String {
    let v = v.trim().trim_end_matches('Z');
    if !v.is_ascii() {
        return v.to_string();
    }
    if v.len() >= 15 {
        format!("{}-{}-{}T{}:{}:{}", &v[0..4], &v[4..6], &v[6..8], &v[9..11], &v[11..13], &v[13..15])
    } else if v.len() >= 8 {
        format!("{}-{}-{}T00:00:00", &v[0..4], &v[4..6], &v[6..8])
    } else {
        v.to_string()
    }
}

fn title_case(s: &str) -> String {
    s.split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// "SODOBNA PROGRAMSKA ORODJA (RV) (RU, LA)" → ("SODOBNA PROGRAMSKA ORODJA", "RV", ["RU, LA"])
fn split_summary(s: &str) -> (String, String, Vec<String>) {
    let s = s.trim();
    match s.find(" (") {
        Some(i) => {
            let subject = s[..i].trim().to_string();
            let tags: Vec<String> = s[i..]
                .split('(')
                .skip(1)
                .filter_map(|p| p.find(')').map(|j| p[..j].trim().to_string()))
                .collect();
            let kind = tags.first().cloned().unwrap_or_default();
            (subject, kind, tags.into_iter().skip(1).collect())
        }
        None => (s.to_string(), String::new(), Vec::new()),
    }
}

/// Labels Wise has used for the teachers line. It was "Predavatelji" and became "Izvajalci" without notice.
const LECTURER_LABELS: [&str; 6] = ["Predavatelji", "Predavatelj", "Izvajalci", "Izvajalec", "Lecturers", "Tutors"];
const GROUP_LABELS: [&str; 2] = ["Skupine", "Groups"];

fn is_label(label: &str, known: &[&str]) -> bool {
    known.iter().any(|k| k.eq_ignore_ascii_case(label))
}

/// Splits an event description ("Izvajalci: A, B\nSkupine: X") into (lecturers, groups).
/// If the teachers label is one we do not know, a description with exactly two labelled lines (teachers + groups)
/// is still read correctly, so a future rename does not silently empty the list.
fn parse_description(d: &str) -> (Vec<String>, String) {
    let lines: Vec<(&str, &str)> = d.lines().filter_map(|l| l.split_once(':')).map(|(a, b)| (a.trim(), b.trim())).collect();
    let groups = lines.iter().find(|(l, _)| is_label(l, &GROUP_LABELS)).map(|(_, v)| v.to_string()).unwrap_or_default();
    let teachers = lines
        .iter()
        .find(|(l, _)| is_label(l, &LECTURER_LABELS))
        .or_else(|| (lines.len() == 2).then(|| lines.iter().find(|(l, _)| !is_label(l, &GROUP_LABELS))).flatten());
    let lecturers = teachers
        .map(|(_, v)| v.split(',').map(|n| title_case(n.trim())).filter(|n| !n.is_empty()).collect())
        .unwrap_or_default();
    (lecturers, groups)
}

fn build_event(f: &HashMap<String, String>) -> Option<Event> {
    let start = fmt_dt(f.get("DTSTART")?);
    let end = f.get("DTEND").map(|v| fmt_dt(v)).unwrap_or_else(|| start.clone());
    let (raw_subject, mut kind, tags) = split_summary(f.get("SUMMARY").map(String::as_str).unwrap_or(""));

    let reservation = raw_subject.eq_ignore_ascii_case("rezervacija");
    let (lecturers, mut groups) = f.get("DESCRIPTION").map(|d| parse_description(d)).unwrap_or_default();
    // A reservation lists every group on the faculty; that is noise.
    if reservation {
        groups.clear();
    }

    let upper = raw_subject.to_uppercase();
    let known = SUBJECTS.iter().find(|s| s.feed_name == upper);
    let (subject, key) = match known {
        Some(s) => (s.name.to_string(), s.key.to_string()),
        None => (title_case(&raw_subject.to_lowercase()), String::new()),
    };
    if reservation {
        kind = "REZ".into();
    }

    Some(Event {
        uid: f.get("UID").cloned().unwrap_or_else(|| format!("{start}-{subject}")),
        start,
        end,
        subject,
        key,
        kind,
        tags,
        room: f.get("LOCATION").cloned().unwrap_or_default().trim().to_string(),
        lecturers,
        groups,
    })
}

pub fn parse(text: &str) -> Vec<Event> {
    let text = unfold(text);
    let mut events = Vec::new();
    let mut cur: Option<HashMap<String, String>> = None;
    for line in text.lines() {
        match line {
            "BEGIN:VEVENT" => cur = Some(HashMap::new()),
            "END:VEVENT" => {
                if let Some(f) = cur.take() {
                    if let Some(e) = build_event(&f) {
                        events.push(e);
                    }
                }
            }
            _ => {
                if let Some(f) = cur.as_mut() {
                    if let Some((head, value)) = line.split_once(':') {
                        let name = head.split(';').next().unwrap_or(head).to_string();
                        f.entry(name).or_insert_with(|| unescape(value));
                    }
                }
            }
        }
    }
    events.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.uid.cmp(&b.uid)));
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:S3555-20261001@feri.wise-tt.com\r\nDTSTART;TZID=Europe/Ljubljana:20261001T110000\r\nDTEND;TZID=Europe/Ljubljana:20261001T130000\r\nSUMMARY:SODOBNA PROGRAMSKA ORODJA (RV) (RU, LA)\r\nLOCATION:G-217\r\nDESCRIPTION:Predavatelji: LEON KUTOŠ\\nSkupine: R-IT 1 VS RV - 2. sk\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:R1\r\nDTSTART;TZID=Europe/Ljubljana:20261007T070000\r\nDTEND;TZID=Europe/Ljubljana:20261007T100000\r\nSUMMARY:Rezervacija\r\nDESCRIPTION:Skupine: A, B,\r\n  C\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[test]
    fn lecturers_under_both_labels_and_unknown_label() {
        // current label
        assert_eq!(parse_description("Izvajalci: DANILO KORŽE, LEON KUTOŠ\nSkupine: R-IT 1 VS"), (vec!["Danilo Korže".into(), "Leon Kutoš".into()], "R-IT 1 VS".into()));
        // previous label
        assert_eq!(parse_description("Predavatelji: JANEZ BREST\nSkupine: R-IT 1 VS").0, vec!["Janez Brest".to_string()]);
        // a label we have never seen, in the normal two-line shape
        assert_eq!(parse_description("Nosilci: NIKO LUKAČ\nSkupine: R-IT 1 VS").0, vec!["Niko Lukač".to_string()]);
        // groups only (reservations): no lecturers, and an unknown extra line is not guessed at
        assert!(parse_description("Skupine: A, B").0.is_empty());
        assert!(parse_description("Opomba: x\nDrugo: y\nSkupine: A").0.is_empty());
    }

    #[test]
    fn parses_sample() {
        let ev = parse(SAMPLE);
        assert_eq!(ev.len(), 2);
        let e = &ev[0];
        assert_eq!(e.subject, "Sodobna programska orodja");
        assert_eq!(e.key, "SPO");
        assert_eq!(e.kind, "RV");
        assert_eq!(e.tags, vec!["RU, LA"]);
        assert_eq!(e.start, "2026-10-01T11:00:00");
        assert_eq!(e.room, "G-217");
        assert_eq!(e.lecturers, vec!["Leon Kutoš"]);
        assert_eq!(e.groups, "R-IT 1 VS RV - 2. sk");
        assert_eq!(ev[1].kind, "REZ");
        assert!(ev[1].groups.is_empty());
    }

    /// Run with `FERI_ICS_FIXTURE=/path/to/feed.ics cargo test real_feed -- --nocapture`.
    #[test]
    fn real_feed() {
        let Ok(path) = std::env::var("FERI_ICS_FIXTURE") else { return };
        let ev = parse(&std::fs::read_to_string(path).unwrap());
        let classes = ev.iter().filter(|e| e.kind != "REZ").count();
        println!("{} events, {} classes", ev.len(), classes);
        assert_eq!(ev.len(), 196);
        assert!(ev.iter().filter(|e| e.kind != "REZ").all(|e| !e.key.is_empty()));
        // Guards against the feed renaming the teachers label again (it did once, and the list silently went empty).
        let with_teacher = ev.iter().filter(|e| e.kind != "REZ" && !e.lecturers.is_empty()).count();
        println!("{with_teacher} of {classes} classes have a lecturer");
        assert!(with_teacher * 10 >= classes * 9, "lecturers missing: only {with_teacher} of {classes} classes have one");
    }
}
