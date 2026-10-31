//! The two scopes (ADR 0006): the project, the git root of the working
//! directory or `--project DIR`; and global, the home directory.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::source::{Source, json_string_field};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Project,
    Global,
}

/// The home directory: `$HOME`, else `%USERPROFILE%`.
pub fn home() -> PathBuf {
    env::var_os("HOME")
        .filter(|h| !h.is_empty())
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn real(path: &Path) -> Option<PathBuf> {
    dunce::canonicalize(path).ok()
}

/// `path` with `.` dropped and each `..` removing the component before it,
/// without following links: what bash's logical `cd DIR && pwd` gives.
pub fn normalize(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.file_name().is_some() {
                    out.pop();
                } else if !out.has_root() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The nearest folder at or above `dir` with a `.git` entry (folder or file).
fn git_root(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .find(|d| d.join(".git").exists())
        .map(Path::to_path_buf)
}

/// A folder that looks like an afk tree: a catalog and a plugin
/// manifest named afk, or agentic-toolkit before the rename (ADR 0018).
pub fn is_toolkit_tree(dir: &Path) -> bool {
    dir.join("catalog.tsv").is_file()
        && fs::read_to_string(dir.join(".claude-plugin/plugin.json"))
            .ok()
            .and_then(|t| json_string_field(&t, "name"))
            .is_some_and(|n| PLUGIN_NAMES.contains(&n.as_str()))
}

/// The start of the reason there is no project when the working directory is
/// outside a git repository; callers add how to start a new project.
pub const NO_REPO: &str = "not inside a git repository";

/// The project root, or why there is none.
pub fn project(project_arg: Option<&Path>, source: &Source) -> Result<PathBuf, String> {
    let root = match project_arg {
        Some(dir) => real(dir)
            .filter(|d| d.is_dir())
            .ok_or_else(|| format!("no such directory: {}", dir.display()))?,
        None => {
            let cwd = env::current_dir().ok().and_then(|d| real(&d));
            match cwd.as_deref().and_then(git_root) {
                Some(root) => root,
                None => {
                    return Err(format!(
                        "{NO_REPO}: run afk from your project, or use --project DIR or --global"
                    ));
                }
            }
        }
    };
    let is_source = match source.dir() {
        Some(dir) => real(dir).is_some_and(|d| d == root),
        None => is_toolkit_tree(&root),
    };
    if is_source {
        return Err(
            "this is the afk clone, not a project: run afk from your project, or use --global"
                .into(),
        );
    }
    if real(&home()).is_some_and(|h| h == root) {
        return Err("the project would be your home directory, which is the global scope: use --global, or --project DIR".into());
    }
    Ok(root)
}

/// The plugin's name, then its name before the rename (ADR 0018).
const PLUGIN_NAMES: [&str; 2] = ["afk", "agentic-toolkit"];

/// Whether `root/.claude/settings.json` enables the afk plugin
/// (`"afk@<marketplace>": true`, or the old `"agentic-toolkit@…"`).
pub fn plugin_on(root: &Path) -> bool {
    let Ok(text) = fs::read_to_string(root.join(".claude").join("settings.json")) else {
        return false;
    };
    PLUGIN_NAMES.iter().any(|n| enabled_in(&text, n))
}

fn enabled_in(text: &str, plugin: &str) -> bool {
    let key = format!("\"{plugin}@");
    let mut rest = text;
    while let Some(at) = rest.find(&key) {
        let after = &rest[at + 1..];
        if let Some(end) = after.find('"') {
            let tail = after[end + 1..].trim_start();
            if let Some(value) = tail.strip_prefix(':')
                && value.trim_start().starts_with("true")
            {
                return true;
            }
        }
        rest = &rest[at + 1..];
    }
    false
}

/// Harnesses found on this machine: their command on PATH or their home folder.
pub fn detected_harnesses(home: &Path) -> Vec<&'static str> {
    const HARNESSES: [(&str, &str, &str); 6] = [
        ("claude", "claude", ".claude"),
        ("codex", "codex", ".codex"),
        ("gemini", "gemini", ".gemini"),
        ("opencode", "opencode", ".config/opencode"),
        ("omp", "omp", ".omp"),
        ("cursor", "cursor-agent", ".cursor"),
    ];
    HARNESSES
        .iter()
        .filter(|(_, cli, dir)| on_path(cli) || home.join(dir).is_dir())
        .map(|(name, _, _)| *name)
        .collect()
}

pub fn on_path(cli: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let exts: Vec<String> = if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(str::to_owned)
            .chain(std::iter::once(String::new()))
            .collect()
    } else {
        vec![String::new()]
    };
    env::split_paths(&path).any(|dir| {
        exts.iter()
            .any(|ext| dir.join(format!("{cli}{ext}")).is_file())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_setting() {
        let tmp = tempfile::tempdir().unwrap();
        let settings = tmp.path().join(".claude");
        fs::create_dir_all(&settings).unwrap();
        assert!(!plugin_on(tmp.path()));
        fs::write(
            settings.join("settings.json"),
            r#"{"enabledPlugins": {"afk@afk" : true}}"#,
        )
        .unwrap();
        assert!(plugin_on(tmp.path()));
        fs::write(
            settings.join("settings.json"),
            r#"{"enabledPlugins": {"afk@afk": false}}"#,
        )
        .unwrap();
        assert!(!plugin_on(tmp.path()));
        fs::write(
            settings.join("settings.json"),
            r#"{"enabledPlugins": {"afk@afk": false, "agentic-toolkit@agentic-toolkit": true}}"#,
        )
        .unwrap();
        assert!(plugin_on(tmp.path()), "the name before the rename counts");
    }

    #[test]
    fn git_root_from_a_subfolder() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".git")).unwrap();
        fs::create_dir_all(tmp.path().join("a/b")).unwrap();
        assert_eq!(
            git_root(&tmp.path().join("a/b")).as_deref(),
            Some(tmp.path())
        );
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    #[test]
    fn resolves_dots_lexically() {
        assert_eq!(
            normalize(Path::new("/a/b/../../c/./d")),
            PathBuf::from("/c/d")
        );
        assert_eq!(normalize(Path::new("/..")), PathBuf::from("/"));
        assert_eq!(normalize(Path::new("../x")), PathBuf::from("../x"));
    }
}
