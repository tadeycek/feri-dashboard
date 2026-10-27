//! Reads and writes the Obsidian study vault (`FERI/Zapiski/Sola`).
//! Notes are written in the same format as the vault's own templates (`Predloge/`).
//! Nothing here deletes or overwrites a note: new notes get a numeric suffix if the name is taken.

use crate::model::{NewNote, NewRok, NoteInfo, Rok, SUBJECTS};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const KINDS: [&str; 2] = ["Predavanja", "Vaje"];

pub fn root(vault_path: &str) -> Result<PathBuf, String> {
    let p = PathBuf::from(vault_path);
    if !p.is_dir() {
        return Err(format!("Vault folder not found: {vault_path}. Fix the path in Settings."));
    }
    Ok(p)
}

fn clean_value(v: &str) -> String {
    let v = v.trim().trim_matches('"').trim_matches('\'').trim();
    v.strip_prefix("[[").and_then(|s| s.strip_suffix("]]")).unwrap_or(v).trim().to_string()
}

fn frontmatter(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return map;
    }
    for l in lines {
        if l.trim() == "---" {
            break;
        }
        if l.starts_with(' ') || l.starts_with('-') || l.starts_with('\t') {
            continue; // list items and nested values
        }
        if let Some((k, v)) = l.split_once(':') {
            map.insert(k.trim().to_string(), clean_value(v));
        }
    }
    map
}

fn safe_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if "/\\:*?\"<>|#^[]".contains(c) || c.is_control() { '-' } else { c })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    let cleaned: String = cleaned.chars().take(100).collect();
    if cleaned.is_empty() { "Brez naslova".into() } else { cleaned }
}

fn unique_path(dir: &Path, stem: &str) -> PathBuf {
    let first = dir.join(format!("{stem}.md"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{stem} {n}.md")))
        .find(|p| !p.exists())
        .expect("unbounded range")
}

fn check_date(d: &str) -> Result<(), String> {
    let ok = d.len() == 10
        && d.char_indices().all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() });
    if ok { Ok(()) } else { Err("Date must be YYYY-MM-DD.".into()) }
}

fn quoted(v: &str) -> String {
    let v = v.trim().replace('"', "'");
    if v.is_empty() { String::new() } else { format!("\"{v}\"") }
}

// ---------- Roki ----------

pub fn list_roki(root: &Path) -> Vec<Rok> {
    let Ok(rd) = fs::read_dir(root.join("Roki")) else { return Vec::new() };
    let mut out: Vec<Rok> = rd
        .flatten()
        .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
        .filter_map(|e| {
            let text = fs::read_to_string(e.path()).ok()?;
            let fm = frontmatter(&text);
            if fm.get("tip").map(String::as_str) != Some("rok") {
                return None;
            }
            let get = |k: &str| fm.get(k).cloned().unwrap_or_default();
            Some(Rok {
                file: e.file_name().to_string_lossy().into_owned(),
                title: e.path().file_stem()?.to_string_lossy().into_owned(),
                predmet: get("predmet"),
                vrsta: get("vrsta"),
                datum: get("datum"),
                ura: get("ura"),
                prostor: get("prostor"),
                opravljeno: get("opravljeno") == "true",
                ocena: get("ocena"),
            })
        })
        .collect();
    out.sort_by(|a, b| a.datum.cmp(&b.datum).then_with(|| a.ura.cmp(&b.ura)));
    out
}

pub fn create_rok(root: &Path, r: &NewRok) -> Result<String, String> {
    check_date(&r.datum)?;
    if r.title.trim().is_empty() {
        return Err("A deadline needs a name.".into());
    }
    let dir = root.join("Roki");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = unique_path(&dir, &safe_name(&r.title));
    let predmet = if r.predmet.is_empty() { String::new() } else { format!("\"[[{}]]\"", r.predmet) };
    let snov: String = {
        let lines: Vec<String> = r.snov.lines().map(str::trim).filter(|l| !l.is_empty()).map(|l| format!("- {l}")).collect();
        if lines.is_empty() { "- ".into() } else { lines.join("\n") }
    };
    let vrsta = if r.vrsta.trim().is_empty() { "pomembno" } else { r.vrsta.trim() };
    let content = format!(
        "---\ntip: rok\npredmet: {predmet}\nvrsta: {vrsta}\ndatum: {}\nura: {}\nprostor: {}\nopravljeno: false\nocena: \n---\n# {}\n\n## Snov\n{snov}\n\n## Priprava\n- [ ] Ponovi zapiske predavanj\n- [ ] Reši stare kolokvije / izpite\n- [ ] \n\n## Po roku\n- Kaj je bilo: \n- Kaj bi naslednjič naredil drugače: \n",
        r.datum,
        quoted(&r.ura),
        quoted(&r.prostor),
        r.title.trim(),
    );
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(path.file_name().unwrap().to_string_lossy().into_owned())
}

/// Flips `opravljeno:` in the frontmatter of one Rok note. Returns the new state.
pub fn toggle_rok(root: &Path, file: &str) -> Result<bool, String> {
    if file.contains('/') || file.contains('\\') || file.contains("..") || !file.ends_with(".md") {
        return Err("Invalid file name.".into());
    }
    let path = root.join("Roki").join(file);
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut in_fm = false;
    let mut seen_open = false;
    let mut state = None;
    let mut out = Vec::new();
    for line in text.split('\n') {
        let t = line.trim();
        if t == "---" {
            if !seen_open {
                seen_open = true;
                in_fm = true;
            } else if in_fm {
                in_fm = false;
            }
        } else if in_fm && state.is_none() && line.starts_with("opravljeno:") {
            let now = line["opravljeno:".len()..].trim() != "true";
            state = Some(now);
            out.push(format!("opravljeno: {now}"));
            continue;
        }
        out.push(line.to_string());
    }
    let state = state.ok_or("The note has no `opravljeno` property.")?;
    fs::write(&path, out.join("\n")).map_err(|e| e.to_string())?;
    Ok(state)
}

// ---------- Notes ----------

pub fn save_note(root: &Path, n: &NewNote) -> Result<NoteInfo, String> {
    check_date(&n.datum)?;
    let subject = SUBJECTS.iter().find(|s| s.name == n.predmet).ok_or("Unknown subject.")?;
    if !KINDS.contains(&n.kind.as_str()) {
        return Err("Note kind must be Predavanja or Vaje.".into());
    }
    if n.title.trim().is_empty() {
        return Err("A note needs a title.".into());
    }
    let dir = root.join("Predmeti").join(subject.name).join(&n.kind);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = unique_path(&dir, &safe_name(&n.title));
    let (tip, extra) = if n.kind == "Vaje" { ("vaje", "oddano: false\n") } else { ("predavanje", "") };
    let content = format!(
        "---\ntip: {tip}\npredmet: \"[[{}]]\"\ndatum: {}\nst: \n{extra}tags:\n  - {tip}\n---\n# {}\n\n## Zapiski\n\n{}\n",
        subject.name,
        n.datum,
        n.title.trim(),
        n.body.trim_end(),
    );
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(info(root, &path, subject.name, &n.kind))
}

fn info(root: &Path, path: &Path, predmet: &str, kind: &str) -> NoteInfo {
    let modified = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    NoteInfo {
        predmet: predmet.into(),
        kind: kind.into(),
        title: path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        path: path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned(),
        modified,
    }
}

pub fn recent_notes(root: &Path, limit: usize) -> Vec<NoteInfo> {
    let mut out = Vec::new();
    for s in SUBJECTS.iter() {
        for k in KINDS {
            let dir = root.join("Predmeti").join(s.name).join(k);
            let Ok(rd) = fs::read_dir(dir) else { continue };
            for e in rd.flatten() {
                if e.path().extension().map_or(false, |x| x == "md") {
                    out.push(info(root, &e.path(), s.name, k));
                }
            }
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out.truncate(limit);
    out
}

// ---------- Workspace (Obsidian-style file access) ----------

#[derive(serde::Serialize, Clone, Debug)]
pub struct Entry {
    /// Path relative to the vault root, `/`-separated.
    pub path: String,
    pub is_dir: bool,
    pub modified: u64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct NoteContent {
    pub content: String,
    pub modified: u64,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct SearchHit {
    pub path: String,
    /// 0 = the file name matched; otherwise the 1-based line number.
    pub line_no: usize,
    pub line: String,
}

const MAX_ENTRIES: usize = 20_000;
const MAX_NOTE_BYTES: u64 = 4 * 1024 * 1024;

fn mtime(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

/// Turns a vault-relative path into an absolute one, refusing anything that could escape the vault
/// or touch hidden folders such as `.obsidian` and `.trash`.
pub fn resolve(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = rel.trim().trim_start_matches('/');
    if rel.is_empty() || rel.contains('\\') || rel.contains('\0') {
        return Err("Invalid path.".into());
    }
    let mut out = root.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.starts_with('.') {
            return Err("Invalid path.".into());
        }
        out.push(part);
    }
    Ok(out)
}

fn rel_of(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).to_string_lossy().replace('\\', "/")
}

/// Every visible file and folder in the vault (hidden ones are skipped).
pub fn tree(root: &Path) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let path = e.path();
            let is_dir = path.is_dir();
            out.push(Entry { path: rel_of(root, &path), is_dir, modified: mtime(&path) });
            if is_dir {
                stack.push(path);
            }
            if out.len() >= MAX_ENTRIES {
                return out;
            }
        }
    }
    out
}

fn require_md(rel: &str) -> Result<(), String> {
    if rel.to_lowercase().ends_with(".md") { Ok(()) } else { Err("Only .md notes are supported.".into()) }
}

pub fn read_note(root: &Path, rel: &str) -> Result<NoteContent, String> {
    require_md(rel)?;
    let path = resolve(root, rel)?;
    let meta = fs::metadata(&path).map_err(|_| "Note does not exist.".to_string())?;
    if meta.len() > MAX_NOTE_BYTES {
        return Err("Note is too large to open.".into());
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok(NoteContent { content, modified: mtime(&path) })
}

/// Saves an existing note. Never creates a file: use `create_note` for that.
pub fn write_note(root: &Path, rel: &str, content: &str) -> Result<u64, String> {
    require_md(rel)?;
    let path = resolve(root, rel)?;
    if !path.is_file() {
        return Err("The note no longer exists. Create it again.".into());
    }
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(mtime(&path))
}

fn unique_rel(root: &Path, rel: &str) -> Result<String, String> {
    let path = resolve(root, rel)?;
    if !path.exists() {
        return Ok(rel.trim_start_matches('/').to_string());
    }
    let (stem, ext) = match rel.rsplit_once('.') {
        Some((s, e)) if !e.contains('/') => (s.to_string(), format!(".{e}")),
        _ => (rel.to_string(), String::new()),
    };
    for n in 2..1000 {
        let cand = format!("{stem} {n}{ext}");
        if !resolve(root, &cand)?.exists() {
            return Ok(cand.trim_start_matches('/').to_string());
        }
    }
    Err("Could not find a free name.".into())
}

pub fn create_note(root: &Path, rel: &str, content: &str) -> Result<String, String> {
    let rel = if rel.to_lowercase().ends_with(".md") { rel.to_string() } else { format!("{rel}.md") };
    let rel = unique_rel(root, &rel)?;
    let path = resolve(root, &rel)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content).map_err(|e| e.to_string())?;
    Ok(rel)
}

pub fn create_folder(root: &Path, rel: &str) -> Result<String, String> {
    let rel = unique_rel(root, rel)?;
    fs::create_dir_all(resolve(root, &rel)?).map_err(|e| e.to_string())?;
    Ok(rel)
}

fn md_files(root: &Path) -> Vec<PathBuf> {
    tree(root)
        .into_iter()
        .filter(|e| !e.is_dir && e.path.to_lowercase().ends_with(".md"))
        .map(|e| root.join(e.path))
        .collect()
}

fn stem_of(rel: &str) -> &str {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.strip_suffix(".md").or_else(|| name.strip_suffix(".MD")).unwrap_or(name)
}

/// Replaces `[[old]]`, `[[old|`, `[[old#` with the new name, like Obsidian's "always update links".
fn relink(text: &str, old: &str, new: &str) -> Option<String> {
    let mut out = text.to_string();
    let mut changed = false;
    for tail in ["]]", "|", "#"] {
        let from = format!("[[{old}{tail}");
        if out.contains(&from) {
            out = out.replace(&from, &format!("[[{new}{tail}"));
            changed = true;
        }
    }
    changed.then_some(out)
}

/// Renames or moves a file or folder. For a renamed note, links in other notes are updated.
/// Returns the new relative path and how many other notes were changed.
pub fn rename(root: &Path, from: &str, to: &str) -> Result<(String, usize), String> {
    let src = resolve(root, from)?;
    let to = if src.is_file() && from.to_lowercase().ends_with(".md") && !to.to_lowercase().ends_with(".md") {
        format!("{to}.md")
    } else {
        to.to_string()
    };
    let dst = resolve(root, &to)?;
    if !src.exists() {
        return Err("Source does not exist.".into());
    }
    if dst.exists() {
        return Err("A file with that name already exists.".into());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::rename(&src, &dst).map_err(|e| e.to_string())?;

    let mut updated = 0;
    let (old_stem, new_stem) = (stem_of(from).to_string(), stem_of(&to).to_string());
    if dst.is_file() && old_stem != new_stem && to.to_lowercase().ends_with(".md") {
        for f in md_files(root) {
            let Ok(text) = fs::read_to_string(&f) else { continue };
            if let Some(next) = relink(&text, &old_stem, &new_stem) {
                if fs::write(&f, next).is_ok() {
                    updated += 1;
                }
            }
        }
    }
    Ok((to, updated))
}

/// Moves a file or folder into the vault's `.trash/` (Obsidian's behaviour). Nothing is deleted.
pub fn trash(root: &Path, rel: &str) -> Result<String, String> {
    let src = resolve(root, rel)?;
    if !src.exists() {
        return Err("File does not exist.".into());
    }
    let trash_dir = root.join(".trash");
    fs::create_dir_all(&trash_dir).map_err(|e| e.to_string())?;
    let name = src.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "datoteka".into());
    let mut dst = trash_dir.join(&name);
    let mut n = 2;
    while dst.exists() {
        dst = trash_dir.join(format!("{name} {n}"));
        n += 1;
    }
    fs::rename(&src, &dst).map_err(|e| e.to_string())?;
    Ok(rel_of(root, &dst))
}

fn snippet(line: &str) -> String {
    let t = line.trim();
    if t.chars().count() > 160 { t.chars().take(160).collect::<String>() + "…" } else { t.to_string() }
}

/// Lower-cases and maps Slovenian/Croatian diacritics to ASCII so "racunalnistvo" finds "računalništvo".
fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'č' | 'ć' => 'c',
            'š' => 's',
            'ž' => 'z',
            'đ' => 'd',
            other => other,
        })
        .collect()
}

pub fn search(root: &Path, query: &str, limit: usize) -> Vec<SearchHit> {
    let q = fold(query.trim());
    if q.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for f in md_files(root) {
        let rel = rel_of(root, &f);
        if fold(stem_of(&rel)).contains(&q) {
            hits.push(SearchHit { path: rel.clone(), line_no: 0, line: String::new() });
        }
        let Ok(text) = fs::read_to_string(&f) else { continue };
        for (i, line) in text.lines().enumerate() {
            if fold(line).contains(&q) {
                hits.push(SearchHit { path: rel.clone(), line_no: i + 1, line: snippet(line) });
                if hits.len() >= limit {
                    return hits;
                }
            }
        }
        if hits.len() >= limit {
            break;
        }
    }
    hits
}

/// Lines in other notes that link to the note called `name` (without extension).
pub fn backlinks(root: &Path, rel: &str) -> Vec<SearchHit> {
    let name = stem_of(rel).to_lowercase();
    let pats = [format!("[[{name}]]"), format!("[[{name}|"), format!("[[{name}#")];
    let mut hits = Vec::new();
    for f in md_files(root) {
        let frel = rel_of(root, &f);
        if frel == rel {
            continue;
        }
        let Ok(text) = fs::read_to_string(&f) else { continue };
        for (i, line) in text.lines().enumerate() {
            let low = line.to_lowercase();
            if pats.iter().any(|p| low.contains(p)) {
                hits.push(SearchHit { path: frel.clone(), line_no: i + 1, line: snippet(line) });
            }
        }
    }
    hits
}

// ---------- Obsidian ----------

fn pct(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn obsidian_uri(root: &Path, rel: &str) -> String {
    let vault = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let file = rel.strip_suffix(".md").unwrap_or(rel);
    format!("obsidian://open?vault={}&file={}", pct(&vault), pct(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_and_names() {
        let fm = frontmatter("---\ntip: rok\npredmet: \"[[Matematika 1]]\"\nura: \"10:00\"\nopravljeno: false\ntags:\n  - x\n---\n# T\n");
        assert_eq!(fm["predmet"], "Matematika 1");
        assert_eq!(fm["ura"], "10:00");
        assert_eq!(fm["opravljeno"], "false");
        assert_eq!(safe_name("1. kolokvij: A/B?"), "1. kolokvij- A-B-");
        assert_eq!(obsidian_uri(Path::new("/x/Sola"), "Roki/1. kolokvij.md"), "obsidian://open?vault=Sola&file=Roki%2F1.%20kolokvij");
    }

    #[test]
    fn rok_roundtrip_and_no_overwrite() {
        let dir = std::env::temp_dir().join(format!("feri-vault-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let new = NewRok { title: "1. kolokvij".into(), predmet: "Matematika 1".into(), vrsta: "kolokvij".into(), datum: "2026-11-20".into(), ura: "10:00".into(), prostor: "A-306".into(), snov: "Limite\nOdvodi".into() };
        let f1 = create_rok(&dir, &new).unwrap();
        let f2 = create_rok(&dir, &new).unwrap();
        assert_eq!(f1, "1. kolokvij.md");
        assert_eq!(f2, "1. kolokvij 2.md");
        let roki = list_roki(&dir);
        assert_eq!(roki.len(), 2);
        assert_eq!(roki[0].predmet, "Matematika 1");
        assert_eq!(roki[0].ura, "10:00");
        assert!(!roki[0].opravljeno);
        assert!(toggle_rok(&dir, &f1).unwrap());
        assert!(list_roki(&dir).iter().find(|r| r.file == f1).unwrap().opravljeno);
        assert!(toggle_rok(&dir, "../x.md").is_err());
        let n = NewNote { predmet: "Matematika 1".into(), kind: "Predavanja".into(), title: "P01 – Uvod".into(), body: "Test".into(), datum: "2026-10-06".into() };
        let a = save_note(&dir, &n).unwrap();
        let b = save_note(&dir, &n).unwrap();
        assert_eq!(a.path, "Predmeti/Matematika 1/Predavanja/P01 – Uvod.md");
        assert_eq!(b.title, "P01 – Uvod 2");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn workspace_ops() {
        let dir = std::env::temp_dir().join(format!("feri-ws-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".obsidian")).unwrap();
        // path safety
        assert!(resolve(&dir, "../x.md").is_err());
        assert!(resolve(&dir, "a/../../x.md").is_err());
        assert!(resolve(&dir, ".obsidian/app.json").is_err());
        assert!(resolve(&dir, "a\\b.md").is_err());
        assert!(resolve(&dir, "").is_err());
        assert!(resolve(&dir, "Predmeti/Matematika 1/P01.md").is_ok());
        // create, read, write
        let a = create_note(&dir, "Pojmi/Limita", "# Limita\n").unwrap();
        assert_eq!(a, "Pojmi/Limita.md");
        assert_eq!(create_note(&dir, "Pojmi/Limita", "x").unwrap(), "Pojmi/Limita 2.md");
        create_note(&dir, "Predmeti/P01", "Glej [[Limita]] in [[Limita|lim]] ter [[Limita#Def]].\n").unwrap();
        assert!(write_note(&dir, "Pojmi/Nova.md", "x").is_err(), "write must not create files");
        write_note(&dir, "Pojmi/Limita.md", "# Limita\nDefinicija\n").unwrap();
        assert!(read_note(&dir, "Pojmi/Limita.md").unwrap().content.contains("Definicija"));
        assert!(read_note(&dir, "x.txt").is_err());
        // tree hides dotfolders
        let t = tree(&dir);
        assert!(t.iter().all(|e| !e.path.starts_with('.')));
        assert!(t.iter().any(|e| e.path == "Pojmi" && e.is_dir));
        // search and backlinks
        assert!(!search(&dir, "DEFINICIJA", 50).is_empty());
        create_note(&dir, "Računalništvo", "Šola in čaj\n").unwrap();
        assert!(search(&dir, "racunalnistvo", 50).iter().any(|h| h.path == "Računalništvo.md" && h.line_no == 0), "folded name match");
        assert!(search(&dir, "sola in caj", 50).iter().any(|h| h.path == "Računalništvo.md" && h.line_no == 1), "folded content match");
        assert!(search(&dir, "definicija", 50).iter().any(|h| h.path == "Pojmi/Limita.md" && h.line_no == 2));
        assert_eq!(backlinks(&dir, "Pojmi/Limita.md").len(), 1);
        // rename updates links in other notes
        let (new, updated) = rename(&dir, "Pojmi/Limita.md", "Pojmi/Limes.md").unwrap();
        assert_eq!(new, "Pojmi/Limes.md");
        assert_eq!(updated, 1);
        let p01 = read_note(&dir, "Predmeti/P01.md").unwrap().content;
        assert_eq!(p01, "Glej [[Limes]] in [[Limes|lim]] ter [[Limes#Def]].\n");
        assert!(rename(&dir, "Pojmi/Limes.md", "Pojmi/Limita 2.md").is_err(), "must not overwrite");
        // trash moves, never deletes
        let t = trash(&dir, "Pojmi/Limes.md").unwrap();
        assert!(dir.join(&t).is_file());
        assert!(!dir.join("Pojmi/Limes.md").exists());
        assert!(trash(&dir, "../etc").is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    /// Read-only check against a real vault: `FERI_VAULT=/path cargo test real_vault -- --nocapture`.
    #[test]
    fn real_vault() {
        let Ok(p) = std::env::var("FERI_VAULT") else { return };
        let root = Path::new(&p);
        let t = tree(root);
        let notes: Vec<_> = t.iter().filter(|e| !e.is_dir && e.path.ends_with(".md")).collect();
        println!("{} entries, {} notes", t.len(), notes.len());
        assert!(t.iter().all(|e| !e.path.split('/').any(|c| c.starts_with('.'))), "hidden entries leaked");
        for n in &notes {
            let r = read_note(root, &n.path).unwrap_or_else(|e| panic!("{}: {e}", n.path));
            assert!(!r.content.is_empty() || n.path.contains("Untitled"), "{} is empty", n.path);
        }
        let hits = search(root, "racunalnistvo", 10);
        println!("search 'racunalnistvo': {} hits, backlinks of Urnik: {}", hits.len(), backlinks(root, "Urnik.md").len());
        assert!(!hits.is_empty());
    }
}
