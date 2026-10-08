# Report check

Builds a throwaway world (a vault with a group, four projects in different states), writes the HTML report with a `skillmirror` binary and drives the file in Chromium through Playwright: the matrix rows, the filter and its "no match" line, the light and dark button, the anchor that opens a diff, any request that leaves the file, and console errors. It prints one `ok` or `FAIL` line per check and exits 1 on a failure. Screenshots are written when a folder is given.

```bash
Code/Development/Report/check_Report.sh                                  # target/debug/skillmirror
Code/Development/Report/check_Report.sh path/to/skillmirror /tmp/shots   # also writes light.png and dark.png
```

Needs: bash, git, node with Playwright (`PLAYWRIGHT_NODE` names its folder, default `/opt/node-tools/node_modules/playwright`), and a Chromium (`PLAYWRIGHT_CHROMIUM` names the binary when Playwright does not find its own). It touches nothing outside a temporary folder: not your vault, not your projects, not your config.

| file | job |
|---|---|
| `check_Report.sh` | makes the world, runs `skillmirror report`, starts the driver |
| `drive_Report.js` | the browser checks |
