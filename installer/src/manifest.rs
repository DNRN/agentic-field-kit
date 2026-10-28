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

/// Every data row, in file order.
pub fn rows(root: &Path) -> Vec<Row> {
    let Ok(text) = fs::read_to_string(file(root)) else {
        return Vec::new();
    };
    text.lines()
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f[0] == "kind" {
                return None;
            }
            let field = |i: usize| f.get(i).copied().unwrap_or_default().to_owned();
            Some(Row {
                kind: field(0),
                name: field(1),
                version: field(2),
                hash: field(3),
                reason: field(4),
                raw_paths: field(5),
            })
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

const HEADER: &str = "kind\tname\tversion\thash\treason\tpaths";

/// Every line except the header and the rows for `name`, as written (minus `\r`).
fn other_lines(root: &Path, name: &str) -> Vec<String> {
    let Ok(text) = fs::read_to_string(file(root)) else {
        return Vec::new();
    };
    text.lines()
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let mut f = line.split('\t');
            f.next() != Some("kind") && f.next() != Some(name)
        })
        .map(str::to_owned)
        .collect()
}

fn write_lines(root: &Path, lines: &[String]) -> io::Result<()> {
    let path = file(root);
    fs::create_dir_all(path.parent().unwrap())?;
    let mut text = String::from(HEADER);
    text.push('\n');
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    let tmp = path.with_extension("tsv.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, &path)
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
    let mut lines = other_lines(root, name);
    lines.push(format!(
        "{kind}\t{name}\t{version}\t{hash}\t{reason}\t{}",
        paths.join(" ")
    ));
    let key = |l: &String| l.split('\t').nth(1).unwrap_or_default().to_owned();
    lines.sort_by(|a, b| {
        key(a)
            .as_bytes()
            .cmp(key(b).as_bytes())
            .then_with(|| a.as_bytes().cmp(b.as_bytes()))
    });
    write_lines(root, &lines)
}

/// Removes `name`'s row; deletes the manifest when no row is left.
pub fn drop(root: &Path, name: &str) -> io::Result<()> {
    if !file(root).exists() {
        return Ok(());
    }
    let lines = other_lines(root, name);
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
