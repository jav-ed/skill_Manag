# Web check

Builds a throwaway world (a vault with a group, four projects, one named like markup), starts `skillmirror web --allow-write` on it and drives the pages in Chromium through Playwright: the first-visit link and its cookie, the overview, the filter, a sync plan and its apply, a push, the history and an undo, the refusals a page script can provoke, any request that leaves the server, and console errors. It prints one `ok` or `FAIL` line per check and exits 1 on a failure. Screenshots are written when a folder is given.

```bash
Code/Development/Web/check_Web.sh                                  # target/debug/skillmirror
Code/Development/Web/check_Web.sh path/to/skillmirror /tmp/shots   # also writes overview.png, plan.png, ...
```

Needs: bash, git, node with Playwright (`PLAYWRIGHT_NODE` names its folder, default `/opt/node-tools/node_modules/playwright`), and a Chromium (`PLAYWRIGHT_CHROMIUM` names the binary when Playwright does not find its own). It touches nothing outside a temporary folder: not your vault, not your projects, not your config.

| file | job |
|---|---|
| `check_Web.sh` | makes the world, starts the server, reads the link |
| `drive_Web.js` | the browser checks |
