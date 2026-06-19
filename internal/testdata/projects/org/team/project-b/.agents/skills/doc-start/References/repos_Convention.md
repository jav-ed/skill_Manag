# Repos Convention

Rules for the `/Repos/` folder at the repo root, the shallow clones it holds, and the `repos_List.md` manifest that tracks them.

## Why the folder exists

Agents often need to read the source code or documentation of a third-party project: to look up an exact API surface, to confirm version-specific behavior, or to read docs that are only complete in the project's repo (not on its website). Cloning the relevant repos locally with `git clone --depth 1` makes that material directly readable without network calls and without pulling full git history. `gh` is also installed, so the clone URL is easy to find with `gh repo view <owner>/<repo>` and docs-only sibling repos can be discovered with `gh search repos`.

## Where things live

- `/Repos/` (repo root) — the clones. Fully gitignored: nothing inside this folder is ever committed.
- `Project_Manag/Docs/Setup/repos_List.md` — the manifest. Lives under `Docs/Setup/` so it stays under git. This file is the source of truth for what should be present in `/Repos/` on a working machine.

The split exists because the clones are large and not source-of-truth (they can always be re-cloned), but the *list* of which clones the project depends on is small, important, and must travel with the project.

## Gitignore

The `bootstrap.sh` script appends `Repos/` to `.gitignore` (creating the file if absent) when run. If the folder appears later in a project that did not bootstrap, add the line by hand. There is no exception pattern: nothing inside `/Repos/` is tracked.

## Cloning rules

1. **Always shallow.** Use `git clone --depth 1 <url> Repos/<name>`. We do not need history; we need the current source or docs.
2. **Always inside `/Repos/`.** Never clone into the project root, into `Project_Manag/`, or into any source folder.
3. **One folder per repo.** Use the repo name as the folder name (`Repos/next.js`, `Repos/anthropic-cookbook`). If two repos collide on name, prefix with the owner (`Repos/vercel-next.js`).
4. **Add an entry to `repos_List.md` in the same change.** A clone without a manifest entry is invisible to the next person who checks the project out: they will not know the clone is expected.

## Manifest format

`Project_Manag/Docs/Setup/repos_List.md` opens with a short summary explaining what the folder is for, then lists each tracked repo. The expected entry shape:

```markdown
- **<repo-name>**: one-line reason this is here (source code reference / documentation / both).
  - URL: https://github.com/<owner>/<repo>
  - Clone: `git clone --depth 1 https://github.com/<owner>/<repo> Repos/<repo-name>`
```

The "reason" line should be specific enough that a reader can decide whether to bother cloning it: "examples of streaming with the SDK" beats "for the SDK".

## Source code vs documentation

Sometimes the source code repo and the documentation repo are different. For example, a framework may publish its docs from a separate repo than its code. When this happens, list both:

```markdown
- **next.js**: source code reference for the framework.
  - URL: https://github.com/vercel/next.js
  - Clone: `git clone --depth 1 https://github.com/vercel/next.js Repos/next.js`
- **next.js-docs**: documentation source for nextjs.org.
  - URL: https://github.com/vercel/next-site
  - Clone: `git clone --depth 1 https://github.com/vercel/next-site Repos/next.js-docs`
```

Use `gh search repos <project> docs` or browse the project's `gh repo view` description to discover docs-only sibling repos before assuming there is only one repo to clone.

## Repopulating on a fresh machine

Anyone checking out the project on a new machine starts with no `/Repos/` folder (it is gitignored). To restore it:

1. Read `Project_Manag/Docs/Setup/repos_List.md`.
2. Run each `Clone:` command in the manifest, in any order.

There is no automated re-clone script by default. The manifest is plain markdown so it stays readable without tooling, and so an agent can scan it and run the commands without needing to execute a project-specific shell script. A `bash`-runnable variant can be added later if the list grows long.

## When `gh` helps

- **Finding the URL:** `gh repo view <owner>/<repo>` — confirms the repo exists, shows the canonical URL, default branch, and a description that often reveals whether docs live elsewhere.
- **Finding sibling docs repos:** `gh search repos "<project> docs"` and `gh search repos --owner=<owner>` — surfaces docs-only repos under the same owner.
- **Cloning via `gh`:** `gh repo clone <owner>/<repo> Repos/<name> -- --depth 1` is equivalent to the `git clone --depth 1` form. Use either; the manifest should still record the `git clone --depth 1` command for portability (`gh` is not installed everywhere).
