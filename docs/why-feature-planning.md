# Why features are planned, not prompted

Agentic coding made the expensive part of a feature — writing it — nearly free. What is left expensive is the part before: deciding exactly what the feature is. Tell an agent "plan this from the description" and it will produce something; whether that something solves the user's issue depends almost entirely on how completely the feature was defined. A feature built from half a description gets the other half guessed.

**1. Half a description becomes a whole feature — by guessing.**
An issue is a conversation, not a contract: the body says what hurt, the comments move the scope, and a later maintainer comment overrides an earlier drive-by one. An agent planning from the body alone silently fills every gap with the most plausible guess — and in the finished plan, a plausible guess looks exactly like a decision. The best case is local flaws. The worst case is a feature that works exactly as planned and still doesn't solve the user's issue, because the guess was in the goal, not the code.

**2. A badly defined feature doesn't just fail — it disrupts a working system.**
New code doesn't land in a vacuum; it lands next to behavior that already works. Wrong assumptions about interfaces, data, or invariants ripple outward from the feature into paths that were fine, and shortcuts taken to keep the guessed plan standing become load-bearing. The cost is not only the wasted build: it is the working system disturbed on the way.

**3. What the feature does to the system is part of the feature.**
Every feature develops the system somehow — it either extends the structure or fragments it. The often-forgotten half of planning is asking what the change does to what already exists. Logic similar to the new feature must be found: does the system reuse it, extend it, or duplicate it? Duplicate logic means two code paths to fix for every one bug, and whether to extract common code is a decision that belongs before the plan is Ready, not a refactor to discover later. The same holds for what the feature makes obsolete: deprecation is scope, not follow-up.

**4. Solid features are built in discussion — and the agent's job is to take part.**
A plan handed down finished is a plan nobody checked. The agent's role is not to receive a spec and disappear with it; it is to understand the feature fully, then ask: every open question comes with options and a recommendation, and the user decides. Rounds of this, the plan updated in place after each, until nothing material is open. That division is the point: the agent's thoroughness finds what you could not have seen, your judgment settles what the agent must not guess. The plan ends as sound as the discussion that shaped it.

**5. The plan is a file, or it is nothing.**
Reasoning that lives only in chat dies with the session — invisible in review, unavailable to the next agent that touches the code. A plan in `docs/plans/`, refined in place round by round, survives: reviewable in the PR, followed while the code is written, kept true by doc-sync afterwards, its decisions recorded as ADRs. Draft → Ready → Done, and no code before Ready.

**Kept honest by the flow, not by hope.** In this toolkit the [feature-planning skill](../skills/feature-planning/SKILL.md) fetches the whole description — the issue and every comment, or asks you for one — then maps impact and obsolescence against a [checklist](../skills/feature-planning/references/impact-checklist.md), delegated where the harness has subagents to the read-only `feature-scout` agent. Each architectural decision goes to [ADR planning](why-adrs.md) as a Proposed ADR, and the plan is refined in question rounds until it is Ready ([ADR 0021](adr/0021-feature-planning-owns-feature-planning.md)). Small, fully specified changes skip all of it and go straight to the change — a gate that cannot tell a flag from a feature gets bypassed, and then it protects nothing.

The pattern underneath: agents made building cheap, which makes defining what to build the expensive part. A flaw found in a question round costs a sentence; found after Ready, it costs a rewrite; found by the user, it costs the feature.
