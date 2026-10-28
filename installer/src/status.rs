//! What is installed where: the status values of the selective-install plan,
//! computed exactly as `install.sh` does.

use std::fs;
use std::path::{Path, PathBuf};

use crate::catalog::{Catalog, Component};
use crate::hash;
use crate::manifest;
use crate::scope::{Scope, plugin_on};
use crate::source::Source;

pub struct Ctx {
    pub source: Source,
    pub catalog: Catalog,
    pub version: String,
    /// The project root, when there is one.
    pub root: Option<PathBuf>,
    pub home: PathBuf,
}

/// Global skill targets, as paths relative to the home directory.
pub const GLOBAL_TARGETS: [&str; 3] = [".agents/skills", ".claude/skills", ".omp/agent/skills"];

pub fn is_ours(status: &str) -> bool {
    matches!(
        status,
        "installed" | "linked" | "outdated" | "modified" | "missing" | "dangling" | "retired"
    )
}

fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

impl Ctx {
    pub fn scope_root(&self, scope: Scope) -> Option<&Path> {
        match scope {
            Scope::Project => self.root.as_deref(),
            Scope::Global => Some(&self.home),
        }
    }

    /// Every path the component may occupy in a scope.
    pub fn candidates(&self, scope: Scope, name: &str) -> Vec<String> {
        let Some(c) = self.catalog.get(name) else {
            return Vec::new();
        };
        match (scope, c.kind.as_str()) {
            (Scope::Project, "skill") => vec![
                format!(".agents/skills/{name}"),
                format!(".claude/skills/{name}"),
            ],
            (Scope::Project, "agent") => vec![format!(".claude/agents/{name}.md")],
            (Scope::Global, "skill") => GLOBAL_TARGETS
                .iter()
                .map(|t| format!("{t}/{name}"))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Empty when installable in the scope (or unknown), else `plugin` or `n/a`.
    pub fn avail(&self, scope: Scope, name: &str) -> &'static str {
        let Some(c) = self.catalog.get(name) else {
            return "";
        };
        match (scope, c.kind.as_str()) {
            (_, "skill") => "",
            (Scope::Project, "agent") => {
                if self.root.as_deref().is_some_and(plugin_on) {
                    "plugin"
                } else {
                    ""
                }
            }
            _ => match c.harnesses.as_str() {
                "claude" | "plugin" if self.scope_root(scope).is_some_and(plugin_on) => "plugin",
                _ => "n/a",
            },
        }
    }

    /// Installed before the manifest existed: a symlink into the toolkit's
    /// `skills/`, or a copy carrying the old marker file.
    fn legacy_ours(&self, path: &Path) -> bool {
        match fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                let Ok(target) = fs::read_link(path) else {
                    return false;
                };
                match self.source.dir() {
                    Some(dir) => {
                        let skills = dunce::simplified(&dir.join("skills")).to_path_buf();
                        let target = dunce::simplified(&target).to_path_buf();
                        target.starts_with(&skills) && target != skills
                    }
                    // Built-in components: a link into any agentic-toolkit clone's skills/.
                    None => target
                        .parent()
                        .filter(|p| p.file_name().is_some_and(|n| n == "skills"))
                        .and_then(Path::parent)
                        .is_some_and(|clone| clone.join("catalog.tsv").is_file()),
                }
            }
            Ok(_) => path.join(".agentic-toolkit-install").is_file(),
            Err(_) => false,
        }
    }

    /// Paths that are ours: the manifest's, then legacy ones.
    pub fn owned(&self, scope: Scope, name: &str) -> Vec<String> {
        let Some(root) = self.scope_root(scope) else {
            return Vec::new();
        };
        let mut paths = manifest::row(root, name)
            .map(|r| r.paths())
            .unwrap_or_default();
        for p in self.candidates(scope, name) {
            if !paths.contains(&p) && exists(&root.join(&p)) && self.legacy_ours(&root.join(&p)) {
                paths.push(p);
            }
        }
        paths
    }

    pub fn component(&self, name: &str) -> Option<&Component> {
        self.catalog.get(name)
    }

    pub fn source_hash(&self, name: &str) -> Option<String> {
        let c = self.catalog.get(name)?;
        self.source
            .content(&c.path)
            .map(|content| hash::content(&content))
    }

    /// One status word.
    pub fn status(&self, scope: Scope, name: &str) -> String {
        let Some(root) = self.scope_root(scope) else {
            return "n/a".into();
        };
        let row = manifest::row(root, name).filter(|r| !r.paths().is_empty());
        let (mhash, paths, legacy) = match row {
            Some(r) => (r.hash.clone(), r.paths(), false),
            None => {
                let paths = self.owned(scope, name);
                if paths.is_empty() {
                    if self
                        .candidates(scope, name)
                        .iter()
                        .any(|p| exists(&root.join(p)))
                    {
                        return "foreign".into();
                    }
                    let av = self.avail(scope, name);
                    return if av.is_empty() { "-".into() } else { av.into() };
                }
                (self.source_hash(name).unwrap_or_default(), paths, true)
            }
        };
        let (mut links, mut missing, mut dangling, mut modified) = (0, false, false, false);
        for p in &paths {
            let full = root.join(p);
            match fs::symlink_metadata(&full) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    links += 1;
                    dangling |= !full.exists();
                }
                Ok(_) => modified |= hash::path(&full).as_deref() != Some(mhash.as_str()),
                Err(_) => missing = true,
            }
        }
        // An edit anywhere wins: nothing may overwrite or delete it without --force.
        let s = if modified {
            "modified"
        } else if self.catalog.get(name).is_none() {
            "retired"
        } else if missing {
            "missing"
        } else if dangling {
            "dangling"
        } else if links == paths.len() {
            "linked"
        } else if legacy || self.source_hash(name).as_deref() == Some(mhash.as_str()) {
            "installed"
        } else {
            "outdated"
        };
        s.into()
    }
}
