# Setup: Internal Repo Paths

This file records first-party repo shortcut names and host-scoped full checkout paths. Use shortcut names in docs first, then resolve them here only when an absolute path matters.

Full paths can differ between hosts and clones. Before relying on a full path, verify which host you are on: run `hostname` and match it with the table heading (the `remote-helper` skill documents the check). The paths below are valid on `javPc`, the local development machine.

External or third-party reference clones do not belong here. Track those in [External reference repos](repos_List.md).

## Repo Shortcuts

| Shortcut | Full path on `javPc` | Use |
|---|---|---|
| `Skill_Vault` | `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/07_Coding_Env/05_Skill_Collec` | The master skill vault that sync, push and list read from. Its `config.yaml` holds the scan `root`, the `mandatory` skills and the scan exclusions. The pointer file `~/.config/skill_Manag/vault` contains this path. |
