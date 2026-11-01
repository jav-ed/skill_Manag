# linker_Diff_View

Research done before the side-by-side changes page was built: which Rust crates and programs already draw a diff in two columns, why none of them is used yet, and which ideas were taken from them. One report, dated 2026-11-01; crates in this area are young and move weekly, so check a version or a minimum Rust version again before relying on a number.

## Docs

- [Diff options](diff_Options.md): ratatui-diff (unpublished, needs Rust 1.98), clankerdiff-ratatui, vtcode-diff, karet-diff, diffs-rs, the text diff libraries (similar, imara-diff, diffy, prettydiff, diffview) and delta, each with version, licence, minimum Rust version, downloads and the reason it was or was not used; what was learned from ratatui-diff and delta; our size for comparison; and the check to run when ratatui-diff is published. Open it to see why the page is our own code, or before moving to a ready-made widget.
