# 0020. The catalog has a details column for the installer's menu

Date: 2026-11-03

## Status

Accepted

Extends [ADR 0007](0007-component-catalog.md) (the catalog's columns). Related: [ADR 0009](0009-installer-in-rust.md) (the installer and its menu).

## Context

The installer's menu shows one line per component, the catalog's `summary`, and its details pane repeats that line. Someone choosing what to install can't tell from five words what a component does, when it runs, or what it needs from them.

The components' own `description` frontmatter is written for the agent: it says when to trigger ("Use when …"), not what the component is for. Hooks have no frontmatter at all.

## Decision

We will add a seventh column to `catalog.tsv`, `details`: two or three sentences per component, written for the person choosing it, saying what it does and when. The header becomes:

```
kind	name	path	harnesses	requires	summary	details
```

- The menu's details pane shows the `summary`, then `details`, above what it showed before (requirements, harness notes, paths).
- `summary` stays one short line for the list and `--list`.
- `details` is plain text with no tabs. `tests/catalog.test.sh` fails when a row has no `details`. The installer shows nothing in its place when a catalog without the column is read (an older `--source` tree).

## Consequences

- A new component means one more piece of text to write, in the same row.
- The catalog's lines get long; that is the price of keeping every component's metadata in one file, which ADR 0007 chose.
- `--list` is unchanged.

## Rejected alternatives

- **Show the frontmatter `description`**: it is written for triggering, not for people, and hooks have none.
- **A file per component (for example `details.md` next to each skill)**: a second place to look for metadata, and agents and hooks are single files with nowhere to put one.
