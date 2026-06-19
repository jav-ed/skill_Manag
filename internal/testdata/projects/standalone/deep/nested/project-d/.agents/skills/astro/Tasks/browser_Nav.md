# Browser Navigation

Use `playwright-cli` to open, inspect, and screenshot the running site.

```bash
playwright-cli open http://localhost:4321
playwright-cli goto http://localhost:4321/en/about
playwright-cli snapshot
playwright-cli screenshot --filename=Scratch/Screenshots/<descriptive-name>.png
playwright-cli close
```

**Screenshot rule:** always save to `Scratch/Screenshots/<descriptive-name>.png`.
The `Scratch/` folder is gitignored — use it freely for any working output.
