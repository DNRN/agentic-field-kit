//! Where the toolkit's components come from (ADR 0010): built into the binary,
//! or a toolkit working tree given with `--source DIR`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

/// A component's files: one file, or a folder as (`/`-separated relative path, bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    File(Vec<u8>),
    Dir(Vec<(String, Vec<u8>)>),
}

#[derive(Debug, Clone)]
pub enum Source {
    Embedded,
    Dir(PathBuf),
}

impl Source {
    /// The toolkit tree given with `--source`, as given (not resolved), so that
    /// links made into it compare the way `install.sh`'s did.
    pub fn dir(&self) -> Option<&Path> {
        match self {
            Source::Embedded => None,
            Source::Dir(dir) => Some(dir),
        }
    }

    fn read(&self, rel: &str) -> io::Result<Vec<u8>> {
        match self {
            Source::Embedded => embedded::FILES
                .iter()
                .find(|(path, _)| *path == rel)
                .map(|(_, bytes)| bytes.to_vec())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, format!("{rel} is not built in"))
                }),
            Source::Dir(dir) => fs::read(dir.join(rel)),
        }
    }

    pub fn catalog_text(&self) -> io::Result<String> {
        let bytes = self.read("catalog.tsv")?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// The toolkit release, from `.claude-plugin/plugin.json` (empty when absent).
    pub fn version(&self) -> String {
        let text = self
            .read(".claude-plugin/plugin.json")
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        json_string_field(&text, "version").unwrap_or_default()
    }

    /// The component at `rel` (a catalog path), or `None` when it doesn't exist.
    pub fn content(&self, rel: &str) -> Option<Content> {
        match self {
            Source::Embedded => {
                if let Some((_, bytes)) = embedded::FILES.iter().find(|(path, _)| *path == rel) {
                    return Some(Content::File(bytes.to_vec()));
                }
                let prefix = format!("{}/", rel.trim_end_matches('/'));
                let files: Vec<(String, Vec<u8>)> = embedded::FILES
                    .iter()
                    .filter_map(|(path, bytes)| {
                        path.strip_prefix(&prefix)
                            .map(|r| (r.to_owned(), bytes.to_vec()))
                    })
                    .collect();
                if files.is_empty() {
                    None
                } else {
                    Some(Content::Dir(files))
                }
            }
            Source::Dir(dir) => read_content(&dir.join(rel)),
        }
    }

    /// Folders directly under `skills/`, with their `SKILL.md` if present.
    pub fn skill_folders(&self) -> Vec<(String, Option<String>)> {
        let mut out: Vec<(String, Option<String>)> = Vec::new();
        match self {
            Source::Embedded => {
                for (path, _) in embedded::FILES {
                    if let Some(rest) = path.strip_prefix("skills/")
                        && let Some((name, _)) = rest.split_once('/')
                        && !out.iter().any(|(n, _)| n == name)
                    {
                        let md = self.read(&format!("skills/{name}/SKILL.md")).ok();
                        out.push((
                            name.to_owned(),
                            md.map(|b| String::from_utf8_lossy(&b).into_owned()),
                        ));
                    }
                }
            }
            Source::Dir(dir) => {
                if let Ok(entries) = fs::read_dir(dir.join("skills")) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            let name = entry.file_name().to_string_lossy().into_owned();
                            let md = fs::read_to_string(entry.path().join("SKILL.md")).ok();
                            out.push((name, md));
                        }
                    }
                }
            }
        }
        out.sort();
        out
    }
}

/// A file or folder on disk, read the way `install.sh` hashes it: a regular
/// file (following a top-level link), or the regular files below a folder,
/// not descending into or listing symlinks (`find . -type f`).
pub fn read_content(path: &Path) -> Option<Content> {
    let meta = fs::metadata(path).ok()?;
    if meta.is_file() {
        return fs::read(path).ok().map(Content::File);
    }
    if !meta.is_dir() {
        return None;
    }
    let mut files = Vec::new();
    walk(path, "", &mut files);
    Some(Content::Dir(files))
}

fn walk(dir: &Path, prefix: &str, files: &mut Vec<(String, Vec<u8>)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let name = entry.file_name().to_string_lossy().into_owned();
        let rel = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        if kind.is_dir() {
            walk(&entry.path(), &rel, files);
        } else if kind.is_file()
            && let Ok(bytes) = fs::read(entry.path())
        {
            files.push((rel, bytes));
        }
    }
}

/// The first `"key": "value"` string in a JSON text, the way install.sh's sed
/// reads `version` from plugin.json. Enough for flat manifests.
pub fn json_string_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let mut rest = text;
    while let Some(at) = rest.find(&needle) {
        let after = rest[at + needle.len()..].trim_start();
        if let Some(value) = after.strip_prefix(':') {
            let value = value.trim_start();
            if let Some(value) = value.strip_prefix('"') {
                return value.find('"').map(|end| value[..end].to_owned());
            }
        }
        rest = &rest[at + needle.len()..];
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_has_the_catalog_and_every_component() {
        let src = Source::Embedded;
        let catalog = crate::catalog::Catalog::parse(&src.catalog_text().unwrap());
        assert!(!catalog.rows.is_empty());
        for row in &catalog.rows {
            assert!(
                src.content(&row.path).is_some(),
                "{} is not built in",
                row.path
            );
        }
    }

    #[test]
    fn embedded_matches_the_working_tree() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let embedded = Source::Embedded;
        let dir = Source::Dir(repo);
        assert_eq!(embedded.version(), dir.version());
        let catalog = crate::catalog::Catalog::parse(&dir.catalog_text().unwrap());
        for row in &catalog.rows {
            let a = embedded
                .content(&row.path)
                .map(|c| crate::hash::content(&c));
            let b = dir.content(&row.path).map(|c| crate::hash::content(&c));
            assert_eq!(a, b, "{}", row.path);
        }
    }

    #[test]
    fn reads_the_version_field() {
        assert_eq!(
            json_string_field(r#"{ "name": "x", "version" : "1.2.3" }"#, "version").as_deref(),
            Some("1.2.3")
        );
        assert_eq!(json_string_field(r#"{ "name": "x" }"#, "version"), None);
    }
}
