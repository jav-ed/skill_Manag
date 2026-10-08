# Rust stack: Prior art

This file surveys the tools that manage, install or sync agent skills and agent configuration, the skill directory conventions of the major coding agents, the `SKILL.md` format rules, and what users of the closest tools ask for. Everything was checked live on 2026-10-07. The ecosystem has converged on the Agent Skills format (a folder with `SKILL.md`, `name` equal to the folder name) and on `.agents/skills/` as the shared project directory: Codex, Gemini CLI, Cursor, GitHub Copilot, OpenCode, Windsurf and Roo Code read it. Claude Code does not (two open issues with 543 and 56 reactions), so a `.claude/skills` bridge is still needed. No surveyed tool combines a nested vault, profiles for new projects, scan-root discovery and local-edit-safe mirroring. The closest, skillshare (Go, 2.7k stars), flattens groups with a `__` prefix that breaks the name-equals-folder rule, overwrites copies whenever the source changes, and has no profiles. Verdicts. Build now: groups as vault-only folders with a flat installed layout, profiles as named lists, `add`, `init`, `status` and `diff`, change-aware sync, a provenance lock with a three-way drift guard, backup with `undo`, `--json`, `--check`, `doctor` with SKILL.md lint, and `targets` (canonical `agents` plus an optional `claude` bridge). Build later: `adopt` (project to vault), dependency closure for `add`, a read-only web view, a `hold` marker for deliberate forks. Do not build: commit-after-sync, watch mode, enable/disable, version pinning, merge with conflict markers, registry or marketplace, security audit.

Related files: [Requirements](rewrite_Requirements.md): the baseline every feature here is judged against. [Engine stack](engine_Stack.md): hashing, git index and YAML crates that the lock and config shapes below depend on. [Web view](web_View.md): decides the delivery of the optional web front-end whose scope this file only bounds. [Name and distribution](name_And_Distribution.md): may rename the tool, so examples below write `<tool>`.

## Headline findings

1. **`.agents/skills` is the shared directory, with one big exception.** Seven agents read it natively (matrix below). Claude Code reads only `.claude/skills`; the request to add `.agents/skills` is still open on 2026-10-07 ([#16345](https://github.com/anthropics/claude-code/issues/16345), 56 reactions; [#31005](https://github.com/anthropics/claude-code/issues/31005), 543 reactions). This repo bridges locally with a symlink `.claude/skills -> ../.agents/skills`, but `.gitignore` line 46 excludes it, so a fresh clone (the SSH case this tool exists for) has skills and no bridge. The tool must treat `.agents/skills` as canonical, model the bridge as an optional target, and let `doctor` and `sync` detect and recreate a missing bridge.
2. **Groups must stay vault-only; the installed layout stays flat.** Claude Code documents direct children only, Cursor walks recursively, Codex has a recursive mode, Windsurf and Gemini CLI do not say. The spec and five agents (Cursor, OpenCode, Roo Code, Kiro, Cline) require `name` to equal the folder. Skillshare's `__` flattening produced exactly the warnings this predicts ([skillshare #110](https://github.com/runkids/skillshare/issues/110)). Our skills also link by relative path (`../coding/SKILL.md`, `../../../Project_Manag/...`), which only resolve at the flat installed depth.
3. **Nobody has shipped local-edit-safe mirroring.** Vercel's `skills` CLI (33k stars) compares upstream against its lock and never hashes the installed files ([#1812](https://github.com/vercel-labs/skills/issues/1812), [#455](https://github.com/vercel-labs/skills/issues/455)); its `verify` and taxonomy PRs were closed unmerged on 2026-06-08. Skillshare's copy mode compares source checksums only. A lock that records the installed hash gives a three-way state (clean, outdated, edited, conflict), which is the feature gap to fill.
4. **`category`, `dependencies` and `version` are not in the spec.** `category` was proposed and closed without adoption ([agentskills #55](https://github.com/agentskills/agentskills/issues/55)); skill dependencies are open and contested ([#100](https://github.com/agentskills/agentskills/issues/100), [#485](https://github.com/agentskills/agentskills/issues/485)); the reference validator rejects unknown keys. Groups and profiles therefore live in folders and `config.yaml`, never in frontmatter. Dependency hints are derived from relative links.
5. **Profiles as named lists have strong precedent.** `anthropics/skills` (180k stars) keeps flat folders and defines groups (`document-skills`, `example-skills`) as path lists in `.claude-plugin/marketplace.json`; skillshare and Vercel users ask for profiles and composable packs ([#202](https://github.com/runkids/skillshare/issues/202), [#2060](https://github.com/vercel-labs/skills/issues/2060)). Skill count may also have a cost: one open Windows report (no reactions yet) says Claude Code idle CPU scales with installed skill directories ([#95306](https://github.com/anthropics/claude-code/issues/95306)); weak evidence, but it points the same way as per-project profiles over "install everything".

## Tools surveyed

### Agent Skills specification and skills-ref
The open format (opened 2025-12-18 per secondary reports; spec at [agentskills.io/specification](https://agentskills.io/specification), checked 2026-10-07). Folder with `SKILL.md`, optional `scripts/`, `references/`, `assets/`. The [client guide](https://agentskills.io/client-implementation/adding-skills-support) recommends scanning `<project>/.agents/skills/` and `~/.agents/skills/` next to a client-native directory, project over user on name collision, scan bounds of depth 4 to 6 and 2000 directories, and lenient validation (warn on name mismatch, skip on empty description). `skills-ref` ([clone](https://github.com/agentskills/agentskills/tree/main/skills-ref), commit 69ef37e, 2026-08-09) is declared a demonstration library, not for production, and rejects any frontmatter key outside `name, description, license, allowed-tools, metadata, compatibility`. Borrow: the rule set, the scan bounds. Avoid: using its strict key list as a hard error, since Claude Code, Cursor and Codex add real keys.

### Vercel `skills` CLI (`npx skills`)
[vercel-labs/skills](https://github.com/vercel-labs/skills): TypeScript, MIT, 33,324 stars, pushed 2026-10-07. `add` from GitHub, GitLab, local paths or archives; installs one canonical copy under `.agents/skills` and, by default, symlinks it into each agent directory (`--copy` for real files); registry of 75+ agents (matrix source). Reads source repos with category folders up to three levels deep (`skills/<category>/<name>`), a shallower `SKILL.md` shadows deeper ones, and installs flat. Lock: project `skills-lock.json` (v1, `computedHash` = SHA-256 over sorted relative path plus content, keys sorted, no timestamps, so parallel branches merge cleanly per the source comment) and global `~/.agents/.skill-lock.json` (v3, GitHub tree SHA, timestamps). Gaps seen in issues: update never hashes installed files (#1812), `update -p` reinstalls all skills every run ([#2314](https://github.com/vercel-labs/skills/issues/2314)), `status`/`verify`/`--frozen-lockfile` requested ([#629](https://github.com/vercel-labs/skills/issues/629), [#500](https://github.com/vercel-labs/skills/issues/500)), enable/disable is the most reacted request ([#634](https://github.com/vercel-labs/skills/issues/634), 24), a declarative manifest ([#165](https://github.com/vercel-labs/skills/issues/165), 19). Anonymous telemetry is on by default. Borrow: sorted, timestamp-free lock; `--list`, `-y`; depth-capped discovery with the shadowing rule. Avoid: symlink default, telemetry, remote-source machinery.

### skillshare
[runkids/skillshare](https://github.com/runkids/skillshare): Go, MIT, 2,728 stars, created 2026-01-14, v0.25.0, pushed 2026-10-07. The nearest product: CLI, TUI, embedded web dashboard (`skillshare ui`), desktop app, MCP serving. Source folder syncs to per-agent targets in `merge` (per-skill symlink, default), `copy` or `symlink` mode. Copy mode keeps `.skillshare-manifest.json` (name to SHA-256, mtimes, `updated_at`) in the target; docs say sync skips equal checksums and overwrites different ones, so edits in the copy are not protected. Groups: nested folders to any depth, flattened as `frontend__react__react-best-practices` while `name` stays bare. Multi-project: a global `projects:` map with per-project `targets` and `include` globs (explicit list, no scan). Safety: automatic backup before sync, trash with 7-day retention and `restore`, `doctor [--json]`, `diff --stat/--patch`, `status --json`, `.skillignore`-based enable/disable, `collect` (target to source). Its bugs show the risks: update deleted a skill after an audit block ([#271](https://github.com/runkids/skillshare/issues/271)), prune removed links it never created ([#314](https://github.com/runkids/skillshare/issues/314)). No profiles ([#202](https://github.com/runkids/skillshare/issues/202), 3 reactions, open) and no watch mode. Borrow: backup plus trash, doctor, status/diff UX, ownership manifest. Avoid: `__` flattening, timestamps in the manifest, scope creep into MCP, hooks, plugins and audit.

### openskills and skillport
[numman-ali/openskills](https://github.com/numman-ali/openskills): TypeScript, 10,776 stars, last push 2026-01-18. Writes an `<available_skills>` block into `AGENTS.md`; its universal mode uses `.agent/skills` (singular), a name that lost to `.agents`. Treat as historical. [gotalab/skillport](https://github.com/gotalab/skillport): Python, MIT, 414 stars, pushed 2026-06-24. CLI plus MCP server; categories and tags live in frontmatter (`metadata.skillport.category`) with env-var filters. This is the frontmatter alternative to folder groups; it makes `SKILL.md` tool-specific and the spec declined `category`. Borrow: nothing.

### rulesync, ruler, skiller
[rulesync](https://github.com/dyoshikawa/rulesync) (TypeScript, MIT, 1,505 stars): `rulesync.jsonc` with per-target feature maps; `generate --dry-run` and `generate --check` (same preview, exit 1 when outputs are stale, flags cannot be combined); `install --frozen` fails when the lock is missing or out of sync; `rulesync add` edits the JSONC keeping comments and restores the previous state if the new source fails; lock `rulesync.lock` pins commit SHAs. Borrow: `--check` and `--frozen` semantics, comment-preserving edit with rollback, a JSON schema URL in the config. [ruler](https://github.com/intellectronica/ruler) (MIT, 2,943 stars): `.ruler/` plus `ruler.toml`, nested `.ruler` for monorepos; its skills column disagrees with vendor docs (it lists `.cursor/skills` and `.gemini/skills`), which is why the matrix below is checked at the source. [skiller](https://github.com/beautyfree/skiller) (45 stars, desktop app, licence listed as "Other" on GitHub): UI over per-agent directories; ignore.

### Claude Code plugins, marketplaces and anthropics/skills
A [plugin](https://code.claude.com/docs/en/discover-plugins) bundles skills, agents, hooks and MCP servers, namespaced `/plugin:skill`, installed at user, project or local scope. Project scope records `enabledPlugins` in the committed `.claude/settings.json` and does not put files into the repo (they live in a plugin cache), so it is neither git-tracked content nor SSH-portable. `claude plugin validate` checks manifests. [anthropics/skills](https://github.com/anthropics/skills) is flat (`skills/<name>`, verified through the API; a summariser claimed categories, which is wrong) and groups through `marketplace.json` entries with `skills: ["./skills/xlsx", ...]`. Claude Code also has `skillOverrides` (`on`, `name-only`, `user-invocable-only`, `off`), so enable/disable already exists inside the agent. Borrow: named path lists as the profile model.

### General sync and config managers
- **chezmoi**: [status](https://www.chezmoi.io/reference/commands/status/) prints two columns (last written vs actual, actual vs target), `verify` exits 0 or 1 ([verify](https://www.chezmoi.io/reference/commands/verify/)). Borrow the two-column idea: column one is installed vs lock (edited), column two is vault vs installed (outdated).
- **GNU stow**: [two-phase](https://www.gnu.org/software/stow/manual/stow.html): scan for conflicts first, stop with no change if any, never delete what it does not own, `-n` simulates, `--adopt` moves existing files into the package and is flagged as dangerous. Stow packages map to our groups (`stow -t proj web android` is `init proj --group web --group android`).
- **copier**: [update](https://copier.readthedocs.io/en/stable/updating/) regenerates old and new template output from the commit recorded in `.copier-answers.yml`, applies the diff over local edits, writes conflict markers or `.rej`, needs a clean git tree. Borrow: a recorded base is what makes a safe update possible. Do not borrow marker merging (agents read `SKILL.md` verbatim).
- **uv, pnpm, cargo**: `uv lock --check` and `uv sync --locked` fail on drift, `--frozen` accepts it ([uv](https://docs.astral.sh/uv/concepts/projects/sync/)); `pnpm install --frozen-lockfile` is the CI default ([pnpm](https://pnpm.io/cli/install)); `cargo update --dry-run` prints without writing, `--locked` errors on change ([cargo](https://doc.rust-lang.org/cargo/commands/cargo-update.html)). Borrow the vocabulary: `--dry-run` prints, `--check` fails on drift.
- **Renovate and Dependabot**: grouping (`groupName`, `groups`), ignore lists, a dependency dashboard, a PR limit ([Renovate](https://docs.renovatebot.com/configuration-options/), [Dependabot](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file)). Borrow: report by group; `status` is the dashboard. A PR per project does not apply.
- **git submodule and subtree**: a [submodule](https://git-scm.com/book/en/v2/Git-Tools-Submodules) pins a whole repository commit, cannot vendor a subfolder, needs `--recurse-submodules` on clone and leaves detached HEADs. Wrong fit for per-skill subsets in 60 repos; this is why copying stays right.

## Agent directory and format matrix

All rows checked 2026-10-07. "Own docs" means the vendor documentation was fetched; "registry" means only the Vercel table or ruler was available (secondary, lower confidence).

| Agent | Project directory | User directory | Notes | Source |
|---|---|---|---|---|
| Claude Code | `.claude/skills/<name>/` (nested `<sub>/.claude/skills/` load when working there) | `~/.claude/skills/` | Docs show direct children only. Does not read `.agents/skills`. Folder may be a symlink. Extra keys: `when_to_use`, `disable-model-invocation`, `user-invocable`, `paths`, `context`, `hooks`. Description plus `when_to_use` cut at 1,536 chars. Keep `SKILL.md` under 500 lines. | [docs](https://code.claude.com/docs/en/skills) |
| Codex | `.agents/skills/` in cwd and every parent up to the repo root | `~/.agents/skills/`, admin `/etc/codex/skills` | Optional `agents/openai.yaml` (`interface`, `policy.allow_implicit_invocation`, `dependencies.tools`). Follows folder symlinks. Duplicate names both listed. Source has a recursive discovery mode with a depth cap and 20,000 entries per root (which roots use it not checked). | [docs](https://learn.chatgpt.com/docs/build-skills), [loader](https://github.com/openai/codex/blob/main/codex-rs/ext/skills/src/loader/discovery.rs) |
| Gemini CLI | `.gemini/skills/` or `.agents/skills/` (alias wins in the same tier) | `~/.gemini/skills/` or `~/.agents/skills/` | Workspace over user over extension over built-in. `/skills enable`, `disable`, `link`. Nesting not documented. | [docs](https://geminicli.com/docs/cli/skills/) |
| Cursor | `.agents/skills/`, `.cursor/skills/`; also `.claude/skills/`, `.codex/skills/` | `~/.agents/skills/`, `~/.cursor/skills/`, `~/.claude/skills/`, `~/.codex/skills/` | Walks the root recursively, so nested groups load. `name` must equal the folder. Keys `paths`, `disable-model-invocation`, `icon`, `color`, `metadata`. | [docs](https://cursor.com/docs/context/skills) |
| GitHub Copilot | `.github/skills/`, `.claude/skills/`, `.agents/skills/` | `~/.copilot/skills/`, `~/.agents/skills/` | Cloud agent, CLI, VS Code, JetBrains, code review. | [docs](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills) |
| OpenCode | `.opencode/skills/`, `.claude/skills/`, `.agents/skills/` (walks up to the git worktree) | `~/.config/opencode/skills/`, `~/.claude/skills/`, `~/.agents/skills/` | Name regex `^[a-z0-9]+(-[a-z0-9]+)*$`, must equal folder. `permission.skill` allow/deny/ask in `opencode.json`. | [docs](https://opencode.ai/docs/skills) |
| Windsurf (Devin docs) | `.devin/skills/` (preferred), `.windsurf/skills/`, `.agents/skills/`, `.claude/skills/` (if enabled) | `~/.codeium/windsurf/skills/`, `~/.config/devin/skills/`, `~/.agents/skills/` | Nesting and limits not documented. Docs moved from docs.windsurf.com to docs.devin.ai. | [docs](https://docs.devin.ai/desktop/cascade/skills) |
| Roo Code | `.roo/skills/`, `.agents/skills/`, plus `skills-<mode>/` variants | `~/.roo/skills/`, `~/.agents/skills/` | `.roo` overrides `.agents`; name equals folder (or symlink name). | [docs](https://roocodeinc.github.io/Roo-Code/features/skills) |
| Cline | `.cline/skills/`, `.clinerules/skills/`, `.claude/skills/` | `~/.cline/skills/` | Own docs do not list `.agents/skills`; the Vercel registry says it does. Unresolved, treat as not read. Per-skill toggle. Name equals folder. | [docs](https://docs.cline.bot/features/skills) |
| Kiro | `.kiro/skills/` | `~/.kiro/skills/` | No `.agents/skills`. Name equals folder, 64 and 1024 limits. | [docs](https://kiro.dev/docs/skills/) |
| Amp, Antigravity, Junie, Goose, Droid, Kilo | Amp `.agents/skills/`; Antigravity `.agents/skills/` (ruler says `.agent/skills/`); Junie `.junie/skills/`; Goose `.goose/skills/`; Droid and Kilo `.agents/skills/` | Amp `~/.config/agents/skills/`; Junie `~/.junie/skills/`; Goose `~/.config/goose/skills/`; Droid `~/.factory/skills/`; Kilo `~/.kilo/skills/` | Registry only, not verified at the source (Amp and Junie doc pages did not render the skills section). Registries disagree on Antigravity. | [Vercel table](https://github.com/vercel-labs/skills#supported-agents), [ruler](https://github.com/intellectronica/ruler) |
| AGENTS.md (not skills) | `AGENTS.md`, nearest file wins | n/a | Read by Codex, Copilot, Cursor, Gemini, Jules, Aider, goose and others. Claude Code does not read it (tracked in #31005). Out of scope for this tool. | [agents.md](https://agents.md/) |

### Which targets the tool should support
- **`agents` (`.agents/skills`)**: canonical, always on, the only target with real `copy` semantics. One copy serves Codex, Gemini CLI, Cursor, Copilot, OpenCode, Windsurf, Roo and the registry-only agents.
- **`claude` (`.claude/skills`)**: optional, mode `link` (directory symlink to `../.agents/skills`) or `copy`. Needed for Claude Code and Cline. Prefer `link`: two real copies are read twice by Cursor, Copilot and OpenCode, and the client guide only promises a warning on collisions. Git can store the symlink, but committing it is the project's choice (this repo ignores it), so `sync` creates a missing bridge, reports it, and refuses to replace a real directory. Whether Cursor lists a bridged skill twice is untested (open point).
- **`kiro` (`.kiro/skills`)**: optional, `copy` only (Kiro reads nothing else).
- Everything else (Junie, Goose, ...): later, through a generic `path:` override, only after a vendor doc confirms it. Do not hard-code the registry rows above.

## SKILL.md rules for linting

Levels: E = error (spec or required by several agents), W = warning, I = info. Sources: [spec](https://agentskills.io/specification), [best practices](https://agentskills.io/skill-creation/best-practices), [Claude Code](https://code.claude.com/docs/en/skills), [skills-ref validator](https://github.com/agentskills/agentskills/blob/main/skills-ref/src/skills_ref/validator.py), [OpenCode](https://opencode.ai/docs/skills).

| Rule | Level | Source |
|---|---|---|
| `SKILL.md` exists, starts with a `---` frontmatter block that parses as YAML | E | spec; client guide |
| `name` present, 1 to 64 chars, lowercase letters, digits, hyphens, no leading, trailing or double hyphen | E | spec; skills-ref |
| `name` equals the folder name | E | spec; Cursor, OpenCode, Roo, Kiro, Cline, skills-ref |
| `name` is ASCII `[a-z0-9-]` (spec allows Unicode lowercase, OpenCode regex does not) | W | spec vs OpenCode |
| `description` present, non-empty, at most 1024 chars | E | spec; Codex limit constant |
| `description` states what it does and when to use it (contains "use when" or equivalent trigger wording) | I (heuristic, ours) | spec; optimizing-descriptions |
| `description` plus `when_to_use` at most 1,536 chars | W | Claude Code truncation |
| `compatibility` at most 500 chars if present | E | spec |
| `metadata` is a string-to-string map | W (spec text says strings; [agentskills #514](https://github.com/agentskills/agentskills/issues/514) notes runtimes store nested values) | spec |
| `allowed-tools` is a space-separated string | W (Claude Code also accepts lists) | spec; Claude Code |
| Frontmatter keys outside the spec set | W for unknown, none for known vendor keys (`when_to_use`, `disable-model-invocation`, `user-invocable`, `paths`, `context`, `agent`, `model`, `effort`, `arguments`, `argument-hint`, `hooks`, `shell`, `icon`, `color`) | skills-ref vs vendor docs |
| Top-level `version` or similar custom key: suggest moving to `metadata` | W | spec |
| `SKILL.md` at most 500 lines, body about 5,000 tokens (approximate as chars / 4) | W | spec progressive disclosure |
| Reference files one level deep from `SKILL.md`; relative links resolve | W for depth, E for dead links | spec file references |
| Unquoted colon inside a plain YAML scalar (lenient clients accept it, strict parsers fail) | E with a quoting hint | client guide "malformed YAML" |
| Reserved names `synced`, `anthropic-skills*` | E | Claude Code |
| Files in the skill folder: report untracked ones, skip symlinks | W | requirements |
| Skill folder contains nested `SKILL.md` below the top one | I (orchestration pattern, [agentskills #376](https://github.com/agentskills/agentskills/issues/376)); discovery stops at the first `SKILL.md` | vercel shadowing rule |

**Link rules specific to this vault.** Skills write links for the installed flat layout. Three kinds exist in the real vault: sibling links `../<name>/SKILL.md` (dependency hints), project-root links `../../../Project_Manag/...` (valid only at depth `.agents/skills/<name>/`), and links inside the skill. The checker must resolve sibling links by skill name against the project's installed set, never against the vault path, and must not try to resolve project-root links inside the vault.

**What the prototype lint found in the 12 skills currently in `.agents/skills`** (run with a throwaway script, 2026-10-07): all names equal their folders and all descriptions are under 1024 chars; `tmux` has a top-level `version:` key (W, move to `metadata.version`); `playwright-cli` is 489 lines, 11 below the 500 line advice; `coding` links `file-tree-optimization` and back, so dependency closure must handle cycles; `file-tree-optimization` needs `coding` and `refac-cli` installed or its links die; `remote-helper` links three levels up into `Project_Manag/Docs` (project-root links); `doc-start` ships `agents/openai.yaml` (Codex metadata, must be copied as is).

## Feature verdicts

Demand is counted in reactions and comments on the closest tools' issues (checked 2026-10-07). Small numbers are normal here: most skill-tool issues in these trackers are listing requests.

| Feature | Who does it and how | Evidence of demand | Cost and risk | Verdict |
|---|---|---|---|---|
| Groups (nested vault) | Vercel reads category folders (depth 3) and installs flat; skillshare nested folders, `__` flatten; Cursor and Hermes read nested natively; anthropics/skills is flat plus manifest lists | Vercel taxonomy [#505](https://github.com/vercel-labs/skills/issues/505) (PR closed), [#1933](https://github.com/vercel-labs/skills/issues/1933) 0 reactions, skillshare [#156](https://github.com/runkids/skillshare/issues/156); spec `category` closed | Low. Discovery rule, name uniqueness, group-versus-skill name clash check. Hermes data loss when a flat skill name equalled a category folder ([#1723](https://github.com/vercel-labs/skills/issues/1723)) | **Build now**, vault-only, flat install |
| Profiles | skillshare per-project `include` globs, Vercel "packs", anthropics marketplace lists, Claude Code plugin scopes | skillshare #202 (3), Vercel #2060 composable packs, [#1185](https://github.com/vercel-labs/skills/issues/1185) | Low as named lists; composition needs a cycle check | **Build now** |
| `add` (opt a project in) | Vercel `add`, skillshare `install -p` | Core command of the 33k-star tool | Low; must respect opt-in rule | **Build now** |
| `init` (new project + profile or groups) | `git init`, `cargo init`, skillshare `init -p`, copier | Requested by the user | Low if thin: mkdir, optional `git init`, install `mandatory` plus selection | **Build now** |
| `status` | skillshare `status`, `check`; chezmoi `status` | Vercel #629 (0), #500 (5), #1812 | Low to medium; needs lock and hashing | **Build now** |
| `diff` | skillshare `diff --stat/--patch`; chezmoi `diff` | Same | Low, the plan already holds per-file changes | **Build now** |
| Change-aware sync | skillshare checksum skip; Vercel re-installs every run (#2314, a bug report) | Bug report on the leader | Low | **Build now** |
| Drift guard + provenance lock | Vercel `skills-lock.json`, skillshare manifest, rulesync lock, copier answers file | [#455](https://github.com/vercel-labs/skills/issues/455) preserve-local, #1812, #500; both leaders lack local-edit protection | Medium: hash format, lock placement, migration of 395 legacy installs | **Build now** |
| Backup and undo | skillshare backup, trash, `restore`; stow never deletes unowned | Skillshare data-loss bugs #271, #314, [#139](https://github.com/runkids/skillshare/issues/139) | Medium: disk use, retention; store only content that differs from what the vault reproduces | **Build now** |
| `--json` | skillshare `status`, `doctor`; rulesync | skillshare [#129](https://github.com/runkids/skillshare/issues/129) (stray notice broke JSON stdout) | Low; keep stdout pure | **Build now** |
| `--check` | rulesync, chezmoi `verify`, `uv --check` | CI use is the stated purpose | Low; define exit codes once | **Build now** |
| `doctor` | skillshare `doctor --json`, Claude Code `/doctor` | n/a | Low to medium | **Build now** |
| Skill lint | `skills-ref validate`, `claude plugin validate` | Spec issues on validation | Low; tiers E, W, I with vendor keys known | **Build now**, inside `doctor` |
| Dependency hints | None standard; spec issues open; Codex `dependencies.tools` covers tools only | #100 (18 comments), #485, #110 | Low if derived from `../<name>/SKILL.md` links, no new frontmatter | `doctor` check **build now**; `add --with-deps` **build later** |
| Other skill dirs (`targets`) | Vercel `--agent`, skillshare targets, rulesync targets | Universal | Low; duplicate-read risk | **Build now** (`agents`, `claude`, `kiro`) |
| `adopt` (project edit back to vault) | skillshare `collect`, rulesync import, stow `--adopt` | Skillshare [#31](https://github.com/runkids/skillshare/issues/31); the README pain "improve in one project, all others outdated" | Medium, reverse flow, must show diff and require the project copy to be `edited` state | **Build later** |
| Commit-after-sync | Only skillshare `commit`, and only for its own source repo | None | High: 60 repos, hooks, unrelated staged changes | **Do not build** |
| Watch mode | None found; agents watch their own dirs | None | High: a daemon that rewrites 60 repos on save | **Do not build** |
| Enable and disable | skillshare via `.skillignore`; agent-native (`skillOverrides`, `/skills disable`, Cline toggle, `permission.skill`) | Vercel #634, 24 reactions (highest) | Opt-in already means "enabled if present"; `delete` plus `undo` covers the rest | **Do not build** |
| Version pinning | Vercel and skillshare pin remote sources to a ref ([#2032](https://github.com/vercel-labs/skills/issues/2032), [#281](https://github.com/runkids/skillshare/issues/281)) | For remote sources only | The vault is one local git repo; a `hold` marker for deliberate forks is the real need | Pinning **do not build**; `hold` **build later** |
| Merge update with markers | copier `update` | Vercel #455 asks only for conflict reporting | Conflict markers inside files agents read verbatim | **Do not build** |
| Profile follow ("profile has N skills missing here") | None | None | Low once the lock records profiles | **Build later** (in `status`) |
| Web view | skillshare dashboard; skiller desktop | skillshare #85, #123 (small) | Medium, security model needed | **Build later**, read-only first |

### Groups
- **Discovery rule**: a skill is a folder containing `SKILL.md`; stop descending at the first one (Vercel's shadowing rule, and it keeps the orchestration pattern of #376 intact). Skip dot-directories. Cap group depth (the client guide suggests 4 to 6 levels) and fail with an error when exceeded, per the no-fallbacks rule.
- **Names**: the leaf folder equals frontmatter `name` equals the installed folder. Names are unique vault-wide, and a group name must not equal any skill name (otherwise `--group web` and `web` are ambiguous, and the Hermes incident shows what a flat name landing on a category folder costs).
- **Installed layout**: `.agents/skills/<name>/`, no prefix, no group record in the project. Moving a skill between groups must not change any project, so the lock never stores the group.
- **Metadata**: none. The folder path is the group. Descriptions of groups, if wanted, belong in `config.yaml` or a README in the group folder, not in `SKILL.md`.

### Profiles
- A profile is a named list of groups, skills and other profiles. It exists because skills are cross-cutting (`secrets` belongs to web and content projects) while a folder holds a skill once; anthropics/skills solves the same problem with manifest lists.
- `mandatory` stays and acts as an implicit base for `init` and `push`. Profiles never run implicitly: `sync` still updates only skills a project already has.
- A project records which profiles were applied (lock field) so `status` can say "profile `web-site` has 2 skills not installed here". It reports; it never installs. That keeps the opt-in rule intact.

### Drift, lock and update flow
- State per skill from three hashes: lock (base), installed (ours), vault (theirs). Equal all: clean. Only vault changed: outdated, sync updates. Only installed changed: edited, sync skips, `adopt` or `--force`. Both changed: conflict, skip and show both sides. No lock entry: unknown (legacy).
- **Legacy bootstrap** for the 395 existing installs: a first `status` marks differing skills unknown; `<tool> baseline` (or a TUI prompt) shows the diff, then records the current installed state as base so future edits are detected. Possible refinement (my idea, not seen elsewhere, feasibility is for the engine stream): compare the installed tree to historical vault revisions of that skill; a match means "outdated, no local edit".
- Flow is push from vault, like `chezmoi apply` and `rulesync generate`; the reverse flow is the explicit `adopt`. No tool in this survey updates automatically in the background, and none should be copied there.
- Pinning remote refs is a different problem from ours; nothing to import.

### Safety rules (collected from stow, skillshare bugs, rulesync)
1. Plan everything first (stow two-phase); abort before any write if the plan has errors.
2. Touch only what the vault names or the lock owns; unknown folders under `.agents/skills` are never read, changed or pruned (skillshare #314).
3. Per-skill atomic swap: stage beside the target, verify, rename; on any failure the old copy stays (skillshare #271).
4. Overwriting an `edited` or `conflict` skill needs `--force` or an interactive yes, and always backs up first.
5. `--dry-run` prints; `--check` is a dry run that exits non-zero on drift; do not allow both (rulesync).
6. Warn when the project working tree has uncommitted changes inside the skill folder (copier's clean-tree precondition, as a warning).

## Proposed config shapes

Naming: **group** (folder in the vault; Vercel and skillshare say category or folder, the user says group), **profile** (skillshare #202 wording; Vercel says pack, Claude Code says plugin), **targets** (skillshare, rulesync and ruler use the same word; Vercel uses `--agent`). New keys use snake_case like `exclude_dirs`. Unknown keys stay hard errors, so each new key is an explicit addition.

**Vault layout.** Group folders are lowercase kebab by recommendation, any non-hidden name is accepted.

```
vault/
  config.yaml
  core/            coding/  doc-start/  skill-writer/      <- group "core"
  web/             astro/  vite/  fumadocs-hosting/  playwright-cli/
  web/seo/         core-web-vitals/  schema-markup/        <- nested group "web/seo"
  content/         derive-posts/  x-post/  post-scheduler/  subtitle-informer/
  android/         ...
  tmux/                                                    <- skill at the top level (no group)
```

**`config.yaml`, existing keys unchanged** (every file valid today stays valid):

```yaml
root: /home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code
mandatory: [coding, doc-start]
exclude_dirs: [testdata]
exclude_paths: [/abs/path/to/fixture]
```

**Same file with the additions** (all optional):

```yaml
# Where skills are written. Omitted means {agents: copy}.
targets:
  agents: copy                 # .agents/skills, canonical
  claude: link                 # .claude/skills -> ../.agents/skills (or: copy)
  kiro: copy                   # .kiro/skills
  # extra: { path: .foo/skills, mode: copy }   # generic form, only for a verified vendor path

# Named selections. Items are groups (folder path), skills (name) or other profiles.
profiles:
  base:
    skills: [coding, doc-start]
  web-site:
    description: Astro site with docs hosting and browser tests   # optional
    extends: [base]
    groups: [web]
    skills: [secrets, ci-cd]
    exclude: [vite]            # optional, subtracts from the resolved set
  android-app:
    extends: [base]
    groups: [android]

# Safety knobs. Defaults shown.
backup:
  keep_runs: 20                # snapshots kept under the XDG state dir
  keep_days: 30
lock: project                  # project (file in .agents/skills) | state (XDG state dir) | off
```

Validation at load: every name in `mandatory`, `skills`, `exclude` exists in the vault; every group path is a real folder; `extends` is acyclic; profile names do not collide with skill or group names; hard error otherwise.

**Provenance lock**, one file per project at `.agents/skills/.<tool>.lock.json`. It sits inside the skills folder so it travels with the skills (the spec scan rule only looks at subfolders with `SKILL.md`, and skillshare already writes a hidden manifest into the skill folders of dozens of agent targets). Sorted keys, no timestamps, no absolute paths, no vault commit id (a vault commit would change the file in every project on every vault commit). Mirrors the merge-friendly choice of Vercel's `skills-lock.json`.

```json
{
  "version": 1,
  "profiles": ["web-site"],
  "skills": {
    "astro":  { "hash": "sha256:9f2c...", "origin": "profile:web-site" },
    "coding": { "hash": "sha256:41ab...", "origin": "mandatory" },
    "vite":   { "hash": "sha256:0d77...", "origin": "manual", "hold": true }
  }
}
```

`hash` is SHA-256 over the skill's files sorted by relative path, each entry as path, mode (octal), length, bytes. Vercel hashes path plus content without mode or delimiters; we add both because permission bits are preserved and concatenation without lengths is ambiguous. `hold: true` marks a deliberate local fork that sync never touches and `status` does not report as drift. An alternative that gets the vault side for free is the git tree id of the skill folder (read from the index, no blob reads); the engine stream decides.

**Command shapes** (names are proposals; the CLI stream owns the final grammar):

```
<tool> init ~/code/new-site --profile web-site     # creates the dir, mandatory + profile; --git for git init
<tool> init . --group web --group content --skill secrets
<tool> add astro vite            |  <tool> add --profile android  |  <tool> add --group web
<tool> status [--json] [--check]                   # matrix of projects x skills with clean/outdated/edited/conflict/unknown
<tool> diff coding --project ~/code/x [--stat]
<tool> sync [--dry-run | --check] [--force]        # skips edited/conflict unless --force
<tool> undo [--list]                               # restore the last (or chosen) backup run
<tool> doctor [--json]                             # vault, config, lock, targets bridge, lint, link checks
```

Exit codes (proposal): 0 clean or applied, 1 drift found (only with `--check`), 2 error (config, I/O, any failed target), 3 finished but held back skills because of local edits. `init` also needs the answer to "which project counts as opted in": see open points.

## What not to build

- **No network sources, registry or marketplace** (skills.sh, skillshare hub, plugin marketplaces). The vault is one local git repo; fetching remote skills is `git clone` plus `add`.
- **No telemetry.** Vercel's CLI sends anonymous usage data by default.
- **No rewriting of skill files**: no `name` prefixing or flattening, no injected frontmatter (`metadata.skillport`, classifiers). Copies stay byte-identical to the vault index content.
- **No new frontmatter fields** (`category`, `dependencies`, `version`, `group`). The spec declined `category`; the reference validator rejects unknown keys; the vault is also read by agents that know nothing about this tool.
- **No symlinked skill content.** Only the optional `.claude/skills` directory bridge is a symlink. Per-skill symlinks break over SSH and are not tracked content.
- **No scope creep into MCP, hooks, plugins, subagents, rules or AGENTS.md** (skillshare and rulesync went there and their issue lists are dominated by it). Skills only.
- **No security or prompt-injection auditing** of skill text (skillshare `audit`); the vault is author-owned, and lint covers format.
- **No watch daemon, no auto-commit or push in projects, no self-update.**
- **No enable/disable toggles**; opt-in plus delete with undo covers it, and agents have native switches.
- **No merge-with-markers** update and **no version pinning** of vault content.
- **No code-project scaffolding** (`cargo new`, copier, cookiecutter do that): `init` creates a directory and installs skills, nothing else.
- **No per-agent format conversion** (Cursor `.mdc`, Copilot instructions): out of scope.

## Risks and open points

1. **Which projects count as opted in.** Today the scan finds `.agents/skills` only. A project that has real directories under `.claude/skills` only (Claude Code default) is invisible. Decide: stay `.agents/skills`-only, or treat `.claude/skills/<name>` real folders as a second discovery signal for `status` and `doctor` (report, never write). Ask the user.
2. **Bridge duplicates and fresh clones.** A `claude: copy` target double-lists skills in Cursor, Copilot and OpenCode (they read `.claude/skills` too). `link` avoids real duplicates, but whether each agent de-duplicates by real path is untested. Test with Cursor and OpenCode before documenting `link` as safe. Separately, an ignored bridge disappears on clone; either commit it (needs symlink support on Windows) or let `sync` recreate it.
3. **Claude Code may adopt `.agents/skills`.** Issues #16345 and #31005 are open; if fixed the `claude` target becomes redundant, so it must stay optional and easy to drop.
4. **Lock placement acceptance.** A committed lock adds a tracked file to every project. The `lock: state` mode (XDG state, keyed by project path) avoids that but loses cross-machine drift checks, and SSH use is a stated goal. Keep both; default to `project`.
5. **Legacy baseline.** 395 installs have no provenance. The first-run flow must not silently overwrite (the rule) and must not block adoption; the `baseline` step and the optional history match above are unproven.
6. **Hidden file tolerance.** The lock file inside `.agents/skills/` relies on agents ignoring non-skill entries. Spec and skillshare practice suggest yes; no agent was tested.
7. **Spec drift.** Open spec issues on nesting ([#59](https://github.com/agentskills/agentskills/issues/59), [#115](https://github.com/agentskills/agentskills/issues/115), [#137](https://github.com/agentskills/agentskills/issues/137)), metadata types (#514), dependencies and versioning ([#46](https://github.com/agentskills/agentskills/issues/46)) can change lint rules. Keep the rule table data-driven and versioned.
8. **Registry-only rows.** Amp, Junie, Goose, Kilo, Droid, Antigravity and Cline's `.agents/skills` claim need vendor confirmation before they get built-in target names.
9. **Moving targets.** Docs moved during this survey (Windsurf to docs.devin.ai, Codex to learn.chatgpt.com); recheck the matrix before each release and store the source URL next to each built-in target in code.
10. **Group depth and rename semantics.** A renamed group or moved skill must be a no-op for projects; the tests must cover it.

## Suggested changes to the requirements doc

1. Vault discovery: add "stop at the first `SKILL.md`", "skip dot-directories", "group names must not equal skill names", "`name` must equal the folder (error)", a depth cap with a hard error.
2. Config: add optional `targets`, `profiles`, `backup`, `lock` keys, keep unknown-key errors; validate names at load.
3. Engine: add a link extractor for `SKILL.md` bodies (sibling, project-root, internal links) for `doctor`; add the three-way state and the `hold` flag to the plan model.
4. Apply: backup retention policy and the `undo` operation; exit-code table (0, 1, 2, 3).
5. Operations list: add `undo`, `baseline` (legacy bootstrap), later `adopt` and `hold`; define `init` as "create directory, install `mandatory` plus profile or groups", not a code scaffolder.
6. Project scan: decide the `.claude/skills` question (open point 1); never descend below `.agents/skills`, and do not follow the `.claude/skills` bridge into a second scan. Declared `link` targets are created when missing (a gitignored bridge vanishes on clone) and never replace a real directory.
7. Web view: read-only first (matrix, group browser, diff); mutating actions only after the drift guard and the security model exist.
8. Add the telemetry and scope-creep exclusions to the quality section.

## Clones

Shallow clones under `Repos/` (gitignored), created for this stream on 2026-10-07:

- `Repos/vercel-labs_skills` ([vercel-labs/skills](https://github.com/vercel-labs/skills), commit 48dc9e8, 2026-10-07): agent registry, lock files, category discovery.
- `Repos/skillshare` ([runkids/skillshare](https://github.com/runkids/skillshare), 2026-10-07): manifest, backup, trash, doctor, organizing-skills docs.
- `Repos/openskills` ([numman-ali/openskills](https://github.com/numman-ali/openskills)): historical.
- `Repos/skillport` ([gotalab/skillport](https://github.com/gotalab/skillport)): category-in-frontmatter alternative.
- `Repos/rulesync` ([dyoshikawa/rulesync](https://github.com/dyoshikawa/rulesync), 2026-10-05): `--check`, `--frozen`, lock docs.
- `Repos/ruler` ([intellectronica/ruler](https://github.com/intellectronica/ruler), 2026-08-19): second registry table.
- `Repos/skiller` ([beautyfree/skiller](https://github.com/beautyfree/skiller)): desktop manager, read only.
- `Repos/agentskills` ([agentskills/agentskills](https://github.com/agentskills/agentskills), commit 69ef37e, 2026-08-09): spec docs and `skills-ref`.

## Verdicts

| Tool or feature | Verdict | Reason | Checked |
|---|---|---|---|
| Groups as vault folders, flat install | Build now | Spec, five agents and skillshare #110 force name equals folder | 2026-10-07 |
| Profiles (named lists, `extends`) | Build now | anthropics/skills and requests #202, #2060 | 2026-10-07 |
| `add`, `init` | Build now | Core of Vercel and skillshare; thin | 2026-10-07 |
| `status`, `diff` | Build now | Leaders lack drift view (#629, #500, #1812) | 2026-10-07 |
| Change-aware sync | Build now | Vercel #2314 shows the cost of skipping it | 2026-10-07 |
| Provenance lock, three-way drift guard | Build now | Only way to avoid silent overwrite; no leader has it | 2026-10-07 |
| Backup and `undo` | Build now | skillshare data-loss bugs #271, #314, #139 | 2026-10-07 |
| `--json`, `--check`, exit codes | Build now | rulesync, chezmoi, uv precedent | 2026-10-07 |
| `doctor` with SKILL.md lint | Build now | Cheap, data-driven rule table above | 2026-10-07 |
| `targets`: `agents` plus `claude` bridge | Build now | Claude Code still ignores `.agents/skills` | 2026-10-07 |
| `kiro` target, generic `path:` targets | Build later | Kiro verified; others registry-only | 2026-10-07 |
| Dependency hints from links in `doctor` | Build now | Real cycles and links in our vault | 2026-10-07 |
| `add --with-deps` | Build later | Needs closure with cycles | 2026-10-07 |
| `adopt` (project to vault) | Build later | After status and diff exist | 2026-10-07 |
| `hold` marker for forks | Build later | Small, after lock | 2026-10-07 |
| Read-only web view | Build later | skillshare ships one; web stream decides | 2026-10-07 |
| Commit-after-sync | Do not build | 60 repos, hooks, no precedent | 2026-10-07 |
| Watch mode | Do not build | No precedent, hazardous | 2026-10-07 |
| Enable and disable | Do not build | Opt-in covers it; agents have native switches | 2026-10-07 |
| Version pinning, merge with markers | Do not build | Remote-source problem; markers corrupt agent input | 2026-10-07 |
| Registry, marketplace, telemetry, audit, MCP and hooks | Do not build | Out of scope for a local skills mirror | 2026-10-07 |
