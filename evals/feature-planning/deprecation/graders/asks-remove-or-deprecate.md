---
type: llm
weight: 2
---

The project already has `notes/legacy_export.py` with `export_csv`, called from `notes/cli.py` (the `export` command) and `notes/backup.py` (`nightly_backup`). The user asked to plan a new export module with one `export(notes, fmt, path)` entry point for CSV and JSON.

PASS if the final response points out that `legacy_export.export_csv` overlaps with or is replaced by the new module and asks the user whether to remove it in this feature (moving its callers) or deprecate it first.
FAIL if it doesn't mention the old export, or decides on its own to remove or keep it without asking.
