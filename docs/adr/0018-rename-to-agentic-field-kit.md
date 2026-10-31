# 0018. The toolkit is renamed Agentic Field Kit, with the command `afk`

Date: 2026-10-31

## Status

Accepted

Related: [ADR 0008](0008-project-install-manifest.md) (the manifest), [ADR 0009](0009-installer-in-rust.md) (the installer), [ADR 0014](0014-git-hook-layout.md) (the hook shim).

## Context

The toolkit and its installer were both called `agentic-toolkit`: a generic name, and a long command to type for every install, update, and `--list`. What the toolkit is for is letting agents work without someone watching: the commit gate, reviews, and doc sync hold when the user is away from the keyboard.

The old name lives in places outside this repository: the manifest `.agents/agentic-toolkit.tsv` committed in every project, the `# agentic-toolkit commit-gate` marker in each clone's `pre-commit` shim, and `"agentic-toolkit@…"` in `.claude/settings.json` where the plugin is enabled.

Checked on 2026-10-31: `afk` is taken on crates.io; `agentic-field-kit` is free there; neither is a Homebrew formula or cask; no `afk` command was on `PATH` on the machine used for this record.

## Decision

We will rename the toolkit Agentic Field Kit, short `afk`.

- The installer binary, its messages, and the Claude Code plugin and marketplace are `afk`; skills are invoked as `/afk:<name>`.
- The Cargo package is `agentic-field-kit`, since `afk` is taken on crates.io, so dist's install scripts are `agentic-field-kit-installer.sh` and `.ps1`.
- The manifest is `.agents/afk.tsv` and the shim marker `# afk <name>`.
- The installer still reads the old names: `.agents/agentic-toolkit.tsv` while `.agents/afk.tsv` is missing; a shim with the old marker as its own, so `--remove` deletes it; `"agentic-toolkit@…"` as the plugin enabled; and a source tree whose plugin is named `agentic-toolkit`.
- `--update` moves the old names to the new ones first, even when every component is current, and says so: it renames the manifest and rewrites each shim with the old marker. Any other write of the manifest moves it too.
- The GitHub repository is `DNRN/agentic-field-kit`; GitHub redirects the old URLs, so links in Accepted ADRs and plans keep working.
- Accepted ADRs, plans, and released CHANGELOG entries keep the old name: they record what was true then.

## Consequences

- Every command is three letters: `afk`, `afk --update`, `afk commit-gate`.
- Upgrading needs a new install: the binary has a new name, so the old `agentic-toolkit` stays on `PATH` until removed by hand (`~/.cargo/bin/agentic-toolkit`).
- Plugin users reinstall: `/plugin install afk@afk` after refreshing the marketplace, then uninstall `agentic-toolkit@agentic-toolkit`. Until then the plugin's skills keep the `/agentic-toolkit:` prefix.
- Each project moves to the new manifest on its next `afk --update` or install, and that change is committed like any other manifest change.
- The compatibility reads stay until a release that drops them, recorded in a later ADR.

## Rejected alternatives

- **Package and binary both `afk`**: the crates.io name is taken; the package name only shows in `cargo install` and the install script names.
- **Keep `agentic-toolkit` as a second binary name**: two names for one tool in every message and doc, for a project before 1.0 with few users.
- **Agent Field Kit**: "Agentic" keeps the link to the old name, and describes the workflow rather than the agents; half the kit (the git hook, rules, policies) isn't an agent.
