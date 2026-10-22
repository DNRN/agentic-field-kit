# Why code review is a gate, not a habit

The mechanism in this repo is four cooperating parts: a [skill](../skills/code-review/SKILL.md) that holds the review procedure, a [reviewer subagent](../agents/code-reviewer.md) with fresh eyes, a [slash command](../commands/code-review.md) for you, and a [hook](../hooks/) that asks before an unreviewed commit. Why it is shaped that way:

**1. The context that wrote the code cannot review it.**
An agent that just spent twenty turns building something holds two handicaps: it already believes the code is correct, and it "remembers" intent the diff doesn't show. Ask it to review and you get a summary of its own reasoning. That's why the reviewer is a separate subagent — fresh context, read-only tools, fed the diff and nothing else. The same applies to you reviewing an LLM's 900-line diff: what you need is a first-pass filter, not a second author.

**2. Review deferred is review skipped.**
"I'll review it later" loses to chat momentum every time — the next task starts, the branch ships, the review never happens. The only reliable moment to ask is the commit boundary: work is declared done, the diff is stable, and fixes are still one command away. A minute after you push, the fix is a PR comment. A week later it's an incident.

**3. Tests verify behavior; review verifies intent.**
Green tests say the code does what the code does. They don't say the change *should* exist, that the public contract change found all its callers, that the "temporary" flag will ever be removed, or that the diff contradicts an Accepted ADR. That judgment layer is exactly what the review checklist covers — including consistency with the project's ADRs, which ties this mechanism to the [ADR skill](why-adrs.md).

**4. A gate that asks beats a gate that blocks — and beats no gate.**
Auto-reviewing every change burns attention on trivia; a hard block gets disabled the first time it interrupts real work. This gate does neither: it denies the commit once and tells the agent to *ask you* — "Run a code review before committing?" One yes/no, your call, every time. Skipping is always possible (`--no-verify`) and always conscious. That is the design: not enforcement, but the end of silent unreviewed commits.

**5. The reviewer reports; it never fixes.**
A reviewer with edit rights reviews its own edits — the loop closes and the judgment disappears. Here the reviewer is read-only by construction: it returns severity-ranked findings with `file:line` and a verdict. The author applies fixes, and the touched hunks get re-reviewed. Findings without a failure mode are forbidden, which keeps the report signal instead of style noise.

The pattern underneath: agentic coding makes writing cheap and review the bottleneck. A review mechanism only works if it fires at the right moment, with fresh eyes, and asks instead of nagging. All four parts exist to guarantee those three properties.

---

## Using it

**What loads where.** The skill works in every harness (`./install.sh` symlinks it everywhere skills are read). The subagent, command, and commit gate load via the Claude Code plugin:

```
/plugin marketplace add DNRN/agentic-toolkit
/plugin install agentic-toolkit@agentic-toolkit
```

**Three ways a review starts:**

1. You ask: `/agentic-toolkit:code-review` (optionally a target — `staged`, a branch, a ref range; defaults to the staged diff).
2. The agent offers: per the skill, before committing completed work it proposes a review first.
3. The gate asks: an unreviewed `git commit` is denied with instructions for the agent to ask you.

**The gated commit flow:**

1. Agent runs `git commit` → hook denies: *unreviewed changes*, and the denial tells the agent to ask you.
2. You say yes → review runs (subagent where available), you get a findings table and a verdict.
   - `fix then commit` → blockers/majors get fixed, touched hunks re-reviewed.
   - `approve` → the reviewed diff is stamped and the commit passes.
3. You say no → the agent commits with `--no-verify`. One conscious skip, no ceremony.

**Limits, on purpose.** The stamp records one reviewed diff — stage anything new and the gate asks again. `git commit -a` is checked against the whole worktree diff, so its stamp is `--stamp-review --all`; the denial message names the right command. The gate fail-opens (missing python3, corrupted input, not a repo, unparseable command) because a wedged commit is worse than an unreviewed one. And `--no-verify` always works: this mechanism makes skipping visible, not impossible.
