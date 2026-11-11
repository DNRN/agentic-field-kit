use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command as Process, ExitCode};
use std::time::Duration;

use clap::Parser;

use agentic_field_kit::catalog::{Catalog, is_name};
use agentic_field_kit::engine::{self, Action, Mode, Options, Plan};
use agentic_field_kit::scope::{self, Scope, home};
use agentic_field_kit::source::Source;
use agentic_field_kit::status::Ctx;
use agentic_field_kit::update::{self, Binary};
use agentic_field_kit::{agents, check, list, newproject, tui};

/// Install afk components into a project or globally.
///
/// Scope: the git root of the working directory (default), --project DIR, or
/// --global. Project installs are copies in .agents/skills, .claude/skills,
/// and .claude/agents, recorded in .agents/afk.tsv; commit them.
/// Global installs are skills in ~/.agents/skills (--harness agents, claude,
/// omp): copies, or with --source symlinks into that toolkit folder.
#[derive(Parser, Debug)]
#[command(name = "afk", version)]
struct Args {
    /// Components to install (or to update or remove)
    names: Vec<String>,
    /// What is installed, in the project and globally
    #[arg(long)]
    list: bool,
    /// Validate skills against the Agent Skills portability rules
    #[arg(long)]
    check: bool,
    /// Update afk itself to the latest release, then refresh what changed
    /// upstream, restore deleted copies, add new requirements
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
    #[arg(long)]
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
    /// Read components from this toolkit folder instead of the built-in ones
    #[arg(long, value_name = "DIR")]
    source: Option<PathBuf>,
}

fn die(msg: &str) -> ExitCode {
    eprintln!("afk: {msg}");
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
    let plan: Plan = engine::plan_action(ctx, opts, action, scope, names, all)?;
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
        for name in &applied.wrote {
            if let Some(next) = engine::next_step(name, scope) {
                println!("{next}");
            }
        }
    }
    if scope == Scope::Project {
        if action == Action::Install {
            ok &= warn_both_scopes(ctx, &plan, run);
        }
        if (!applied.wrote.is_empty() || applied.changed)
            && let Some(hint) = engine::commit_hint(ctx.root.as_deref().expect("project root"))
        {
            println!("{hint}");
        }
    }
    if plan.has_changes() {
        println!("{}", if ok { "done" } else { "done with skips" });
    }
    Ok(ok)
}

/// Same skill in the project and globally: warn, and on a terminal offer to
/// remove the global copy.
fn warn_both_scopes(ctx: &Ctx, plan: &Plan, run: &Run) -> bool {
    let mut ok = true;
    for (name, g) in engine::both_scopes(ctx, plan) {
        println!("{}", engine::both_scopes_warning(&name, &g));
        if g == "foreign" {
            continue;
        }
        let s = &name;
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
                std::slice::from_ref(s),
                false,
            ) {
                Ok(done) => ok &= done,
                Err(e) => {
                    eprintln!("afk: {e}");
                    ok = false;
                }
            }
        } else {
            println!("  Remove it with: afk --global --remove {s}");
        }
    }
    ok
}

/// Whether to look for a newer afk for the notice (ADR 0022): on a terminal,
/// with the built-in components, and not turned off.
fn checking(ctx: &Ctx) -> bool {
    is_tty() && ctx.source.dir().is_none() && !update::off()
}

/// The binary step of `--update` (ADR 0022). `None`: go on and refresh the
/// components in this process. `Some`: this process is done, with that
/// exit status. `refresh` says whether there is a scope to refresh.
fn update_binary(ctx: &Ctx, run: &Run, refresh: bool) -> Option<ExitCode> {
    if !run.yes && !is_tty() {
        // Unattended: no network request, nothing replaced.
        if let Some(newer) = update::cached(&ctx.version, &ctx.home) {
            println!("afk {newer} is available: run afk --update on a terminal, or add --yes");
        }
        return None;
    }
    let replace = match update::binary(&ctx.version, &ctx.home) {
        Ok(Binary::Current) => {
            println!("afk {} is the latest release.", ctx.version);
            return None;
        }
        Ok(Binary::Manual(latest)) => {
            println!(
                "afk {latest} is available. Its install script didn't install this afk; update it with: {}",
                update::MANUAL
            );
            return None;
        }
        Ok(Binary::Replace(replace)) => replace,
        Err(e) => {
            eprintln!("afk: can't check for a newer afk: {e}");
            return None;
        }
    };
    let (old, new) = (&ctx.version, replace.latest.clone());
    if run.dry_run {
        println!("Would update afk {old} → {new}, then refresh components with it.");
        return None;
    }
    if !run.yes && !confirm(&format!("Update afk {old} → {new}?")) {
        return None;
    }
    // Before the update: on Linux the replaced file's link reads "(deleted)".
    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            eprintln!("afk: can't find this afk's path: {e}");
            return Some(ExitCode::FAILURE);
        }
    };
    if let Err(e) = replace.apply() {
        eprintln!("afk: the update to {new} failed: {e}");
        return Some(ExitCode::FAILURE);
    }
    println!("Updated afk {old} → {new}.");
    if !refresh {
        println!("Run afk --update in each project to refresh its components.");
        return Some(ExitCode::SUCCESS);
    }
    // Components come from the new binary: run it with the same arguments.
    match Process::new(&exe)
        .args(env::args_os().skip(1))
        .env(update::UPDATED_VAR, "1")
        .status()
    {
        Ok(status) => Some(ExitCode::from(
            u8::try_from(status.code().unwrap_or(1)).unwrap_or(1),
        )),
        Err(e) => {
            eprintln!("afk: can't start the new afk: {e}");
            eprintln!("Next: run afk --update again to refresh the components.");
            Some(ExitCode::FAILURE)
        }
    }
}

fn main() -> ExitCode {
    // Usage errors exit 1; --help and --version exit 0.
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            let _ = e.print();
            return if e.use_stderr() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    if args.global && args.project.is_some() {
        return die("choose --project DIR or --global, not both");
    }
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
        // Absolute and with `..` resolved, without following links (bash's `cd && pwd`).
        Some(dir) if dir.is_dir() => match std::path::absolute(dir) {
            Ok(dir) => Source::Dir(scope::normalize(&dir)),
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

    enum Command {
        List,
        Menu,
        Act(Action),
    }
    let command = if args.list {
        Command::List
    } else if args.update {
        Command::Act(Action::Update)
    } else if args.remove {
        Command::Act(Action::Remove)
    } else if !args.names.is_empty() || args.all {
        Command::Act(Action::Install)
    } else if is_tty() {
        Command::Menu
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
    if args.all && !matches!(command, Command::Act(Action::Install)) {
        return die("--all installs; --update and --remove without names cover everything");
    }
    if matches!(command, Command::List) && !args.names.is_empty() {
        return die("--list takes no component names");
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
    let opts = Options {
        force: args.force,
        mode_arg,
        mode,
        harnesses: args.harness,
    };

    let action = match command {
        Command::List => {
            let newer = checking(&ctx)
                .then(|| update::check(&ctx.version, &ctx.home).wait(Duration::from_secs(2)))
                .flatten();
            print!("{}", list::table(&ctx, why.as_deref(), newer.as_deref()));
            return ExitCode::SUCCESS;
        }
        Command::Menu => {
            // Outside any git repository, with no scope given: offer to start
            // a project (ADR 0016).
            let new_project = !args.global
                && args.project.is_none()
                && why
                    .as_deref()
                    .is_some_and(|w| w.starts_with(scope::NO_REPO));
            let home = ctx.home.clone();
            let check = checking(&ctx).then(|| update::check(&ctx.version, &ctx.home));
            let mut app = tui::App::new(ctx, opts, args.global, args.dry_run, why);
            if new_project && let Ok(cwd) = std::env::current_dir() {
                app.start_new_project(&dunce::canonicalize(&cwd).unwrap_or(cwd));
            }
            let later = check.and_then(|check| {
                app.newer = check.newer;
                check.later
            });
            let out = match tui::run(app, later) {
                Ok(out) => out,
                Err(e) => return die(&e.to_string()),
            };
            for line in &out.log {
                println!("{line}");
            }
            if let Some((agent, dir)) = out.handoff {
                let shown = newproject::display(&dir, &home);
                println!("Starting {} in {shown}…", agent.name);
                return match agents::launch(agent, &dir) {
                    Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
                    Err(e) => {
                        eprintln!("afk: can't start {}: {e}", agent.command);
                        eprintln!(
                            "Next: open your agent in {shown} and ask it to start the project."
                        );
                        ExitCode::FAILURE
                    }
                };
            }
            return if out.ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
        Command::Act(action) => action,
    };
    let flags = Run {
        dry_run: args.dry_run,
        yes: args.yes,
    };
    if action == Action::Update
        && ctx.source.dir().is_none()
        && !update::off()
        && !update::updated()
        && let Some(code) = update_binary(&ctx, &flags, args.global || ctx.root.is_some())
    {
        return code;
    }
    let scope = if args.global {
        Scope::Global
    } else if ctx.root.is_some() {
        Scope::Project
    } else {
        let why = why.as_deref().unwrap_or("no project");
        let code = die(why);
        if why.starts_with(scope::NO_REPO) {
            for line in engine::new_project_hint() {
                eprintln!("{line}");
            }
        }
        return code;
    };
    match run(&ctx, &opts, &flags, action, scope, &args.names, args.all) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => die(&e),
    }
}
