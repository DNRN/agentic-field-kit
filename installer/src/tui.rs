//! The terminal UI (ADR 0009, plan R3): a component table with status per
//! scope, marks for install (+), remove (-), and update (~), a review of the
//! plan, and applying it through the engine. The UI never writes files itself.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Cell as TableCell, Clear, Paragraph, Row as TableRow, Table, TableState, Wrap,
};
use ratatui::{DefaultTerminal, Frame};

use crate::brand;
use crate::engine::{self, Action, Options};
use crate::githook::{self, State};
use crate::list;
use crate::manifest;
use crate::scope::{Scope, detected_harnesses, plugin_on};
use crate::status::{Ctx, is_ours};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Install,
    Remove,
    Update,
}

impl Mark {
    fn symbol(self) -> &'static str {
        match self {
            Mark::Install => "+",
            Mark::Remove => "-",
            Mark::Update => "~",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RowInfo {
    pub name: String,
    pub kind: String,
    pub summary: String,
    pub project: String,
    pub global: String,
}

/// The plan under review. `y` applies it only if nothing blocks it, it isn't
/// a dry run, and every line has been on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub lines: Vec<String>,
    /// Each action's plan as reviewed; applying re-plans each action after
    /// the ones before it and stops if the plan no longer matches.
    pub reviewed: Vec<(Action, Vec<String>)>,
    /// Why the plan can't be applied, if it can't.
    pub blocked: Option<String>,
    pub scroll: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    List,
    Review(Review),
    Result(Vec<String>),
    Help,
}

pub struct App {
    pub ctx: Ctx,
    /// Options for the global scope (`--harness`, `--mode`, `--force`); the
    /// project scope ignores harness and mode.
    pub opts: Options,
    pub scope: Scope,
    /// `--dry-run`: plans are shown, never applied.
    pub dry_run: bool,
    /// Why there is no project, when there is none.
    pub why: Option<String>,
    pub rows: Vec<RowInfo>,
    pub cursor: usize,
    pub marks: BTreeMap<String, Mark>,
    pub screen: Screen,
    pub message: String,
    pub quit: bool,
    /// Every line applied this session, printed after the UI closes.
    pub log: Vec<String>,
    /// Something applied this session couldn't be done (exit status 1).
    pub failed: bool,
    /// Set while drawing: the review's wrapped line count and visible rows,
    /// and whether its last line has been on screen.
    review_rows: Cell<(usize, usize)>,
    review_seen_end: Cell<bool>,
}

impl App {
    pub fn new(ctx: Ctx, opts: Options, global: bool, dry_run: bool, why: Option<String>) -> App {
        let scope = if global || ctx.root.is_none() {
            Scope::Global
        } else {
            Scope::Project
        };
        let message = match (&ctx.root, &why) {
            (None, Some(why)) if !global => format!("{why}. Showing the global scope."),
            _ => String::new(),
        };
        let mut app = App {
            ctx,
            opts,
            scope,
            dry_run,
            why,
            rows: Vec::new(),
            cursor: 0,
            marks: BTreeMap::new(),
            screen: Screen::List,
            message,
            quit: false,
            log: Vec::new(),
            failed: false,
            review_rows: Cell::new((0, 0)),
            review_seen_end: Cell::new(false),
        };
        app.refresh();
        app
    }

    /// Re-reads every status from disk.
    pub fn refresh(&mut self) {
        let ctx = &self.ctx;
        self.rows = list::rows(ctx)
            .into_iter()
            .map(|name| {
                let (kind, summary) = match ctx.component(&name) {
                    Some(c) => (c.kind.clone(), c.summary.clone()),
                    None => {
                        let row = ctx
                            .root
                            .as_deref()
                            .and_then(|r| manifest::row(r, &name))
                            .or_else(|| manifest::row(&ctx.home, &name));
                        (
                            row.map(|r| r.kind).unwrap_or_default(),
                            "No longer in the toolkit.".into(),
                        )
                    }
                };
                let project = if ctx.root.is_some() {
                    ctx.status(Scope::Project, &name)
                } else {
                    "n/a".into()
                };
                let global = ctx.status(Scope::Global, &name);
                RowInfo {
                    name,
                    kind,
                    summary,
                    project,
                    global,
                }
            })
            .collect();
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
    }

    fn status(&self, row: &RowInfo) -> String {
        match self.scope {
            Scope::Project => row.project.clone(),
            Scope::Global => row.global.clone(),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if matches!(self.screen, Screen::Review(_)) {
            self.on_review_key(key.code);
            return;
        }
        match self.screen {
            Screen::List => self.on_list_key(key.code),
            Screen::Result(_) => match key.code {
                KeyCode::Char('q') => self.quit = true,
                _ => self.screen = Screen::List,
            },
            Screen::Help | Screen::Review(_) => self.screen = Screen::List,
        }
    }

    fn on_review_key(&mut self, code: KeyCode) {
        let (total, visible) = self.review_rows.get();
        let last = total.saturating_sub(visible);
        let Screen::Review(review) = &mut self.screen else {
            return;
        };
        match code {
            KeyCode::Up | KeyCode::Char('k') => review.scroll = review.scroll.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => review.scroll = (review.scroll + 1).min(last),
            KeyCode::PageUp => review.scroll = review.scroll.saturating_sub(visible.max(1)),
            KeyCode::PageDown => review.scroll = (review.scroll + visible.max(1)).min(last),
            KeyCode::Home => review.scroll = 0,
            KeyCode::End => review.scroll = last,
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let blocked = review.blocked.clone();
                if self.dry_run {
                    self.screen = Screen::List;
                    self.message = "Dry run: nothing changed.".into();
                } else if let Some(why) = blocked {
                    self.message = why;
                } else if !self.review_seen_end.get() {
                    self.message = "Scroll to the end of the plan first (↓ or End).".into();
                } else {
                    self.apply();
                }
            }
            _ => {
                self.screen = Screen::List;
                self.message = "Nothing changed.".into();
            }
        }
    }

    fn on_list_key(&mut self, code: KeyCode) {
        self.message.clear();
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.quit = true,
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(self.rows.len().saturating_sub(1))
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.rows.len().saturating_sub(1),
            KeyCode::Char(' ') => self.toggle(self.cursor),
            KeyCode::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if i < self.rows.len() {
                    self.cursor = i;
                    self.toggle(i);
                }
            }
            KeyCode::Char('u') => self.mark_updates(),
            KeyCode::Char('g') => self.switch_scope(),
            KeyCode::Enter => self.review(),
            KeyCode::Char('?') => self.screen = Screen::Help,
            _ => {}
        }
    }

    /// A number or space marks an uninstalled component for install and an
    /// installed one for removal; marking again clears it.
    pub fn toggle(&mut self, i: usize) {
        let Some(row) = self.rows.get(i).cloned() else {
            return;
        };
        if self.marks.remove(&row.name).is_some() {
            return;
        }
        let st = self.status(&row);
        match st.as_str() {
            "-" if self.ctx.component(&row.name).is_none() => {
                self.message = format!("{}: no longer in the toolkit", row.name);
            }
            "-" => {
                self.marks.insert(row.name, Mark::Install);
            }
            "foreign" => {
                self.message = format!(
                    "{}: exists but was not installed by agentic-toolkit; left alone",
                    row.name
                )
            }
            "plugin" => {
                self.message = format!("{}: provided by the agentic-toolkit plugin", row.name)
            }
            "n/a" => {
                let plugin_only = self
                    .ctx
                    .component(&row.name)
                    .is_some_and(|c| c.harnesses == "plugin");
                let hook = self
                    .ctx
                    .component(&row.name)
                    .is_some_and(|c| c.kind == "hook");
                self.message = if plugin_only {
                    format!("{}: ships with the Claude Code plugin only", row.name)
                } else if hook {
                    format!(
                        "{}: git hooks are per repository; install it in a project",
                        row.name
                    )
                } else {
                    let scope = if self.scope == Scope::Project {
                        "project"
                    } else {
                        "global"
                    };
                    format!("{}: not available in the {scope} scope", row.name)
                };
            }
            _ => {
                self.marks.insert(row.name, Mark::Remove);
            }
        }
    }

    /// Marks every component `--update` would change.
    pub fn mark_updates(&mut self) {
        let names: Vec<String> = self
            .rows
            .iter()
            .filter(|r| {
                matches!(
                    self.status(r).as_str(),
                    "outdated" | "missing" | "dangling" | "retired" | "inactive"
                )
            })
            .map(|r| r.name.clone())
            .collect();
        if names.is_empty() {
            self.message = "Nothing to update.".into();
        }
        for n in names {
            self.marks.insert(n, Mark::Update);
        }
    }

    pub fn switch_scope(&mut self) {
        match self.scope {
            Scope::Global if self.ctx.root.is_none() => {
                self.message = self.why.clone().unwrap_or_default();
                return;
            }
            Scope::Global => self.scope = Scope::Project,
            Scope::Project => self.scope = Scope::Global,
        }
        self.marks.clear();
    }

    /// The marked actions. An update of a component the install step already
    /// writes is left out, so each write is planned, shown, and applied once.
    fn requests(&self) -> Vec<(Action, Vec<String>)> {
        let marked = |want: Mark| -> Vec<String> {
            self.marks
                .iter()
                .filter(|(_, m)| **m == want)
                .map(|(n, _)| n.clone())
                .collect()
        };
        let installs = marked(Mark::Install);
        let mut updates = marked(Mark::Update);
        if !installs.is_empty() {
            let opts = self.scope_opts();
            if let Ok(plan) = engine::plan_action(
                &self.ctx,
                &opts,
                Action::Install,
                self.scope,
                &installs,
                false,
            ) {
                let written: Vec<&str> = plan
                    .steps
                    .iter()
                    .filter(|s| s.verb == engine::Verb::Write)
                    .map(|s| s.name.as_str())
                    .collect();
                updates.retain(|n| !written.contains(&n.as_str()));
            }
        }
        [
            (Action::Install, installs),
            (Action::Update, updates),
            (Action::Remove, marked(Mark::Remove)),
        ]
        .into_iter()
        .filter(|(_, names)| !names.is_empty())
        .collect()
    }

    fn scope_opts(&self) -> Options {
        match self.scope {
            Scope::Global => self.opts.clone(),
            Scope::Project => Options {
                harnesses: Vec::new(),
                mode_arg: None,
                ..self.opts.clone()
            },
        }
    }

    /// Everything `names` require, directly or through other requirements.
    fn requirements(&self, names: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut todo: Vec<String> = names.to_vec();
        while let Some(n) = todo.pop() {
            for r in self
                .ctx
                .component(&n)
                .map(|c| c.requires.clone())
                .unwrap_or_default()
            {
                if !out.contains(&r) {
                    out.push(r.clone());
                    todo.push(r);
                }
            }
        }
        out
    }

    /// The plans for the marks; confirming applies them, so removals the user
    /// didn't name are shown as removals.
    pub fn review(&mut self) {
        let requests = self.requests();
        if requests.is_empty() {
            self.message = "Nothing marked.".into();
            return;
        }
        let opts = self.scope_opts();
        let mut lines = Vec::new();
        let mut reviewed = Vec::new();
        let mut blocked = None;
        let installs: Vec<String> = requests
            .iter()
            .filter(|(a, _)| *a == Action::Install)
            .flat_map(|(_, n)| n.clone())
            .collect();
        let needed = self.requirements(&installs);
        for (action, names) in &requests {
            if *action == Action::Remove
                && let Some(n) = names.iter().find(|n| needed.contains(n))
            {
                let why = format!(
                    "! {n} is needed by a component marked for install; unmark one of them"
                );
                lines.push(why.clone());
                blocked = Some(why);
                continue;
            }
            match engine::plan_action(&self.ctx, &opts, *action, self.scope, names, false) {
                Ok(plan) => {
                    let shown = engine::show(&self.ctx, &opts, self.scope, &plan, true);
                    lines.extend(shown.iter().cloned());
                    reviewed.push((*action, shown));
                }
                Err(e) => {
                    let why = format!("! {e}");
                    lines.push(why.clone());
                    blocked = Some(why);
                }
            }
        }
        self.review_seen_end.set(false);
        self.screen = Screen::Review(Review {
            lines,
            reviewed,
            blocked,
            scroll: 0,
        });
    }

    /// Plans and applies each reviewed action in turn, as `--yes` would. An
    /// action whose plan differs from the reviewed one is not applied.
    pub fn apply(&mut self) {
        let Screen::Review(review) = &self.screen else {
            return;
        };
        let reviewed = review.reviewed.clone();
        let opts = self.scope_opts();
        let mut lines = Vec::new();
        let mut ok = true;
        let mut touched_project = false;
        for (action, names) in self.requests() {
            let plan =
                match engine::plan_action(&self.ctx, &opts, action, self.scope, &names, false) {
                    Ok(plan) => plan,
                    Err(e) => {
                        lines.push(format!("! {e}"));
                        ok = false;
                        continue;
                    }
                };
            let shown = engine::show(&self.ctx, &opts, self.scope, &plan, true);
            if reviewed.iter().find(|(a, _)| *a == action).map(|(_, s)| s) != Some(&shown) {
                lines.push(format!(
                    "! the {} plan changed after the steps before it; not applied, review again",
                    match action {
                        Action::Install => "install",
                        Action::Update => "update",
                        Action::Remove => "remove",
                    }
                ));
                ok = false;
                continue;
            }
            lines.extend(shown);
            ok &= !plan.skipped;
            if plan.has_changes() {
                match engine::apply(&self.ctx, &opts, self.scope, &plan, true) {
                    Ok(done) => {
                        lines.extend(done.messages.iter().cloned());
                        ok &= !done.skipped;
                        touched_project |= self.scope == Scope::Project
                            && (!done.wrote.is_empty() || done.changed);
                    }
                    Err(e) => {
                        lines.push(format!("! {e}"));
                        ok = false;
                    }
                }
            }
            if self.scope == Scope::Project && action == Action::Install {
                for (name, g) in engine::both_scopes(&self.ctx, &plan) {
                    lines.push(engine::both_scopes_warning(&name, &g));
                    if g != "foreign" {
                        lines.push(format!(
                            "  Remove it with: agentic-toolkit --global --remove {name}"
                        ));
                    }
                }
            }
        }
        if touched_project && let Some(root) = self.ctx.root.as_deref() {
            lines.push(engine::commit_hint(root));
        }
        lines.push(if ok {
            "done".into()
        } else {
            "done with skips".into()
        });
        self.log.extend(lines.iter().cloned());
        self.failed |= !ok;
        self.marks.clear();
        self.refresh();
        self.screen = Screen::Result(lines);
    }
}

// ---- drawing -------------------------------------------------------------------

fn colored() -> bool {
    std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
}

/// The colour scheme: the brand oranges for chrome, and status colours in
/// mid tones that stay apart from them. xterm 256-colour indices of middle
/// lightness, so each reads on dark and on light backgrounds. `plain` is the
/// style under `NO_COLOR`.
fn themed(color: Style, plain: Style) -> Style {
    if colored() { color } else { plain }
}

/// Amber, for titles, headings, and keys.
const AMBER: Color = Color::Indexed(208);
/// Tan, for labels.
const TAN: Color = Color::Indexed(137);
const GREEN: Color = Color::Indexed(71);
const YELLOW: Color = Color::Indexed(178);
const RED: Color = Color::Indexed(167);
const BLUE: Color = Color::Indexed(67);

fn status_style(status: &str) -> Style {
    let fg = |c| themed(Style::default().fg(c), Style::default());
    match status {
        "installed" | "linked" => fg(GREEN),
        "outdated" | "missing" | "dangling" | "inactive" => fg(YELLOW),
        "modified" | "retired" | "foreign" => fg(RED),
        "plugin" => fg(BLUE),
        _ => Style::default().add_modifier(Modifier::DIM),
    }
}

fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// The name, the component in the details, marks.
fn accent() -> Style {
    themed(bold().fg(brand::ORANGE), bold())
}

/// Box titles, headings, and the keys in the key line.
fn title() -> Style {
    themed(bold().fg(AMBER), bold())
}

/// Labels in the header and the table's column names.
fn label() -> Style {
    themed(Style::default().fg(TAN), Style::default())
}

fn border() -> Style {
    themed(Style::default().fg(brand::RUST), Style::default())
}

/// The row under the cursor.
fn selected() -> Style {
    themed(
        bold().bg(brand::BROWN).fg(Color::Indexed(230)),
        Style::default().add_modifier(Modifier::REVERSED),
    )
}

fn warning() -> Style {
    themed(Style::default().fg(YELLOW), Style::default())
}

fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

fn boxed(name: &str) -> Block<'static> {
    let block = Block::bordered()
        .border_style(border())
        .title_style(title());
    if name.is_empty() {
        block
    } else {
        block.title(format!(" {name} "))
    }
}

/// "↑↓ move · q quit" with each key in the title style and the rest dim.
fn key_line(keys: &str) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, part) in keys.split(" · ").enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", dim()));
        }
        let (key, what) = part.split_once(' ').unwrap_or((part, ""));
        spans.push(Span::styled(key.to_owned(), title()));
        if !what.is_empty() {
            spans.push(Span::styled(format!(" {what}"), dim()));
        }
    }
    Line::from(spans)
}

fn header(app: &App) -> Vec<Line<'static>> {
    let ctx = &app.ctx;
    let project = match &ctx.root {
        Some(root) => root.display().to_string(),
        None => format!(
            "none ({})",
            app.why
                .as_deref()
                .unwrap_or_default()
                .split(':')
                .next()
                .unwrap_or_default()
        ),
    };
    let plugin = if ctx.root.as_deref().is_some_and(plugin_on) {
        "enabled for this project"
    } else if plugin_on(&ctx.home) {
        "enabled for your user"
    } else {
        "not enabled"
    };
    let field = |name: &'static str, value: String| {
        Line::from(vec![Span::styled(name, label()), Span::raw(value)])
    };
    vec![
        Line::from(vec![
            Span::styled(brand::NAME, accent()),
            Span::styled(" · ", dim()),
            Span::raw(format!("agentic-toolkit {}", ctx.version)),
        ]),
        Line::from(""),
        field("Project  ", project),
        field("Plugin   ", plugin.to_owned()),
        field("Harness  ", detected_harnesses(&ctx.home).join(", ")),
    ]
}

fn details(app: &App) -> Text<'static> {
    let Some(row) = app.rows.get(app.cursor) else {
        return Text::default();
    };
    let ctx = &app.ctx;
    let mut lines = vec![
        Line::from(Span::styled(row.name.clone(), accent())),
        Line::from(row.summary.clone()),
        Line::from(""),
    ];
    if let Some(c) = ctx.component(&row.name) {
        if !c.requires.is_empty() {
            lines.push(Line::from(format!("Needs {}.", c.requires.join(", "))));
        }
        let needed_by: Vec<&str> = ctx
            .catalog
            .rows
            .iter()
            .filter(|o| o.requires.contains(&row.name))
            .map(|o| o.name.as_str())
            .collect();
        if !needed_by.is_empty() {
            lines.push(Line::from(format!("Needed by {}.", needed_by.join(", "))));
        }
        match c.harnesses.as_str() {
            "claude" => lines.push(Line::from("Claude Code only.")),
            "plugin" => lines.push(Line::from("Ships with the Claude Code plugin only.")),
            _ => {}
        }
    }
    if is_ours(&row.project) && !matches!(row.global.as_str(), "-" | "n/a" | "plugin") {
        lines.push(Line::from(Span::styled(
            "! also installed globally: Codex lists it twice.",
            warning(),
        )));
    }
    let scope_name = if app.scope == Scope::Project {
        "project"
    } else {
        "global"
    };
    let paths = if is_ours(&app.status(row)) {
        ctx.owned(app.scope, &row.name)
    } else {
        engine::layout(ctx, &app.scope_opts(), app.scope, &row.name)
    };
    if !paths.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Paths ({scope_name})"),
            title(),
        )));
        for p in paths {
            lines.push(Line::from(format!("  {}", engine::disp(app.scope, &p))));
        }
    }
    if app.scope == Scope::Project
        && let (Some(c), Some(root)) = (ctx.component(&row.name), ctx.root.as_deref())
        && c.kind == "hook"
        && c.harnesses != "plugin"
    {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Git hook", title())));
        let st = githook::state(root, &row.name);
        let shown = st
            .path()
            .map(|p| githook::disp(root, p))
            .unwrap_or_default();
        let text = match &st {
            State::NoRepo => "  not a git repository".to_owned(),
            State::Absent(_) => format!("  {shown}: not there yet"),
            State::Ours(_) if st.enabled() => format!("  {shown}: runs it"),
            State::Ours(_) => format!("  {shown}: not executable"),
            State::Runs(_) => format!("  {shown} (yours): runs it"),
            State::Foreign(_) => format!("  {shown} is not ours. Add:"),
        };
        lines.push(Line::from(text));
        if matches!(st, State::Foreign(_)) {
            lines.push(Line::from(format!("  {}", githook::line_to_add(&row.name))));
        }
    }
    Text::from(lines)
}

fn table(app: &App) -> Table<'static> {
    let scope_style = |s: Scope| {
        if app.scope == s {
            accent().add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default()
        }
    };
    let head = TableRow::new(vec![
        TableCell::from(" "),
        TableCell::from("#"),
        TableCell::from("component"),
        TableCell::from("kind"),
        TableCell::from(Span::styled("project", scope_style(Scope::Project))),
        TableCell::from(Span::styled("global", scope_style(Scope::Global))),
    ])
    .style(label().add_modifier(Modifier::BOLD));
    let rows: Vec<TableRow> = app
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mark = app.marks.get(&r.name).map(|m| m.symbol()).unwrap_or(" ");
            TableRow::new(vec![
                TableCell::from(Span::styled(mark.to_owned(), accent())),
                TableCell::from(Span::styled(format!("{}", i + 1), dim())),
                TableCell::from(r.name.clone()),
                TableCell::from(r.kind.clone()),
                TableCell::from(Span::styled(r.project.clone(), status_style(&r.project))),
                TableCell::from(Span::styled(r.global.clone(), status_style(&r.global))),
            ])
        })
        .collect();
    Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(14),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(9),
        ],
    )
    .header(head)
    .row_highlight_style(selected())
    .block(boxed("Components"))
}

/// Lines cut to `width` characters, so the popup knows how many rows it needs.
fn wrap(lines: &[String], width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for line in lines {
        let chars: Vec<char> = line.chars().collect();
        if chars.is_empty() {
            out.push(String::new());
        }
        for chunk in chars.chunks(width) {
            out.push(chunk.iter().collect());
        }
    }
    out
}

/// A centred box with `lines` from `scroll` on, and `keys` at the bottom.
/// Returns the wrapped line count and how many rows were visible.
fn popup(
    frame: &mut Frame,
    title: &str,
    lines: &[String],
    keys: &str,
    scroll: usize,
) -> (usize, usize) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).min(100);
    let wrapped = wrap(lines, width.saturating_sub(2) as usize);
    let height = (wrapped.len() as u16 + 4)
        .min(area.height.saturating_sub(2))
        .max(5);
    let visible = height.saturating_sub(4) as usize;
    let scroll = scroll.min(wrapped.len().saturating_sub(visible));
    let [v] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [rect] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(v);
    frame.render_widget(Clear, rect);
    let mut text: Vec<Line> = wrapped
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|l| Line::from(l.clone()))
        .collect();
    text.push(Line::from(""));
    let more = if scroll + visible < wrapped.len() {
        format!("↓ {} more · ", wrapped.len() - scroll - visible)
    } else {
        String::new()
    };
    text.push(Line::from(Span::styled(format!("{more}{keys}"), accent())));
    let popup_border = themed(Style::default().fg(brand::ORANGE), Style::default());
    frame.render_widget(
        Paragraph::new(text).block(boxed(title).border_style(popup_border)),
        rect,
    );
    (wrapped.len(), visible)
}

const KEYS: &str = "↑↓ move · space/1-9 mark · u update · g scope · enter review · ? help · q quit";

const HELP: [&str; 9] = [
    "Marks: + install (with what it needs), - remove, ~ update.",
    "space or a row number marks the component under it; again clears the mark.",
    "u marks everything --update would change: outdated, missing, retired.",
    "g switches between the project and the global scope (marks are cleared).",
    "enter shows the plan; y applies it, any other key goes back.",
    "",
    "Project installs are copies in .agents/ and .claude/; commit them.",
    "Locally edited copies are never overwritten or removed (use --force).",
    "Same skill in both scopes: Codex lists it twice; Claude Code uses the global one.",
];

/// Draws the whole screen, with the Entropy Fox when there is room for it.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let show_logo = brand::logo_fits(area);
    let header_height = if show_logo { brand::LOGO_HEIGHT } else { 5 };
    let [top, body, status, keys] = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let text_area = if show_logo {
        let [logo_area, _gap, text] = Layout::horizontal([
            Constraint::Length(brand::LOGO_WIDTH),
            Constraint::Length(2),
            Constraint::Min(10),
        ])
        .areas(top);
        frame.render_widget(Paragraph::new(brand::logo(colored())), logo_area);
        // The five header lines start a third of the way down, with room
        // below for the lines that wrap beside the logo at 80 columns.
        let [_, below] = Layout::vertical([
            Constraint::Length(brand::LOGO_HEIGHT / 3),
            Constraint::Min(1),
        ])
        .areas(text);
        below
    } else {
        top
    };
    let lines = Paragraph::new(header(app));
    let lines = if show_logo {
        lines.wrap(Wrap { trim: false })
    } else {
        lines
    };
    frame.render_widget(lines, text_area);

    let [left, right] =
        Layout::horizontal([Constraint::Length(47), Constraint::Min(20)]).areas(body);
    let mut state = TableState::default().with_selected(Some(app.cursor));
    frame.render_stateful_widget(table(app), left, &mut state);
    frame.render_widget(
        Paragraph::new(details(app))
            .wrap(Wrap { trim: false })
            .block(boxed("")),
        right,
    );

    frame.render_widget(Paragraph::new(app.message.clone()), status);
    frame.render_widget(Paragraph::new(key_line(KEYS)), keys);

    match &app.screen {
        Screen::List => {}
        Screen::Review(review) => {
            let keys = if app.dry_run {
                "dry run: any key back"
            } else if review.blocked.is_some() {
                "can't apply: any key back"
            } else {
                "↑↓ scroll · y apply · any other key back"
            };
            let (total, visible) = popup(frame, "Plan", &review.lines, keys, review.scroll);
            app.review_rows.set((total, visible));
            if review.scroll.min(total.saturating_sub(visible)) + visible >= total {
                app.review_seen_end.set(true);
            }
        }
        Screen::Result(lines) => {
            popup(frame, "Done", lines, "any key back · q quit", 0);
        }
        Screen::Help => {
            let lines: Vec<String> = HELP.iter().map(|s| (*s).to_owned()).collect();
            popup(frame, "Help", &lines, "any key back", 0);
        }
    }
}

/// Runs the UI until the user quits; the terminal is restored on every exit,
/// including a panic. Returns the lines applied, for the scrollback, and
/// whether everything applied could be done.
pub fn run(mut app: App) -> io::Result<(Vec<String>, bool)> {
    let mut terminal: DefaultTerminal = ratatui::init();
    let result = (|| -> io::Result<()> {
        while !app.quit {
            terminal.draw(|frame| draw(frame, &app))?;
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
        Ok(())
    })();
    ratatui::restore();
    result.map(|()| (app.log, !app.failed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::engine::Mode;
    use crate::source::Source;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::fs;
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

    fn app_from(f: &Fixture, source: Source, dry_run: bool) -> App {
        let catalog = Catalog::parse(&source.catalog_text().unwrap());
        let ctx = Ctx {
            version: source.version(),
            source,
            catalog,
            root: Some(f.project.clone()),
            home: f.home.clone(),
        };
        let opts = Options {
            force: false,
            mode_arg: None,
            mode: Mode::Copy,
            harnesses: Vec::new(),
        };
        App::new(ctx, opts, false, dry_run, None)
    }

    fn app_with(f: &Fixture, dry_run: bool) -> App {
        app_from(f, Source::Embedded, dry_run)
    }

    fn app(f: &Fixture) -> App {
        app_with(f, false)
    }

    fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
        if from.is_dir() {
            fs::create_dir_all(to).unwrap();
            for entry in fs::read_dir(from).unwrap().flatten() {
                copy_tree(&entry.path(), &to.join(entry.file_name()));
            }
        } else {
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        }
    }

    /// A private toolkit tree, so a test can change a component "upstream".
    fn toolkit(f: &Fixture) -> PathBuf {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let dir = f.home.parent().unwrap().join("toolkit");
        for p in ["catalog.tsv", "skills", "agents", "hooks", ".claude-plugin"] {
            copy_tree(&repo.join(p), &dir.join(p));
        }
        dir
    }

    #[test]
    fn an_update_that_the_install_already_does_is_planned_once() {
        let f = fixture();
        let dir = toolkit(&f);
        let mut app = app_from(&f, Source::Dir(dir.clone()), false);
        let (review, reviewer) = (row(&app, "code-review"), row(&app, "code-reviewer"));
        press(&mut app, &format!("{review}\ny\n"));
        let skill = dir.join("skills/code-review/SKILL.md");
        let text = fs::read_to_string(&skill).unwrap() + "\nupstream change\n";
        fs::write(&skill, text).unwrap();
        app.refresh();
        assert_eq!(app.rows[review - 1].project, "outdated");
        press(&mut app, &format!("u{reviewer}\n"));
        let updates = review_lines(&app)
            .iter()
            .filter(|l| l.contains("update  .agents/skills/code-review"))
            .count();
        assert_eq!(updates, 1, "{:?}", review_lines(&app));
        press(&mut app, "y");
        assert!(!app.failed, "{:?}", app.log);
        assert_eq!(app.rows[review - 1].project, "installed");
        assert!(f.project.join(".claude/agents/code-reviewer.md").is_file());
    }

    /// Each key after a draw, as the real loop does, on a terminal of `size`.
    fn press_on(app: &mut App, keys: &str, size: (u16, u16)) {
        for c in keys.chars() {
            screen(app, size.0, size.1);
            let code = match c {
                '\n' => KeyCode::Enter,
                '↓' => KeyCode::End,
                c => KeyCode::Char(c),
            };
            app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
        }
    }

    fn press(app: &mut App, keys: &str) {
        press_on(app, keys, (100, 30));
    }

    fn row(app: &App, name: &str) -> usize {
        app.rows.iter().position(|r| r.name == name).unwrap() + 1
    }

    fn screen(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn review_lines(app: &App) -> Vec<String> {
        match &app.screen {
            Screen::Review(review) => review.lines.clone(),
            other => panic!("not reviewing: {other:?}"),
        }
    }

    #[test]
    fn mark_review_and_apply_installs() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "doc-sync");
        press(&mut app, &format!("{n}\n"));
        assert!(review_lines(&app).contains(&"  copy    .agents/skills/doc-sync".to_owned()));
        press(&mut app, "y");
        assert!(matches!(app.screen, Screen::Result(_)));
        assert!(f.project.join(".agents/skills/doc-sync/SKILL.md").is_file());
        assert_eq!(app.rows[n - 1].project, "installed");
    }

    #[test]
    fn marking_an_installed_component_removes_it() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "doc-sync");
        press(&mut app, &format!("{n}\ny"));
        press(&mut app, &format!("\n{n}\ny"));
        assert!(!f.project.join(".agents/skills/doc-sync").exists());
    }

    #[test]
    fn quitting_or_declining_changes_nothing() {
        let f = fixture();
        let mut app = app(&f);
        press(&mut app, "12\nn");
        assert_eq!(app.screen, Screen::List);
        press(&mut app, "q");
        assert!(app.quit);
        assert!(!f.project.join(".agents").exists());
    }

    #[test]
    fn g_switches_to_the_global_scope() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "adr-planning");
        press(&mut app, &format!("g{n}\ny"));
        assert!(
            f.home
                .join(".agents/skills/adr-planning/SKILL.md")
                .is_file()
        );
        assert!(!f.project.join(".agents").exists());
    }

    #[test]
    fn the_hook_installs_into_the_project_with_its_shim() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "commit-gate");
        press(&mut app, &format!("{n}\n"));
        let lines = review_lines(&app);
        assert!(
            lines.contains(&"  copy    .agents/hooks/commit-gate.sh".to_owned()),
            "{lines:?}"
        );
        assert!(
            lines.contains(&"  hook    .git/hooks/pre-commit".to_owned()),
            "{lines:?}"
        );
        press(&mut app, "y");
        assert!(!app.failed, "{:?}", app.log);
        assert!(f.project.join(".agents/hooks/commit-gate.sh").is_file());
        assert!(f.project.join(".git/hooks/pre-commit").is_file());
        assert_eq!(app.rows[n - 1].project, "installed");
    }

    #[test]
    fn the_hook_says_why_it_cant_be_global() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "commit-gate");
        press(&mut app, &format!("g{n}"));
        assert!(app.marks.is_empty());
        assert!(
            app.message.contains("git hooks are per repository"),
            "{}",
            app.message
        );
    }

    #[test]
    fn an_inactive_hook_is_marked_by_update() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "commit-gate");
        press(&mut app, &format!("{n}\ny\n"));
        fs::remove_file(f.project.join(".git/hooks/pre-commit")).unwrap();
        app.refresh();
        assert_eq!(app.rows[n - 1].project, "inactive");
        press(&mut app, "u\ny");
        assert!(!app.failed, "{:?}", app.log);
        assert!(f.project.join(".git/hooks/pre-commit").is_file());
        assert_eq!(app.rows[n - 1].project, "installed");
    }

    #[test]
    fn the_review_lists_the_requirement_that_applying_removes() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "code-reviewer");
        press(&mut app, &format!("{n}\ny\n"));
        press(&mut app, &format!("{n}\n"));
        assert!(
            review_lines(&app)
                .contains(&"  remove  .agents/skills/code-review  (no longer needed)".to_owned())
        );
        press(&mut app, "y");
        assert!(!f.project.join(".agents/skills/code-review").exists());
    }

    #[test]
    fn u_marks_what_update_would_change() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "doc-sync");
        press(&mut app, &format!("{n}\ny\n"));
        fs::remove_dir_all(f.project.join(".claude/skills/doc-sync")).unwrap();
        app.refresh();
        press(&mut app, "u\ny");
        assert!(f.project.join(".claude/skills/doc-sync/SKILL.md").is_file());
    }

    #[test]
    fn at_80_by_24_the_logo_shows_and_the_table_scrolls() {
        let f = fixture();
        let mut app = app(&f);
        let text = screen(&app, 80, 24);
        assert!(text.contains(brand::NAME));
        assert!(text.contains(FOX_LINE), "no logo at 80×24:\n{text}");
        // ADR 0012: the table keeps 7 component rows beside the logo.
        for name in app.rows.iter().take(7).map(|r| r.name.clone()) {
            assert!(text.contains(&name), "{name} missing:\n{text}");
        }
        // The rest scroll into view with the cursor.
        press_on(&mut app, "↓", (80, 24));
        let text = screen(&app, 80, 24);
        let last = app.rows.last().unwrap().name.clone();
        assert!(text.contains(&last), "{last} missing after End:\n{text}");
    }

    #[test]
    fn a_taller_screen_shows_every_row_with_the_logo() {
        let f = fixture();
        let app = app(&f);
        let height = brand::LOGO_HEIGHT + app.rows.len() as u16 + 5;
        let text = screen(&app, 80, height);
        assert!(text.contains(FOX_LINE));
        for r in &app.rows {
            assert!(
                text.contains(&r.name),
                "{} missing at 80×{height}:\n{text}",
                r.name
            );
        }
    }

    /// A line of the logo that nothing else on the screen draws.
    const FOX_LINE: &str = "_/##########################=-._";

    #[test]
    fn below_80_by_24_the_name_stays_and_the_logo_goes() {
        let f = fixture();
        let app = app(&f);
        for (width, height) in [(79, 24), (120, 23)] {
            let text = screen(&app, width, height);
            assert!(text.contains(brand::NAME), "{width}×{height}:\n{text}");
            assert!(!text.contains(FOX_LINE), "{width}×{height}:\n{text}");
        }
    }

    #[test]
    fn a_dry_run_never_applies() {
        let f = fixture();
        let mut app = app_with(&f, true);
        let n = row(&app, "doc-sync");
        press(&mut app, &format!("{n}\ny"));
        assert_eq!(app.message, "Dry run: nothing changed.");
        assert!(!f.project.join(".agents").exists());
    }

    #[test]
    fn removing_what_a_marked_install_needs_is_blocked() {
        let f = fixture();
        let mut app = app(&f);
        let review = row(&app, "code-review");
        let reviewer = row(&app, "code-reviewer");
        press(&mut app, &format!("{review}\ny"));
        press(&mut app, &format!("\n{review}{reviewer}\ny"));
        assert!(
            app.message
                .contains("code-review is needed by a component marked for install"),
            "{}",
            app.message
        );
        assert!(
            !f.project.join(".claude/agents/code-reviewer.md").exists(),
            "nothing applied"
        );
        assert!(f.project.join(".agents/skills/code-review").exists());
    }

    #[test]
    fn a_plan_taller_than_the_terminal_must_be_scrolled_before_applying() {
        let f = fixture();
        let mut app = app(&f);
        let reviewer = row(&app, "code-reviewer");
        press(&mut app, &format!("{reviewer}\ny\n"));
        let (adr, doc) = (row(&app, "adr-planning"), row(&app, "doc-sync"));
        press_on(&mut app, &format!("{adr}{doc}{reviewer}\ny"), (80, 10));
        assert!(
            app.message.starts_with("Scroll to the end"),
            "{}",
            app.message
        );
        assert!(!f.project.join(".agents/skills/adr-planning").exists());
        press_on(&mut app, "↓y", (80, 10));
        assert!(f.project.join(".agents/skills/adr-planning").exists());
        assert!(!f.project.join(".claude/agents/code-reviewer.md").exists());
    }

    #[test]
    fn failures_are_reported_for_the_exit_status() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "doc-sync");
        press(&mut app, &format!("{n}\ny\n"));
        fs::write(
            f.project.join(".agents/skills/doc-sync/SKILL.md"),
            "edited\n",
        )
        .unwrap();
        app.refresh();
        press(&mut app, &format!("{n}\ny"));
        assert!(
            app.failed,
            "a modified copy can't be removed without --force"
        );
    }
}
