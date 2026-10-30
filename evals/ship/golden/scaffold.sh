#!/usr/bin/env bash
set -euo pipefail
# A small CLI with its flags documented in the README. The staged change adds
# a tested --verbose flag; the README doesn't mention it yet.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p tests
cat > greet.py <<'PY'
import argparse


def greeting(name: str, verbose: bool = False) -> str:
    return f"Hello, {name}!"


def main(argv=None) -> None:
    parser = argparse.ArgumentParser(prog="greet")
    parser.add_argument("name")
    args = parser.parse_args(argv)
    print(greeting(args.name))


if __name__ == "__main__":
    main()
PY
cat > tests/test_greet.py <<'PY'
import unittest

from greet import greeting


class GreetTest(unittest.TestCase):
    def test_greeting(self):
        self.assertEqual(greeting("Ada"), "Hello, Ada!")
PY
cat > README.md <<'MD'
# greet

Prints a greeting.

## Usage

    python greet.py NAME

| Flag | Effect |
| --- | --- |
| `NAME` | Who to greet |
MD
git add -A
git commit -qm "Add greet CLI"
cat > greet.py <<'PY'
import argparse


def greeting(name: str, verbose: bool = False) -> str:
    text = f"Hello, {name}!"
    if verbose:
        text += f" (greeted {len(name)}-letter name)"
    return text


def main(argv=None) -> None:
    parser = argparse.ArgumentParser(prog="greet")
    parser.add_argument("name")
    parser.add_argument("--verbose", action="store_true", help="say more about the greeting")
    args = parser.parse_args(argv)
    print(greeting(args.name, args.verbose))


if __name__ == "__main__":
    main()
PY
cat > tests/test_greet.py <<'PY'
import unittest

from greet import greeting


class GreetTest(unittest.TestCase):
    def test_greeting(self):
        self.assertEqual(greeting("Ada"), "Hello, Ada!")

    def test_verbose(self):
        self.assertEqual(greeting("Ada", verbose=True), "Hello, Ada! (greeted 3-letter name)")
PY
git add greet.py tests/test_greet.py
