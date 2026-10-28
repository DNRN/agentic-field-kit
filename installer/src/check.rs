//! `--check`: every skill against the Agent Skills portability rules, as
//! `install.sh --check` checks them.

use crate::catalog::is_name;
use crate::source::Source;

/// The report lines, and whether every skill passed.
pub fn run(source: &Source) -> (Vec<String>, bool) {
    let mut out = Vec::new();
    for (name, skill_md) in source.skill_folders() {
        let Some(md) = skill_md else {
            out.push(format!("FAIL {name}: missing SKILL.md"));
            continue;
        };
        if !is_name(&name) {
            out.push(format!(
                "FAIL {name}: folder name must be lowercase alphanumeric with single hyphens"
            ));
        }
        let lines: Vec<&str> = md.lines().map(|l| l.trim_end_matches('\r')).collect();
        let name_ok = lines.iter().any(|l| {
            l.strip_prefix("name:")
                .is_some_and(|v| v.trim_start_matches(' ') == name)
        });
        if !name_ok {
            out.push(format!(
                "FAIL {name}: frontmatter 'name: {name}' missing or mismatched"
            ));
        }
        // install.sh: grep '^description: *..*.' — at least two characters after the colon.
        let description_ok = lines.iter().any(|l| {
            l.strip_prefix("description:")
                .is_some_and(|v| v.chars().count() >= 2)
        });
        if !description_ok {
            out.push(format!("FAIL {name}: empty or missing description"));
        }
    }
    let ok = out.is_empty();
    if ok {
        out.push("all skills OK".into());
    }
    (out, ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn skill(root: &std::path::Path, folder: &str, md: Option<&str>) {
        let dir = root.join("skills").join(folder);
        fs::create_dir_all(&dir).unwrap();
        if let Some(md) = md {
            fs::write(dir.join("SKILL.md"), md).unwrap();
        }
    }

    #[test]
    fn reports_each_rule() {
        let tmp = tempfile::tempdir().unwrap();
        skill(
            tmp.path(),
            "good",
            Some("---\nname: good\ndescription: Use when testing.\n---\n"),
        );
        skill(tmp.path(), "no-md", None);
        skill(
            tmp.path(),
            "Bad_Name",
            Some("---\nname: Bad_Name\ndescription: x y\n---\n"),
        );
        skill(
            tmp.path(),
            "mismatch",
            Some("---\nname: other\ndescription: \n---\n"),
        );
        let (lines, ok) = run(&Source::Dir(tmp.path().to_path_buf()));
        assert!(!ok);
        assert_eq!(
            lines,
            [
                "FAIL Bad_Name: folder name must be lowercase alphanumeric with single hyphens",
                "FAIL mismatch: frontmatter 'name: mismatch' missing or mismatched",
                "FAIL mismatch: empty or missing description",
                "FAIL no-md: missing SKILL.md",
            ]
        );
    }

    #[test]
    fn built_in_skills_pass() {
        let (lines, ok) = run(&Source::Embedded);
        assert!(ok, "{lines:?}");
    }
}
