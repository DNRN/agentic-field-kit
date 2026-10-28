# 0008. Installs are recorded in a manifest; the content hash is the version

Date: 2026-10-28

## Status

Proposed

Plan: [selective install](../plans/selective-install.md). Related: [ADR 0006](0006-project-scope-install-default.md) (install scope), [ADR 0007](0007-component-catalog.md) (catalog).

## Context

Users install a few components into a project, then later add, remove, and update them by pulling the toolkit and rerunning the installer. To do that, the installer has to know, for every component in a scope:

- **Is it ours?** It must never replace or remove anything it didn't install. Today a copy carries a marker file. An agent is a single `.md` file, so it has nowhere to put one.
- **What did it write?** A skill can land in two directories ([ADR 0006](0006-project-scope-install-default.md)). An agent brings in the skill it depends on.
- **Was it asked for, or pulled in?** Removing `code-reviewer` should also offer to remove `code-review` when the user never asked for it and nothing else needs it.
- **Has it changed, and on which side?** The toolkit may have changed since the install (update it), or the user may have edited the copy (keep their edits).

Detecting change with per-component version numbers needs a bump on every edit and a test that catches a missed bump. A content hash changes with any edit by itself.

## Decision

We will record every install in one manifest per scope: `<project>/.agents/agentic-toolkit.tsv` for a project (committed with the copies), and `~/.agents/agentic-toolkit.tsv` for `--global`. Only the installer writes it. It is sorted, with one tab-separated row per component:

```
kind	name	version	hash	reason	paths
skill	code-review	0.2.0	3f2a…	dependency	.agents/skills/code-review .claude/skills/code-review
agent	code-reviewer	0.2.0	9c1e…	explicit	.claude/agents/code-reviewer.md
```

- `hash` is the sha256 of the component's content when it was installed: for a folder, of each file's sha256 and relative path, sorted. Carriage returns are removed before hashing, so line-ending conversion by git doesn't count as a change. A global install made of symlinks records `-`: a link always shows the clone's current content, and `-` tells `--update` to restore a deleted one as a link.
- `version` is the toolkit release from `.claude-plugin/plugin.json`, for reading the changelog. The installer never decides anything from it. Components get no version numbers of their own.
- `reason` is `explicit` for a component the user named, `dependency` for one pulled in through `requires`.
- `paths` lists everything written, relative to the scope root (the project, or `$HOME` for global).
- The manifest decides what is ours, but only for paths the installer writes for that name (`.agents/skills/<name>`, `.claude/skills/<name>`, `.omp/agent/skills/<name>`, `.claude/agents/<name>.md`); a committed manifest can't point it elsewhere. The disk decides the state: content whose hash differs from the manifest's is `modified`; a component no longer in the catalog is `retired`; a listed path that is gone is `missing`; one whose source hash differs is `outdated`. When several apply, the first wins, so an edit always needs `--force`.
- `--update` copies outdated components again, restores missing ones, installs dependencies added since, and offers to remove retired ones. It skips `modified` without `--force`. `--remove` follows the same rule for modified copies.
- Global installs made before the manifest still count as ours: a symlink into this clone's `skills/`, or a copy with the old `.agentic-toolkit-install` marker.

## Consequences

- Updating is `git pull` in the clone, then `install.sh --update` in the project. No component needs a version bump, and no edit can be missed.
- The project's installed set is one file, readable in review and diffable in git.
- Two branches that each install a component can conflict in the manifest. Sorted rows, one per component, keep the conflict to adjacent lines.
- A user who deletes a copy by hand sees it as `missing` until `--update` restores it or `--remove` drops it from the manifest.

## Rejected alternatives

- **Marker file in each copy (the earlier plan)**: agents are single files and need a second mechanism, a ledger. Neither records the reason or all the paths.
- **Version numbers per component**: a forgotten bump hides an update, and it needs a test for every content change. Pinning an older version of one tool still means checking out an older tag of the toolkit.
- **Scan the target directories only, no record**: can't tell our copy from a same-named one the user wrote, and can't tell a dependency from a component the user asked for.
- **JSON manifest**: the installer would need `jq` or `python3` (see [ADR 0007](0007-component-catalog.md)).
