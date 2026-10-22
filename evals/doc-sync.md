# doc-sync - golden tasks

Covers `skills/doc-sync/`, `agents/doc-auditor.md`, `commands/doc-sync.md`.

## Task 1: renamed flag is updated in docs

Scratch repo with `docs/guide.md` documenting a CLI flag `--old`. Staged diff renames it to `--new` in the code. Ask: "Sync docs for my staged changes."

Success criteria:

- `docs/guide.md` is surgically edited: `--old` → `--new` where the flag is used; no wholesale regeneration; unrelated sections untouched.
- Report lists the doc as updated; diff shows only the necessary hunks.

## Task 2: superseded ADR no longer described as current

Repo has `docs/adr/0001-rest.md` (Accepted) and `docs/adr/0002-grpc.md` superseding it; `docs/architecture.md` still describes REST as the current API style. Staged diff completes the gRPC migration.

Success criteria:

- `docs/architecture.md` updated to gRPC, linking ADR 0002 and noting 0001 as superseded.
- Agent does not rewrite the ADRs themselves.

## Task 3: new flow feature gets one accurate mermaid diagram

Staged diff adds a three-step request flow (submit → poll → fetch). No doc exists for it.

Success criteria:

- New doc under the project's docs convention, in the project's voice, containing exactly one mermaid diagram matching the actual code paths.
- No diagrams for non-structural features; no second diagram of the same flow.

## Task 4: no observable change, no doc churn

Staged diff renames internal variables only. Ask to sync docs.

Success criteria:

- No doc edits; report says why (no observable surface).

## Negative cases

- Docs unrelated to the change are reorganized or restyled: failure.
- Auditor edits files directly: failure (proposals only; author applies).
- User waives the docs check: agent commits with `--no-verify`; it does not stamp.

## Hook: commit-gate.sh docs check

Deterministic, so covered by `tests/commit-gate.test.sh`: code without docs raises the check, a docs stamp or a `.md` edit in the commit clears it. Run `tests/run.sh`.
