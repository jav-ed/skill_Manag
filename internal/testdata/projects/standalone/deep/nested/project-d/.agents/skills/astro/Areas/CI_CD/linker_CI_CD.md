# CI/CD

Everything about deploying an Astro project via Woodpecker CI to a Linux server behind Caddy. The pipeline shape is the same across every Astro repo we ship: `install → build → og_images → compress → deploy`. What varies per project is the deploy target (the site directory on the server), the repo path on the agent machine, and whether the forge/Woodpecker trigger path is already active.

This folder is currently the working CI/CD skill area inside the Astro skill. Once the workflow is proven end-to-end, move it into a dedicated CI/CD skill and leave this folder as a small router that tells agents to read that skill.

Default assumption: the CI platform already exists. In normal project work, do not start with server/bootstrap setup. Start with the project repo, its `.woodpecker.yml`, its `CI/` scripts, local dry-runs, and whether the repo is enabled in Woodpecker. Only enter platform checks when the user asks for first-time setup or when evidence shows Forgejo, Woodpecker, OAuth, or webhooks are missing or broken.

## Routing

- [Platform](../../../ci-cd/Platform/linker_Platform.md): shared Forgejo/Woodpecker service checks, API discovery, and secret-backed access. Use when the platform itself may be down or misconfigured.
- [Application](../../../ci-cd/Application/linker_Application.md): one Astro repo's CI/CD integration, trigger checks/setup, deploy, rollback, verification, and debugging.
- [Architecture](../../../ci-cd/Architecture/architecture.md): how the pipeline works under the hood — the five steps and what each does, why local backend (no containers), why external scripts (log truncation workaround), `skip_clone: true` implications, release naming + retention, atomic symlink swap, and the footguns (host coupling, uncommitted-changes-get-built, no failure notification by default, `shared/` directory drift, optional `fnox.toml`, OG generator contract drift). Read once when you want to understand *why* the pipeline is shaped this way.
- [Limitations](../../../ci-cd/Deferred/limitations.md): current boundaries while this CI/CD area is treated as the source of truth before extraction into its own skill — single local runner, single-worker assumptions, skill-owned `fnox.toml`, and what must change before multi-machine or multi-worker use.
- [Template/](../../../ci-cd/Template/): the canonical `.woodpecker.yml` + `CI/` folder to copy into a new Astro repo. Five placeholders — `__Project_Abs_Path__`, `__Deploy_Host__`, `__Deploy_Base__`, `__Compressor_Bin__`, `__Og_Image_Dir__` — get substituted in one `sed` pass during first setup.

## When to open which

- "I have a new Astro repo and need a pipeline" → [Application integration](../../../ci-cd/Application/Integration/first_Setup.md)
- "Pushes do not trigger Woodpecker for this repo" → [Application trigger checks](../../../ci-cd/Application/Trigger/repo_Trigger_Checks.md)
- "Is Forgejo / Woodpecker / OAuth working globally?" → [Platform](../../../ci-cd/Platform/linker_Platform.md)
- "How do I deploy / rollback / verify a release?" → [Application active use](../../../ci-cd/Application/Active_Use/daily_Workflow.md)
- "Why does the pipeline work this way?" → [Architecture](../../../ci-cd/Architecture/architecture.md)
- "What assumptions are we knowingly accepting right now?" → [Limitations](../../../ci-cd/Deferred/limitations.md)
- "Where's the template I copy from?" → [Template/](../../../ci-cd/Template/)
- "How do I set up the Woodpecker server itself?" → [Platform](../../../ci-cd/Platform/linker_Platform.md), then the CI/CD runtime repo docs

## Per-repo specifics

If your project has a `Project_Manag/Docs/Architecture/CI_CD/` tree (the org's standard project-docs layout), record the per-repo values there: this site's deploy target, this site's Caddy file, this site's domain. The per-repo docs should *route through* this folder for the org-wide pattern and only state what is unique to the project.
