# Style Export Template

Use this template when a project has a local MapLibre style builder and the approved style needs to be handed to the map-server operator as minified JSON plus gzip, Brotli, and Zstandard sidecars.

The template is not imported by Astro directly. It is copied into the project-level tooling folder, usually:

```text
Code/Maps/export_<Project>_Map_Style.ts
```

Keep the project style builder in the same tooling area, for example:

```text
Code/Maps/project_Vector_Map_Style.ts
```

Do not keep the style builder under `src/Scripts/Browser_Client/` once the frontend loads hosted style JSON. At that point the builder is an authoring/export source, not runtime browser code.

The generated files should go into an ignored scratch folder, usually:

```text
Scratch/G12_Handoff/map_Styles/
```

## Copy Into A Project

1. Copy [export_Map_Style.ts](./export_Map_Style.ts) into `Code/Maps/`.
2. Rename it for the project, for example `export_Partner_Map_Style.ts`.
3. Put the project's local style builder next to it, for example `Code/Maps/project_Vector_Map_Style.ts`.
4. Adjust the import so it points at the project's local style builder.
5. Replace tenant, tileset, public map host, private map host, and attribution text.
6. Add a package script, for example:

```json
{
  "scripts": {
    "maps:export-style": "bun Code/Maps/export_Partner_Map_Style.ts"
  }
}
```

7. Run the script and verify the output before copying it to the map server:

```bash
bun run maps:export-style
jq -e '.version == 8 and (.layers | length > 0)' Scratch/G12_Handoff/map_Styles/<tenant>.public.json
gzip -t Scratch/G12_Handoff/map_Styles/*.json.gz
brotli --test Scratch/G12_Handoff/map_Styles/*.json.br
zstd -t Scratch/G12_Handoff/map_Styles/*.json.zst
```

## Output Contract

For each style target, the exporter writes:

```text
<name>.json
<name>.json.gz
<name>.json.br
<name>.json.zst
```

The `.json` file is minified. The sidecars are precompressed files for Caddy. Do not commit these outputs; keep them under `Scratch/` or another ignored artifact folder.
