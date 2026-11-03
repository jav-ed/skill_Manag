# Setup: Internal Repo Paths

This file records first-party repo shortcut names and host-scoped full checkout paths. Use shortcut names in docs first, then resolve them here only when an absolute path matters.

Full paths can differ between hosts and clones. Before relying on a full path, verify which host you are on: run `hostname` and match it with the table heading (the `remote-helper` skill documents the check). The paths below are valid on `javPc`, the local development machine.

External or third-party reference clones do not belong here. Track those in [External reference repos](repos_List.md).

## Repo Shortcuts

| Shortcut | Full path on `javPc` | Use |
|---|---|---|
| `Skill_Vault` | `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/07_Coding_Env/05_Skill_Collec` | The master skill vault that sync, push and list read from. Its `config.yaml` holds the scan `root`, the `mandatory` skills and the scan exclusions. The pointer file `~/.config/skill_Manag/vault` contains this path. |
| `Skillmirror_Docs` | `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/01_Projects/07_Infrastruc/03_AI_Tools/01_AI_Helper/02_Skill_Manager/Skillmirror_Docs` | The public documentation site of skillmirror, a sibling of this repository (Forgejo `ai-tools/skillmirror_Docs`). This repository is the source of truth for what the tool does; the site explains it. |
| `Installations` | `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/01_Projects/07_Infrastruc/01_Installations` | Platform tools, investigations and the g12 setup. |
| `Web_Runtime` | `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/01_Projects/01_Web_Service/01_Zetun_Web/10_Web_Runtime` | The Caddy repository: the routes and the runtime folders the docs site is deployed into. |

## Local Layout

This repository and the docs repository are independent Git repositories, kept side by side in one workspace folder that is itself not a repository (its `README.md` explains the pair):

```text
02_Skill_Manager/
  README.md
  Skillmirror_Cli_Code/    this repository (GitHub jav-ed/skill_Manag)
  Skillmirror_Docs/        the documentation site (Forgejo ai-tools/skillmirror_Docs)
```

Do not place either repository under `Repos/`; that ignored folder is for disposable upstream clones. Until 2026-11-03 this repository was the workspace folder itself; agent memory is keyed by the folder path, so the memory of the old path was copied to the new one.

