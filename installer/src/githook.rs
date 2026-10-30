//! The per-clone half of a `hook` component (ADR 0014): a shim at git's
//! `pre-commit` path that runs the committed copy at `.agents/hooks/<name>.sh`.
//! The manifest records the copy, never the shim. A `pre-commit` hook that
//! agentic-toolkit didn't write is never changed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where a `hook` component's copy lives in the project.
pub fn copy_path(name: &str) -> String {
    format!(".agents/hooks/{name}.sh")
}

/// The first lines of every shim; how the installer knows the file is its own.
pub fn marker(name: &str) -> String {
    format!("# agentic-toolkit {name}")
}

/// The line to add to a `pre-commit` hook the installer doesn't own. Git runs
/// hooks at the top of the worktree; the line fails open without the copy.
pub fn line_to_add(name: &str) -> String {
    let p = copy_path(name);
    format!("[ ! -f {p} ] || bash {p} --pre-commit || exit 1")
}

/// The shim's content. It fails open when the copy is absent (an older branch,
/// the component removed in another clone).
pub fn shim(name: &str) -> String {
    format!(
        "#!/bin/sh\n{}\ngate=\"$(git rev-parse --show-toplevel)/{}\"\n[ -f \"$gate\" ] || exit 0\nexec bash \"$gate\" --pre-commit\n",
        marker(name),
        copy_path(name)
    )
}

/// git's `pre-commit` path for the project: `git rev-parse --git-path`, so
/// `core.hooksPath` and linked worktrees count. Without git, `.git/hooks/` when
/// `.git` is a folder. `None` outside a git repository.
pub fn pre_commit_path(root: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-path", "hooks/pre-commit"])
        .output();
    if let Ok(out) = out
        && out.status.success()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let p = PathBuf::from(text.trim_end_matches(['\n', '\r']));
        if !p.as_os_str().is_empty() {
            return Some(if p.is_absolute() { p } else { root.join(p) });
        }
    }
    let git = root.join(".git");
    git.is_dir().then(|| git.join("hooks").join("pre-commit"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Not a git repository: nowhere to enable a hook.
    NoRepo,
    /// No `pre-commit` hook yet.
    Absent(PathBuf),
    /// The installer's shim.
    Ours(PathBuf),
    /// Someone else's hook that already runs the copy (the line was added).
    Runs(PathBuf),
    /// Someone else's hook that doesn't run the copy.
    Foreign(PathBuf),
}

impl State {
    /// Git runs the copy on commit.
    pub fn enabled(&self) -> bool {
        match self {
            State::Ours(p) | State::Runs(p) => executable(p),
            _ => false,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            State::NoRepo => None,
            State::Absent(p) | State::Ours(p) | State::Runs(p) | State::Foreign(p) => Some(p),
        }
    }
}

pub fn state(root: &Path, name: &str) -> State {
    let Some(path) = pre_commit_path(root) else {
        return State::NoRepo;
    };
    let Ok(bytes) = fs::read(&path) else {
        return if fs::symlink_metadata(&path).is_ok() {
            State::Foreign(path)
        } else {
            State::Absent(path)
        };
    };
    let text = String::from_utf8_lossy(&bytes);
    if text.lines().any(|l| l.trim_end() == marker(name)) {
        State::Ours(path)
    } else if text.contains(&copy_path(name)[1..]) {
        // `agents/hooks/<name>.sh`, as the plugin gate detects it.
        State::Runs(path)
    } else {
        State::Foreign(path)
    }
}

#[cfg(unix)]
fn executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

/// Git for Windows runs a hook whatever its mode.
#[cfg(not(unix))]
fn executable(path: &Path) -> bool {
    path.is_file()
}

/// Writes the shim, executable, at `path`.
pub fn write_shim(path: &Path, name: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, shim(name))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

/// How a hook path is shown: relative to the project when inside it.
pub fn disp(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".git/hooks")).unwrap();
        tmp
    }

    #[test]
    fn shim_runs_the_copy_and_fails_open() {
        let s = shim("commit-gate");
        assert!(s.starts_with("#!/bin/sh\n# agentic-toolkit commit-gate\n"));
        assert!(s.contains("/.agents/hooks/commit-gate.sh\""));
        assert!(s.contains("[ -f \"$gate\" ] || exit 0"));
        assert!(s.contains("--pre-commit"));
    }

    #[test]
    fn states() {
        let tmp = repo();
        let root = tmp.path();
        let hook = pre_commit_path(root).unwrap();
        assert_eq!(state(root, "commit-gate"), State::Absent(hook.clone()));
        write_shim(&hook, "commit-gate").unwrap();
        let st = state(root, "commit-gate");
        assert_eq!(st, State::Ours(hook.clone()));
        assert!(st.enabled());
        fs::write(&hook, "#!/bin/sh\nnpx lint-staged\n").unwrap();
        assert_eq!(state(root, "commit-gate"), State::Foreign(hook.clone()));
        fs::write(
            &hook,
            format!(
                "#!/bin/sh\nnpx lint-staged\n{}\n",
                line_to_add("commit-gate")
            ),
        )
        .unwrap();
        assert_eq!(state(root, "commit-gate"), State::Runs(hook));
    }

    #[test]
    fn outside_a_repository_there_is_no_hook() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(state(tmp.path(), "commit-gate"), State::NoRepo);
    }

    #[cfg(unix)]
    #[test]
    fn a_shim_git_would_skip_is_not_enabled() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = repo();
        let hook = pre_commit_path(tmp.path()).unwrap();
        write_shim(&hook, "commit-gate").unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!state(tmp.path(), "commit-gate").enabled());
    }
}
