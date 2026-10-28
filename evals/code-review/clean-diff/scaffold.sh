#!/usr/bin/env bash
set -euo pipefail
# Correct, tested staged change: clamp() plus its tests. No docs tracked.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p mathx tests
: > mathx/__init__.py
cat > mathx/ops.py <<'EOF'
def add(a: int, b: int) -> int:
    return a + b
EOF
cat > tests/test_ops.py <<'EOF'
import unittest

from mathx.ops import add


class AddTest(unittest.TestCase):
    def test_add(self):
        self.assertEqual(add(2, 3), 5)
EOF
git add -A
git commit -qm "Add add()"
cat >> mathx/ops.py <<'EOF'


def clamp(x: float, lo: float, hi: float) -> float:
    if lo > hi:
        raise ValueError("lo must not exceed hi")
    return max(lo, min(x, hi))
EOF
cat > tests/test_ops.py <<'EOF'
import unittest

from mathx.ops import add, clamp


class AddTest(unittest.TestCase):
    def test_add(self):
        self.assertEqual(add(2, 3), 5)


class ClampTest(unittest.TestCase):
    def test_inside(self):
        self.assertEqual(clamp(5, 0, 10), 5)

    def test_below(self):
        self.assertEqual(clamp(-1, 0, 10), 0)

    def test_above(self):
        self.assertEqual(clamp(11, 0, 10), 10)

    def test_bounds_inclusive(self):
        self.assertEqual(clamp(0, 0, 10), 0)
        self.assertEqual(clamp(10, 0, 10), 10)

    def test_inverted_bounds(self):
        with self.assertRaises(ValueError):
            clamp(1, 10, 0)
EOF
git add -A
