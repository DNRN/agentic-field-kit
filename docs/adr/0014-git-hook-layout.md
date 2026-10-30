# 0014. The gate script is committed; each clone enables it with a shim

Date: 2026-10-30

## Status

Accepted

Plan: [roadmap phase 1](../plans/phase1-enforce-themes.md). Related: [ADR 0008](0008-project-install-manifest.md) (manifest), [ADR 0013](0013-commit-gate-as-git-hook.md) (gate as git hook).

## Context

The installer gains a `hook` kind so `agentic-toolkit commit-gate` puts the gate into a project. Git runs only `$(git rev-parse --git-path hooks)/pre-commit`: `.git/hooks/` by default, or the directory in `core.hooksPath` (husky sets it). Neither is shared by a clone: `.git/` is never committed, and `core.hooksPath` is per-clone config. So some per-clone step is unavoidable.

Options:

- **Script in `.git/hooks/pre-commit`**: nothing is committed, every clone runs the installer, and the manifest (committed, per project) would record a path that is per clone.
- **Committed hooks folder plus `core.hooksPath`**: shared, but setting it hides every other hook in `.git/hooks/` and fights husky.
- **Committed script plus a per-clone shim**: the gate is committed like any component; `.git/hooks/pre-commit` is three lines that run it.

Existing `pre-commit` hooks (husky, the pre-commit framework, hand-written) must never be overwritten.

## Decision

We will install the gate as a committed copy at `.agents/hooks/commit-gate.sh`, recorded in the manifest like a skill (path, content hash). The installer then writes a shim at `$(git rev-parse --git-path hooks/pre-commit)`:

```sh
#!/bin/sh
# agentic-toolkit commit-gate
gate="$(git rev-parse --show-toplevel)/.agents/hooks/commit-gate.sh"
[ -f "$gate" ] || exit 0
exec bash "$gate" --pre-commit
```

The shim fails open when the script is absent (an older branch checked out, the component removed in another clone).

- The manifest does not record the shim. It is per-clone state; `--list` shows the component as `not enabled` when the copy is current but the shim is missing, and `--update` writes it.
- If a `pre-commit` hook exists without the marker line, the installer writes nothing there, reports it, and prints the line to add to that hook (`bash .agents/hooks/commit-gate.sh --pre-commit || exit 1`). It never edits a hook it did not write.
- The marker is how the rest of the toolkit detects the git gate: the plugin's `PreToolUse` mode allows the commit when the `pre-commit` file contains `agents/hooks/commit-gate.sh`, so a user with both is asked once.
- `--remove` deletes the copy and the shim only if the shim still has the marker.

## Consequences

- A teammate who clones the project sees the gate in the repository and the manifest; one `agentic-toolkit --update` enables it. Until then their commits are not gated, and `--list` says so.
- Stamp commands in the denial message become repository-relative (`bash .agents/hooks/commit-gate.sh --stamp-review`) and work in every harness.
- Husky and pre-commit framework users add one line by hand. We accept the friction to never corrupt a hook we don't own.
- `not enabled` is a new status word in `--list` and the UI.

## Rejected alternatives

- **Script only in `.git/hooks/`**: not reviewable in the repository, and a teammate can't see the project expects it.
- **`core.hooksPath` to a committed folder**: silently disables the clone's other hooks and conflicts with husky.
- **Chaining by renaming the existing hook to `pre-commit.local`**: edits a file another tool owns; its updates would overwrite or orphan the chain.
- **Recording the shim in the manifest**: the manifest is committed, so every clone but one would read a path that isn't there.
