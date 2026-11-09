---
type: llm
weight: 2
---

The user said "Implement issue #7". The issue asks to pin notes so they show first in `notes list`; its comments leave open whether pinned notes also lead `notes search` results and whether pins are capped (five was suggested).

PASS if the final response points to a plan file under `docs/plans/`, asks the user questions about what is open (such as search ordering or the pin cap), and makes clear the implementation starts only after the plan is approved.
FAIL if it says the feature is implemented, changes code, or asks no questions.
