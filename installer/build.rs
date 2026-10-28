//! Builds `catalog.tsv`, `.claude-plugin/plugin.json`, and every path the
//! catalog lists into the binary (ADR 0010). Generates `$OUT_DIR/embedded.rs`
//! with one `(repository-relative path, bytes)` pair per file.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let installer = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo = installer.parent().unwrap().to_path_buf();
    let catalog = repo.join("catalog.tsv");
    println!("cargo::rerun-if-changed={}", catalog.display());

    let mut files: Vec<String> = vec!["catalog.tsv".into(), ".claude-plugin/plugin.json".into()];
    let text = fs::read_to_string(&catalog).expect("read catalog.tsv");
    for line in text.lines() {
        let fields: Vec<&str> = line.trim_end_matches('\r').split('\t').collect();
        if fields.len() < 3
            || fields[0].is_empty()
            || fields[0].starts_with('#')
            || fields[0] == "kind"
        {
            continue;
        }
        let path = repo.join(fields[2]);
        println!("cargo::rerun-if-changed={}", path.display());
        collect(&repo, &path, &mut files);
    }
    println!(
        "cargo::rerun-if-changed={}",
        repo.join(".claude-plugin/plugin.json").display()
    );
    files.sort();
    files.dedup();

    let mut out = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for rel in &files {
        let abs = repo.join(rel);
        writeln!(
            out,
            "    ({rel:?}, include_bytes!({:?})),",
            abs.to_str().unwrap()
        )
        .unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded.rs");
    fs::write(dest, out).unwrap();
}

/// Regular files under `path` (or `path` itself), as `/`-separated paths
/// relative to the repository. Symlinks are skipped, as `find -type f` does.
fn collect(repo: &Path, path: &Path, files: &mut Vec<String>) {
    let meta = fs::symlink_metadata(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    if meta.is_file() {
        let rel = path.strip_prefix(repo).unwrap();
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_str().unwrap().to_owned())
            .collect();
        files.push(parts.join("/"));
    } else if meta.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for entry in entries {
            collect(repo, &entry, files);
        }
    }
}
