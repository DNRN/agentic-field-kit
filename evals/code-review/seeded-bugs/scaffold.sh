#!/usr/bin/env bash
# Repo with three seeded defects in the staged diff of shop/store.py:
# off-by-one loop bound (sum_last), SQL built by concatenation (find_user),
# new VIP discount with no test (apply_discount). No docs, so the gate's docs
# check stays off and the case measures review only.
set -euo pipefail
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p shop tests
: > shop/__init__.py
cat > shop/store.py <<'EOF'
import sqlite3


def find_user(conn: sqlite3.Connection, name: str):
    return conn.execute("SELECT id, name FROM users WHERE name = ?", (name,)).fetchone()


def sum_last(items: list[int], k: int) -> int:
    """Sum of the last k items."""
    return sum(items[-k:]) if k else 0


def apply_discount(price: float, code: str) -> float:
    if code == "SAVE10":
        return round(price * 0.9, 2)
    return price
EOF
cat > tests/test_store.py <<'EOF'
import unittest

from shop.store import apply_discount, sum_last


class StoreTest(unittest.TestCase):
    def test_save10(self):
        self.assertEqual(apply_discount(100, "SAVE10"), 90)

    def test_unknown_code(self):
        self.assertEqual(apply_discount(100, "NOPE"), 100)

    def test_sum_last(self):
        self.assertEqual(sum_last([1, 2, 3, 4], 2), 7)
EOF
git add -A
git commit -qm "Add store helpers"
cat > shop/store.py <<'EOF'
import sqlite3


def find_user(conn: sqlite3.Connection, name: str):
    query = "SELECT id, name FROM users WHERE name = '" + name + "'"
    return conn.execute(query).fetchone()


def sum_last(items: list[int], k: int) -> int:
    """Sum of the last k items."""
    total = 0
    for i in range(len(items) - k, len(items) + 1):
        total += items[i]
    return total


def apply_discount(price: float, code: str) -> float:
    if code == "SAVE10":
        return round(price * 0.9, 2)
    if code == "VIP":
        return round(price * 0.7, 2)
    return price
EOF
git add -A
