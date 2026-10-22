# Why docs must be kept true, not just written

Agentic coding inverted the usual scarcity: code is now the cheap part of a commit, prose is the expensive one — because prose is the only artifact nobody tests. The doc-sync mechanism exists because of one uncomfortable fact: **a stale document is an active lie, and agents believe lies**.

**1. Docs are the agent's onboarding. Every session starts blank.**
An agent's context is the repository. The README and `docs/` are what it reads before touching code — so documentation is no longer just for humans; it is executable context. A doc that says "run `make bootstrap`" when that target was removed three merges ago doesn't slow an agent down; it sends it confidently down a path that no longer exists. Humans distrust missing docs. Agents trust any doc.

**2. Nothing in CI fails when prose lies.**
Tests pin behavior. Linters pin style. ADRs pin decisions. No tool fails the build when the documented flag was renamed, the endpoint was removed, or the default changed. That gap doesn't stay small: every merge that skips a doc update widens it, silently, and the first reader to hit the gap pays the full cost of rediscovering the truth.

**3. Documentation is the human interface.**
Code is written for the compiler and read under duress. Docs are for the human who needs the system's shape in five minutes: what it does, how to run it, how the pieces fit. When agents do the writing, that interface is maintained by nobody unless a mechanism maintains it. Well-documented code is not a virtue statement — it is the difference between a repo a human can adopt and one only its author-agent can drive.

**4. A stale diagram is the most confident lie in the repo.**
A mermaid diagram that matches the real code paths is the fastest structure-transfer a reader gets. A diagram that shows a removed step is worse than no diagram: it asserts structure with visual authority. Structure — flows, states, dependencies — is exactly what drifts silently, which is why diagrams are updated with the code that changed them, one per concept, never decorative.

**5. Docs and ADRs answer different halves of the same question.**
The ADR records *why* a decision was made and which alternatives died. The docs record *what is true now*. When the two drift apart you get the worst case: an ADR marked superseded while the doc still teaches the old design as current. Keeping them correlated — docs linking governing ADRs, superseded designs leaving the docs when they leave the code — is what makes the whole record readable end to end.

**6. Deferred doc updates are skipped doc updates.**
"Update the README before the release" is the "I'll review it later" of documentation. Momentum always wins. The only reliable moment is the commit that changed the code — same boundary, same ask, one yes/no. That is why documentation check lives in the [commit gate](why-code-review.md) next to code review, not in a checklist nobody opens.

The pattern underneath: tests verify behavior, ADRs verify design — and docs carry everything else a reader needs to trust the system. Most agent workflows automate the first two or neither. The third is the one that rots first, because it is the only one no tool verifies by default. [Doc sync](../skills/doc-sync/SKILL.md) makes the repo verify it: at the moment work is declared done, docs either match reality or the commit asks why not.
