//! The content hash of ADR 0008 (the same as the bash installer's, before 0.2.0):
//!
//! - a file: sha256 of its bytes with every `\r` removed (`tr -d '\r'`);
//! - a folder: sha256 of the lines `"<file hash> ./<path>\n"` for every regular
//!   file below it except `.agentic-toolkit-install`, paths sorted bytewise
//!   (`find . -type f | LC_ALL=C sort`).

use sha2::{Digest, Sha256};
use std::path::Path;

use crate::source::{Content, File, read_content};

const OLD_MARKER: &str = ".agentic-toolkit-install";

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn file(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    for chunk in bytes.split(|&b| b == b'\r') {
        hasher.update(chunk);
    }
    hex(&hasher.finalize())
}

pub fn dir(files: &[(String, File)]) -> String {
    let mut lines: Vec<(String, &[u8])> = files
        .iter()
        .filter(|(rel, _)| rel.rsplit('/').next() != Some(OLD_MARKER))
        .map(|(rel, f)| (format!("./{rel}"), f.bytes.as_slice()))
        .collect();
    lines.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut hasher = Sha256::new();
    for (path, bytes) in lines {
        hasher.update(file(bytes).as_bytes());
        hasher.update(b" ");
        hasher.update(path.as_bytes());
        hasher.update(b"\n");
    }
    hex(&hasher.finalize())
}

pub fn content(content: &Content) -> String {
    match content {
        Content::File(f) => file(&f.bytes),
        Content::Dir(files) => dir(files),
    }
}

/// The hash of a file or folder on disk; `None` when it can't be read.
pub fn path(path: &Path) -> Option<String> {
    read_content(path).map(|c| content(&c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Expected values computed with the bash installer's hash_of on the same fixtures
    // (bash 5.3, GNU coreutils), so these tests need no bash.

    #[test]
    fn file_ignores_carriage_returns() {
        assert_eq!(file(b"a\r\nb\r\n"), file(b"a\nb\n"));
        assert_eq!(
            file(b"a\nb\n"),
            "911169ddaaf146aff539f58c26c489af3b892dff0fe283c1c264c65ae5aa59a2"
        );
    }

    #[test]
    fn empty_folder() {
        assert_eq!(
            dir(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn folder_matches_install_sh() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("references/deep")).unwrap();
        fs::write(root.join("SKILL.md"), "---\nname: fixture\n---\r\nBody\r\n").unwrap();
        fs::write(root.join("references/rubric.md"), "rubric\n").unwrap();
        fs::write(root.join("references/deep/notes.txt"), "deep\n").unwrap();
        fs::write(root.join("Ünïcode.md"), "ü\n").unwrap();
        fs::write(root.join("a.b"), "dot\n").unwrap();
        fs::write(root.join(OLD_MARKER), "/old/clone\n").unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
        assert_eq!(
            path(root).unwrap(),
            "912ee33a0725814877a3ba516b0f64fc060e8b10ad412be8558020d6eb23d1c3"
        );
    }
}
