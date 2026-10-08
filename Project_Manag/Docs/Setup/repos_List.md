# Setup: External Reference Repos

Third-party or external repos cloned locally for source-code or documentation reference. The clones live in `/Repos/` at the repo root, which is fully gitignored. This file lives under `Project_Manag/Docs/Setup/` so it stays under git, and is the source of truth for what should be present in `/Repos/`. To repopulate on a fresh machine, run the clone commands below.

First-party repos do not belong here. Track those in [Internal repo paths](internal_Repo_Paths.md).

Convention: every clone uses `git clone --depth 1` and lands inside `/Repos/`. See `.agents/skills/doc-start/References/repos_Convention.md` for the full rules.

## Repos

### Go TUI reference

- **bubbles**: source and docs of charmbracelet/bubbles, the TUI component library behind the current Go TUI (filepicker, progress, spinner, paginator). Read it when changing any Go TUI component. The `list` and `table` components have no mouse support, so mouse is wired manually via `msg.Y` plus component APIs.
  - URL: https://github.com/charmbracelet/bubbles
  - Clone: `git clone --depth 1 https://github.com/charmbracelet/bubbles Repos/bubbles`

### Skill managers and the SKILL.md format

- **agentskills**: Agent Skills specification and the `skills-ref` validator. Read it for frontmatter rules, name-equals-folder and the lint rule table.
  - URL: https://github.com/agentskills/agentskills
  - Clone: `git clone --depth 1 https://github.com/agentskills/agentskills Repos/agentskills`
- **vercel-labs_skills**: Vercel `skills` CLI: agent registry, lock files, category discovery. The largest comparable tool.
  - URL: https://github.com/vercel-labs/skills
  - Clone: `git clone --depth 1 https://github.com/vercel-labs/skills Repos/vercel-labs_skills`
- **skillshare**: Go skill sync tool: manifest, backup, trash, doctor, nested folders. Closest competitor, read for failure cases.
  - URL: https://github.com/runkids/skillshare
  - Clone: `git clone --depth 1 https://github.com/runkids/skillshare Repos/skillshare`
- **openskills**: Historical skill installer; shows the older conventions.
  - URL: https://github.com/numman-ali/openskills
  - Clone: `git clone --depth 1 https://github.com/numman-ali/openskills Repos/openskills`
- **skillport**: Skill manager with categories in frontmatter, the alternative to folder groups.
  - URL: https://github.com/gotalab/skillport
  - Clone: `git clone --depth 1 https://github.com/gotalab/skillport Repos/skillport`
- **rulesync**: Rules and skills generator: `--check`, `--frozen` and lock file design.
  - URL: https://github.com/dyoshikawa/rulesync
  - Clone: `git clone --depth 1 https://github.com/dyoshikawa/rulesync Repos/rulesync`
- **ruler**: Second agent-directory registry table, used to cross-check target paths.
  - URL: https://github.com/intellectronica/ruler
  - Clone: `git clone --depth 1 https://github.com/intellectronica/ruler Repos/ruler`
- **skiller**: Desktop skill manager. Source-available licence (SUL-1.0): read ideas only, never copy code.
  - URL: https://github.com/beautyfree/skiller
  - Clone: `git clone --depth 1 https://github.com/beautyfree/skiller Repos/skiller`

### TUI stack (Rust)

- **ratatui**: Renderer: workspace split, `CellDiffOption`, List internals, changelog.
  - URL: https://github.com/ratatui/ratatui
  - Clone: `git clone --depth 1 https://github.com/ratatui/ratatui Repos/ratatui`
- **ratatui-templates**: ratatui project templates (cloned as `templates`, renamed): event-driven and component patterns.
  - URL: https://github.com/ratatui/templates
  - Clone: `git clone --depth 1 https://github.com/ratatui/templates Repos/ratatui-templates`
- **tui-widgets**: Widget collection: scrollview, popup, prompts. Read to judge, not used.
  - URL: https://github.com/ratatui/tui-widgets
  - Clone: `git clone --depth 1 https://github.com/ratatui/tui-widgets Repos/tui-widgets`
- **ratatui-textarea**: Multi-line editor widget, size and scope check.
  - URL: https://github.com/ratatui/ratatui-textarea
  - Clone: `git clone --depth 1 https://github.com/ratatui/ratatui-textarea Repos/ratatui-textarea`
- **tachyonfx**: Effects crate, judged as decoration.
  - URL: https://github.com/ratatui/tachyonfx
  - Clone: `git clone --depth 1 https://github.com/ratatui/tachyonfx Repos/tachyonfx`
- **tui-realm**: Framework on ratatui: shows the mouse gap in its stock components.
  - URL: https://github.com/veeso/tui-realm
  - Clone: `git clone --depth 1 https://github.com/veeso/tui-realm Repos/tui-realm`
- **rat-salsa**: Framework with first-class mouse and focus utilities and a file dialog.
  - URL: https://github.com/thscharler/rat-salsa
  - Clone: `git clone --depth 1 https://github.com/thscharler/rat-salsa Repos/rat-salsa`
- **tui-tree-widget**: Tree widget with `rendered_at` and `click_at` (cloned from `tui-rs-tree-widget`, renamed).
  - URL: https://github.com/EdJoPaTo/tui-rs-tree-widget
  - Clone: `git clone --depth 1 https://github.com/EdJoPaTo/tui-rs-tree-widget Repos/tui-tree-widget`
- **ratatui-explorer**: File explorer widget: resets scroll state every frame, no mouse.
  - URL: https://github.com/tatounee/ratatui-explorer
  - Clone: `git clone --depth 1 https://github.com/tatounee/ratatui-explorer Repos/ratatui-explorer`
- **cursive**: Retained-mode TUI library: no hover events.
  - URL: https://github.com/gyscos/cursive
  - Clone: `git clone --depth 1 https://github.com/gyscos/cursive Repos/cursive`
- **yazi**: File manager with its own terminal layer and per-event layout, a reference for mouse handling.
  - URL: https://github.com/sxyazi/yazi
  - Clone: `git clone --depth 1 https://github.com/sxyazi/yazi Repos/yazi`
- **gitui**: Git TUI: std threads plus channel select loop, key-release filter, dev profile settings.
  - URL: https://github.com/gitui-org/gitui
  - Clone: `git clone --depth 1 https://github.com/gitui-org/gitui Repos/gitui`
- **television**: Fuzzy finder TUI: pure mouse function over a stored layout.
  - URL: https://github.com/alexpasmantier/television
  - Clone: `git clone --depth 1 https://github.com/alexpasmantier/television Repos/television`

### Engine stack (Rust)

- **ripgrep**: Home of the `ignore` crate: parallel walker, overrides, `WalkState`. Also the clippy corpus.
  - URL: https://github.com/BurntSushi/ripgrep
  - Clone: `git clone --depth 1 https://github.com/BurntSushi/ripgrep Repos/ripgrep`
- **walkdir**: Sequential walker, a dependency of `ignore`.
  - URL: https://github.com/BurntSushi/walkdir
  - Clone: `git clone --depth 1 https://github.com/BurntSushi/walkdir Repos/walkdir`
- **jwalk**: Parallel walker, measured slower than `ignore`.
  - URL: https://github.com/Byron/jwalk
  - Clone: `git clone --depth 1 https://github.com/Byron/jwalk Repos/jwalk`
- **gitoxide**: `gix-index` and `gix-discover`: reading the git index without a subprocess.
  - URL: https://github.com/GitoxideLabs/gitoxide
  - Clone: `git clone --depth 1 https://github.com/GitoxideLabs/gitoxide Repos/gitoxide`
- **git2-rs**: libgit2 bindings. Licence-blocked: the bundled C library is GPL-2.0 with a linking exception plus LGPL xdiff.
  - URL: https://github.com/rust-lang/git2-rs
  - Clone: `git clone --depth 1 https://github.com/rust-lang/git2-rs Repos/git2-rs`
- **tempfile**: Temporary files and directories for staging and atomic persist.
  - URL: https://github.com/Stebalien/tempfile
  - Clone: `git clone --depth 1 https://github.com/Stebalien/tempfile Repos/tempfile`
- **atomic-write-file**: Crash-tested single-file atomic writes, rejected in favour of `tempfile` plus rename.
  - URL: https://github.com/andreacorbellini/rust-atomic-write-file
  - Clone: `git clone --depth 1 https://github.com/andreacorbellini/rust-atomic-write-file Repos/atomic-write-file`
- **reflink-copy**: Reflink copies, unsupported on ext4, not needed.
  - URL: https://github.com/cargo-bins/reflink-copy
  - Clone: `git clone --depth 1 https://github.com/cargo-bins/reflink-copy Repos/reflink-copy`
- **trash-rs**: FreeDesktop trash, judged for backups and rejected there.
  - URL: https://github.com/ArturKovacs/trash-rs
  - Clone: `git clone --depth 1 https://github.com/ArturKovacs/trash-rs Repos/trash-rs`
- **similar**: Text diff library used for unified and inline diffs.
  - URL: https://github.com/mitsuhiko/similar
  - Clone: `git clone --depth 1 https://github.com/mitsuhiko/similar Repos/similar`
- **imara-diff**: Faster histogram diff, kept as an upgrade path.
  - URL: https://github.com/pascalkuthe/imara-diff
  - Clone: `git clone --depth 1 https://github.com/pascalkuthe/imara-diff Repos/imara-diff`
- **diffy**: Diff and patch library, no advantage here.
  - URL: https://github.com/bmwill/diffy
  - Clone: `git clone --depth 1 https://github.com/bmwill/diffy Repos/diffy`
- **serde-saphyr**: Strict typed YAML with error snippets, the config reader.
  - URL: https://github.com/bourumir-wyngs/serde-saphyr
  - Clone: `git clone --depth 1 https://github.com/bourumir-wyngs/serde-saphyr Repos/serde-saphyr`
- **saphyr**: YAML parser building blocks, untyped trees.
  - URL: https://github.com/saphyr-rs/saphyr
  - Clone: `git clone --depth 1 https://github.com/saphyr-rs/saphyr Repos/saphyr`
- **yaml-rust2**: YAML parser, comments lost on emit.
  - URL: https://github.com/Ethiraric/yaml-rust2
  - Clone: `git clone --depth 1 https://github.com/Ethiraric/yaml-rust2 Repos/yaml-rust2`
- **serde-yaml-ng**: Maintained fork of the archived `serde_yaml`, last release 2024.
  - URL: https://github.com/acatton/serde-yaml-ng
  - Clone: `git clone --depth 1 https://github.com/acatton/serde-yaml-ng Repos/serde-yaml-ng`
- **yaml-edit**: Comment-preserving YAML editor on a lossless tree, tested with 5,000 random configs: list replacement bug.
  - URL: https://github.com/jelmer/yaml-edit
  - Clone: `git clone --depth 1 https://github.com/jelmer/yaml-edit Repos/yaml-edit`
- **toml-rs**: `toml_edit`, plan C if YAML editing is dropped.
  - URL: https://github.com/toml-rs/toml
  - Clone: `git clone --depth 1 https://github.com/toml-rs/toml Repos/toml-rs`
- **gray-matter-rs**: Frontmatter parser, no gain over a manual split.
  - URL: https://github.com/the-alchemists-of-arland/gray-matter-rs
  - Clone: `git clone --depth 1 https://github.com/the-alchemists-of-arland/gray-matter-rs Repos/gray-matter-rs`
- **markdown-frontmatter**: Frontmatter parser, errors on files without frontmatter.
  - URL: https://github.com/imbolc/markdown-frontmatter
  - Clone: `git clone --depth 1 https://github.com/imbolc/markdown-frontmatter Repos/markdown-frontmatter`
- **BLAKE3**: Hash implementation, speed irrelevant at 2 MB of skills.
  - URL: https://github.com/BLAKE3-team/BLAKE3
  - Clone: `git clone --depth 1 https://github.com/BLAKE3-team/BLAKE3 Repos/BLAKE3`

### Web view and distribution

- **axum**: Web framework for the loopback server (stage 1).
  - URL: https://github.com/tokio-rs/axum
  - Clone: `git clone --depth 1 https://github.com/tokio-rs/axum Repos/axum`
- **tiny-http**: Synchronous server alternative, rejected for open CVEs and no release since 2022.
  - URL: https://github.com/tiny-http/tiny-http
  - Clone: `git clone --depth 1 https://github.com/tiny-http/tiny-http Repos/tiny-http`
- **dufs**: Local file server, reference for loopback and auth choices.
  - URL: https://github.com/sigoden/dufs
  - Clone: `git clone --depth 1 https://github.com/sigoden/dufs Repos/dufs`
- **miniserve**: Local file server, reference for embedded assets and flags.
  - URL: https://github.com/svenstaro/miniserve
  - Clone: `git clone --depth 1 https://github.com/svenstaro/miniserve Repos/miniserve`
- **mdBook**: Embeds its web assets and serves locally on axum, the precedent for a small embedded UI.
  - URL: https://github.com/rust-lang/mdBook
  - Clone: `git clone --depth 1 https://github.com/rust-lang/mdBook Repos/mdBook`
- **jupyter_server**: Token and origin checking model of a local server that can run code.
  - URL: https://github.com/jupyter-server/jupyter_server
  - Clone: `git clone --depth 1 https://github.com/jupyter-server/jupyter_server Repos/jupyter_server`
- **cargo-dist**: Release builds, checksums and attestations for GitHub Releases.
  - URL: https://github.com/axodotdev/cargo-dist
  - Clone: `git clone --depth 1 https://github.com/axodotdev/cargo-dist Repos/cargo-dist`
- **release-plz**: Workspace layout reference; its conventional-commit assumption rules it out for releases.
  - URL: https://github.com/release-plz/release-plz
  - Clone: `git clone --depth 1 https://github.com/release-plz/release-plz Repos/release-plz`
