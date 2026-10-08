# Review rounds: Reproducers

Runnable reproducers for the open findings of [round_2_Full.md](../round_2_Full.md). They print what they observe and assert nothing; each section below shows the output that proves the bug while it is open (captured on the tip of 2026-10-08, tree hash `d01061c0fe7217c4`) and the output that must appear once it is fixed. **No fix exists yet**, so the "when fixed" lines are derived from the proposed fixes, not observed. Round 1 needs no reproducers: each of its fixes has a permanent test in `Crates/`, listed in [round_1_Core.md](../round_1_Core.md).

## Files and where they belong

| File here | Copy it to | Findings |
|---|---|---|
| `Core/review_common/mod.rs` | `Crates/Core/tests/review_common/mod.rs` | helpers for both Core files |
| `Core/review_config.rs` | `Crates/Core/tests/review_config.rs` | M4 (s1, s2), L3 (p1), L4 (p2) |
| `Core/review_apply.rs` | `Crates/Core/tests/review_apply.rs` | M3 (a1), L1 (a2), L2 (a3), a4 stays correct |
| `Tui/review.rs` | `Crates/Tui/src/tests/review.rs` | M1 (u8, u2b), M2 (u1), M5 (u5), L5 (u2), L6 (u3), L7 (u6) |
| `Tui/review_draw.rs` | `Crates/Tui/src/tests/review_draw.rs` | u4, the draw fuzz |
| `Cli/cli_repro2.sh` | anywhere, run with `BIN=...` | M6 (K1), L9 (K2), negative checks K3 to K6 |

- **Names**: the Rust files and the `review_common` folder keep Rust module naming (snake case) because `mod review_common;` needs that exact name; the docs naming rule (lowercase file names, uppercase folders) applies to the Markdown around them.
- **Why two Core files and two Tui files**: the repository's line gate (`just loc-gate`) fails a `.rs` file with more than 300 code lines, and the original files were longer. The split changes nothing else.
- **One edit to existing code** for the Tui files: in `Crates/Tui/src/tests/mod.rs` add `mod review;` and `mod review_draw;` after `mod render;`, and change the `Harness` fields `rx` and `terminal` to `pub(super)`. Do this in a throwaway copy (see [review_Method.md](../review_Method.md)); do not commit the edit.
- The Core files build `ApplyOptions { threads, ..ApplyOptions::default() }`, so they compile against the tip (which has a `backup` field) and against trees without it only if you change that helper back to `ApplyOptions { threads }`.
- Do not commit the files into `Crates/` as they are: they print instead of asserting. Turn each one into an asserting test with the fix, using the "when fixed" lines as the assertions.

## Commands

From a throwaway copy of the repository (copy recipe in [review_Method.md](../review_Method.md)), with `CARGO_TARGET_DIR` pointing outside the copy's `target`:

```bash
cargo test --offline -p skillmirror-core --test review_config --test review_apply -- --nocapture --test-threads=1
cargo test --offline -p skillmirror-tui review -- --nocapture --test-threads=1
cargo build --offline -p skillmirror
BIN=$CARGO_TARGET_DIR/debug/skillmirror bash Project_Manag/Docs/Investigation/Review_Rounds/Repro/Cli/cli_repro2.sh
```

Run time on the reviewer's machine: Core about 12 s (P1 takes 8 s by design), Tui about 70 s (the draw fuzz dominates), the script a few seconds. Temp directories show as `/tmp/.tmpXXXX`; paths below are shortened to `TMP`.

## Series S: `save_config` (M4)

While open (`s1`, selected lines):

```
S blank line inside the list         ERR invalid vault config TMP/config.yaml: the rewritten file does not hold the new mandatory list | file untouched: true
S col-0 comment inside the list      ERR invalid vault config TMP/config.yaml: the rewritten file does not hold the new mandatory list | file untouched: true
S quoted key                         ERR invalid vault config TMP/config.yaml: error: line 3 column 1: duplicate mapping key: mandatory, ...
S BOM at start, key on line 1        ERR invalid vault config TMP/config.yaml: error: line 3 column 1: duplicate mapping key: mandatory, ...
S CRLF file                          OK  -> root: /a\nmandatory:\n  - c\nexclude_dirs:\n  - x\n
```

When fixed: the first four lines print `OK ->` with exactly one `mandatory` key holding `c`; the CRLF line keeps `\r\n` (the output shows `\r\n`, not `\n`). Lines that already print `OK` for indented comments, flow lists, `---`, no trailing newline, awkward names must stay as they are.

While open (`s2`):

```
S2 config.yaml is still a symlink: false
S2 dotfiles copy now: "root: /x\nmandatory: [a]\n"
S2 vault copy now:    "root: /x\nmandatory:\n  - b\n"
```

When fixed: `still a symlink: true` and the dotfiles copy now holds `- b` (or the save is refused with an error that names the real path, and nothing changes).

## Series P: profiles (L3, L4)

While open (`p1`, `p2`):

```
P1 20 levels: load ok in 1.95s
P1 22 levels: load ok in 8.05s
P1 stopping: too slow
P2 profile b1: Ok({"x"})
P2 profile b2: Ok({"x", "y"})
```

When fixed: every `P1` level loads in a few milliseconds, no `stopping` line. `P2` prints the same set for `b1` and `b2` (`{"x"}` if the contract says excludes apply to the whole closure).

## Series A: apply (M3, L1, L2)

While open:

```
A1 300 rounds on fresh projects: b failed 300x (expected), a failed 109x, c failed 109x
A1 sample failure of an innocent sibling: Apply(Swap { stage: "TMP/p0/.agents/.stage-...-0", dest: "TMP/p0/.agents/skills/a", source: Os { code: 2, kind: NotFound, ... } })
A2 same-size edit with restored mtime: "Updated"; now "NEW!\n"
A2 ordinary edit: "Failed(Apply(DestinationChanged { dest: \"TMP/proj/.agents/skills/sk\" }))"; file kept: "OLD1 plus my edit\n"
A3 plan kind: Ok(Unchanged)
A3 outcome after an identical re-save: Failed(Apply(DestinationChanged { dest: "TMP/proj/.agents/skills/sk" }))
```

The A1 count varies with load (203 per sibling on the snapshot run, 109 on the tip run); any value above 0 is the bug.

When fixed: `A1 ... a failed 0x, c failed 0x` and no sample line; `A2` first line is `Failed(Apply(DestinationChanged ...))` with the edit `"EDIT\n"` kept, the ordinary-edit line unchanged; `A3` outcome is `Unchanged`.

Must stay as it is (`a4`, correct behavior):

```
A4 outcome: Failed(Apply(DestinationChanged { dest: "TMP/proj/.agents/skills/sk" })); notes.md kept: true; SKILL.md: "OLD\n"; .agents: ["skills"]
```

## Series U: interface (M1, M2, M5, L5, L6, L7)

While open:

```
U1 after Enter on the Sync page: Sync/running 0/3
U1 after clicking the header arrow: menu
U1 Delete page opened from the stale session: Delete/select
U1 after the first job reported: Delete/done(results of Sync)
U2 header: "Select skills to delete   3 / 3 selected"; rows shown with a mark: 1
U2 dialog title: "Delete 3 skills in 2 projects? ..."
U2b one Enter later: Push/running 0/6
U3 running: Sync/running 0/3, should_quit: true
U5 wizard result: saved with error: invalid vault config TMP/vault/config.yaml: mandatory lists "tmux" twice
U5 pointer file exists: true
U6 first scan's result populated the second page: List/select
U8 my_notes.md survived the sync: false
U8 results page mentions it: false
U4 15200 draws, 0 panics
```

When fixed:

- `U1`: the header click during a run is ignored (the phase stays `Sync/running ...`), or the stale event is dropped; the Delete page never shows `results of Sync`.
- `U2`: depends on the policy chosen for L5 (Enter acts on visible rows only, or the header shows shown and selected counts and the dialog lists names).
- `U2b` and `U8`: Enter on Push or Sync prints `.../confirm` and the confirm page lists `my_notes.md` as removed; confirming then ends with either the file kept and `DestinationChanged` in the results, or the file listed as removed in the preview the user accepted.
- `U3`: `should_quit: false` after the first Ctrl-C during `Running`.
- `U5`: `pointer file exists: false`.
- `U6`: unchanged; it documents the benign sibling of L7, which needs a setup change in between to show the bug.
- `U4`: must stay `0 panics`.

## Series K: command line (M6, L9)

While open (`cli_repro2.sh`, selected lines):

```
$ skillmirror init TMP/new1 --yes   ... 1 skill in 1 project: 1 failed   [exit 4]
new1 exists after the failed init: YES
new1b exists: YES; has .git: YES
error: TMP/new1b is not empty   [exit 3]        (the second try of the same command)
new2/.git exists: no; GIT_DIR target created: YES
```

When fixed: `new1 exists after the failed init: no`, `new1b exists: no; has .git: no`, the second `init` repeats the failed row (exit 4) instead of `is not empty`, and `new2/.git exists: YES; GIT_DIR target created: no`.

K3 to K6 are negative checks and must keep their output: dry runs create no directory (`proj3/.agents exists: no`, `new3 exists: no`), `add` twice ends `up to date`, `add` through a linked `.agents` ends `error: ... must be a real directory, not a link or a file` with exit 3, and unknown group, unknown profile and an empty selection are exit 3 with a hint. K7 (finding L10) is a note in the script: it needs a terminal and is not run.
