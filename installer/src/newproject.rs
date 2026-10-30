//! The new project screen's facts (ADR 0016): where the user stands, what is
//! there, which choice to highlight, and whether a subfolder name is usable.
//! Reads the disk; writes nothing.

use std::fs;
use std::path::{Path, PathBuf};

/// File names that mark a folder as a project's.
pub const PROJECT_FILES: [&str; 14] = [
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "setup.py",
    "requirements.txt",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Gemfile",
    "composer.json",
    "mix.exs",
    "CMakeLists.txt",
    "Makefile",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Here,
    Subfolder,
    Browse,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub path: PathBuf,
    /// Entries in the folder, hidden ones included.
    pub entries: usize,
    /// Which of [`PROJECT_FILES`] it has.
    pub project_files: Vec<String>,
    /// Subfolders that are git repositories (a folder of projects).
    pub repositories: usize,
    /// The home directory or a filesystem root: never a project itself.
    pub home_or_root: bool,
}

pub fn inspect(dir: &Path, home: &Path) -> Folder {
    let (mut entries, mut repositories, mut project_files) = (0, 0, Vec::new());
    if let Ok(read) = fs::read_dir(dir) {
        for entry in read.flatten() {
            entries += 1;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                repositories += 1;
            } else if PROJECT_FILES.contains(&name.as_str()) {
                project_files.push(name);
            }
        }
    }
    project_files.sort();
    let same = |a: &Path, b: &Path| match (dunce::canonicalize(a), dunce::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    };
    Folder {
        path: dir.to_path_buf(),
        entries,
        project_files,
        repositories,
        home_or_root: same(dir, home) || dir.parent().is_none(),
    }
}

impl Folder {
    /// The choices on the screen, in order: "here" only where a project may go.
    pub fn choices(&self) -> Vec<Choice> {
        let mut out = Vec::new();
        if !self.home_or_root {
            out.push(Choice::Here);
        }
        out.extend([Choice::Subfolder, Choice::Browse, Choice::Quit]);
        out
    }

    /// The highlighted choice: here for an empty folder or one with project
    /// files; a subfolder for home, a folder of repositories, or other files.
    pub fn default_choice(&self) -> Choice {
        if self.home_or_root || self.repositories > 0 {
            Choice::Subfolder
        } else if self.entries == 0 || !self.project_files.is_empty() {
            Choice::Here
        } else {
            Choice::Subfolder
        }
    }

    /// One line on what the folder holds.
    pub fn describe(&self) -> String {
        if self.entries == 0 {
            return "Empty folder, not a git repository.".into();
        }
        let mut s = format!(
            "{} {}, not a git repository",
            self.entries,
            if self.entries == 1 {
                "entry"
            } else {
                "entries"
            }
        );
        if !self.project_files.is_empty() {
            s.push_str(&format!(
                "; project files: {}",
                self.project_files.join(", ")
            ));
        }
        if self.repositories > 0 {
            s.push_str(&format!(
                "; {} {} git repositories",
                self.repositories,
                if self.repositories == 1 {
                    "subfolder is"
                } else {
                    "subfolders are"
                }
            ));
        }
        s.push('.');
        s
    }
}

/// The subfolder `name` would create in `dir`, or why it can't.
pub fn subfolder(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() {
        return Err("Type a folder name.".into());
    }
    if name != name.trim() {
        return Err("No spaces at the start or end of the name.".into());
    }
    if name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err("One folder name, without / or \\.".into());
    }
    let path = dir.join(name);
    if fs::symlink_metadata(&path).is_ok() {
        return Err(format!("{name} already exists here."));
    }
    Ok(path)
}

/// A path for display, with the home directory as `~`.
pub fn display(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".into(),
        Ok(rest) => format!("~/{}", rest.to_string_lossy().replace('\\', "/")),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        let dir = home.join("code").join("new");
        fs::create_dir_all(&dir).unwrap();
        (tmp, home, dir)
    }

    #[test]
    fn an_empty_folder_defaults_to_here() {
        let (_t, home, dir) = folder();
        let f = inspect(&dir, &home);
        assert_eq!(f.entries, 0);
        assert_eq!(f.default_choice(), Choice::Here);
        assert_eq!(f.choices()[0], Choice::Here);
        assert_eq!(f.describe(), "Empty folder, not a git repository.");
    }

    #[test]
    fn project_files_default_to_here() {
        let (_t, home, dir) = folder();
        fs::write(dir.join("package.json"), "{}").unwrap();
        fs::write(dir.join("index.js"), "").unwrap();
        let f = inspect(&dir, &home);
        assert_eq!(f.project_files, ["package.json"]);
        assert_eq!(f.default_choice(), Choice::Here);
        assert!(f.describe().contains("project files: package.json"));
    }

    #[test]
    fn other_files_and_folders_of_repositories_default_to_a_subfolder() {
        let (_t, home, dir) = folder();
        fs::write(dir.join("notes.txt"), "").unwrap();
        assert_eq!(inspect(&dir, &home).default_choice(), Choice::Subfolder);
        let code = home.join("code");
        fs::create_dir_all(code.join("shop/.git")).unwrap();
        let f = inspect(&code, &home);
        assert_eq!(f.repositories, 1);
        assert_eq!(f.default_choice(), Choice::Subfolder);
    }

    #[test]
    fn home_never_offers_here() {
        let (_t, home, _dir) = folder();
        let f = inspect(&home, &home);
        assert!(f.home_or_root);
        assert!(!f.choices().contains(&Choice::Here));
        assert_eq!(f.default_choice(), Choice::Subfolder);
        assert!(inspect(Path::new("/"), &home).home_or_root);
    }

    #[test]
    fn subfolder_names() {
        let (_t, _home, dir) = folder();
        fs::create_dir_all(dir.join("taken")).unwrap();
        assert_eq!(subfolder(&dir, "shop").unwrap(), dir.join("shop"));
        for bad in ["", " shop", "a/b", "a\\b", ".", "..", "taken"] {
            assert!(subfolder(&dir, bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn display_uses_tilde_for_home() {
        let home = Path::new("/home/u");
        assert_eq!(display(Path::new("/home/u/code/x"), home), "~/code/x");
        assert_eq!(display(home, home), "~");
        assert_eq!(display(Path::new("/srv/x"), home), "/srv/x");
    }
}
