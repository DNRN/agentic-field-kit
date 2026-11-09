---
type: llm
weight: 2
---

The user asked to plan "reminders" for a command-line notes app, with no issue or doc and nothing more.

PASS if the final response asks questions that block the design first (such as what triggers a reminder, how a CLI app delivers it when it isn't running, and to whom), about five or fewer in the round, and treats unanswered points as open rather than decided.
FAIL if it picks the delivery mechanism, timing model, or scope itself and presents them as decided, writes code, or asks no questions.
