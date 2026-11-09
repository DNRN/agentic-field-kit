#!/usr/bin/env bash
set -euo pipefail
# Notes CLI with a Draft plan for note tags after round 1: three open questions, which the prompt answers.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p notes tests
: > notes/__init__.py
: > tests/__init__.py
cat > notes/__main__.py <<'EOF'
from notes.cli import main

main()
EOF
cat > notes/store.py <<'EOF'
import json
from pathlib import Path

DB = Path("notes.json")


def load() -> list[dict]:
    if not DB.exists():
        return []
    return json.loads(DB.read_text(encoding="utf-8"))


def save(notes: list[dict]) -> None:
    DB.write_text(json.dumps(notes, indent=2, ensure_ascii=False), encoding="utf-8")


def add(title: str, body: str) -> dict:
    notes = load()
    note = {"id": len(notes) + 1, "title": title, "body": body}
    notes.append(note)
    save(notes)
    return note


def search(query: str) -> list[dict]:
    q = query.lower()
    return [n for n in load() if q in n["title"].lower() or q in n["body"].lower()]
EOF
cat > notes/cli.py <<'EOF'
import argparse

from notes import store


def main() -> None:
    parser = argparse.ArgumentParser(prog="notes")
    sub = parser.add_subparsers(dest="command", required=True)
    add = sub.add_parser("add")
    add.add_argument("title")
    add.add_argument("body", nargs="?", default="")
    sub.add_parser("list")
    search = sub.add_parser("search")
    search.add_argument("query")
    args = parser.parse_args()
    if args.command == "add":
        note = store.add(args.title, args.body)
        print(f"added {note['id']}")
    elif args.command == "list":
        for note in store.load():
            print(f"{note['id']}: {note['title']}")
    elif args.command == "search":
        for note in store.search(args.query):
            print(f"{note['id']}: {note['title']}")
EOF
cat > tests/test_store.py <<'EOF'
import os
import tempfile
import unittest
from pathlib import Path

from notes import store


class StoreTest(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        store.DB = Path(self.dir.name) / "notes.json"

    def tearDown(self):
        self.dir.cleanup()

    def test_add_and_search(self):
        store.add("Groceries", "milk, eggs")
        self.assertEqual([n["title"] for n in store.search("milk")], ["Groceries"])


if __name__ == "__main__":
    unittest.main()
EOF
cat > README.md <<'EOF'
# notes

Command-line notes app. `python -m notes add <title> [body]`, `python -m notes list`, `python -m notes search <query>`.
EOF
cat > AGENTS.md <<'EOF'
# notes

Command-line notes app. Python 3.11, standard library only.

- Run: `python -m notes <command>`
- Test: `python -m unittest`

## Feature planning

- Tracker: GitHub issues, via `./tools/gh` (a repo-local stand-in for `gh` with the same `issue view` commands)
- Scout model: sonnet
EOF
mkdir -p tools
cat > tools/gh <<'EOF'
#!/usr/bin/env bash
# Repo-local stand-in for the GitHub CLI: `gh auth status` and `gh issue view <N> [--comments]`.
set -euo pipefail
case "${1:-} ${2:-}" in
  "auth status")
    echo "github.com"
    echo "  Logged in to github.com account eval-user"
    exit 0
    ;;
  "issue view") shift 2 ;;
  *)
    echo "tools/gh: supports 'auth status' and 'issue view <N> [--comments]'" >&2
    exit 1
    ;;
esac
num="" comments=0
for arg in "$@"; do
  case "$arg" in
    -c | --comments) comments=1 ;;
    --json* | -w | --web)
      echo "tools/gh: $arg is not supported; use --comments" >&2
      exit 1
      ;;
    *[0-9]) num="${arg##*[!0-9]}" ;;
  esac
done
case "$num" in
  *)
    echo "GraphQL: Could not resolve to an issue or pull request with the number of ${num:-?}." >&2
    exit 1
    ;;
esac
EOF
chmod +x tools/gh
mkdir -p docs/plans
cat > docs/plans/note-tags.md <<'EOF'
# Plan: Note tags

**Status.** Draft

## Source

None: described by the user on 2026-11-02.

## Summary

Notes get free-form tags. `notes add` takes `--tag` (repeatable), and `notes list --tag <tag>` shows only the notes with that tag.

## Type

New capability.

## Benefits

Users with many notes can group and filter them without inventing title prefixes.

## Impact

- `notes/store.py`: notes gain a `tags` list; notes saved before this have none.
- `notes/cli.py`: `add --tag`, `list --tag`.
- `README.md`: usage line.
- Tests: `tests/test_store.py`.

## Deprecations

None found.

## Risks and quirks

- Existing `notes.json` files have no `tags` key; loading must treat it as empty.

## Decisions

None yet.

## Assumptions

- Tags are stored on the note in `notes.json`; no separate index.

## Open questions

Blocking:

1. Are tags case-sensitive (are `Work` and `work` one tag or two)?
2. Is there a maximum number of tags per note?

Non-blocking:

3. Should this feature include renaming a tag across all notes?

## Acceptance criteria

- `notes add "Standup" --tag work --tag daily` stores both tags.
- `notes list --tag work` lists only notes tagged `work`.
- Notes saved before this feature still load and list.

## Implementation steps

1. Store: save `tags` on add; default to an empty list on load.
2. CLI: `--tag` on `add` and `list`.
3. Tests and README.

## Refinement log

| Round | Question | Answer |
| --- | --- | --- |
| 1 | Who is it for? | Anyone with more than a few dozen notes |
| 1 | Filter by several tags at once? | Not now; one `--tag` on `list` |
EOF
git add -A
git commit -qm "Initial notes CLI and note tags plan"
