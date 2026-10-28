# 0007. Installable components are declared in a catalog

Date: 2026-10-23

## Status

Proposed

Plan: [selective install](../plans/selective-install.md). Related: [ADR 0006](0006-project-scope-install-default.md) (install scope), [ADR 0008](0008-project-install-manifest.md) (what is installed where).

## Context

To offer a choice, the installer needs a list of what can be installed. Today it scans `skills/*/` and knows nothing else. The Claude Code plugin is all or nothing. Selective install needs facts that no single component file holds:

- **Kind and target per harness.** Skills go to every harness. `agents/*.md` are Claude Code subagents. `commit-gate` is a hook that is wired through `hooks/hooks.json`.
- **Dependencies.** `code-reviewer` reads the rubric in the `code-review` skill, and `doc-auditor` reads the one in `doc-sync` ([ADR 0005](0005-rubrics-live-in-skills.md)). `commit-gate` tells the agent to run both skills. Installing one of these without its dependency gives a component that fails.
- **A one-line summary for a menu.** A skill's `description` is its discovery contract ("use when …"). It is too long for a menu row, and it shouldn't be bent to fit one.

Hooks have no frontmatter. Skills may add only a `metadata` string map, and Claude Code subagent frontmatter has a fixed set of fields.

## Decision

We will declare every installable component in one committed file, `catalog.tsv`, at the repository root. The file has one row per component and tab-separated columns:

```
kind	name	path	harnesses	requires	summary
skill	code-review	skills/code-review	all	-	Severity-ranked review of a diff before commit
agent	code-reviewer	agents/code-reviewer.md	claude	code-review	Read-only fresh-eyes reviewer subagent
```

- The installer reads only the catalog. Installing a component installs everything in its `requires` column first, and the installer prints what was added.
- `harnesses` is `all` (every harness), `claude` (Claude Code only), or `plugin` (only through the Claude Code plugin); `requires` is a comma-separated list of names, or `-`.
- `name` and `description` stay in the component's own frontmatter. `tests/run.sh` fails when a component on disk has no catalog row, when a row points to a missing path, when a `requires` names an unknown component, or when a row's `name` differs from its frontmatter `name`.
- The format is plain TSV so the installer can stay in bash, with no `jq` and no associative arrays (macOS ships bash 3.2).

## Consequences

- A new component means one more file to edit. The test catches a missed edit, the same way `tests/plugin-manifest.test.sh` does for `plugin.json`.
- The installer can show the dependencies and the supported harnesses before it changes anything.
- The `harnesses` column says which components work outside Claude Code. Today that is skills only.

## Rejected alternatives

- **Derive the list from folders (today)**: covers skills only. Kind, dependencies, and harness support would end up hard-coded in `install.sh`.
- **Dependencies in frontmatter `metadata`**: works for skills only. Hooks have no frontmatter, and an unknown field in subagent frontmatter isn't guaranteed to be ignored by every harness.
- **Reuse `.claude-plugin/plugin.json`**: specific to Claude Code, has no dependency field, and doesn't list skills.
- **JSON or YAML catalog**: the installer would then need `jq`, `python3`, or a YAML parser just to show a menu.
