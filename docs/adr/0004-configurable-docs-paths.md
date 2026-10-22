# 0004. Docs check paths configured per repository

Date: 2026-10-22

## Status

Accepted

Supersedes the docs-check trigger rule in [ADR 0003](0003-doc-sync-at-commit-gate.md) (Decision item 4); the rest of 0003 stands.

## Context

ADR 0003 decides what counts as documentation by file extension: any `.md` is a doc, everything else is code. That rule fails in two kinds of repository:

- **Markdown is the product.** In this repo, skills, agents, commands, and evals are `.md` files. Changing a skill touches "docs" by definition, so the docs check never fires, which is exactly the forgetting the gate exists to catch.
- **Docs are not markdown.** `.mdx`, `.rst`, `.adoc`, and `.txt` docs never count as docs, so editing them doesn't satisfy the check, and changing them counts as a code change.

The same rule drives three things in `commit-gate.sh`: whether the commit touches docs, whether the repo tracks any docs at all, and the code-only diff hash, which lets doc edits after a review keep the review stamp. All three need to use the same definition, or the stamps and the checks disagree.

The setting has to reach everyone who commits: collaborators, every harness, and CI.

## Decision

We will read the docs definition from a committed file, `.commit-gate-docs`, at the repository root:

- One git pathspec per line, relative to the repository root (`docs/`, `README.md`, `*.rst`). Long-form magic (`:(glob)docs/**`) is allowed; blank lines and `#` comments are ignored.
- Without the file, or when it has no pathspecs, the gate uses `*.md`, ADR 0003's behavior.
- All three uses read the same list: docs touched = the diff matches a pathspec; repo has docs = `git ls-files` matches a pathspec; code-only diff = the diff with every pathspec excluded. Stamping and checking share one function, so they cannot disagree.
- The file is read from the working tree at commit time.

This repo commits `.commit-gate-docs` with `docs/`, `README.md`, and `*/README.md`; skills, agents, commands, and evals count as code.

## Consequences

- In this repo, a skill change without a docs edit now raises the docs check; repos without the file behave exactly as before.
- The definition travels with the repository: clones, CI, and every harness see the same rule.
- A changed `.commit-gate-docs` also changes which hashes count as code-only, so docs stamps recorded under the old list stop matching and the gate asks again. Acceptable: changing the definition is rare and deliberate.
- One more root dotfile to explain; documented in `hooks/README.md` and the doc-sync skill.

## Rejected alternatives

- **`git config commitgate.docs`** (the original plan): lives in `.git/config`, which is never committed. Collaborators and CI would silently get the default.
- **Environment variable**: per-shell, invisible, and not set in harness-spawned hook processes.
- **Key in `.claude-plugin/plugin.json` or harness settings**: ties a repo-level fact to one harness and to the plugin's install, not to the repository being committed.
- **Heuristic** (directories named `docs`/`doc`, files named `README*`): guesses wrong in exactly the repos this decision exists for, and a guessed rule can't be corrected without code changes.
