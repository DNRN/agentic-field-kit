# 0005. Rubrics live in skills; skills are the entry points; subagents receive the rubric's location

Date: 2026-10-22

## Status

Accepted

Supersedes, in [ADR 0002](0002-code-review-mechanism.md) and [ADR 0003](0003-doc-sync-at-commit-gate.md): the subagent input contract (Decision item 2 of each: "receives only the diff command and repo path" / "given the diff and candidate docs") and the command layer (Decision item 3 of each). The skill, subagent, and commit-gate layers stand.

## Context

The review checklist existed twice, in `skills/code-review/SKILL.md` and `agents/code-reviewer.md`, and the doc audit rubric likewise in `skills/doc-sync/SKILL.md` and `agents/doc-auditor.md`. The commands restated the procedures a third time. Copies drift, and a reviewer that checks something different from what the skill describes undermines both.

The single copy must work in every harness this repo targets (Claude Code, Codex, Gemini CLI, Cursor, OpenCode, omp):

- Skill folders are the only component every harness installs, and the Agent Skills standard loads files under a skill's `references/` on demand.
- Agent files are harness-specific. omp loads Claude plugin `agents/` folders, but harness features such as Claude Code's `${CLAUDE_PLUGIN_ROOT}` substitution or `skills:` preloading do nothing, or leave broken text, elsewhere.
- A subagent cannot be assumed to find a skill's files on its own; the agent that loaded the skill knows where the skill lives.
- The commands and skills shared names. In Claude Code both registered as `agentic-toolkit:code-review` (and `agentic-toolkit:doc-sync`), and the command shadowed the skill: loading the skill by name from inside the command re-invoked the command. A bare `code-review` resolves to Claude Code's built-in skill instead. Claude Code already exposes plugin skills as slash commands, so the command files added only an argument-to-scope mapping.

## Decision

We will keep each rubric once, inside its skill, and pass its location to whoever applies it:

1. **`skills/code-review/references/rubric.md`** and **`skills/doc-sync/references/rubric.md`** hold everything a reviewer or auditor needs: stance, what to check, what counts as a finding, severities, and the report format. doc-sync's rubric also holds the documentation standards the author writes to.
2. **Skills** keep only the procedure and point at their rubric. To delegate, the skill passes the subagent the diff command, the repository path (plus candidate docs for doc-sync), and the rubric's location in a form the subagent can read (an absolute path or a harness URI such as `skill://code-review/references/rubric.md`). The author's reasoning is still never passed. Without subagents, the author applies the same rubric.
3. **Agent files** are thin wrappers: harness frontmatter plus "read the rubric you were given and follow it; if none was given, find the skill's `references/rubric.md`; if it can't be found, stop." Any read-only subagent given the same three inputs behaves the same, so harnesses without the bundled agents lose nothing.
4. **No commands.** `commands/code-review.md` and `commands/doc-sync.md` are removed; the skills are the explicit entry points (`/agentic-toolkit:code-review` and `/agentic-toolkit:doc-sync` in Claude Code, each harness's own skill invocation elsewhere). Each skill maps an optional target (`staged`, `worktree`, a branch, a ref range) to its diff scope.

## Consequences

- One copy of each rubric; skill, agent, and generic subagents cannot disagree.
- The rubric reaches every harness wherever the skill is installed, by symlink, copy, or plugin.
- Delegation depends on the delegating agent passing a readable location. A reviewer without it searches for the skill and stops if it cannot find it, rather than reviewing from guesswork.
- The subagent's input grows by one item (the rubric location), which is instructions, not the author's reasoning, so ADR 0002's fresh-eyes property holds.
- One entry point per mechanism in every harness, with no name collisions; the commit gate's denial text (`/agentic-toolkit:code-review`) now names the skill.
- Whether the main agent delegates is still the model's call under the harness's own rules: in Claude Code headless runs it sometimes audited docs itself, citing a harness instruction not to spawn agents unasked. The rubric makes the self-audit path equivalent in content, not in independence.

## Rejected alternatives

- **Rubric in the agent file, skill points to the agent**: agent files are not installed outside Claude Code-style plugins, so skill-only harnesses lose the rubric.
- **Rubric in `prompts/`**: not part of any skill folder, so `install.sh` does not install it and harnesses cannot load it.
- **Generated copies checked in, with a CI diff check**: still two copies in the repository, and edits to the wrong copy fail late.
- **Claude Code `skills:` preload or `${CLAUDE_PLUGIN_ROOT}` paths in the agent**: works in one harness only, and `skills:` injects `SKILL.md`, whose procedure tells the reader to delegate, not the rubric.
- **Rename the commands** (e.g. `/agentic-toolkit:review`): fixes the collision, but keeps two Claude-only files that only forward to the skill, and two entry points per mechanism.
