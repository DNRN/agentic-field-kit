//! Where the toolkit's components come from (ADR 0010): built into the binary,
//! or a toolkit working tree given with `--source DIR`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
}

/// One file's bytes, and whether it is executable (kept on copy, ignored by the hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub bytes: Vec<u8>,
    pub executable: bool,
}

/// A component's files: one file, or a folder as (`/`-separated relative path, file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    File(File),
    Dir(Vec<(String, File)>),
}

#[derive(Debug, Clone)]
pub enum Source {
    Embedded,
    /// An absolute toolkit tree, as given (not resolved through links), so that
    /// links made into it are recognised by plain path comparison.
    Dir(PathBuf),
}

fn embedded_file(rel: &str) -> Option<File> {
    embedded::FILES
        .iter()
        .find(|(path, _, _)| *path == rel)
        .map(|(_, bytes, executable)| File {
            bytes: bytes.to_vec(),
            executable: *executable,
        })
}

impl Source {
    pub fn dir(&self) -> Option<&Path> {
        match self {
            Source::Embedded => None,
            Source::Dir(dir) => Some(dir),
        }
    }

    fn read(&self, rel: &str) -> io::Result<Vec<u8>> {
        match self {
            Source::Embedded => embedded_file(rel).map(|f| f.bytes).ok_or_else(|| {
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

    /// Where a component lives on disk, for a symlink to point at (`--source` only).
    pub fn link_target(&self, rel: &str) -> Option<PathBuf> {
        self.dir().map(|dir| dir.join(rel))
    }

    /// The component at `rel` (a catalog path), or `None` when it doesn't exist.
    pub fn content(&self, rel: &str) -> Option<Content> {
        match self {
            Source::Embedded => {
                if let Some(file) = embedded_file(rel) {
                    return Some(Content::File(file));
                }
                let prefix = format!("{}/", rel.trim_end_matches('/'));
                let files: Vec<(String, File)> = embedded::FILES
                    .iter()
                    .filter_map(|(path, bytes, executable)| {
                        path.strip_prefix(&prefix).map(|r| {
                            (
                                r.to_owned(),
                                File {
                                    bytes: bytes.to_vec(),
                                    executable: *executable,
                                },
                            )
                        })
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
                for (path, _, _) in embedded::FILES {
                    if let Some((name, _)) = path
                        .strip_prefix("skills/")
                        .and_then(|rest| rest.split_once('/'))
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

#[cfg(unix)]
fn is_executable(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_: &fs::Metadata) -> bool {
    false
}

fn read_file(path: &Path) -> Option<File> {
    let meta = fs::metadata(path).ok()?;
    let bytes = fs::read(path).ok()?;
    Some(File {
        bytes,
        executable: is_executable(&meta),
    })
}

/// A file or folder on disk, read the way the hash sees it: a regular
/// file (following a top-level link), or the regular files below a folder,
/// not descending into or listing symlinks (`find . -type f`).
pub fn read_content(path: &Path) -> Option<Content> {
    let meta = fs::metadata(path).ok()?;
    if meta.is_file() {
        return read_file(path).map(Content::File);
    }
    if !meta.is_dir() {
        return None;
    }
    let mut files = Vec::new();
    walk(path, "", &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Some(Content::Dir(files))
}

fn walk(dir: &Path, prefix: &str, files: &mut Vec<(String, File)>) {
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
            && let Some(file) = read_file(&entry.path())
        {
            files.push((rel, file));
        }
    }
}

fn write_file(dest: &Path, file: &File) -> io::Result<()> {
    fs::write(dest, &file.bytes)?;
    #[cfg(unix)]
    if file.executable {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest)?.permissions();
        perms.set_mode(perms.mode() | ((perms.mode() & 0o444) >> 2));
        fs::set_permissions(dest, perms)?;
    }
    Ok(())
}

/// Writes a component's files at `dest`, which must not exist (`cp -R`).
pub fn write_content(dest: &Path, content: &Content) -> io::Result<()> {
    match content {
        Content::File(file) => write_file(dest, file),
        Content::Dir(files) => {
            fs::create_dir_all(dest)?;
            for (rel, file) in files {
                let path = rel
                    .split('/')
                    .fold(dest.to_path_buf(), |p, part| p.join(part));
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                write_file(&path, file)?;
            }
            Ok(())
        }
    }
}

/// The first `"key": "value"` string in a JSON text, such as `version` in
/// plugin.json. Enough for flat manifests.
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

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

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
        let embedded = Source::Embedded;
        let dir = Source::Dir(repo());
        assert_eq!(embedded.version(), dir.version());
        let catalog = crate::catalog::Catalog::parse(&dir.catalog_text().unwrap());
        for row in &catalog.rows {
            assert_eq!(
                embedded.content(&row.path),
                dir.content(&row.path),
                "{}",
                row.path
            );
        }
    }

    #[test]
    fn written_content_reads_back_the_same() {
        let tmp = tempfile::tempdir().unwrap();
        let src = Source::Embedded;
        let catalog = crate::catalog::Catalog::parse(&src.catalog_text().unwrap());
        for (i, row) in catalog.rows.iter().enumerate() {
            let content = src.content(&row.path).unwrap();
            let dest = tmp.path().join(i.to_string());
            write_content(&dest, &content).unwrap();
            assert_eq!(read_content(&dest), Some(content), "{}", row.path);
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

#[cfg(all(test, unix))]
mod unix_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn copies_keep_the_executable_bit() {
        let tmp = tempfile::tempdir().unwrap();
        // Sorted by path, as read_content returns folders.
        let content = Content::Dir(vec![
            (
                "notes.md".into(),
                File {
                    bytes: b"x\n".to_vec(),
                    executable: false,
                },
            ),
            (
                "run.sh".into(),
                File {
                    bytes: b"#!/bin/sh\n".to_vec(),
                    executable: true,
                },
            ),
        ]);
        let dest = tmp.path().join("skill");
        write_content(&dest, &content).unwrap();
        assert_ne!(
            fs::metadata(dest.join("run.sh"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0
        );
        assert_eq!(
            fs::metadata(dest.join("notes.md"))
                .unwrap()
                .permissions()
                .mode()
                & 0o111,
            0
        );
        assert_eq!(read_content(&dest), Some(content));
    }
}
