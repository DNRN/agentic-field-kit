//! The terminal UI (ADR 0009, plan R3): a component table with status per
//! scope, marks for install (+), remove (-), and update (~), a review of the
//! plan, and applying it through the engine. The UI never writes files itself.

use std::collections::BTreeMap;
use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Cell, Clear, Paragraph, Row as TableRow, Table, TableState, Wrap};
use ratatui::{DefaultTerminal, Frame};

use crate::brand::{self, Logo};
use crate::engine::{self, Action, Options};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    List,
    Review(Vec<String>),
    Result(Vec<String>),
    Help,
}

pub struct App {
    pub ctx: Ctx,
    /// Options for the global scope (`--harness`, `--mode`, `--force`); the
    /// project scope ignores harness and mode.
    pub opts: Options,
    pub scope: Scope,
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
}

impl App {
    pub fn new(ctx: Ctx, opts: Options, global: bool, why: Option<String>) -> App {
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
            why,
            rows: Vec::new(),
            cursor: 0,
            marks: BTreeMap::new(),
            screen: Screen::List,
            message,
            quit: false,
            log: Vec::new(),
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
        match self.screen.clone() {
            Screen::List => self.on_list_key(key.code),
            Screen::Review(_) => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => self.apply(),
                _ => {
                    self.screen = Screen::List;
                    self.message = "Nothing changed.".into();
                }
            },
            Screen::Result(_) => match key.code {
                KeyCode::Char('q') => self.quit = true,
                _ => self.screen = Screen::List,
            },
            Screen::Help => self.screen = Screen::List,
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
                self.message = if plugin_only {
                    format!(
                        "{}: plugin only until roadmap phase 1 (portable commit gate)",
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
                    "outdated" | "missing" | "dangling" | "retired"
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

    fn requests(&self) -> Vec<(Action, Vec<String>)> {
        [Action::Install, Action::Update, Action::Remove]
            .into_iter()
            .map(|action| {
                let want = match action {
                    Action::Install => Mark::Install,
                    Action::Update => Mark::Update,
                    Action::Remove => Mark::Remove,
                };
                let names = self
                    .marks
                    .iter()
                    .filter(|(_, m)| **m == want)
                    .map(|(n, _)| n.clone())
                    .collect::<Vec<_>>();
                (action, names)
            })
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
        for (action, names) in requests {
            match engine::plan_action(&self.ctx, &opts, action, self.scope, &names, false) {
                Ok(plan) => lines.extend(engine::show(&self.ctx, &opts, self.scope, &plan, true)),
                Err(e) => lines.push(format!("! {e}")),
            }
        }
        self.screen = Screen::Review(lines);
    }

    /// Plans and applies each marked action in turn, as `--yes` would.
    pub fn apply(&mut self) {
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
            lines.extend(engine::show(&self.ctx, &opts, self.scope, &plan, true));
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
        self.marks.clear();
        self.refresh();
        self.screen = Screen::Result(lines);
    }
}

// ---- drawing -------------------------------------------------------------------

fn colored() -> bool {
    std::env::var_os("NO_COLOR").is_none_or(|v| v.is_empty())
}

fn status_style(status: &str) -> Style {
    if !colored() {
        return Style::default();
    }
    match status {
        "installed" | "linked" => Style::default().fg(Color::Green),
        "outdated" | "missing" | "dangling" => Style::default().fg(Color::Yellow),
        "modified" | "retired" | "foreign" => Style::default().fg(Color::Red),
        "plugin" => Style::default().fg(Color::Cyan),
        _ => Style::default().add_modifier(Modifier::DIM),
    }
}

fn accent() -> Style {
    if colored() {
        Style::default()
            .fg(brand::ORANGE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
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
    vec![
        Line::from(vec![
            Span::styled(brand::NAME, accent()),
            Span::raw(format!(" · agentic-toolkit {}", ctx.version)),
        ]),
        Line::from(""),
        Line::from(format!("Project  {project}")),
        Line::from(format!("Plugin   {plugin}")),
        Line::from(format!(
            "Harness  {}",
            detected_harnesses(&ctx.home).join(", ")
        )),
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
            if colored() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
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
        lines.push(Line::from(format!("Paths ({scope_name})")));
        for p in paths {
            lines.push(Line::from(format!("  {}", engine::disp(app.scope, &p))));
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
        Cell::from(" "),
        Cell::from("#"),
        Cell::from("component"),
        Cell::from("kind"),
        Cell::from(Span::styled("project", scope_style(Scope::Project))),
        Cell::from(Span::styled("global", scope_style(Scope::Global))),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));
    let rows: Vec<TableRow> = app
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mark = app.marks.get(&r.name).map(|m| m.symbol()).unwrap_or(" ");
            TableRow::new(vec![
                Cell::from(Span::styled(mark.to_owned(), accent())),
                Cell::from(format!("{}", i + 1)),
                Cell::from(r.name.clone()),
                Cell::from(r.kind.clone()),
                Cell::from(Span::styled(r.project.clone(), status_style(&r.project))),
                Cell::from(Span::styled(r.global.clone(), status_style(&r.global))),
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
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .block(Block::bordered().title(" Components "))
}

fn popup(frame: &mut Frame, title: &str, lines: &[String], keys: &str) {
    let area = frame.area();
    let width = area.width.saturating_sub(4).min(100);
    let height = (lines.len() as u16 + 4)
        .min(area.height.saturating_sub(2))
        .max(5);
    let [v] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [rect] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(v);
    frame.render_widget(Clear, rect);
    let mut text: Vec<Line> = lines.iter().map(|l| Line::from(l.clone())).collect();
    text.push(Line::from(""));
    text.push(Line::from(Span::styled(keys.to_owned(), accent())));
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(format!(" {title} "))),
        rect,
    );
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

/// Draws the whole screen; `logo` is the Entropy Fox when the terminal can show it.
pub fn draw(frame: &mut Frame, app: &App, logo: Option<&mut Logo>) {
    let area = frame.area();
    let show_logo = logo.is_some() && brand::logo_fits(area);
    let header_height = if show_logo { brand::LOGO_HEIGHT } else { 5 };
    let [top, body, status, keys] = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    let text_area = match logo {
        Some(logo) if show_logo => {
            let [logo_area, gap, text] = Layout::horizontal([
                Constraint::Length(brand::LOGO_WIDTH),
                Constraint::Length(2),
                Constraint::Min(10),
            ])
            .areas(top);
            let _ = gap;
            logo.render(frame, logo_area);
            let [_, centered] =
                Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(text);
            centered
        }
        _ => top,
    };
    frame.render_widget(Paragraph::new(header(app)), text_area);

    let [left, right] =
        Layout::horizontal([Constraint::Length(47), Constraint::Min(20)]).areas(body);
    let mut state = TableState::default().with_selected(Some(app.cursor));
    frame.render_stateful_widget(table(app), left, &mut state);
    frame.render_widget(
        Paragraph::new(details(app))
            .wrap(Wrap { trim: false })
            .block(Block::bordered()),
        right,
    );

    frame.render_widget(Paragraph::new(app.message.clone()), status);
    frame.render_widget(
        Paragraph::new(Span::styled(
            KEYS,
            Style::default().add_modifier(Modifier::DIM),
        )),
        keys,
    );

    match &app.screen {
        Screen::List => {}
        Screen::Review(lines) => popup(frame, "Plan", lines, "y apply · any other key back"),
        Screen::Result(lines) => popup(frame, "Done", lines, "any key back · q quit"),
        Screen::Help => {
            let lines: Vec<String> = HELP.iter().map(|s| (*s).to_owned()).collect();
            popup(frame, "Help", &lines, "any key back");
        }
    }
}

/// Runs the UI until the user quits; the terminal is restored on every exit,
/// including a panic. Returns the lines applied, for the scrollback.
pub fn run(mut app: App) -> io::Result<Vec<String>> {
    let mut terminal: DefaultTerminal = ratatui::init();
    let mut logo = brand::Logo::detect();
    let result = (|| -> io::Result<()> {
        while !app.quit {
            terminal.draw(|frame| draw(frame, &app, logo.as_mut()))?;
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
        Ok(())
    })();
    ratatui::restore();
    result.map(|()| app.log)
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

    fn app(f: &Fixture) -> App {
        let source = Source::Embedded;
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
        App::new(ctx, opts, false, None)
    }

    fn press(app: &mut App, keys: &str) {
        for c in keys.chars() {
            let code = match c {
                '\n' => KeyCode::Enter,
                c => KeyCode::Char(c),
            };
            app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
        }
    }

    fn row(app: &App, name: &str) -> usize {
        app.rows.iter().position(|r| r.name == name).unwrap() + 1
    }

    fn screen(app: &App, width: u16, height: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, app, None)).unwrap();
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
            Screen::Review(lines) => lines.clone(),
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
    fn the_hook_says_why_it_cant_be_selected() {
        let f = fixture();
        let mut app = app(&f);
        let n = row(&app, "commit-gate");
        press(&mut app, &n.to_string());
        assert!(app.marks.is_empty());
        assert!(
            app.message.contains("plugin only until roadmap phase 1"),
            "{}",
            app.message
        );
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
    fn the_screen_fits_80_by_24() {
        let f = fixture();
        let app = app(&f);
        let text = screen(&app, 80, 24);
        for name in [
            "adr-planning",
            "code-review",
            "doc-sync",
            "code-reviewer",
            "doc-auditor",
            "commit-gate",
        ] {
            assert!(text.contains(name), "{name} missing:\n{text}");
        }
        assert!(text.contains(brand::NAME));
    }
}
