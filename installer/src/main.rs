use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use agentic_toolkit::catalog::{Catalog, is_name};
use agentic_toolkit::engine::{self, Mode, Options, Plan, Verb};
use agentic_toolkit::scope::{self, Scope, home};
use agentic_toolkit::source::Source;
use agentic_toolkit::status::Ctx;
use agentic_toolkit::{check, list};

/// Install agentic-toolkit components into a project or globally.
///
/// Scope: the git root of the working directory (default), --project DIR, or
/// --global. Project installs are copies in .agents/skills, .claude/skills,
/// and .claude/agents, recorded in .agents/agentic-toolkit.tsv; commit them.
/// Global installs are skills in ~/.agents/skills (--harness agents, claude,
/// omp): copies, or with --source symlinks into that toolkit folder.
#[derive(Parser, Debug)]
#[command(name = "agentic-toolkit", version)]
struct Args {
    /// Components to install (or to update or remove)
    names: Vec<String>,
    /// What is installed, in the project and globally
    #[arg(long)]
    list: bool,
    /// Validate skills against the Agent Skills portability rules
    #[arg(long)]
    check: bool,
    /// Refresh what changed upstream, restore deleted copies, add new requirements
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
    /// Global installs: symlink (needs --source) or copy
    #[arg(long)]
    mode: Option<String>,
    /// Also overwrite or remove locally modified copies
    #[arg(long)]
    force: bool,
    /// Print the plan only
    #[arg(long)]
    dry_run: bool,
    /// Do not ask (also removes retired and unneeded components)
    #[arg(long, short = 'y')]
    yes: bool,
    /// Open the menu without a terminal
    #[arg(long)]
    interactive: bool,
    /// Read components from this toolkit folder instead of the built-in ones
    #[arg(long, value_name = "DIR")]
    source: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Install,
    Update,
    Remove,
}

fn die(msg: &str) -> ExitCode {
    eprintln!("agentic-toolkit: {msg}");
    ExitCode::FAILURE
}

fn is_tty() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn confirm(question: &str) -> bool {
    print!("{question} [y/N] ");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    if io::stdin().lock().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(
        answer.trim_end_matches(['\r', '\n']),
        "y" | "Y" | "yes" | "YES"
    )
}

struct Run {
    dry_run: bool,
    yes: bool,
}

/// Plans, shows, confirms, and applies one action; the exit status, or the
/// reason to stop.
fn run(
    ctx: &Ctx,
    opts: &Options,
    run: &Run,
    action: Action,
    scope: Scope,
    names: &[String],
    all: bool,
) -> Result<bool, String> {
    let plan: Plan = match action {
        Action::Install => engine::plan_install(ctx, opts, scope, names, all)?,
        Action::Update => engine::plan_update(ctx, opts, scope, names)?,
        Action::Remove => engine::plan_remove(ctx, opts, scope, names)?,
    };
    if plan.steps.is_empty() {
        println!("Nothing to do.");
        return Ok(true);
    }
    let mut can_ask_remove = run.yes || (is_tty() && !run.dry_run);
    for line in engine::show(ctx, opts, scope, &plan, can_ask_remove) {
        println!("{line}");
    }
    if run.dry_run {
        return Ok(!plan.skipped);
    }
    let mut ok = !plan.skipped;
    let mut applied = engine::Applied::default();
    if plan.has_changes() {
        if is_tty() && !run.yes {
            if !confirm("Apply?") {
                println!("Nothing changed.");
                return Ok(!plan.skipped);
            }
            can_ask_remove = true;
        }
        applied =
            engine::apply(ctx, opts, scope, &plan, can_ask_remove).map_err(|e| e.to_string())?;
        for line in &applied.messages {
            println!("{line}");
        }
        ok &= !applied.skipped;
    }
    if scope == Scope::Project {
        if action == Action::Install {
            ok &= warn_both_scopes(ctx, &plan, run);
        }
        if !applied.wrote.is_empty() || applied.changed {
            let root = ctx.root.as_deref().expect("project root");
            println!(
                "Commit the changes so teammates get the same set: git -C \"{}\" add .agents .claude",
                root.display()
            );
        }
    }
    if plan.has_changes() {
        println!("{}", if ok { "done" } else { "done with skips" });
    }
    Ok(ok)
}

/// Same skill in the project and globally: Codex lists it twice; Claude Code
/// and Cursor use the global copy (selective-install plan, "Evidence").
fn warn_both_scopes(ctx: &Ctx, plan: &Plan, run: &Run) -> bool {
    let mut ok = true;
    for s in &plan.steps {
        if !matches!(s.verb, Verb::Write | Verb::Ok | Verb::Promote)
            || ctx.component(&s.name).is_none_or(|c| c.kind != "skill")
        {
            continue;
        }
        let g = ctx.status(Scope::Global, &s.name);
        if matches!(g.as_str(), "-" | "n/a" | "plugin") {
            continue;
        }
        println!(
            "! {} is also installed globally ({g}): Codex lists it twice; Claude Code and Cursor use the global copy.",
            s.name
        );
        if g == "foreign" {
            continue;
        }
        if is_tty() && !run.yes && confirm("  Remove the global copy?") {
            let opts = Options {
                force: false,
                mode_arg: None,
                mode: Mode::Copy,
                harnesses: Vec::new(),
            };
            let global = Run {
                dry_run: false,
                yes: true,
            };
            match self::run(
                ctx,
                &opts,
                &global,
                Action::Remove,
                Scope::Global,
                std::slice::from_ref(&s.name),
                false,
            ) {
                Ok(done) => ok &= done,
                Err(e) => {
                    eprintln!("agentic-toolkit: {e}");
                    ok = false;
                }
            }
        } else {
            println!(
                "  Remove it with: agentic-toolkit --global --remove {}",
                s.name
            );
        }
    }
    ok
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
        Some(dir) if dir.is_dir() => match std::path::absolute(dir) {
            Ok(dir) => Source::Dir(dir),
            Err(e) => return die(&format!("{}: {e}", dir.display())),
        },
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
        .find(|h| engine::global_rel(h).is_none())
    {
        return die(&format!("unknown harness: {bad}"));
    }
    let mode_arg = match args.mode.as_deref() {
        None => None,
        Some("symlink") => Some(Mode::Symlink),
        Some("copy") => Some(Mode::Copy),
        Some(_) => return die("mode must be symlink or copy"),
    };
    if mode_arg == Some(Mode::Symlink) && source.dir().is_none() {
        return die(
            "--mode symlink links into a toolkit folder: add --source DIR, or use --mode copy",
        );
    }
    if let Some(dir) = &args.project
        && !dir.is_dir()
    {
        return die(&format!("no such directory: {}", dir.display()));
    }

    let catalog = match source.catalog_text() {
        Ok(text) => Catalog::parse(&text),
        Err(e) => return die(&format!("can't read catalog.tsv: {e}")),
    };

    let action = if args.list {
        None
    } else if args.update {
        Some(Action::Update)
    } else if args.remove {
        Some(Action::Remove)
    } else if !args.names.is_empty() || args.all {
        Some(Action::Install)
    } else if args.interactive || is_tty() {
        return die(
            "the menu is not in the Rust installer yet (plan R3): name components, or use --list, --update, --remove",
        );
    } else {
        let names: Vec<&str> = catalog.names().collect();
        return die(&format!(
            "name the components to install, or use --all, --list, --update, --remove (see --help). Components: {}",
            names.join(" ")
        ));
    };
    if args.all && !args.names.is_empty() {
        return die("give component names or --all, not both");
    }
    if args.all && action != Some(Action::Install) {
        return die("--all installs; --update and --remove without names cover everything");
    }
    if action.is_none() && !args.names.is_empty() {
        return die("--list and the menu take no component names");
    }

    let (root, why) = match scope::project(args.project.as_deref(), &source) {
        Ok(root) => (Some(root), None),
        Err(why) => (None, Some(why)),
    };
    // Links point into a toolkit folder; Windows can't make them without Developer Mode.
    let mode = mode_arg.unwrap_or(if cfg!(windows) || source.dir().is_none() {
        Mode::Copy
    } else {
        Mode::Symlink
    });
    let ctx = Ctx {
        version: source.version(),
        source,
        catalog,
        root,
        home: home(),
    };

    let Some(action) = action else {
        print!("{}", list::table(&ctx, why.as_deref()));
        return ExitCode::SUCCESS;
    };
    let scope = if args.global {
        Scope::Global
    } else if ctx.root.is_some() {
        Scope::Project
    } else {
        return die(why.as_deref().unwrap_or("no project"));
    };
    let opts = Options {
        force: args.force,
        mode_arg,
        mode,
        harnesses: args.harness,
    };
    let flags = Run {
        dry_run: args.dry_run,
        yes: args.yes,
    };
    match run(&ctx, &opts, &flags, action, scope, &args.names, args.all) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => die(&e),
    }
}
