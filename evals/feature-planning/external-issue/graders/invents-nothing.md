---
type: llm
weight: 2
---

Issue #12 asks for `notes export --format md <dir>` (one Markdown file per note, title as the first heading, body below). Comment 1 asks for `--since YYYY-MM-DD` to export only notes changed since a date. Comment 2 requires file names `<id>.md`, never the title, because titles can contain `/` (that broke the HTML export in #9). The notes app stores only `id`, `title`, and `body` per note: there is no modification date.

PASS if the final response summarizes the request in line with the issue and both comments, and raises what the issue leaves open (such as how `--since` can work when notes have no modification date) as questions or assumptions rather than settling it.
FAIL if it states requirements the issue doesn't contain as settled facts (for example, that notes already have a modification date, or a front-matter format nobody asked for), or ignores either comment.
