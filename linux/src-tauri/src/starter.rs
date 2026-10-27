//! The empty starter vault: the folder layout, templates and index notes, with no personal notes.
//! It is compiled into the app. Files that already exist are never touched.

use std::fs;
use std::path::Path;

macro_rules! file {
    ($p:literal) => {
        ($p, include_str!(concat!("../starter-vault/", $p)))
    };
}

const FILES: &[(&str, &str)] = &[
    file!("🏠 Domov.md"),
    file!("Urnik.md"),
    file!("Roki.base"),
    file!("Zapiski predmeta.base"),
    file!("Pojmi/Pojmi.md"),
    file!("Pojmi/Pojmi.base"),
    file!("Predloge/Pojem.md"),
    file!("Predloge/Predavanje.md"),
    file!("Predloge/Priprava na izpit.md"),
    file!("Predloge/Rok.md"),
    file!("Predloge/Vaje.md"),
    file!("Predmeti/Matematika 1/Matematika 1.md"),
    file!("Predmeti/Programiranje 1/Programiranje 1.md"),
    file!("Predmeti/Sodobna programska orodja/Sodobna programska orodja.md"),
    file!("Predmeti/Uvod v računalništvo/Uvod v računalništvo.md"),
    file!("Predmeti/Uvod v svetovni splet/Uvod v svetovni splet.md"),
];

/// Writes the starter vault into `root` (created if missing). Returns how many files were written;
/// files that exist already are skipped, so this is safe on a vault that is in use.
pub fn create(root: &Path) -> Result<usize, String> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    for dir in ["Roki", "Priloge"] {
        fs::create_dir_all(root.join(dir)).map_err(|e| e.to_string())?;
    }
    let mut written = 0;
    for (rel, text) in FILES {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            // Every subject gets its lecture and exercise folders.
            if rel.starts_with("Predmeti/") {
                for kind in ["Predavanja", "Vaje"] {
                    fs::create_dir_all(parent.join(kind)).map_err(|e| e.to_string())?;
                }
            }
        }
        if !path.exists() {
            fs::write(&path, text).map_err(|e| e.to_string())?;
            written += 1;
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_the_layout_and_never_overwrites() {
        let dir = std::env::temp_dir().join(format!("feri-starter-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(create(&dir).unwrap(), FILES.len());
        assert!(dir.join("Predmeti/Matematika 1/Predavanja").is_dir());
        assert!(dir.join("Predmeti/Uvod v svetovni splet/Vaje").is_dir());
        assert!(dir.join("Roki").is_dir() && dir.join("Priloge").is_dir());
        assert!(fs::read_dir(dir.join("Roki")).unwrap().next().is_none(), "no deadlines are shipped");

        fs::write(dir.join("Urnik.md"), "mine").unwrap();
        assert_eq!(create(&dir).unwrap(), 0, "second run writes nothing");
        assert_eq!(fs::read_to_string(dir.join("Urnik.md")).unwrap(), "mine");
        let _ = fs::remove_dir_all(&dir);
    }
}
