# Handoff: next steps

*What is left, in order, each item with a "done when". Section A is the work that closes the gaps listed in `handoff_Overview.md`; section B is the history of what the lead finished (kept so a reader knows what exists and where its tests are); section C is the parked ideas. When you finish an item, tick it here and note what you learned in the pitfalls file.*

Status marks: `[ ]` open, `[~]` partly done, `[x]` done.

## A. Open work, in order

### A1. [ ] Run it on the real vault and tree, read-only (gap 1)

Design: [handoff_Overview.md](handoff_Overview.md), "First hour", step 5. `skills`, `status`, `sync --dry-run`, `info`, `doctor`, `report`, `web` without `--allow-write`. Compare `list --json | jq length` with the earlier count (436 skills in 65 projects). `doctor` also reads the headers of the real vault's skills: two had an unquoted `: ` in the description (`post-scheduler` and the old `secrets`), which `doctor` reports; fixing them is the user's call in the vault.

Done when: a short report to the user with the output of each command, every surprise explained or turned into a failing test first, and no write has happened.

### A2. [ ] Use it and list what is awkward (gap 2)

The terminal interface and the web interface have been checked by tests and by the lead's reading of screenshots, never judged by the user. Use both on a throwaway world, then on the real tree. Things worth a look: how the Skills browser reads with 24 skills in groups, the overview with 65 projects (the web table and its filter), the plan page when 60 projects change, the first-run wizard from nothing (`vault init`, `new`, `adopt` and the Setup screen), light and dark theme, small windows, a slow terminal over SSH.

Done when: the list is with the user, small fixes are in with tests, choices are the user's.

### A3. [ ] Close the CI gap (gap 3)

Add `just check-features` and `just ui-check` to the `check-deps` job of `.github/workflows/ci.yml` (the second needs a node 22.12 step, `actions/setup-node`, and `npm ci` is done by the script). Optionally the Chromium check `Code/Development/Web/check_Web.sh` as its own job (Playwright and a browser need installing in the job). Needs a token with the `workflows` permission, so this is a person's job or a session the user gives that permission.

Done when: a pull request that edits `Ui/` without rebuilding `Crates/Web/assets/ui/` fails CI (try it once on a throwaway branch, then close the pull request).

### A4. [ ] Push the tag (gap 4)

`git tag go-oracle c7310f9 && git push origin go-oracle`, after asking the user. Then update the sentence in `Docs/Architecture/go_Legacy.md` ("tag `go-oracle` once the maintainer has pushed it") and in [cutover_Runbook.md](cutover_Runbook.md).

### A5. [ ] A fresh review (gap 5)

Two things deserve eyes that did not write them: the backup and undo code (`Crates/Core/src/backup/`, `ops/history.rs`) because it holds the user's data, and the web server (`Crates/Web/src/server/`: `guard.rs`, `plans.rs`, `jobs.rs`) because it is the only network-facing part. Method: `Docs/Investigation/Review_Rounds/review_Method.md` (reproducers become failing tests; draw fuzz; race loops). The security list the server must keep is in `Docs/Architecture/front_Ends.md`.

Done when: no High and no Medium finding is open; every fix started as a failing test.

### A6. [ ] Other machines (gap 6)

Run `just parity` as a normal user (87 / 44 / 0 was the result before the lead ran as root; the scenarios that depend on read-only permissions behave differently for root, `Scratch/Oracle/parity/report.md` lists each after a run), look at `skillmirror web` in Firefox, build once on a machine with 8 GB and write the real number into `Docs/Setup/build_Resources.md`.

### A7. [ ] Decisions that wait for the user

[open_Questions.md](open_Questions.md): one batch, with the lead's default for each.

### A7b. [ ] Use the AGENTS.md feature on the real vault and finish its edges (gap 10)

Built on 2026-10-10 and tested only in throwaway worlds. Do, read-only first: `skillmirror agents status` on the real tree (every project is `missing` until a file exists; that is normal), then put the text you want in `<vault>/AGENTS.md` (or keep the built-in five rules), make sure `doc-start`, `coding` and `file-tree-optimization` are mandatory, and only with the user's yes run `skillmirror init` for a new project or `skillmirror agents add --project DIR` for one existing project. Open edges, none of them blocking: the web view has no row for it (`Docs/Architecture/web_Api.md`, `Ui/`), `doctor` does not read `<vault>/AGENTS.md`, `status` does not say how many blocks are out of date, a roll-out to every project at once is `agents add` per project today (no `--all` for `add`, on purpose: it writes into about 60 projects), and the `wrote > 0` condition in `commands/pipeline.rs` survives a mutation check.

Done when: the user has seen the block in a real project, the text they want is in the vault, and each edge above is either built or crossed out here with the user's word.

### A8. [ ] Promote this folder, then ask about the branch

[cutover_Runbook.md](cutover_Runbook.md), last section.

## B. Finished by the lead (what exists, where its tests are)

1. **[x] Backup store and `undo`.** `Crates/Core/src/backup/` (store, `Run`, `undo`, tests in `tests.rs`, `apply_tests.rs`, `delete_tests.rs`, `undo_tests.rs`); `ops::delete` moves the folder to `.trash-*` first, then into the store, and renames it back when the store fails; the CLI starts a run before each write and prints `Backup: run <id>`; `undo` and `history` (`Cli/tests/undo.rs`, `undo_terminal.rs`); the TUI results page names the run. Three mutations (no swap-back, unchecked run id, prune after every run) were each caught. Contract Q34.
2. **[x] Review round 2** (`Docs/Investigation/Review_Rounds/round_2_Full.md`): M1 to M6, L1 to L12 and I1, each as a failing test first. Decisions in the contract: Q35 (profile `exclude`), Q36 (plan before confirm, job discipline), Q37 (selection and filter), Q38 (no fsync), Q39 (leftovers). Declined with a reason: comments inside a replaced `mandatory` list go with the old items; a read-only `config.yaml` is replaced with its mode kept; no cancel flag for a running job (Ctrl-C asks twice).
3. **[x] Phase 5 rest.** Scan progress (Q45), `status` (Q41), `diff` (Q42), `doctor` (Q43), the `targets` bridge (Q44). Open in the bridge: no test makes `add` apply with zero successful writes, so the `wrote > 0` guard in `commands/pipeline.rs` survives a mutation check. `doctor` does not probe `renameat2` on the real filesystem (it would write into the user's projects); it checks the filesystem type.
4. **[x] Terminal interface completion.** Scan problems (`i`, Q46), History with undo (Q47), Add and Init (Q48), the changes page behind `v` (Q49), the Skills browser with a detail card (Q55). One real-terminal test per screen in `Crates/Cli/tests/interface_screens.rs`.
5. **[x] Reports and web.** `skillmirror report` (Q50), `skillmirror web` with the security minimum of the decision record (loopback, one-time token traded for a `HttpOnly; SameSite=Strict` cookie, Host/Origin/Sec-Fetch-Site checks, JSON plus `X-Skillmirror: 1` on every change, strict CSP, 64 KB body limit) and writes behind `--allow-write` with a plan first and a plan that runs once (Q56). The interface is the `Ui/` project (Astro shell, Solid, lucide-solid, Motion), meeting Rust in the documented JSON API (`Docs/Architecture/web_Api.md`) and in the built files committed under `Crates/Web/assets/ui/`.
6. **[x] What a regular user needs.** Scoped `sync`/`push`/`status` (skill names, group, profile, project; Q51), `info` (Q52), `new`, `adopt`, `vault init` (Q53), `config`, `mandatory` (Q54).
7. **[x] Build cost.** `[profile.dev] debug = false, incremental = false`, documented in `Docs/Setup/build_Resources.md` with measured numbers and the switches to turn them back on.
8. **[x] Documentation pass.** README, architecture, `sync_Concept.md`, the contract rows, the web API page, `doc_Start.md`; the Astro leftovers of an earlier website idea are gone.
9. **[x] The AGENTS.md of a project (2026-10-10).** Contract Q57 to Q62, decision record section 8. Core `agents/` (text, block, inspect, plan, apply), single-file entries in the backup store (`Subject::Agents`, `file_entry.rs`, `undo_file.rs`), `agents status|sync|add` and `init` writing the file (`--no-agents-md`), the Agents page and `m` on the Init page. Tests: `agents/*_tests.rs` in Core, `Cli/tests/agents.rs` and `agents_write.rs`, `Tui/src/tests/agents.rs` and `init_agents.rs`, `Cli/tests/interface_agents.rs`, and the AGENTS.md section of `Code/Development/Smoke/check_Smoke.sh`. 24 of 25 guard mutations are caught.
10. **[x] Cutover, the code part.** The Go tree is deleted; the merge into `main` is done (pull request 1). What is left of the cutover is A4 and A8.

## C. Separate track: skills (not part of the rewrite; unchanged since the first handoff)

The first lead edited three skills in this repo's own `.agents/skills/` (kept out of the pushed commits): `secrets` (new: owns Sops, Age, Fnox and a new `Age/keys_And_Recipients.md`), `default-tools` (no longer owns secrets, carries its commands itself), `skill-writer` (refreshed from the vault plus an "Independence" section). The user must review them, copy them to the vault by hand and add `secrets` to `mandatory` in the vault config; until then nobody may run a real sync or push on that machine. Also open: `remote-helper` is bound to one project; sibling links in `coding` and `file-tree-optimization` (`../coding`, `../refac-cli`, `../../default-tools`); `post-scheduler` and the old vault `secrets` have invalid frontmatter (an unquoted `: `). These are the user's call; do not touch the vault.

## D. Parked ideas and one pending report

- Pending report for the user: a short list of crates and tools that are good but blocked by the Hippocratic licence. The current list is in section 7 of the decision record (`git2`, `skiller`, the `cargo-binstall` libraries, `bacon`, `slint`); search the research reports for "licen" to complete it and send the user one consolidated list.
- Registry cache (`$XDG_CACHE_HOME/skillmirror/` list of projects): only after the cold scan is measured (needs `drop_caches`, ask the user) and only if it is still slow. The scan is 0.7 s warm today.
- Provenance lock: deferred by the user ("maybe later"); would give three-way drift states. Do not start without the user.
- Parked: skill dependency hints (a skill should not link `../coding`; the tool could warn), further agent directories beyond `.claude/skills` (a row in `KNOWN_TARGETS`, `config/vault_config.rs`), a `--commit` option after sync, watch mode. All judged "not now" in the decision record.
