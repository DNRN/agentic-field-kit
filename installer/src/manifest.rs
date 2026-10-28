//! The manifest of ADR 0008, `<scope root>/.agents/agentic-toolkit.tsv`:
//! kind, name, version, hash, reason, paths. Read tolerantly: a trailing `\r`
//! (a CRLF checkout) is ignored, and only paths the installer writes for a
//! row's name count, so a committed manifest can't point anywhere else.

use std::fs;
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
