# Audit — Output Schema & URL Discovery

## How URL discovery works (dev mode)

1. If `dist/` exists → derives localhost URLs from `dist/**/*.html` paths (covers all pages, all languages)
2. If no `dist/` → reads a `pages.ts` config file (fallback, rarely needed)

Always emits a staleness warning when using dist-derived URLs — rebuild first if pages have changed.

## JSON schema (report.json)

```json
{
  "generatedAt": "<ISO 8601>",
  "mode": "dev | dist",
  "source": "<dist path or base URL>",
  "note": "<path to pages list file>",
  "summary": { "totalPages": 42, "pagesWithViolations": 7, "totalViolations": 12 },
  "pages": [
    {
      "source": "<URL or file path>",
      "violations": [
        {
          "id": "<axe rule id>",
          "impact": "critical | serious | moderate | minor",
          "description": "<readable description>",
          "helpUrl": "<axe docs URL>",
          "totalNodes": 8,
          "nodes": [{ "target": ["<CSS selector>"], "html": "<truncated>", "failureSummary": "<fix hint>" }]
        }
      ]
    }
  ]
}
```

`target` (CSS selector) is never truncated — use it to find the element in code. `html` is visual context only.
