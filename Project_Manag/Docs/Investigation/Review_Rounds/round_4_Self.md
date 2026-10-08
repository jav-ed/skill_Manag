# Review rounds: Round 4, Self review

A review of the work of 2026-10-08 that came after round 3 (the targets bridge, the scan progress line, the interface pages for scan problems, history, add, init and changes, the HTML report) by the same session that wrote it, because the user ruled out any other agent for this work. It is weaker than a second pair of eyes, so a fresh review of the same code is still welcome. The method is the one of round 3: read each change asking what it broke, turn every suspicion into a test, count it as a finding only when the test failed on the code as it was, and prove each guard by breaking it and watching a test fail.

## Findings

| # | Severity | Finding | Status |
|---|---|---|---|
| S8 | Low | `status` and the report told a link problem (a real folder where the link should go) from a skill that could not be compared by the name `bridge <target>` in the list of failed skills. A skill folder that is really called `bridge claude` would have been taken for a link problem. | Fixed. A link problem is its own field, `project_problems` (data, JSON and text); `failed` holds only skills; both count as problems for the summary and the exit code. Tests: `ops::status_tests::a_missing_bridge_is_drift_and_a_blocked_one_is_a_problem`, `a_skill_folder_named_like_a_link_problem_is_still_a_skill`, `Cli/tests/bridge_links.rs::a_link_in_the_way_is_a_problem_of_the_project_and_not_of_a_skill` (mutation-checked). |
| S9 | Low | `report -o PATH` replaced a symbolic link at PATH by the report (the rename swaps the link itself) after finding the marker in the file behind it. Nothing outside the link was written, but a link the user made was silently gone. | Fixed. A link is never replaced, whatever it points at; the command refuses with exit 3 and a hint. Test: `Cli/tests/report.rs::a_link_is_never_replaced_even_when_it_points_at_a_report` (mutation-checked). |
| S10 | Info | Nothing proved that init, in the interface, still refuses a folder that got files between the question and the yes (the check is in `create_new`, which runs in the job). | Test added, mutation-checked: `Tui/src/tests/init.rs::a_folder_that_got_files_after_the_question_is_never_touched`. No code change was needed. |
| S11 | Info | The report file mode (0600, because it embeds file contents) was written down in the contract but not asserted. | Asserted in `Cli/tests/report.rs::report_writes_one_html_file_and_names_it`. |

Two guards could not be proved by a test and were kept on purpose: the `wrote > 0` check before the links of `add` are made (only reachable when every target fails while the apply itself works; the same guard keeps a failed `init` from linking a project that was taken back), and `files.sort()` in the report data (git already returns tracked files sorted).

## Suspicions that did not hold

- **A backup run is left behind when `init` cannot create its folder**: `Backups::begin` creates nothing on disk until the run stores something, so no empty run folder exists.
- **A name typed for init could escape the parent folder**: a name with a slash, `.` or `..` is refused on the page, and the folder is made with one `join` of a single component.
- **The report could run a value as script**: every value goes through `maud`; a test fills names, descriptions, file names, paths, messages and diff lines with `<script>` and tags and requires exactly one `<script` in the page. The page's Content-Security-Policy forbids any request even if one slipped through; in Chromium the page asked for nothing outside the file.
- **The scan line writes into `--json` output**: it draws on stderr only, and is not created at all for `--json`, a pipe or `TERM=dumb`; three tests prove it.

## Left as it is, on purpose

- The changes page reads the diff of every file of the plan in a job when `v` is pressed (the command `diff` does the same). A plan that touches thousands of large files makes that page slow and large; it is on demand and never blocks the interface, and the page cuts at 5000 lines.
- The report puts at most 400 lines of each file's diff into a card; `skillmirror diff` shows all of them.
- The folder picker lists directories on the interface thread (as the setup wizard always did); a very slow network folder would stall a key press. Everything else on disk is in a job.

## What a fresh review should look at

The backup and undo code (`backup/`), `apply/` (the swap and the stale check), the job discipline of the new pages (`app/jobs_events.rs`, `jobs_install.rs`), and the escaping of the report (`Crates/Web/src/report/`).
