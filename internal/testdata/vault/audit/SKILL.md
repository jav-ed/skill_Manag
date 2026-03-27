---
name: audit
description: Run the accessibility audit CLI tool (axe-core). Use when auditing a web project for accessibility violations — dist mode (static HTML) or dev mode (Playwright against a live server).
---

# Audit

Axe-core accessibility audit CLI. Globally linked — run from the project root so relative paths resolve correctly.

## Modes

### dist — static HTML, no browser (run after build)

```bash
audit dist ./dist
audit dist ./dist --tags 22aa --min-impact critical --output ./report.json
```

| Flag | Default | Description |
|---|---|---|
| `path` | `./dist` | Path to dist/ folder (positional) |
| `--tags` | `21aa` | WCAG level: `2a` `2aa` `21aa` `22aa` |
| `--min-impact` | `serious` | Exit 1 if violations at this level or above: `minor` `moderate` `serious` `critical` |
| `--output` | _(none)_ | Write JSON report to this path |

### dev — Playwright against a live server

```bash
audit dev http://localhost:4321
audit dev http://localhost:4321 --output ./report.json
```

| Flag | Default | Description |
|---|---|---|
| `url` | `http://localhost:4321` | Base URL (positional) |
| `--dist` | `./dist` | Path to dist/ for URL discovery |
| `--tags` | `21aa` | WCAG level: `2a` `2aa` `21aa` `22aa` |
| `--min-impact` | `serious` | Exit 1 if violations at this level or above |
| `--output` | _(none)_ | Write JSON report to this path |

### report — read and filter a saved report (no re-crawl)

```bash
audit report ./report.json
audit report ./report.json --top 10 --min-impact critical
```

| Flag | Default | Description |
|---|---|---|
| `file` | _(required)_ | Path to a saved `report.json` (positional) |
| `--top` | _(all)_ | Show at most N violations, sorted critical → minor |
| `--min-impact` | `minor` | Skip violations below this level |

## Output

`--output` writes two files:

| File | Content |
|---|---|
| `report.json` | Violations-only pages, nodes capped at 5, html truncated at 200 chars |
| `report.pages.json` | Flat list of all inspected pages — use to verify discovery coverage |

The summary block (`totalPages`, `pagesWithViolations`, `totalViolations`) is near the top of `report.json` — read lines 1–30 first before loading the full file.

## References

- [JSON schema, URL discovery details](./references/schema.md)
