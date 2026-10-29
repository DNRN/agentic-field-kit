# Evals

Golden tasks for components whose behavior depends on a model, in the [`claude plugin eval`](https://code.claude.com/docs/en/plugin-evals) format. Each case runs in a fresh Claude Code session with the plugin loaded and again without it, so the report shows what the plugin contributes (`WITH`, `W/OUT`, `Δ`).

Hooks and scripts are deterministic, so they get regression tests in [`tests/`](../tests/) (`tests/run.sh`) instead of cases here.

## Run

From the repo root:

```bash
claude plugin eval . --trust-plugin --scaffold --allow-tools Bash Edit Write
```

- `--scaffold` runs each case's `scaffold.sh`, which builds the scratch git repo the task works in. It runs as you, outside the sandbox; these scripts only write into the run's empty workspace.
- `--allow-tools Bash Edit Write`: the cases read diffs with `git`, edit docs, and write ADRs. Bash runs in Claude Code's sandbox, which on Linux needs `bubblewrap` and `socat`.
- Iterate on one case with one arm and one run: `--case doc-sync-renamed-flag --runs 1 --ablation none`. Filter by component with `--tag code-review`.
- Cost: the default is 3 runs per arm, 108 agent runs for the suite. One run per arm (`--runs 1`, 36 runs) cost about $4 on 2026-10-23, when there were 26.
- Results land in `evals/results/` (gitignored). The command exits 1 when any case scores below `--threshold` (default 1.0).

## Layout

One directory per component, one case directory per golden task:

```text
evals/<component>/<case>/
├── prompt.md      # frontmatter: tags, limits, tools; body: what the user types
├── case.yaml      # case name and context.scaffold_script
├── scaffold.sh    # builds the scratch repo (files, commits, staged diff)
└── graders/*.md   # one check each
```

| Component | Case | Checks |
| --- | --- | --- |
| code-review | `seeded-bugs` | Finds the off-by-one, SQL injection, and untested behavior; verdict `fix then commit`; reads `git diff --cached`; edits nothing |
| | `clean-diff` | Approves a correct, tested diff without inventing findings |
| | `formatting-only` | Commits a whitespace-only diff without a review |
| | `skip-review` | User declines review: commits with `--no-verify`, never stamps |
| doc-sync | `renamed-flag` | Renamed flag fixed in the guide; unrelated section untouched; no new docs |
| | `superseded-adr` | Architecture doc describes gRPC and links the superseding ADR; ADRs not edited |
| | `new-flow-diagram` | New doc under `docs/` with exactly one mermaid diagram of submit → poll → fetch, linked from the index |
| | `no-observable-change` | Internal rename: no doc edits, says why |
| | `waived-check` | User waives the docs check: commits with `--no-verify`, never stamps docs |
| adr-planning | `plan-feature` | Bootstraps `docs/adr/`, writes Proposed ADRs, plan references them, briefs each ADR and asks for a verdict |
| | `existing-format` | Follows the project's ADR home, numbering, and sections |
| | `contradiction` | Surfaces the conflict with an Accepted ADR and drafts a superseding one |
| | `verdict` | Records a rejection and an acceptance in the Status sections; keeps the rejected ADR; follows up on the plan |
| | `bug-fix` | Fixes a failing test without creating an ADR or loading the skill |
| project-init | `skip-dialogue` | Empty directory, user stays local and skips the questions: git repository, `AGENTS.md` pointing at `PROJECT.md`, `CLAUDE.md` importing `AGENTS.md`, open questions instead of guesses; no remote |
| | `brief-given` | Brief given up front: `PROJECT.md` records the stated stack and non-goal and invents no technology |
| | `existing-claude-md` | Rules in an existing `CLAUDE.md` move to `AGENTS.md`; `CLAUDE.md` imports it |
| | `feature-request` | Feature work in a repository without `AGENTS.md`: adds the flag, creates no `AGENTS.md` or `PROJECT.md`, doesn't load the skill |

## Writing graders

- Prefer free, deterministic graders: `regex` over a produced file (`target: { source: file, path: … }`), `tool_used`, `file_exists`. Keep `llm` graders for short outputs, written as concrete PASS and FAIL conditions.
- Give each case one grader on the result and one on how it was reached.
- `skill-fired` (`tool_used: Skill`) and `rubric-read` are plugin-fired indicators: reported, never scored, since the baseline arm cannot load the plugin's skills.
- `tool_used` counts subagent calls too, so `rubric-read` passes when the reviewer subagent read the rubric.
- `file_exists` only sees files created during the run; check edited files by content.
- A commit is checked through the HEAD reflog: `{ source: file, path: .git/logs/HEAD }`.
- New case: `claude plugin eval init --bare <name>`, then move it under its component and add `case.yaml` and `scaffold.sh` as above. Every component needs at least one negative case (the trigger must not fire, or must not block).

## Known issues

- **"a file in the plugin has more than one name (a hard link)"** on every case: cargo's `target/` (the installer's build output) is full of hard links, and the runner reads the whole plugin folder, ignored files included. Move it out for the run (`mv target ../target.aside`, then back), or build with `CARGO_TARGET_DIR` outside the repository.
- **`git` is masked inside runs** (`permission denied: git`, exit 126), seen with Claude Code 2.1.288 and Arch's git, whose `/usr/lib/git-core` helpers are symlinks to `/usr/bin/git`. A normal sandboxed session runs git fine. Workaround: put a copy of the git binary in a directory you own outside `/tmp` and put it first on `PATH` (a copy under `/tmp` is masked too):

  ```bash
  mkdir -p ~/.cache/eval-git && cp "$(command -v git)" ~/.cache/eval-git/
  PATH=~/.cache/eval-git:$PATH claude plugin eval . --trust-plugin --scaffold --allow-tools Bash Edit Write
  ```

- **`code-review-formatting-only` fails with the plugin** (observed 2026-10-23): `commit-gate.sh` denies a whitespace-only diff as unreviewed, though the skill skips review for formatting-only changes. Tracked as [D11](../docs/plans/design-improvements.md#d11-commit-gate-ignores-formatting-only-diffs).
