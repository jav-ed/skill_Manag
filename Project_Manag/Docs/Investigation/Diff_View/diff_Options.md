# Diff view: what exists in Rust

Decision: build the side-by-side changes page ourselves on `similar` (already a dependency, 3.2 since the update of 2026-11-01), with our own pairing of replaced lines and our own word marks, drawn with plain ratatui. No ready-made diff widget can be used today. The one that fits best, joshka/ratatui-diff, is not on crates.io (the name holds a `0.0.0` placeholder), the supply-chain gate forbids git dependencies (`deny.toml`: `unknown-git = "deny"`, `allow-git = []`), and it needs Rust 1.98 where the workspace promises 1.89. The other crates are engines without a widget, parts of an agent or an editor, or too new to trust. `delta`, the best known side-by-side view in a terminal, is a program that prints text and has no library. The decision is in [Side-by-side diff](../../Decisions/side_By_Side_Diff.md); this file holds the evidence. Re-check ratatui-diff when it is published (last section).

## Method and caveats

- Date 2026-11-01. Versions, downloads, licences, minimum Rust versions (MSRV) and descriptions come from the crates.io API (`https://crates.io/api/v1/crates/NAME`); stars, push dates and releases from `gh api repos/OWNER/NAME`. Nothing was built into the app. ratatui-diff and delta were cloned (`Repos/ratatui-diff` at `dc8765b`, 2026-10-30; `Repos/delta` at `3c2269c`, 2026-03-08) and read: manifests, README, CHANGELOG and the files named below. The other crates were judged from their metadata and descriptions only, not run.
- What the page had to do, from the request of 2026-11-01 (a screenshot of a diff editor): old text left, new text right; line numbers; red and green line tint with a stronger tint on the changed words; hatched filler where a side has no line; long lines wrapped with both sides kept in step; a ruler at the right edge; reachable from the sync list before anything is ticked; the same page behind the question of a run and for AGENTS.md; one change shown once when twenty projects get exactly the same change; mouse and keyboard; failures listed, never dropped.
- Our own size, for the comparison: about 530 code lines in Core (`ops/inspect/diff/sides`), 770 in the state, layout, keys and mouse (`screens/diff`), 680 in the drawing (`ui/pages/diff`), plus 640 lines of tests.

## Ready-made ratatui widgets

- **ratatui-diff** (joshka, 10 stars on GitHub, pushed 2026-10-30; source version 0.1.0, edition 2024, `MIT OR Apache-2.0` in `Cargo.toml`, the GitHub API reports Apache-2.0). Its CHANGELOG says "Unreleased: First release". On crates.io the name `ratatui-diff` is a `0.0.0` placeholder (MIT, 195 downloads, repository field `ratatui/ratatui`, updated 2026-06-12). `rust-version = "1.98"`. Dependencies: `ratatui-core` 0.1, `similar` 3, `diffy` 0.5, `unicode-segmentation`, `unicode-width` 0.2, and optional `syntect` and `two-face` for colours. `src/` is 5,239 lines (`widget.rs` alone 2,612). Features that match the request: unified and split view, "similarity-aligned replacements", word highlights, wrapping, line numbers, hunk and file movement, folding, sticky file headers. Features beyond it: search, selection with exact source text, hit-testing of cells, horizontal scroll, whitespace markers, syntax colours. What stops it today: (1) not published, and a git dependency is refused by `cargo deny`; (2) MSRV 1.98 would lift the workspace floor from 1.89 by nine releases for one page; (3) it takes one `DiffDocument` and knows nothing of our grouping of equal changes, fold ids, ruler, per-target failures or the AGENTS.md flow, so most of the page would still be ours. Verdict: not now, read it, check again.
- **clankerdiff-ratatui** 0.8.2 (MIT, 1,962 downloads, 26 versions since 2026-09-08, MSRV 1.98): "Embeddable Ratatui diff review widget". Same MSRV wall, three months old. No.

## Diff engines and renderer-neutral rows

- **vtcode-diff** 0.180.5 (MIT or Apache-2.0, 4,773 downloads, MSRV 1.98.1): "Bounded structured text diffs and renderer-neutral terminal preview rows". A part of an agent application, MSRV above ours. No.
- **karet-diff** 0.7.1 (MIT or Apache-2.0, 2,582 downloads, MSRV 1.92): "Pure text diffing engine (histogram line diff, intra-line highlights, unified-diff parse, hunk staging) for karet". An engine made for the karet project and no widget; its two features we need (histogram lines, intra-line marks) are what `similar` and our `words.rs` already give. No.
- **diffs-rs** 0.1.1 (Apache-2.0, 21 downloads, created 2026-10-27, MSRV 1.88): "Fast terminal diff viewer and embeddable Rust diff engine". Created five days before this was read. No.

## Text diff libraries

- **similar** 3.2.1 (Apache-2.0, 236.8M downloads, MSRV 1.85). Already used by `diff` and `skillmirror diff`; the page was built on 2.7.0 and moved to 3 on 2026-11-01, where the only change the code needed was that `TextDiff` no longer gives its lines as a slice (`iter_old_slices` instead of `old_slices`). It gives line operations (`TextDiff::from_lines`, `DiffOp`) and the Myers algorithm over any slice, which is what the word marks use on tokens. Yes. 
- **imara-diff** 0.2.0 (Apache-2.0, 35.8M downloads, MSRV 1.71): faster histogram diff, no text-level helpers. Kept as the upgrade path if large files ever need it; files over 1 MiB are listed and not shown, so the sizes stay small. No.
- **diffy** 0.5.2 (MIT or Apache-2.0, 19.2M downloads, MSRV 1.85): diff and patch; no advantage here.
- **prettydiff** 0.9.0 (MIT, 9.0M downloads, MSRV 1.70) and **diffview** 0.1.0 (MIT, 281 downloads): "Side-by-side diff for two files" and "Side-by-side terminal diff viewer". Both are made to print finished text for a terminal (I did not read their code), which does not fit our own layout with hover, click and folds. No.

## Programs

- **delta** 0.20.1 (MIT, 32.6k stars, released 2026-10-04): crate `git-delta`, edition 2018, one `[[bin]]` and no `[lib]`, so there is nothing to depend on. Its `--side-by-side` view prints ANSI text. Running it from the interface would need the program installed, a parser from ANSI back into ratatui cells, and would give up click, fold, ruler and jumps. No. Worth reading: `src/edits.rs` (`infer_edits`: which removed line goes with which added line, by an edit-distance limit), `src/align.rs` (a Needleman-Wunsch table for the edits inside a line), `src/wrapping.rs` and `src/features/side_by_side.rs` (long lines in two columns).
- `git diff --word-diff` and `git diff --color-words`: a subprocess, a git repository around both sides (the project copy is not always in one) and text to parse. No.

## What we took from them

Ideas only; no code was copied (ratatui-diff is MIT or Apache-2.0, delta is MIT, and the project licence is Hippocratic).

- From the ratatui-diff docs (`src/lib.rs` line 64): replaced lines are paired by similarity anchors; stretches without an anchor, and runs that are too large, pair in source order. Ours: `pairs.rs`, bigram Dice similarity of at least 0.5, at most 256 lines and 64 KiB per run, then in order.
- From delta (`src/edits.rs`): a limit on how different two lines may be and still count as the same line. Ours is the same similarity threshold.
- Words as units, not characters: runs of letters, runs of blanks and single other characters, with the marks of changed words that only blanks separate joined into one. When the marks cover more than 70% of the visible characters on both sides, the line was replaced and not edited, so it is tinted whole and carries no marks.
- Both sides of a wrapped row are as tall as the taller side and padded, so the columns stay in step (the idea of delta's wrapping); the layout counts rows with the same `wrap` function the drawing uses, so a reserved row is a row that is filled.

## Verdict table

| Option | Kind | Licence | MSRV | Verdict |
| --- | --- | --- | --- | --- |
| similar 3.2 | engine, ours | Apache-2.0 | 1.85 | yes |
| own pairing, marks, layout, drawing | code, ours | project | 1.89 | yes, about 2,000 lines |
| ratatui-diff (joshka) | widget | MIT or Apache-2.0 | 1.98 | not now, unpublished |
| clankerdiff-ratatui | widget | MIT | 1.98 | no |
| vtcode-diff | rows, part of an app | MIT or Apache-2.0 | 1.98.1 | no |
| karet-diff | engine, part of an editor | MIT or Apache-2.0 | 1.92 | no |
| diffs-rs | viewer and engine | Apache-2.0 | 1.88 | no, too new |
| imara-diff, diffy | engines | Apache-2.0 / MIT or Apache-2.0 | 1.71 / 1.85 | no |
| prettydiff, diffview | printers | MIT | 1.70 / none | no |
| delta | program | MIT | edition 2018 | read only |

## Re-check

When ratatui-diff is on crates.io as `0.1` or later: run `curl -s https://crates.io/api/v1/crates/ratatui-diff` and look at `rust_version` and `license`. If the MSRV is below the workspace floor (or the floor is raised on purpose), compare what it has that the page has not (search, selection, copy, horizontal scroll, syntax colours; the ideas listed in the decision) with what it would take to feed it our groups, fold ids and failures. Use `git log` on `Repos/ratatui-diff` for what changed since `dc8765b`. A move is a ruling for the user, not something to do on the way.
