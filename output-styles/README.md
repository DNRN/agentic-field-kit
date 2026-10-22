# Output styles

Global persona/behavior variants applied to the whole session (e.g. reviewer mode, terse mode), as an alternative to per-task prompting.

## Format

```
output-styles/<kebab-name>.md
```

- Markdown; describes tone, verbosity, and defaults the agent should adopt session-wide.
- Keep orthogonal to skills: a style changes *how* things are said/done globally; a skill adds a *capability*.
