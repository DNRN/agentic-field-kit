//! Planning and applying installs, updates, and removals, with the rules of
//! `install.sh` (docs/plans/selective-install.md): never touch what isn't
//! ours, never overwrite or delete a local edit without `--force`, and remove
//! what the user didn't name only after a confirmation or `--yes`. Only
//! [`apply`] writes to disk.

use std::fs;
use std::io;
use std::path::Path;

use crate::catalog::Component;
use crate::hash;
use crate::manifest;
use crate::scope::{Scope, plugin_on};
use crate::source::write_content;
use crate::status::{Ctx, is_ours};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Symlink,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Link,
    Copy,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub force: bool,
    /// `--mode`, when given.
    pub mode_arg: Option<Mode>,
    /// The mode for new global paths: `--mode`, else symlink when links are possible.
    pub mode: Mode,
    /// `--harness` values; empty when none were given.
    pub harnesses: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    /// Copy or link the paths.
    Write,
    Remove,
    /// Remove only after a confirmation or `--yes`.
    AskRemove,
    /// Record the reason as explicit.
    Promote,
    Skip,
    Ok,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub verb: Verb,
    pub name: String,
    pub reason: String,
    pub paths: Vec<String>,
    /// For `Write`, the status before; otherwise the explanation shown.
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub steps: Vec<Step>,
    /// Something asked for can't be done (exit status 1).
    pub skipped: bool,
}

impl Plan {
    fn add(&mut self, verb: Verb, name: &str, reason: &str, paths: Vec<String>, note: &str) {
        self.steps.push(Step {
            verb,
            name: name.into(),
            reason: reason.into(),
            paths,
            note: note.into(),
        });
    }

    fn skip(&mut self, name: &str, note: &str) {
        self.add(Verb::Skip, name, "-", Vec::new(), note);
        self.skipped = true;
    }

    fn ok(&mut self, name: &str, note: &str) {
        self.add(Verb::Ok, name, "-", Vec::new(), note);
    }

    pub fn has_changes(&self) -> bool {
        self.steps.iter().any(|s| {
            matches!(
                s.verb,
                Verb::Write | Verb::Remove | Verb::AskRemove | Verb::Promote
            )
        })
    }
}

pub fn global_rel(harness: &str) -> Option<&'static str> {
    match harness {
        "agents" => Some(".agents/skills"),
        "claude" => Some(".claude/skills"),
        "omp" => Some(".omp/agent/skills"),
        _ => None,
    }
}

fn exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// How a path is shown: relative to the project, or `~/` for global.
pub fn disp(scope: Scope, p: &str) -> String {
    match scope {
        Scope::Global => format!("~/{p}"),
        Scope::Project => p.to_owned(),
    }
}

/// Paths a fresh install writes.
fn layout(ctx: &Ctx, opts: &Options, scope: Scope, name: &str) -> Vec<String> {
    let Some(c) = ctx.component(name) else {
        return Vec::new();
    };
    match (scope, c.kind.as_str()) {
        (Scope::Project, "skill") => {
            let mut paths = vec![format!(".agents/skills/{name}")];
            if !ctx.root.as_deref().is_some_and(plugin_on) {
                paths.push(format!(".claude/skills/{name}"));
            }
            paths
        }
        (Scope::Project, "agent") => vec![format!(".claude/agents/{name}.md")],
        (Scope::Global, "skill") => {
            let harnesses: Vec<&str> = if opts.harnesses.is_empty() {
                vec!["agents"]
            } else {
                opts.harnesses.iter().map(String::as_str).collect()
            };
            harnesses
                .iter()
                .filter_map(|h| global_rel(h))
                .map(|t| format!("{t}/{name}"))
                .collect()
        }
        _ => Vec::new(),
    }
}

/// Link or copy: `--mode` if given, else the form the path has, else the form
/// recorded for the component (hash `-` means links), else the default.
/// Links need a toolkit folder to point into (`--source`).
pub fn form_for(ctx: &Ctx, opts: &Options, scope: Scope, full: &Path, name: &str) -> Form {
    if scope == Scope::Project {
        return Form::Copy;
    }
    let mode_form = |m: Mode| {
        if m == Mode::Symlink {
            Form::Link
        } else {
            Form::Copy
        }
    };
    let form = if let Some(m) = opts.mode_arg {
        mode_form(m)
    } else if is_symlink(full) {
        Form::Link
    } else if exists(full) {
        Form::Copy
    } else if let Some(row) = manifest::row(&ctx.home, name) {
        if row.hash == "-" {
            Form::Link
        } else {
            Form::Copy
        }
    } else {
        mode_form(opts.mode)
    };
    if form == Form::Link && ctx.source.dir().is_none() {
        Form::Copy
    } else {
        form
    }
}

fn form_ok(ctx: &Ctx, opts: &Options, scope: Scope, full: &Path, name: &str) -> bool {
    let want = form_for(ctx, opts, scope, full, name);
    if is_symlink(full) {
        want == Form::Link
    } else {
        want == Form::Copy
    }
}

fn union(own: &[String], lay: &[String]) -> Vec<String> {
    let mut out = own.to_vec();
    for p in lay {
        if !out.contains(p) {
            out.push(p.clone());
        }
    }
    out
}

/// Appends `name` to `order` after everything it requires.
fn visit(
    ctx: &Ctx,
    name: &str,
    order: &mut Vec<String>,
    visiting: &mut Vec<String>,
) -> Result<(), String> {
    if order.iter().any(|n| n == name) {
        return Ok(());
    }
    if visiting.iter().any(|n| n == name) {
        return Err(format!("dependency cycle at {name}"));
    }
    let c = ctx
        .component(name)
        .ok_or_else(|| format!("unknown component: {name} (see --list)"))?;
    visiting.push(name.to_owned());
    for r in &c.requires {
        visit(ctx, r, order, visiting)?;
    }
    order.push(name.to_owned());
    Ok(())
}

fn reason_of(root: &Path, name: &str) -> String {
    manifest::row(root, name)
        .map(|r| r.reason)
        .unwrap_or_else(|| "explicit".into())
}

fn requires(ctx: &Ctx, name: &str) -> Vec<String> {
    ctx.component(name)
        .map(|c| c.requires.clone())
        .unwrap_or_default()
}

fn plan_write(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    plan: &mut Plan,
    name: &str,
    reason: &str,
    explicit: bool,
) {
    let root = ctx.scope_root(scope).expect("scope root");
    match ctx.avail(scope, name) {
        "" => {}
        "plugin" => {
            return plan.skip(
                name,
                "provided by the agentic-toolkit plugin enabled for this scope",
            );
        }
        _ => {
            let note = if ctx.component(name).is_some_and(|c| c.harnesses == "plugin") {
                "plugin only until roadmap phase 1 (portable commit gate)"
            } else {
                "not available in this scope"
            };
            return plan.skip(name, note);
        }
    }
    let st = ctx.status(scope, name);
    let lay = layout(ctx, opts, scope, name);
    let own = ctx.owned(scope, name);
    if let Some(p) = lay
        .iter()
        .find(|p| exists(&root.join(p)) && !own.contains(p))
    {
        return plan.skip(
            name,
            &format!(
                "{} exists, not installed by agentic-toolkit",
                disp(scope, p)
            ),
        );
    }
    let row = manifest::row(root, name);
    let mreason = row.as_ref().map(|r| r.reason.clone()).unwrap_or_default();
    let reason = if explicit {
        "explicit".to_owned()
    } else if !mreason.is_empty() {
        mreason.clone()
    } else {
        reason.to_owned()
    };
    let need = match st.as_str() {
        "modified" => {
            if !opts.force {
                return plan.skip(name, "modified locally; --force overwrites it");
            }
            true
        }
        "installed" | "linked" => {
            let recorded = row.as_ref().map(|r| r.paths()).unwrap_or_default();
            lay.iter()
                .any(|p| !recorded.contains(p) || !form_ok(ctx, opts, scope, &root.join(p), name))
        }
        _ => true,
    };
    if need {
        plan.add(Verb::Write, name, &reason, union(&own, &lay), &st);
    } else if reason == "explicit" && mreason == "dependency" {
        plan.add(Verb::Promote, name, "explicit", Vec::new(), "");
    } else {
        plan.ok(name, "already installed");
    }
}

/// `names`, or with `all` every component the scope supports, plus what they require.
pub fn plan_install(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    names: &[String],
    all: bool,
) -> Result<Plan, String> {
    let mut plan = Plan::default();
    let names: Vec<String> = if all {
        ctx.catalog
            .names()
            .filter(|n| ctx.avail(scope, n).is_empty())
            .map(str::to_owned)
            .collect()
    } else {
        names.to_vec()
    };
    let (mut order, mut visiting) = (Vec::new(), Vec::new());
    for n in &names {
        if ctx.component(n).is_none() {
            return Err(format!("unknown component: {n} (see --list)"));
        }
        // Not installable here: say why, and don't pull in what it requires.
        if ctx.avail(scope, n).is_empty() {
            visit(ctx, n, &mut order, &mut visiting)?;
        } else {
            plan_write(ctx, opts, scope, &mut plan, n, "explicit", true);
        }
    }
    for n in &order {
        let explicit = names.contains(n);
        plan_write(
            ctx,
            opts,
            scope,
            &mut plan,
            n,
            if explicit { "explicit" } else { "dependency" },
            explicit,
        );
    }
    Ok(plan)
}

/// Everything ours in the scope: manifest rows, then legacy installs.
fn installed_targets(ctx: &Ctx, scope: Scope, root: &Path) -> Vec<String> {
    let mut targets = manifest::names(root);
    for n in ctx.catalog.names() {
        if !targets.iter().any(|t| t == n) && !ctx.owned(scope, n).is_empty() {
            targets.push(n.to_owned());
        }
    }
    targets
}

pub fn plan_update(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    names: &[String],
) -> Result<Plan, String> {
    let root = ctx.scope_root(scope).expect("scope root");
    let mut plan = Plan::default();
    let targets = if names.is_empty() {
        installed_targets(ctx, scope, root)
    } else {
        names.to_vec()
    };
    for n in &targets {
        let st = ctx.status(scope, n);
        match st.as_str() {
            "installed" | "linked" => plan.ok(n, "up to date"),
            "outdated" | "missing" | "dangling" => plan.add(
                Verb::Write,
                n,
                &reason_of(root, n),
                ctx.owned(scope, n),
                &st,
            ),
            "modified" => {
                if ctx.component(n).is_none() {
                    if opts.force {
                        plan.add(
                            Verb::AskRemove,
                            n,
                            "-",
                            ctx.owned(scope, n),
                            "retired: no longer in the toolkit",
                        );
                    } else {
                        plan.skip(n, "retired, and modified locally; --force removes it");
                    }
                } else if opts.force {
                    plan.add(
                        Verb::Write,
                        n,
                        &reason_of(root, n),
                        ctx.owned(scope, n),
                        &st,
                    );
                } else {
                    plan.skip(n, "modified locally; --force overwrites it");
                }
            }
            "retired" => plan.add(
                Verb::AskRemove,
                n,
                "-",
                ctx.owned(scope, n),
                "retired: no longer in the toolkit",
            ),
            _ => plan.skip(n, "not installed"),
        }
    }
    // Requirements added to the toolkit since these components were installed.
    let (mut order, mut visiting) = (Vec::new(), Vec::new());
    for n in &targets {
        for r in requires(ctx, n) {
            if ctx.status(scope, &r) == "-" {
                visit(ctx, &r, &mut order, &mut visiting)?;
            }
        }
    }
    for n in &order {
        plan_write(ctx, opts, scope, &mut plan, n, "dependency", false);
    }
    Ok(plan)
}

pub fn plan_remove(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    names: &[String],
) -> Result<Plan, String> {
    let root = ctx.scope_root(scope).expect("scope root");
    let mut plan = Plan::default();
    let mut targets: Vec<String> = Vec::new();
    if names.is_empty() {
        targets = installed_targets(ctx, scope, root);
    } else {
        for n in names {
            if is_ours(&ctx.status(scope, n)) {
                targets.push(n.clone());
            } else {
                plan.skip(n, "not installed by agentic-toolkit here");
            }
        }
    }
    let mut edited = Vec::new();
    targets.retain(|n| {
        if !opts.force && ctx.status(scope, n) == "modified" {
            plan.skip(n, "modified locally; --force removes it");
            edited.push(n.clone());
            false
        } else {
            true
        }
    });
    let installed = manifest::names(root);
    for m in &installed {
        if targets.contains(m) {
            continue;
        }
        for r in requires(ctx, m) {
            if !targets.contains(&r) {
                continue;
            }
            if !edited.contains(m) {
                return Err(format!(
                    "{r} is required by {m}; remove both: agentic-toolkit --remove {r} {m}"
                ));
            }
            plan.skip(
                &r,
                &format!("required by {m}, which is modified locally; --force removes both"),
            );
            targets.retain(|n| n != &r);
        }
    }
    // Requirements nobody needs any more, pulled in only for what is removed.
    let mut auto = Vec::new();
    let mut changed = true;
    while changed {
        changed = false;
        for m in &installed {
            if targets.contains(m)
                || manifest::row(root, m).is_none_or(|r| r.reason != "dependency")
            {
                continue;
            }
            let needed = installed
                .iter()
                .any(|n| !targets.contains(n) && requires(ctx, n).contains(m));
            if needed {
                continue;
            }
            if targets.iter().any(|n| requires(ctx, n).contains(m)) {
                if !opts.force && ctx.status(scope, m) == "modified" {
                    continue;
                }
                targets.push(m.clone());
                auto.push(m.clone());
                plan.add(
                    Verb::AskRemove,
                    m,
                    "-",
                    ctx.owned(scope, m),
                    "no longer needed",
                );
                changed = true;
            }
        }
    }
    for n in &targets {
        if auto.contains(n) {
            continue;
        }
        let mut paths = ctx.owned(scope, n);
        if scope == Scope::Global && !opts.harnesses.is_empty() {
            let prefixes: Vec<String> = opts
                .harnesses
                .iter()
                .filter_map(|h| global_rel(h))
                .map(|t| format!("{t}/"))
                .collect();
            paths.retain(|p| prefixes.iter().any(|t| p.starts_with(t.as_str())));
        }
        if !paths.is_empty() {
            plan.add(Verb::Remove, n, "-", paths, "");
        }
    }
    Ok(plan)
}

/// The plan as `install.sh` prints it. `can_ask_remove`: removals the user
/// didn't name will happen (confirmation or `--yes`).
pub fn show(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    plan: &Plan,
    can_ask_remove: bool,
) -> Vec<String> {
    let root = ctx.scope_root(scope).expect("scope root");
    let mut out = Vec::new();
    for s in &plan.steps {
        match s.verb {
            Verb::Write => {
                let update = matches!(
                    s.note.as_str(),
                    "outdated" | "modified" | "missing" | "dangling"
                );
                for p in &s.paths {
                    let how = if update {
                        "update"
                    } else if form_for(ctx, opts, scope, &root.join(p), &s.name) == Form::Link {
                        "link"
                    } else {
                        "copy"
                    };
                    let req = if s.reason == "dependency" {
                        "  (required)"
                    } else {
                        ""
                    };
                    out.push(format!("  {how:<7} {}{req}", disp(scope, p)));
                }
            }
            Verb::Remove | Verb::AskRemove => {
                if s.verb == Verb::AskRemove && !can_ask_remove {
                    out.push(format!(
                        "  {:<7} {} ({}): remove it with --remove {}",
                        "keep", s.name, s.note, s.name
                    ));
                } else {
                    let note = if s.note.is_empty() {
                        String::new()
                    } else {
                        format!("  ({})", s.note)
                    };
                    for p in &s.paths {
                        out.push(format!("  {:<7} {}{note}", "remove", disp(scope, p)));
                    }
                }
            }
            Verb::Promote => {
                out.push(format!(
                    "  {:<7} {} now counts as installed by you, not as a requirement",
                    "keep", s.name
                ));
            }
            Verb::Skip => out.push(format!("  {:<7} {}: {}", "skip", s.name, s.note)),
            Verb::Ok => out.push(format!("  {:<7} {}: {}", "ok", s.name, s.note)),
        }
    }
    out
}

#[derive(Debug, Default)]
pub struct Applied {
    /// Components copied or linked.
    pub wrote: Vec<String>,
    /// Something was removed.
    pub changed: bool,
    /// Something planned couldn't be done (exit status 1).
    pub skipped: bool,
    /// Lines to show.
    pub messages: Vec<String>,
}

/// Removes a file, folder, or link (`rm -rf`); a missing path is fine.
fn remove_any(path: &Path) -> io::Result<()> {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if meta.file_type().is_symlink() {
        // A directory symlink on Windows is removed as a directory.
        fs::remove_file(path).or_else(|_| fs::remove_dir(path))
    } else if meta.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    if target.is_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

fn with_path(path: &Path, e: io::Error) -> io::Error {
    io::Error::new(e.kind(), format!("{}: {e}", path.display()))
}

/// Folders a project removal emptied.
fn tidy(root: &Path) {
    for d in [
        ".agents/skills",
        ".agents",
        ".claude/skills",
        ".claude/agents",
        ".claude",
    ] {
        let _ = fs::remove_dir(root.join(d));
    }
}

fn write_component(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    root: &Path,
    c: &Component,
    step: &Step,
) -> io::Result<()> {
    let content = ctx.source.content(&c.path).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is missing from the toolkit", c.path),
        )
    })?;
    let forms: Vec<Form> = step
        .paths
        .iter()
        .map(|p| form_for(ctx, opts, scope, &root.join(p), &c.name))
        .collect();
    let hash = if forms.iter().all(|f| *f == Form::Link) {
        "-".to_owned()
    } else {
        hash::content(&content)
    };
    for (p, form) in step.paths.iter().zip(forms) {
        let full = root.join(p);
        remove_any(&full).map_err(|e| with_path(&full, e))?;
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| with_path(parent, e))?;
        }
        match form {
            Form::Link => {
                let target = ctx
                    .source
                    .link_target(&c.path)
                    .expect("links need --source");
                symlink(&target, &full).map_err(|e| with_path(&full, e))?;
            }
            Form::Copy => write_content(&full, &content).map_err(|e| with_path(&full, e))?,
        }
    }
    manifest::put(
        root,
        &c.kind,
        &c.name,
        &ctx.version,
        &hash,
        &step.reason,
        &step.paths,
    )
}

/// Carries out the plan. `can_ask_remove` as for [`show`].
pub fn apply(
    ctx: &Ctx,
    opts: &Options,
    scope: Scope,
    plan: &Plan,
    can_ask_remove: bool,
) -> io::Result<Applied> {
    let root = ctx.scope_root(scope).expect("scope root");
    let mut done = Applied::default();
    for s in &plan.steps {
        match s.verb {
            Verb::Write => {
                let Some(c) = ctx.component(&s.name) else {
                    done.messages.push(format!(
                        "  {:<7} {}: not in the catalog any more",
                        "skip", s.name
                    ));
                    done.skipped = true;
                    continue;
                };
                write_component(ctx, opts, scope, root, c, s)?;
                done.wrote.push(s.name.clone());
            }
            Verb::Remove | Verb::AskRemove => {
                if s.verb == Verb::AskRemove && !can_ask_remove {
                    continue;
                }
                for p in &s.paths {
                    let full = root.join(p);
                    remove_any(&full).map_err(|e| with_path(&full, e))?;
                }
                if let Some(row) = manifest::row(root, &s.name) {
                    let left: Vec<String> = row
                        .paths()
                        .into_iter()
                        .filter(|p| !s.paths.contains(p))
                        .collect();
                    if left.is_empty() {
                        manifest::drop(root, &s.name)?;
                    } else {
                        manifest::put(
                            root,
                            &row.kind,
                            &s.name,
                            &ctx.version,
                            &row.hash,
                            &row.reason,
                            &left,
                        )?;
                    }
                }
                done.changed = true;
            }
            Verb::Promote => {
                if let Some(row) = manifest::row(root, &s.name) {
                    manifest::put(
                        root,
                        &row.kind,
                        &s.name,
                        &ctx.version,
                        &row.hash,
                        "explicit",
                        &row.paths(),
                    )?;
                }
            }
            Verb::Skip | Verb::Ok => {}
        }
    }
    if scope == Scope::Project {
        tidy(root);
    }
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::source::Source;
    use std::path::PathBuf;

    struct Fixture {
        _tmp: tempfile::TempDir,
        project: PathBuf,
        home: PathBuf,
    }

    fn fixture() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let project = tmp.path().join("project");
        let home = tmp.path().join("home");
        fs::create_dir_all(project.join(".git")).unwrap();
        fs::create_dir_all(&home).unwrap();
        Fixture {
            _tmp: tmp,
            project,
            home,
        }
    }

    fn ctx(f: &Fixture) -> Ctx {
        let source = Source::Embedded;
        let catalog = Catalog::parse(&source.catalog_text().unwrap());
        Ctx {
            version: source.version(),
            source,
            catalog,
            root: Some(f.project.clone()),
            home: f.home.clone(),
        }
    }

    fn copy_opts() -> Options {
        Options {
            force: false,
            mode_arg: None,
            mode: Mode::Copy,
            harnesses: Vec::new(),
        }
    }

    fn install(ctx: &Ctx, scope: Scope, names: &[&str]) {
        let names: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        let plan = plan_install(ctx, &copy_opts(), scope, &names, false).unwrap();
        apply(ctx, &copy_opts(), scope, &plan, true).unwrap();
    }

    #[test]
    fn built_in_components_install_into_a_project() {
        let f = fixture();
        let ctx = ctx(&f);
        install(&ctx, Scope::Project, &["code-reviewer"]);
        assert_eq!(ctx.status(Scope::Project, "code-reviewer"), "installed");
        assert_eq!(ctx.status(Scope::Project, "code-review"), "installed");
        assert!(
            f.project
                .join(".claude/skills/code-review/references/rubric.md")
                .is_file()
        );
        assert_eq!(
            manifest::row(&f.project, "code-review").unwrap().reason,
            "dependency"
        );
        assert_eq!(
            manifest::row(&f.project, "code-reviewer").unwrap().reason,
            "explicit"
        );
    }

    #[test]
    fn without_a_toolkit_folder_global_installs_are_copies() {
        let f = fixture();
        let ctx = ctx(&f);
        install(&ctx, Scope::Global, &["adr-planning"]);
        let path = f.home.join(".agents/skills/adr-planning");
        assert!(path.is_dir() && !is_symlink(&path));
        assert_ne!(manifest::row(&f.home, "adr-planning").unwrap().hash, "-");
        assert_eq!(ctx.status(Scope::Global, "adr-planning"), "installed");
    }

    #[test]
    fn a_link_recorded_by_install_sh_is_restored_as_a_copy_without_a_toolkit_folder() {
        let f = fixture();
        let ctx = ctx(&f);
        let paths = [".agents/skills/adr-planning".to_owned()];
        manifest::put(
            &f.home,
            "skill",
            "adr-planning",
            "0.1.0",
            "-",
            "explicit",
            &paths,
        )
        .unwrap();
        let full = f.home.join(&paths[0]);
        assert_eq!(
            form_for(&ctx, &copy_opts(), Scope::Global, &full, "adr-planning"),
            Form::Copy
        );
        assert_eq!(ctx.status(Scope::Global, "adr-planning"), "missing");
        let plan = plan_update(&ctx, &copy_opts(), Scope::Global, &[]).unwrap();
        apply(&ctx, &copy_opts(), Scope::Global, &plan, false).unwrap();
        assert!(full.is_dir() && !is_symlink(&full));
        assert_eq!(ctx.status(Scope::Global, "adr-planning"), "installed");
    }

    #[test]
    fn retired_and_orphaned_removals_wait_for_a_confirmation() {
        let f = fixture();
        let ctx = ctx(&f);
        install(&ctx, Scope::Project, &["code-reviewer"]);
        let plan = plan_remove(
            &ctx,
            &copy_opts(),
            Scope::Project,
            &["code-reviewer".to_owned()],
        )
        .unwrap();
        apply(&ctx, &copy_opts(), Scope::Project, &plan, false).unwrap();
        assert_eq!(ctx.status(Scope::Project, "code-review"), "installed");
        let plan = plan_remove(&ctx, &copy_opts(), Scope::Project, &[]).unwrap();
        apply(&ctx, &copy_opts(), Scope::Project, &plan, true).unwrap();
        assert!(
            !f.project.join(".agents").exists(),
            "manifest and empty folders are removed"
        );
    }
}
