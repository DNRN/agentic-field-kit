# 0002. Layered code review: skill procedure, reviewer subagent, commit-time gate

Date: 2026-10-22

## Status

Accepted

## Context

We want finished work reviewed before it is committed, with the agent *asking* whether to review rather than always auto-running or never running. Constraints: skills are the only portable component (all harnesses); subagents and hooks load only via the Claude Code plugin; the author's own context is a poor reviewer of its own output.

Forces:

- A review run by the same context that wrote the code reviews nothing — confirmation bias plus stale intent.
- Model-remembered rules ("review before commit") are unreliable; skills fire on relevance, not on events.
- A deterministic gate must not wedge commits when state is unexpected.

## Decision

We will build code review as four cooperating layers, one per trigger mechanism:

1. **`skills/code-review/`** — the portable procedure: scope the diff, fresh eyes, checklist, verified findings, verdict. Works in every harness. Carries the behavioral rule "offer review before committing; proceed unreviewed only on explicit user decline."
2. **`agents/code-reviewer.md`** — Claude Code subagent: fresh context, read-only tools, strong model. Receives only the diff command and repo path, never the author's reasoning.
3. **`commands/code-review.md`** — explicit `/agentic-toolkit:code-review` entry point for user-triggered review.
4. **`hooks/scripts/commit-gate.sh`** (wired in `hooks/hooks.json`, PreToolUse on `git commit`) — the deterministic "ask before committing": denies an unreviewed commit with a reason instructing the agent to ask the user; a stamp file records the reviewed diff hash so the approved commit passes; `--no-verify` is the explicit bypass. Fail-open on any unexpected state. Originally `review-gate.sh`; generalized into the shared commit gate with the documentation check of ADR 0003.

Review is offered at commit time, not auto-run after every feature: the commit boundary is when work is declared done and fixes are still cheap to apply.

## Consequences

- Claude Code gets a deterministic gate; other harnesses get the model-driven rule from the skill — acceptable split, since hooks are the least portable category by nature.
- The reviewer is isolated and read-only by construction; findings cite `file:line` verified against source.
- The stamp is a nudge, not enforcement — `--no-verify` always exists. We accept this: the gate exists to make skipping a conscious act.
- Four files to maintain per review mechanism instead of one; each stays single-concern.

## Rejected alternatives

- **Skill only**: maximally portable, but nothing fires at commit time; the agent remembers the rule only sometimes.
- **Dedicated agent only**: fresh eyes, but Claude-plugin-only and homeless as a procedure — no portable knowledge, no user entry point, no gate.
- **Command only**: user-triggered, so it can never be the "agent asks" path; ages badly per `commands/README.md`.
- **Auto-run review after every completed feature**: wastes tokens on trivial changes and interrupts mid-task flow; the commit boundary is the correct single gate.
- **CI/server-side review only**: outside the agent loop, too late to fix cheaply, not portable across harnesses.
