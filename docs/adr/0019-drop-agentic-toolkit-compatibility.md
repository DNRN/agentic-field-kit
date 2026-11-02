# 0019. The installer no longer reads the names from before the rename

Date: 2026-11-02

## Status

Accepted

Supersedes the compatibility reads in [ADR 0018](0018-rename-to-agentic-field-kit.md). Related: [ADR 0008](0008-project-install-manifest.md) (the manifest), [ADR 0014](0014-git-hook-layout.md) (the hook shim).

## Context

ADR 0018 renamed the toolkit `afk` and kept the installer reading the old names (`.agents/agentic-toolkit.tsv`, the `# agentic-toolkit <name>` shim marker, the `agentic-toolkit` plugin name), with `--update` moving them to the new ones. It left dropping those reads to a later ADR.

The rename was never released: the last release, 0.3.0, still ships the `agentic-toolkit` binary, and as of 2026-11-02 no project uses the toolkit besides its own author's tests.

## Decision

We will drop the compatibility reads before the first release under the new name, 0.4.0. The installer reads and writes only `.agents/afk.tsv`, treats only a shim marked `# afk <name>` as its own, and recognises only the `afk` plugin. `--update` no longer renames anything.

The 0.1.0 `install.sh` legacy (symlinks into a clone, the `.agentic-toolkit-install` marker; ADR 0008) is a separate decision and stays.

## Consequences

- Less code and fewer tests in the installer, and no rename step in `--update`.
- A project installed with 0.3.0 or earlier isn't recognised: its manifest and shim are `foreign` to afk. To move it, run `agentic-toolkit --remove --all` with the old binary before upgrading, or delete `.agents/agentic-toolkit.tsv`, the installed copies, and the `pre-commit` shim by hand, then install with `afk`.
- Accepted ADRs, plans, and released CHANGELOG entries still use the old name, as ADR 0018 decided.

## Rejected alternatives

- **Keep the reads for one release**: they protect projects that don't exist, and every later release would carry them until a further ADR removes them.
