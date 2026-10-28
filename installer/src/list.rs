//! `--list`: the status table, line for line as `install.sh --list` prints it.

use std::fmt::Write as _;

use crate::manifest;
use crate::scope::{Scope, detected_harnesses, plugin_on};
use crate::status::{Ctx, is_ours};

/// Catalog names, then names from either manifest that the catalog no longer has.
pub fn rows(ctx: &Ctx) -> Vec<String> {
    let mut rows: Vec<String> = ctx.catalog.names().map(str::to_owned).collect();
    let mut extra = ctx.root.as_deref().map(manifest::names).unwrap_or_default();
    extra.extend(manifest::names(&ctx.home));
    for n in extra {
        if !rows.contains(&n) {
            rows.push(n);
        }
    }
    rows
}

/// `why` is the reason there is no project, when there is none.
pub fn table(ctx: &Ctx, why: Option<&str>) -> String {
    let mut out = String::new();
    let mut plug = "not enabled";
    match (&ctx.root, why) {
        (Some(root), _) => {
            writeln!(
                out,
                "agentic-toolkit {} → project {}",
                ctx.version,
                root.display()
            )
            .unwrap();
            if plugin_on(root) {
                plug = "enabled for this project";
            }
        }
        (None, why) => {
            let why = why.unwrap_or_default();
            writeln!(
                out,
                "agentic-toolkit {} → no project ({})",
                ctx.version,
                why.split(':').next().unwrap_or_default()
            )
            .unwrap();
        }
    }
    if plug == "not enabled" && plugin_on(&ctx.home) {
        plug = "enabled for your user";
    }
    writeln!(
        out,
        "Harnesses found: {}",
        detected_harnesses(&ctx.home).join(", ")
    )
    .unwrap();
    writeln!(out, "Claude Code plugin: {plug}").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "  {:>2}  {:<14} {:<6} {:<9} {:<9} summary",
        "#", "component", "kind", "project", "global"
    )
    .unwrap();
    let mut dup = false;
    for (i, n) in rows(ctx).iter().enumerate() {
        let (kind, mut sum, mut notes) = match ctx.component(n) {
            Some(c) => {
                let mut notes = match c.harnesses.as_str() {
                    "claude" => "Claude Code only".to_owned(),
                    "plugin" => "plugin only until roadmap phase 1".to_owned(),
                    _ => String::new(),
                };
                if !c.requires.is_empty() {
                    if !notes.is_empty() {
                        notes.push_str("; ");
                    }
                    notes.push_str(&format!("needs {}", c.requires.join(", ")));
                }
                (c.kind.clone(), c.summary.clone(), notes)
            }
            None => {
                let row = ctx
                    .root
                    .as_deref()
                    .and_then(|r| manifest::row(r, n))
                    .or_else(|| manifest::row(&ctx.home, n));
                (
                    row.map(|r| r.kind).unwrap_or_default(),
                    String::new(),
                    "no longer in the toolkit".to_owned(),
                )
            }
        };
        let st_p = if ctx.root.is_some() {
            ctx.status(Scope::Project, n)
        } else {
            "n/a".into()
        };
        let st_g = ctx.status(Scope::Global, n);
        if is_ours(&st_p) && !matches!(st_g.as_str(), "-" | "n/a" | "plugin") {
            if !notes.is_empty() {
                notes.push_str("; ");
            }
            notes.push_str("! in both scopes");
            dup = true;
        }
        if sum.chars().count() > 31 {
            sum = sum.chars().take(30).collect::<String>() + "…";
        }
        writeln!(
            out,
            "  {:>2}  {:<14} {:<6} {:<9} {:<9} {}",
            i + 1,
            n,
            kind,
            st_p,
            st_g,
            sum
        )
        .unwrap();
        if !notes.is_empty() {
            writeln!(out, "      {notes}").unwrap();
        }
    }
    writeln!(out).unwrap();
    if dup {
        writeln!(
            out,
            "! in both scopes: Codex lists it twice; Claude Code and Cursor use the global copy."
        )
        .unwrap();
    }
    out
}
