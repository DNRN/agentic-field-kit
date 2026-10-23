#!/usr/bin/env bash
set -euo pipefail
# A failing unit test from an off-by-one; ADRs exist, but a bug fix gets none.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs/adr shop tests
cat > docs/adr/0001-record-architecture-decisions.md <<'EOF'
# 0001. Record architecture decisions

Date: 2026-01-05

## Status
Accepted

## Context
We need to record the architectural decisions made on this project.

## Decision
We will use Architecture Decision Records.

## Consequences
Decisions are numbered, immutable, and superseded rather than edited.
EOF
: > shop/__init__.py
: > tests/__init__.py
cat > shop/cart.py <<'EOF'
def total(prices: list[float]) -> float:
    result = 0.0
    for i in range(1, len(prices)):
        result += prices[i]
    return result
EOF
cat > tests/test_cart.py <<'EOF'
import unittest

from shop.cart import total


class CartTest(unittest.TestCase):
    def test_total(self):
        self.assertEqual(total([1.0, 2.0, 3.0]), 6.0)


if __name__ == "__main__":
    unittest.main()
EOF
git add -A
git commit -qm "Add cart"
