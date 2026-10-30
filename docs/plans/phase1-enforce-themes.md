# Plan: roadmap phase 1, enforce existing themes

Implementation plan for [roadmap phase 1](roadmap.md#phase-1-enforce-existing-themes): turn the rules the model must remember (review before commit, docs kept true, ADRs immutable) into checks that run in every harness.

Decisions, all `Accepted` (2026-10-30):

- [ADR 0013](../adr/0013-commit-gate-as-git-hook.md): `commit-gate.sh` gains a `--pre-commit` mode and runs as a git hook; one script, two triggers. Supersedes ADR 0002's "Claude Code only" consequence.
- [ADR 0014](../adr/0014-git-hook-layout.md): the gate is installed as a committed copy at `.agents/hooks/commit-gate.sh`; each clone enables it with a marker-carrying shim in `$(git rev-parse --git-path hooks/pre-commit)`. The manifest records the copy, not the shim. Foreign `pre-commit` hooks are never edited.
- [ADR 0015](../adr/0015-accepted-adrs-guarded-at-commit.md): the git hook denies commits that change an Accepted ADR outside its `## Status` section.

Unchanged: [ADR 0004](../adr/0004-configurable-docs-paths.md) (`.commit-gate-docs`), [ADR 0008](../adr/0008-project-install-manifest.md) (manifest, hash; the list of paths the installer owns gains `.agents/hooks/<name>.sh`), [ADR 0010](../adr/0010-components-embedded-in-binary.md) (the script is already embedded through its catalog row).

## Order

| # | Item | Depends on | ADR | PR |
| --- | --- | --- | --- | --- |
| P0 | This plan; ADRs 0013–0015 accepted (done 2026-10-30) | — | 0013–0015 | 1 |
| P1 | `--pre-commit` mode and installed-name messages (done 2026-10-30) | P0 | 0013 | 1 |
| P2 | One gate at a time (plugin defers to the git hook) | P1 | 0014 | 2 |
| P3 | `hook` kind in the installer | P1 | 0014 | 3 |
| P4 | `adr-guard` | P1 | 0015 | 4 |
| P5 | `adr-index` SessionStart hook | — | — | 5 |
| P6 | `ship` skill | P1 | — | 6 |
| P7 | `adr-planning`: create and supersede on demand | — | — | 6 |
| P8 | Docs, CHANGELOG, release 0.3.0 | P1–P7 | — | 7 |

P5 and P7 depend on nothing and can go in parallel with P1–P4.

## P1. `--pre-commit` mode

**Change.**

- `hooks/scripts/commit-gate.sh --pre-commit`: skip stdin, `python3`, and `parse-commit.py`; `top` from `git rev-parse --show-toplevel`; diff kind always `--cached` (git has already staged `commit -a` into a temporary index, so `--all` stamps match). Reuse `load_docs`, `diff_hashes`, `stamped`, and the pending logic unchanged; move the shared part into functions both modes call rather than duplicating it.
- Denial: the same pending list and "ask the user" text on stderr, exit 1. Success and every unexpected state: silent, exit 0.
- Skill names as installed, decided from disk: `code-review` / `doc-sync` when `.agents/skills/<name>` exists in the project, `/agentic-toolkit:<name>` otherwise. Stamp command: `bash .agents/hooks/commit-gate.sh --stamp-review` when that copy exists, the script's own path otherwise. The plugin mode uses the same function, so both messages agree.
- `skills/code-review` and `skills/doc-sync`: the stamp step says "run the stamp command the gate printed" instead of naming the plugin path.

**Tests** (`tests/commit-gate.test.sh`, new `--pre-commit` section; real repositories in a temp dir, `git commit` with the hook installed):

- unreviewed staged change → exit 1, stderr names code review; after `--stamp-review` → commit succeeds;
- `commit -a` after `--stamp-review --all` → succeeds;
- code change without docs → docs pending; `--stamp-docs` clears it; a doc edit in the same commit clears it;
- `--no-verify` → succeeds; empty commit, not a repo, unreadable `.commit-gate-docs` → succeeds (fail open);
- message says `code-review` when `.agents/skills/code-review` exists, `/agentic-toolkit:code-review` when not.

## P2. One gate at a time

**Change.** In plugin mode, after finding `top`: if `$(git -C "$top" rev-parse --git-path hooks/pre-commit)` exists, is executable, and contains `agents/hooks/commit-gate.sh`, exit 0. The git hook will ask instead.

**Tests.** Plugin mode with the shim present → allow (no JSON); with a foreign `pre-commit` hook → deny as today; with the shim but `--no-verify` in the command → allow (unchanged).

## P3. `hook` kind in the installer

**Change.**

- `catalog.tsv`: `hook	commit-gate	hooks/scripts/commit-gate.sh	all	code-review,doc-sync	Ask before unreviewed commits`. `harnesses` becomes `all`; the "plugin only until roadmap phase 1" note in `list.rs` and the TUI's "can't be selected" text go.
- `engine`: target for a `hook` is `.agents/hooks/<name>.sh` in project scope; global scope is refused (`n/a` in the global column), since git hooks are per repository. `status::owned` accepts the new path pattern. Copy keeps the executable bit.
- `apply` writes the shim after the copy, at `git rev-parse --git-path hooks/pre-commit` (run in the project; honours `core.hooksPath` and worktrees). Not a git repository → install the copy, report "not a git repository: hook not enabled".
- Existing `pre-commit` without the marker → write nothing there, report the hook's path and the line to add, exit 0 with the component `not enabled`. With `--force` the answer is the same: never overwrite.
- `status`: new word `not enabled` when the copy is `installed` but the shim is missing. Precedence: after `modified`, `retired`, `missing`, `outdated` (content problems first). `--update` writes a missing shim; `--check`-style read-only commands only report.
- `--remove`: delete the copy; delete the shim only if it carries the marker; leave a foreign hook alone.
- TUI: `commit-gate` selectable in project scope; the detail pane shows the shim state and, for a foreign hook, the line to add.
- `--dry-run` / plan popup lists the shim as its own line: `hook    .git/hooks/pre-commit` (or the `core.hooksPath` location), `skip    .husky/pre-commit  (not ours: add …)`.

**Tests.**

- Rust unit tests: target path, owned-path filter rejects a manifest row pointing outside `.agents/hooks/`, status precedence with `not enabled`, shim content.
- `tests/install.test.sh`: install writes copy, manifest row, and executable shim; a real `git commit` is then gated; deleting the shim → `not enabled`, `--update` restores it; foreign hook untouched, reported; `core.hooksPath` set → shim lands there; `--remove` removes copy and shim, keeps a foreign hook; global install refused.
- TUI `TestBackend`: the hook can be marked and its plan lists the shim.
- CI on Linux, macOS, Windows. Windows: the shim runs under Git for Windows' `sh`; the test commits through `git` in Git Bash.

## P4. `adr-guard`

**Change.**

- In `--pre-commit` mode, after the review and docs checks (a denial lists every failing check at once): for each staged path matching `docs/adr/[0-9][0-9][0-9][0-9]-*.md` with status `M`, `D`, or `R` against `HEAD`, read `git show HEAD:<path>`; if its Status section's first non-blank line starts with `Accepted`, compare `HEAD` and index versions with the `## Status` section (heading to the next `## `) removed. Any difference, deletion, or rename denies with the path and "write a superseding ADR (adr-planning) and change only the old one's Status".
- ADR checks never get a stamp; the waiver is `--no-verify`.
- Plugin `PreToolUse` on Edit/Write: deferred. It needs `python3` for the JSON and a reliable way to tell whether an `Edit`'s `old_string` falls inside the Status section. Revisit after P8 if Claude Code users ask for earlier feedback; the git check already holds.

**Tests.** Edit an Accepted ADR's Context → denied; change only its Status to `Superseded by 0016` → allowed; add a note line under Status → allowed; edit a Proposed ADR → allowed; delete an Accepted ADR → denied; new ADR → allowed; a Status section that is missing or unparsable → allowed (fail open). Includes a CRLF ADR.

## P5. `adr-index`

**Change.** `hooks/scripts/adr-index.sh`, wired as `SessionStart` in `hooks/hooks.json`. Prints `Accepted ADRs (docs/adr/):` and one line per Accepted ADR, `NNNN <title> — <path>`, read from each file's first `# ` line and Status. Silent when there is no `docs/adr/` or no Accepted ADR. Bash and `sed`/`grep` only; fails silent. Catalog row: `hook	adr-index	hooks/scripts/adr-index.sh	plugin	-	List Accepted ADRs at session start`. Superseded ADRs are left out, so the list is what holds now.

**Tests** (`tests/adr-index.test.sh`): fixture with Accepted, Proposed, Rejected, and Superseded ADRs prints only the Accepted ones in order; no `docs/adr/` prints nothing; a malformed file is skipped. Output stays under 80 lines for 60 ADRs (one line each).

## P6. `ship`

**Change.** `skills/ship/SKILL.md`, `disable-model-invocation`-style explicit trigger ("use only when the user asks to ship, or runs /ship"). Procedure: scope the diff (staged, or `--all`); run `code-review`, apply or record the user's call on findings, stamp; run `doc-sync`, stamp; draft the commit message (references the ADR when the change implements one); show it and commit only after the user confirms. Uses the stamp command the gate prints (P1), so it never hard-codes the plugin path. Catalog row, `requires` `code-review,doc-sync`.

**Evals** (`evals/ship/`): golden — a small staged change; the agent reviews, stamps, syncs docs, commits, and the gate does not fire. Negative — "commit this" without asking to ship does not trigger the skill. Negative — reviewer finds a seeded bug; the agent stops and asks instead of committing.

## P7. `adr-planning`: create and supersede

**Change.** Two sections in `skills/adr-planning/SKILL.md`: "Record a decision on demand" (user asks for an ADR outside a planning flow: same numbering and format, still `Proposed` until the user accepts) and "Supersede an Accepted ADR" (new ADR stating what it replaces; old ADR's Status only becomes `Superseded by NNNN`, or `Accepted. <part> superseded by NNNN` for a partial one, which is what ADR 0015 lets through). Update the description so it triggers on "write an ADR" and "supersede ADR NNNN".

**Evals** (`evals/adr-planning/`): `on-demand` — "record that we use Postgres" creates the next-numbered Proposed ADR; `supersede` — reversing an Accepted decision produces a new ADR and touches only the old one's Status, and the commit passes the P4 check. The existing `contradiction` case keeps passing.

## P8. Docs and release

- README: the git gate, `agentic-toolkit commit-gate`, `--update` per clone, husky/pre-commit-framework one-liner, `--no-verify`; the plugin and git gate together ask once.
- `hooks/README.md`: both modes of `commit-gate.sh`, `adr-guard`, `adr-index`, the `hook` kind's paths.
- `docs/why-code-review.md`, `docs/why-adrs.md`: the gate works in every harness; ADRs are enforced.
- [Selective install S5](selective-install.md#s5-commit-gate-outside-the-plugin) marked done; [Rust installer](rust-installer.md) "Out of scope" line updated; roadmap phase 1 links this plan.
- ADR 0002's Status gains "consequence on harness coverage superseded by 0013".
- CHANGELOG 0.3.0: Added — git `pre-commit` gate, `hook` kind, `adr-guard`, `adr-index`, `ship`, ADR create/supersede. Changed — gate messages name the skills as installed; plugin gate defers to the git hook.
- Release as in the installer plan: `claude plugin tag --push`, `git tag installer/v0.3.0`.

## Acceptance

From the roadmap, with the check that proves each:

| Criterion | Check |
| --- | --- |
| Gate fires from Codex, Cursor, OpenCode, and a plain terminal commit; a stamp unblocks it; `--no-verify` bypasses | P1 tests (terminal); manual run per harness, noted in the PR |
| `agentic-toolkit commit-gate` installs, `--update` restores, `--remove` removes, foreign hooks never overwritten, `--list` no longer says plugin-only | P3 tests on Linux, macOS, Windows CI |
| Plugin and git hook both installed: one unreviewed commit asks once | P2 test |
| Edit to an Accepted ADR denied; Status-only change allowed | P4 tests |
| A new Claude Code session lists Accepted ADRs without reading `docs/adr/` | P5 test plus one manual session |
| `ship` and `adr-planning` pass `agentic-toolkit --check --source .` and their evals | CI plus eval run |

## Risks

- **A gate that blocks a human's terminal commit** surprises people who installed it for their agent. The message leads with `--no-verify`'s meaning, and README says so; P3 never installs it without being named.
- **Windows `bash` on `PATH`**: Git for Windows runs hooks with its own `sh`, and the shim `exec`s `bash`, which Git for Windows ships next to it. Verified in CI, not assumed.
- **Hash drift between `git diff --cached` in the hook and `git diff HEAD` at stamp time** would make `--all` stamps never match. Checked by hand on 2026-10-30; P1's `commit -a` test keeps it checked.
- **ADR status parsing** across formats (bold `**Status:** Accepted`, front matter). ADR 0015 covers this repository's format; anything else fails open and is noted in `hooks/README.md`.

## Out of scope

- Plugin `PreToolUse` variant of `adr-guard` (see P4).
- Configurable ADR location (ADR 0015 consequences).
- Hooks for other harnesses' native configs; the git hook replaces them.
