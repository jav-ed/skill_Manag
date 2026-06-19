# Styles — cascade layers, breakpoints, checklists

Tailwind v4 organises CSS into named cascade layers (`base`, `components`, `utilities`). Projects in this org use three buckets explicitly, plus a small set of `@custom-variant` breakpoints declared in the entry stylesheet. This file documents both, plus the checklist for adding / renaming / moving a stylesheet.

## Cascade layers

| Layer | What goes in | Lives in |
|---|---|---|
| `@layer base` | Element-level defaults: `html`, `body`, headings, links, `:focus-visible`, color-scheme, scrollbar tuning. | `Base/root_Vars.css` or equivalent base file |
| `@layer components` | Reusable class patterns: `.shell`, interaction primitives, callouts, lightbox, nav menus, prose. | `Base/interaction_Primitives.css`, header CSS, `Pages/<Page>/<feature>.css` |
| _unlayered_ | Overrides that must beat utility classes — e.g. `[data-theme="dark"] .bg-white { ... !important }`. | A final base override file if the project has one |

Tailwind utilities sit between `components` and `utilities` in the layer order. Unlayered CSS wins against utilities — use it only when a hardcoded utility in markup must be re-tinted (e.g. `.bg-white` overridden for dark mode, when fixing the markup would be a bigger refactor than the override).

Long-term, prefer fixing the markup to use tokens (`bg-background` / `bg-card`) over adding more unlayered overrides. Every unlayered color override is a band-aid for token-bypassing markup.

## Custom-variant breakpoints

Tailwind v4's built-in breakpoints (`sm` 640px, `md` 768px, `lg` 1024px, `xl` 1280px, `2xl` 1536px) cover most needs. When a project genuinely needs finer mobile-range targeting, add `@custom-variant` declarations in `Base/base_Main.css`:

```css
@custom-variant m_ss (@media (width <= 375px));                          /* iPhone SE */
@custom-variant m_sm (@media (width >= 376px) and (width <= 416px));    /* Samsung S */
@custom-variant m_xs (@media (width >= 417px) and (width <= 639px));    /* small mobile */
```

Then `m_ss:hidden` / `m_sm:text-sm` / etc. compile like any built-in Tailwind variant.

Two rules:

1. **Name reflects scope, not device.** `m_ss` reads as "mobile, smallest size" — survives the iPhone SE being discontinued. Avoid `iphone_se` or `pixel_5`.
2. **If the list grows past 3–4 entries, move it into a dedicated `Base/breakpoints.css`** and import that from `base_Main.css`. Keeps the entry file scannable.

## Checklist — adding a new stylesheet

1. Decide: shared chrome (`Base/`) or page-scoped (`Pages/<PageName>/`)?
2. File name in `snake_Case.css`, descriptive of what it does (not where it sits).
3. If `Base/`: add `@import "./your_File.css";` to `base_Main.css` in cascade order. Keep `theme.css` before `derived_Color_Vars.css`, and keep derived files after the base variables they depend on.
4. If `Pages/<PageName>/`: import it directly from the layout that needs it — `import "../Styles/Pages/<PageName>/your_File.css"` in the `.astro` frontmatter.
5. Wrap rules in the appropriate `@layer` (`base` or `components`). Unlayered only if you specifically need to beat utility classes.
6. Reference tokens (`var(--foreground)`, `bg-primary`) — never hardcoded colors. See [tokens.md](tokens.md).
7. If the file targets long-form content, mirror the prose pattern: put it in `Pages/Posts/` and have each long-form layout opt in.

## Checklist — renaming or moving a stylesheet

CSS filenames get referenced in doc comments throughout the codebase, not just in `import` statements. When you rename, also grep:

```bash
grep -rn "old_file_name\|old/path" src/
```

Update both code imports and doc comments. Catalog of files Partner's last styles migration touched (illustrative — your delta may differ):

- `layouts/Layout.astro` — the global import line.
- Page layouts that opt into page-scoped CSS (`Blog_Post_Layout.astro`, etc.).
- `Base/base_Main.css` — the import chain.
- `Base/base_Main.css` — header doc comment listing sibling files.
- Any `.astro` / `.ts` / `.js` file whose comments mention the old filename ("Styled in <name>.css", "see <name>.css for rules", etc.).

The bare-filename mentions (e.g. "documented in prose.css") will still resolve via `find . -name <name>.css` even if you skip them, but updating helps future readers.

## Checklist — moving CSS from Base/ to Pages/<PageName>/

When chrome that was loading globally turns out to be page-scoped (e.g. prose on Partner):

1. `mv Base/<file>.css Pages/<PageName>/<file>.css`.
2. Remove the `@import "./<file>.css";` line from `base_Main.css`.
3. Remove the file's entry from `base_Main.css`'s header doc comment.
4. In every page/layout that uses the markup the file targets, add a direct import: `import "../Styles/Pages/<PageName>/<file>.css";`.
   - Vite dedupes multiple imports of the same file, so having all 5 long-form layouts import the same `prose.css` is fine.
5. Update sibling Base file doc comments (`base_Main.css` etc.) that referenced the moved file.
6. Probe each affected route in the dev server and confirm the relevant classes still render correctly.
