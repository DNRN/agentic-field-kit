# Output styles

Global persona/behavior variants applied to the whole session (e.g. reviewer mode, terse mode), as an alternative to per-task prompting.

## Format

```
output-styles/<kebab-name>.md
```

- Markdown; describes tone, verbosity, and defaults the agent should adopt session-wide.
- Keep orthogonal to skills: a style changes *how* things are said/done globally; a skill adds a *capability*.
- Register each new file in `.claude-plugin/plugin.json` under `outputStyles`. The explicit list keeps this README from loading as a style; an unlisted file never loads (`tests/plugin-manifest.test.sh` fails on it).
