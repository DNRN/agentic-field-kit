# Why ADRs matter in agentic coding

Agentic coding changes the economics of writing software: implementation becomes fast, cheap, and parallel. The scarce resource is no longer producing code — it's keeping track of what was decided and why. Architecture Decision Records are the mechanism that keeps that knowledge from evaporating.

**1. Your agent has no memory. Your ADRs do.**
Every agent session starts blank. Its context is whatever lives in the repo. If a decision exists only in a conversation — "we chose polling over webhooks because the firewall blocks inbound connections" — the next session doesn't have it. It will happily build a webhook. ADRs put intent into the one place every agent reads: the repository.

**2. Agents optimize locally; architecture is a global constraint.**
An agent solves the task in front of it. The locally convenient path routinely violates a decision made three weeks ago for reasons invisible in the code. Code shows *what*, never *why* — and when an agent infers the why, it confidently gets it wrong and builds on top of the wrong reason. An ADR records the constraint together with its rationale, so the agent can respect both.

**3. ADRs shut down the re-litigation machine.**
LLMs generate plausible alternatives by the dozen. Without recorded decisions and rejected alternatives, every session is a fresh chance to re-propose the migration you already turned down. "Rejected: gRPC — no Go experience on the team, current latency is within budget" is the most valuable sentence in the file. It converts an open question back into a settled one.

**4. Without ADRs, architectural drift is silent.**
Session 1 decides in chat. Session 2 violates it. Tests stay green — they verify behavior, not design. Review can't tell intent from accident. Fifty agent-sessions later the architecture has drifted and nobody can say where it started. With ADRs the deviation is at least visible: either the code is wrong, or the decision genuinely changed and gets superseded the proper way.

**5. Humans move up to the decision layer.**
When agents draft, humans review. ADRs make the reviewable unit bigger than the diff: the decision itself. The agent writes a one-page record with status `Proposed`; you confirm or reject — and only then does implementation start. You spend your time on rationale instead of reverse-engineering architecture from a 900-line diff.

**6. One record, every reader.**
New teammate, new agent, new tool, future you: everyone inherits the same context from the same file. ADRs are onboarding documentation for humans and machines at once.

**Kept honest by the tools, not by memory.** In this toolkit a session starts with the Accepted ADRs listed in its context (the `adr-index` hook), and a commit that rewrites an Accepted ADR instead of superseding it is denied (the commit gate's ADR check, [ADR 0015](adr/0015-accepted-adrs-guarded-at-commit.md)). The skill asks for both; the hooks make them happen.

The pattern underneath: agentic coding moves the bottleneck from writing code to keeping decisions coherent. Tests verify behavior; ADRs verify design. Most agent workflows have the first — the second is the missing half.
