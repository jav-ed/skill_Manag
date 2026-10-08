# Ui

The web interface of `skillmirror web`. It is its own project, separate from the Rust workspace: an [Astro](https://astro.build) shell, [Solid](https://www.solidjs.com) views, [Lucide](https://lucide.dev) icons (`lucide-solid`), [Motion](https://motion.dev) for the movement, built with [Vite](https://vite.dev) (Astro's own). The Rust server and this project meet in two places only:

- the **JSON API** the views read and the changes they send, written down in [`Project_Manag/Docs/Architecture/web_Api.md`](../Project_Manag/Docs/Architecture/web_Api.md) and pinned by `Crates/Web/src/server/tests/read.rs`;
- the **built files**: `npm run build` writes `dist/`, `Code/Development/Web/build_Ui.sh` copies it to `Crates/Web/assets/ui/`, and `Crates/Web/build.rs` puts those files into the binary. They are committed, so building or installing the Rust tool needs no node.

Nothing in the Rust workspace reads this folder, and nothing here reads Rust. If the interface ever moves to a repository of its own, this folder moves with its history and `Crates/Web/assets/ui/` becomes a release artifact.

## Work on it

Needs node 22.12 or newer.

```bash
cd Ui
npm ci                 # the exact versions of package-lock.json
npm run check          # type check
npm run build          # dist/
```

To see it against a real server, start one on a throwaway vault (`skillmirror web --allow-write`, link printed in the terminal) and develop against the built files: `Code/Development/Web/build_Ui.sh` after every change, reload the page. (`npm run dev` serves the pages but has no API behind it.) Before committing a change: `Code/Development/Web/build_Ui.sh` so that the committed files match, and `Code/Development/Web/check_Web.sh` to drive the result in Chromium.

## Rules that the server's policy makes

The server answers every page with `default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:`. So:

- no inline `<script>` or `<style>` and no `style="..."` in markup (Astro is configured not to inline anything: `astro.config.mjs`); set styles from script (`el.style`, Motion) or with classes;
- no Astro islands (they emit an inline hydration script): the shell page loads one module, `src/app/main.tsx`, which mounts the whole Solid app;
- text goes in as text, never as markup: no `innerHTML`, no `{@html}` equivalents. Names of skills and projects come from the user's disk and may be anything;
- a change is a `POST` with `Content-Type: application/json` and the header `X-Skillmirror: 1`; `src/app/api.ts` does that, use it;
- only addresses of the same server: no fonts, images or scripts from elsewhere.

`Crates/Web/src/server/tests/files.rs` fails when a built page has an inline script or style, and `Code/Development/Web/drive_Web.js` fails on any policy violation in the browser.

## Layout

| Path | What |
|---|---|
| `astro.config.mjs` | no inlining, static output, one page per view |
| `src/pages/[...route].astro`, `src/layouts/Shell.astro` | the pages: all the same shell around `#app` |
| `public/` | `theme.js` (runs before paint so the saved theme shows at once), `favicon.svg` |
| `src/app/main.tsx`, `App.tsx` | start, sidebar, view switch, toasts |
| `src/app/router.ts` | a small router: the address decides the view, links with `data-link` are followed in place |
| `src/app/api.ts`, `types.ts` | the typed calls and the shapes of the answers |
| `src/app/views/` | Overview, Run (sync and push), Vault (the skills page), History, Doctor, Settings |
| `src/app/flow.tsx`, `panels.tsx` | the review, run and result of a change; the plan and undo panels |
| `src/app/components.tsx`, `motion.ts`, `toast.ts`, `styles.css` | shared parts |
