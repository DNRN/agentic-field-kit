//! The manifest of ADR 0008, `<scope root>/.agents/agentic-toolkit.tsv`:
//! kind, name, version, hash, reason, paths. Read tolerantly: a trailing `\r`
//! (a CRLF checkout) is ignored, and only paths the installer writes for a
//! row's name count, so a committed manifest can't point anywhere else.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::catalog::is_name;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub kind: String,
    pub name: String,
    pub version: String,
    pub hash: String,
    pub reason: String,
    /// The paths column as written, before filtering.
    pub raw_paths: String,
}

impl Row {
    /// Paths this row may claim: the component's own folders or file.
    pub fn paths(&self) -> Vec<String> {
        let n = &self.name;
        let allowed = [
            format!(".agents/skills/{n}"),
            format!(".claude/skills/{n}"),
            format!(".omp/agent/skills/{n}"),
            format!(".claude/agents/{n}.md"),
            crate::githook::copy_path(n),
        ];
        self.raw_paths
            .split_whitespace()
            .filter(|p| allowed.iter().any(|a| a == p))
            .map(str::to_owned)
            .collect()
    }
}

pub fn file(root: &Path) -> PathBuf {
    root.join(".agents").join("agentic-toolkit.tsv")
}

/// The manifest's non-empty lines as bytes, without `\n` or a trailing `\r`.
/// Bytes, not text: every row survives a rewrite even
/// when they aren't UTF-8. A missing manifest has no lines.
fn raw_lines(root: &Path) -> io::Result<Vec<Vec<u8>>> {
    match fs::read(file(root)) {
        Ok(bytes) => Ok(bytes
            .split(|&b| b == b'\n')
            .map(|l| l.strip_suffix(b"\r").unwrap_or(l))
            .filter(|l| !l.is_empty())
            .map(<[u8]>::to_vec)
            .collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(io::Error::new(
            e.kind(),
            format!("{}: {e}", file(root).display()),
        )),
    }
}

fn field(line: &[u8], i: usize) -> &[u8] {
    line.split(|&b| b == b'\t').nth(i).unwrap_or_default()
}

/// Every data row, in file order. An unreadable manifest has none.
pub fn rows(root: &Path) -> Vec<Row> {
    let text = |l: &[u8], i: usize| String::from_utf8_lossy(field(l, i)).into_owned();
    raw_lines(root)
        .unwrap_or_default()
        .iter()
        .filter(|l| field(l, 0) != b"kind")
        .map(|l| Row {
            kind: text(l, 0),
            name: text(l, 1),
            version: text(l, 2),
            hash: text(l, 3),
            reason: text(l, 4),
            raw_paths: text(l, 5),
        })
        .collect()
}

/// The first row for `name`.
pub fn row(root: &Path, name: &str) -> Option<Row> {
    rows(root).into_iter().find(|r| r.name == name)
}

/// Names in the manifest that are valid component names.
pub fn names(root: &Path) -> Vec<String> {
    rows(root)
        .into_iter()
        .map(|r| r.name)
        .filter(|n| is_name(n))
        .collect()
}

const HEADER: &[u8] = b"kind\tname\tversion\thash\treason\tpaths";

/// Every line except the header and the rows for `name`, as written.
fn other_lines(root: &Path, name: &str) -> io::Result<Vec<Vec<u8>>> {
    Ok(raw_lines(root)?
        .into_iter()
        .filter(|l| field(l, 0) != b"kind" && field(l, 1) != name.as_bytes())
        .collect())
}

fn write_lines(root: &Path, lines: &[Vec<u8>]) -> io::Result<()> {
    let path = file(root);
    fs::create_dir_all(path.parent().unwrap())?;
    let mut bytes = HEADER.to_vec();
    bytes.push(b'\n');
    for line in lines {
        bytes.extend_from_slice(line);
        bytes.push(b'\n');
    }
    let tmp = path.with_extension("tsv.tmp");
    let named = |e: io::Error| io::Error::new(e.kind(), format!("{}: {e}", path.display()));
    fs::write(&tmp, bytes).map_err(named)?;
    fs::rename(&tmp, &path).map_err(named)
}

/// Replaces `name`'s row. Rows stay sorted by name (`LC_ALL=C sort -t TAB -k2,2`).
pub fn put(
    root: &Path,
    kind: &str,
    name: &str,
    version: &str,
    hash: &str,
    reason: &str,
    paths: &[String],
) -> io::Result<()> {
    let mut lines = other_lines(root, name)?;
    lines.push(
        format!(
            "{kind}\t{name}\t{version}\t{hash}\t{reason}\t{}",
            paths.join(" ")
        )
        .into_bytes(),
    );
    lines.sort_by(|a, b| field(a, 1).cmp(field(b, 1)).then_with(|| a.cmp(b)));
    write_lines(root, &lines)
}

/// Removes `name`'s row; deletes the manifest when no row is left.
pub fn drop(root: &Path, name: &str) -> io::Result<()> {
    if !file(root).exists() {
        return Ok(());
    }
    let lines = other_lines(root, name)?;
    if lines.is_empty() {
        return fs::remove_file(file(root));
    }
    write_lines(root, &lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, text: &str) {
        fs::create_dir_all(root.join(".agents")).unwrap();
        fs::write(file(root), text).unwrap();
    }

    #[test]
    fn reads_crlf_rows() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            "kind\tname\tversion\thash\treason\tpaths\r\n\
             skill\tcode-review\t0.2.0\tabc\texplicit\t.agents/skills/code-review .claude/skills/code-review\r\n",
        );
        let r = row(tmp.path(), "code-review").unwrap();
        assert_eq!(r.reason, "explicit");
        assert_eq!(
            r.paths(),
            [".agents/skills/code-review", ".claude/skills/code-review"]
        );
    }

    #[test]
    fn paths_outside_the_component_are_ignored() {
        let r = Row {
            kind: "skill".into(),
            name: "code-review".into(),
            version: String::new(),
            hash: String::new(),
            reason: "explicit".into(),
            raw_paths: ".agents/skills/code-review ../victim /etc .agents/skills/other .claude/agents/code-review.md".into(),
        };
        assert_eq!(
            r.paths(),
            [
                ".agents/skills/code-review",
                ".claude/agents/code-review.md"
            ]
        );
    }

    #[test]
    fn names_skip_invalid_ones() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "kind\tname\nskill\tok-name\nskill\t../evil\n");
        assert_eq!(names(tmp.path()), ["ok-name"]);
    }
}

#[cfg(test)]
mod write_tests {
    use super::*;

    #[test]
    fn put_keeps_rows_sorted_and_other_lines_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put(
            root,
            "skill",
            "doc-sync",
            "0.1.0",
            "h1",
            "explicit",
            &[".agents/skills/doc-sync".into()],
        )
        .unwrap();
        put(
            root,
            "skill",
            "adr-planning",
            "0.1.0",
            "h2",
            "dependency",
            &[".agents/skills/adr-planning".into()],
        )
        .unwrap();
        put(
            root,
            "skill",
            "doc-sync",
            "0.2.0",
            "h3",
            "explicit",
            &[".agents/skills/doc-sync".into()],
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(file(root)).unwrap(),
            "kind\tname\tversion\thash\treason\tpaths\n\
             skill\tadr-planning\t0.1.0\th2\tdependency\t.agents/skills/adr-planning\n\
             skill\tdoc-sync\t0.2.0\th3\texplicit\t.agents/skills/doc-sync\n"
        );
        drop(root, "doc-sync").unwrap();
        drop(root, "adr-planning").unwrap();
        assert!(!file(root).exists(), "an empty manifest is deleted");
    }
}

#[cfg(test)]
mod byte_tests {
    use super::*;

    #[test]
    fn a_line_that_is_not_utf8_survives_a_rewrite() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        put(
            root,
            "skill",
            "code-review",
            "0.1.0",
            "h",
            "explicit",
            &[".agents/skills/code-review".into()],
        )
        .unwrap();
        let mut bytes = fs::read(file(root)).unwrap();
        bytes.extend_from_slice(b"# caf\xe9\n");
        fs::write(file(root), &bytes).unwrap();
        put(
            root,
            "skill",
            "adr-planning",
            "0.1.0",
            "h",
            "explicit",
            &[".agents/skills/adr-planning".into()],
        )
        .unwrap();
        let after = fs::read(file(root)).unwrap();
        assert!(after.windows(6).any(|w| w == b"# caf\xe9"));
        assert!(row(root, "code-review").is_some() && row(root, "adr-planning").is_some());
    }
}
