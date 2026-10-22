# Hooks

Event-triggered scripts. Config lives in `hooks/hooks.json` (shape: `{"hooks": {<Event>: [...]}}`), scripts in `hooks/scripts/`.

Least portable category: hook event models and config formats differ per harness (Claude Code `hooks.json`, Cursor `hooks.json`, OpenCode JS plugins, Codex lifecycle config). Keep the scripts generic and self-contained; the wiring is per-harness.

## Format

- Declare matchers per event (`PreToolUse`, `PostToolUse`, `Stop`, …); keep hooks fast and silent on success.
- Reference bundled scripts via `"${CLAUDE_PLUGIN_ROOT}"/hooks/scripts/<script>.sh` — never absolute local paths.
- Hooks must be idempotent and safe when the tool state is unexpected; they run on every matching event.
- Every hook ships with an eval or manual test note under `evals/`.
