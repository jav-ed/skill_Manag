# Side-by-side diff

Decision, 2026-11-01: the changes of a sync, push, add, init or agents run are shown on one page of our own, the old text on the left and the new text on the right, as in the diff editor of an editor. The page is opened with `v` on a skill of the sync or push list (before anything is ticked) and with `v` on the question of a run. It replaces the one-column unified page that only the question had. It is built on `similar` 3 (already a dependency), with our own pairing of replaced lines, our own word marks and plain ratatui drawing. The evidence is in [Diff options](../Investigation/Diff_View/diff_Options.md).

## What the page is

- **Two sides.** Left is the project now, right is what the run brings in (for AGENTS.md: the file now and the file after). Line numbers, a red tint for removed lines and a green one for added lines, a stronger tint on the words that changed, hatching where a side has no line, long lines wrapped with both sides kept in step, a ruler at the right edge, a head for each skill and file.
- **One change, once.** Targets of one skill with exactly the same files and texts are one group: "the same change in 20 projects" and the names of the projects, not twenty pages. The group with most projects comes first.
- **Helpers.** `n` and `p` the next and previous change (the header buttons do the same), `]` and `[` (or tab) files, `}` and `{` skills, `g` and `G` the ends, `ctrl+d` and `ctrl+u` half pages, `s` side by side or inline, `c` folded context or whole files, a click on a fold opens it, a click or drag on the ruler goes there, the wheel scrolls. A terminal narrower than 70 columns shows one column; narrower than 30 says so.
- **Honest about what it cannot show.** A target that cannot be compared or applied is listed with the reason, a binary, too large (over 1 MiB) or unreadable file is named, a page of more than 20,000 items stops with the way to see the rest (`skillmirror diff`). Nothing is dropped without a word. Looking writes nothing.
- **One reading.** The rows and the unified text of `skillmirror diff` come from the same file reading (`ops/inspect/diff/files.rs`), so the two cannot disagree.

## Rejected

- **A ready-made widget now.** ratatui-diff fits best and is the one to watch, but it is not published, a git dependency is refused by the supply-chain gate, and it needs Rust 1.98 where the workspace promises 1.89. The other crates are engines, parts of other programs or too new.
- **Showing `delta` or `git diff --word-diff` output.** A subprocess, a program that must be installed, text to take apart again, and no click, fold, ruler or jump.
- **Two pages** (one unified for the question, one side by side for the list). One page, one set of keys, one set of tests.
- **Marking every word of a replaced line.** A line whose words are mostly new is tinted whole; marks only help where a line was edited.

## Rulings still open (the user's)

1. Move to ratatui-diff when it is published and its Rust floor is acceptable? It would bring search, selection and horizontal scroll, and would take over the drawing; the grouping, fold ids, ruler and failures would stay ours.
2. Ideas not built, in the order they would help: search with `/`, a switch that ignores whitespace, horizontal scroll instead of wrapping, copying the text of a line, syntax colours, a click on a project in the right pane to see only its changes, and the same page on the Override page (vault left, project right).
