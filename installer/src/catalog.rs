//! `catalog.tsv` (ADR 0007): kind, name, path, harnesses, requires, summary.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub kind: String,
    pub name: String,
    pub path: String,
    pub harnesses: String,
    /// Names this component can't work without; empty for `-`.
    pub requires: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub rows: Vec<Component>,
}

impl Catalog {
    /// Rows in file order; the header, comments, and blank lines are skipped.
    pub fn parse(text: &str) -> Catalog {
        let rows = text
            .lines()
            .filter_map(|line| {
                let f: Vec<&str> = line
                    .trim_end_matches('\r')
                    .split('\t')
                    .filter(|s| !s.is_empty())
                    .collect();
                let kind = *f.first()?;
                if kind.starts_with('#') || kind == "kind" {
                    return None;
                }
                let field = |i: usize| f.get(i).copied().unwrap_or_default().to_owned();
                let requires = match f.get(4).copied() {
                    None | Some("-") => Vec::new(),
                    Some(list) => list
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect(),
                };
                Some(Component {
                    kind: kind.to_owned(),
                    name: field(1),
                    path: field(2),
                    harnesses: field(3),
                    requires,
                    summary: field(5),
                })
            })
            .collect();
        Catalog { rows }
    }

    pub fn get(&self, name: &str) -> Option<&Component> {
        self.rows.iter().find(|c| c.name == name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.rows.iter().map(|c| c.name.as_str())
    }
}

/// Kebab-case: `^[a-z0-9]+(-[a-z0-9]+)*$`.
pub fn is_name(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !s.contains("--")
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rows_and_requires() {
        let c = Catalog::parse(
            "kind\tname\tpath\tharnesses\trequires\tsummary\n\
             skill\tcode-review\tskills/code-review\tall\t-\tReview\r\n\
             # comment\n\n\
             hook\tcommit-gate\thooks/scripts/commit-gate.sh\tplugin\tcode-review,doc-sync\tGate\n",
        );
        assert_eq!(c.rows.len(), 2);
        assert!(c.get("code-review").unwrap().requires.is_empty());
        assert_eq!(c.get("code-review").unwrap().summary, "Review");
        assert_eq!(
            c.get("commit-gate").unwrap().requires,
            ["code-review", "doc-sync"]
        );
    }

    #[test]
    fn kebab_names() {
        for ok in ["a", "code-review", "a1-b2"] {
            assert!(is_name(ok), "{ok}");
        }
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "../x", "a b"] {
            assert!(!is_name(bad), "{bad}");
        }
    }
}
