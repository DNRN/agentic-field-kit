use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use agentic_toolkit::catalog::{Catalog, is_name};
use agentic_toolkit::scope::{self, home};
use agentic_toolkit::source::Source;
use agentic_toolkit::status::Ctx;
use agentic_toolkit::{check, list};

/// Install agentic-toolkit components into a project or globally.
///
/// Scope: the git root of the working directory (default), --project DIR, or
/// --global. Install, update, and remove are still done by install.sh until
/// the Rust port reaches them (docs/plans/rust-installer.md, R2).
#[derive(Parser, Debug)]
#[command(name = "agentic-toolkit", version)]
struct Args {
    /// Components to install
    names: Vec<String>,
    /// What is installed, in the project and globally
    #[arg(long)]
    list: bool,
    /// Validate skills against the Agent Skills portability rules
    #[arg(long)]
    check: bool,
    /// Refresh what changed upstream
    #[arg(long)]
    update: bool,
    /// Remove what this installer installed
    #[arg(long)]
    remove: bool,
    /// Install every component the scope supports
    #[arg(long)]
    all: bool,
    /// The project folder (default: the git root of the working directory)
    #[arg(long, value_name = "DIR")]
    project: Option<PathBuf>,
    /// Use the global scope (your home directory)
    #[arg(long, conflicts_with = "project")]
    global: bool,
    /// Global target: agents, claude, or omp (repeatable)
    #[arg(long, value_name = "NAME")]
    harness: Vec<String>,
    /// Global installs: symlink or copy
    #[arg(long)]
    mode: Option<String>,
    /// Also overwrite or remove locally modified copies
    #[arg(long)]
    force: bool,
    /// Print the plan only
    #[arg(long)]
    dry_run: bool,
    /// Do not ask
    #[arg(long, short = 'y')]
    yes: bool,
    /// Open the menu without a terminal
    #[arg(long)]
    interactive: bool,
    /// Read components from this toolkit tree instead of the built-in ones
    #[arg(long, value_name = "DIR")]
    source: Option<PathBuf>,
}

fn die(msg: &str) -> ExitCode {
    eprintln!("agentic-toolkit: {msg}");
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args = Args::parse();
    let actions = [args.list, args.check, args.update, args.remove]
        .iter()
        .filter(|a| **a)
        .count();
    if actions > 1 {
        return die("choose one of --list, --update, --remove, --check");
    }
    if let Some(bad) = args.names.iter().find(|n| !is_name(n)) {
        return die(&format!("not a component name: {bad} (see --list)"));
    }
    let source = match &args.source {
        Some(dir) if dir.is_dir() => Source::Dir(dir.clone()),
        Some(dir) => return die(&format!("no such toolkit folder: {}", dir.display())),
        None => Source::Embedded,
    };

    if args.check {
        let (lines, ok) = check::run(&source);
        for line in lines {
            println!("{line}");
        }
        return if ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    if !args.global && (!args.harness.is_empty() || args.mode.is_some()) {
        return die(
            "--harness and --mode apply to --global only: project installs are copies (ADR 0006)",
        );
    }
    if let Some(bad) = args
        .harness
        .iter()
        .find(|h| !matches!(h.as_str(), "agents" | "claude" | "omp"))
    {
        return die(&format!("unknown harness: {bad}"));
    }
    if args
        .mode
        .as_deref()
        .is_some_and(|m| m != "symlink" && m != "copy")
    {
        return die("mode must be symlink or copy");
    }
    if let Some(dir) = &args.project
        && !dir.is_dir()
    {
        return die(&format!("no such directory: {}", dir.display()));
    }

    if !args.list {
        return die(
            "install, --update, --remove, and the menu are not in the Rust installer yet: use install.sh (plan R2)",
        );
    }
    if !args.names.is_empty() {
        return die("--list and the menu take no component names");
    }

    let catalog = match source.catalog_text() {
        Ok(text) => Catalog::parse(&text),
        Err(e) => return die(&format!("can't read catalog.tsv: {e}")),
    };
    let (root, why) = match scope::project(args.project.as_deref(), &source) {
        Ok(root) => (Some(root), None),
        Err(why) => (None, Some(why)),
    };
    let ctx = Ctx {
        version: source.version(),
        source,
        catalog,
        root,
        home: home(),
    };
    print!("{}", list::table(&ctx, why.as_deref()));
    ExitCode::SUCCESS
}
