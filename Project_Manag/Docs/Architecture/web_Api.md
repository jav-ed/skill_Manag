# web_Api

The JSON API of `skillmirror web`: the contract between the Rust server (`Crates/Web/src/server/api/`) and the web interface (`Ui/`). The shapes here are pinned by `Crates/Web/src/server/tests/read.rs` and `flow.rs`; `Ui/src/app/types.ts` mirrors them. A change to a shape changes both sides and this page.

## Rules for every request

- Every request needs the session cookie (the one-time link in the terminal is traded for it) and a `Host` of `127.0.0.1:PORT` or `localhost:PORT`; see [front_Ends.md](front_Ends.md) for the whole guard. Without them the answer is 401 or 403 and no JSON.
- Reads are `GET`. A change is a `POST` with `Content-Type: application/json` and the header `X-Skillmirror: 1`, at most 64 KB.
- Without `--allow-write` the changing endpoints answer 403.
- An error answer is `{"error": "a sentence for the person"}` with a status: 400 the request is wrong, 403 not allowed, 404 no such plan or job, 409 another change is running, 422 the engine refused it (unknown skill, project not found, and so on), 500 the vault or the configuration cannot be read.
- Paths in answers are absolute; `label` is the same project as the pages name it (below the scan root).

## Reads

| `GET` | Answer |
|---|---|
| `/api/session` | `{allow_write, version}` |
| `/api/overview` | `{at, vault, root, projects[], in_sync, drift, problems, issues[]}`; a project is `{path, label, state: in_sync\|drift\|problem, current, outdated[{skill, added, changed, removed}], missing[], missing_links[], not_in_vault[], problems[{name, message, hint}], link_problems[{name, message, hint}]}`; an issue is `{path, message}` |
| `/api/skills/sync`, `/api/skills/push` | `{rows[], projects[{path, label}]}`; a row is `{name, group, mandatory, current, outdated, missing, problems}`. Sync: skills a project has and the vault knows; push: the mandatory skills |
| `/api/vault` | `{skills[], foreign[]}`; a skill is `{name, group, mandatory, description, header_problem, files[], untracked[], profiles[], projects[{project, state: current\|outdated\|missing\|not_in_vault\|problem, detail}]}`; `foreign` holds folders that projects have and the vault does not |
| `/api/history` | `{runs[{id, date, command, skills, projects, error}]}`, newest first |
| `/api/doctor` | `{findings[{severity: note\|warning\|error, check, subject, message, hint}]}` |
| `/api/settings` | `{vault: {path, source}\|null, root, mandatory[], targets[], exclude_dirs[], exclude_paths[], profiles[{name, description}], allow_write}` |

## Changes

| `POST` | Body | Answer |
|---|---|---|
| `/api/rescan` | `{}` | `{at, projects}`; the next read looks at the disk again |
| `/api/plan` | `{kind: "sync"\|"push", skills[], project?}` | `{plan, kind, rows[], create, update, unchanged, failed, projects}`; a row is `{project, skill, action: create\|update\|unchanged\|failed, detail, files[{path, kind: added\|changed\|mode\|removed, added, removed, text, note}]}`. Looks at the disk now, writes nothing, keeps the plan for 15 minutes (8 at most) |
| `/api/undo-plan` | `{run}` | `{plan, run, date, command, lines[{project, skill, was, step: restore\|remove\|gone\|failed, message}], actionable, failed}`; writes nothing |
| `/api/apply` | `{plan}` (a plan or an undo plan) | `{job}`; takes the plan out, so it runs once |

| `GET` | Answer |
|---|---|
| `/api/job/{id}` | `{state: "running", done, total}`, or `{state: "done", title, lines[{project, skill, outcome, detail}], failed, backup, warnings[]}`, or `{state: "failed", message}` |

## Not API

`/report` is the static HTML report of the same data (its own policy, inline script and style). Every other address is a file of the interface (`/`, `/skills`, `/sync`, `/push`, `/history`, `/doctor`, `/settings` and `/_astro/...`) or 404.
