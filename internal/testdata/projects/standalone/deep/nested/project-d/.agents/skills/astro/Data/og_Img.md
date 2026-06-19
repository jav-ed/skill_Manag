# Data — Og_Img/

`src/Data/Og_Img/` holds the project-owned inputs for generated OG images. These files are configuration and contracts: a project engineer changes them when the page set, template set, output format, reusable assets, or theme mapping changes. The generator code that reads and validates those inputs lives in `src/Scripts/OG_Images/`.

Use this page when deciding where an OG value belongs, registering a template, adding an asset role, or changing output paths. For the manifest build machinery itself, use [Scripts/OG_Images](../Scripts/og_Images.md) and [OG images pipeline](../Scripts/og_Images_Pipeline.md).

## Boundary

| Concern | Home |
|---|---|
| Which templates this project may emit | `src/Data/Og_Img/og_Image_Config.ts` |
| Which template a collection normally uses | `src/Data/Og_Img/og_Image_Config.ts` |
| One-off per-page template override field | MDX `seo.og_Img.template_Override` |
| Per-entry source image for image-led templates | MDX `seo.og_Img.image` |
| Reusable project image/SVG asset roles for OG templates | `src/Data/Og_Img/og_Asset_Config.ts` |
| Rare route-level hero image choices that cannot live in content | `src/Data/Og_Img/og_Asset_Config.ts` |
| Route-level icon choices for generic split-icon pages | `src/Data/Og_Img/og_Icon_Config.ts` |
| OG theme tokens consumed by the external renderer | `src/Data/Og_Img/og_Theme_Contract.js` |
| Output format, output folder, manifest cache path, source image root | `src/Data/Og_Img/og_Output_Config.js` |
| Validation, collection walking, manifest route, font resolution | `src/Scripts/OG_Images/` |
| Template JSX and renderer version routing | External `Og_Image` repo |

The rule is simple: editable project inputs live in Data. Machinery that computes, validates, or emits the manifest lives in Scripts.

## Current files

| File | Purpose |
|---|---|
| `og_Image_Config.ts` | Closed template catalogue plus `collection_Template_Defaults`. Every Astro content collection that emits pages needs a default here unless each entry has an explicit `seo.og_Img.template_Override`. Unknown template names fail through the Zod schema. |
| `og_Asset_Config.ts` | Asset registry with project-root-relative paths plus rare route-level hero image paths that cannot live in content frontmatter. The external renderer runs outside Astro's import graph, so asset paths are strings like `Public/favicon.svg` or `src/Assets/Imgs/services/vorsorge_hero.webp`. |
| `og_Icon_Config.ts` | Icon registry for `zet_General_Split_Icon`. A collection can select a renderer-owned built-in icon by name or a project-owned SVG asset id registered in `og_Asset_Config.ts`. There is intentionally no default icon. Missing mappings hard-fail. |
| `og_Output_Config.js` | Output format and paths shared by the site, manifest builder, and renderer. This includes `og_Img_Format`, `og_Img_Ext`, `rel_Path_Og_Fold`, `rel_Path_Json`, `rel_Path_Theme_Config`, and `og_Imgs_Rel_Path`. |
| `og_Theme_Contract.js` | Token map for the renderer. It mirrors `src/Styles/Base/theme.css` manually because the external renderer cannot rely on Astro's CSS cascade. |

## Template defaults and overrides

The normal path is collection-level defaults:

1. Register the template name in `og_Template_Names`.
2. Map each content collection key to a template in `collection_Template_Defaults`.
3. Let the manifest worker resolve every page through that default.

Use MDX `seo.og_Img.template_Override` only when a single page intentionally differs from its collection. Do not add override fields casually. Reusing SEO title or description fields for template behavior is discouraged because those fields have their own purpose and their content may change independently.

Image paths and image templates are separate decisions:

- `seo.og_Img.image` declares an image asset for the OG pipeline, but it does not choose an image template by itself.
- `seo.og_Img.template_Override` chooses the template when one entry wants an image layout or any other non-default layout.
- If `seo.og_Img.image` is present but the resolved template is text-only, the manifest builder fails. Remove the image or choose an image template explicitly.
- If an entry wants an image-specific template, provide both the template override and the image path in the same `seo.og_Img` block, unless the collection has a dedicated canonical image field already consumed by the manifest helper, such as blog `cover` or service `hero_image`.
- Do not infer "first image in MDX body" as the OG image. Body order is prose, not an OG contract.

The worker hard-fails when an existing collection lacks a template default and no per-entry override is present. That is intentional: a new route should not silently ship without an OG card.

Templates with extra required project data have sibling config files. For `zet_General_Split_Icon`, selecting the template in `og_Image_Config.ts` is not enough. Add the same collection key to `og_Icon_Config.ts` and choose either a built-in icon name or a custom SVG asset id. Missing icon config must hard-fail; do not add a fallback icon.

OG image `title` and `description` are display copy for a rendered card, not normal SEO prose. Write them without terminal full stops. The manifest helper strips one final ASCII full stop from `title` and `description` before render so older prose-shaped entries do not show noisy dots on cards. Meaningful punctuation such as question marks stays intact.

## Asset paths

Asset paths are project-root relative because the external renderer receives a manifest and reads the files directly. Do not use Astro imports inside `og_Asset_Config.ts`.

Allowed asset roles should be selective:

- `brand_Mark` stays a shared/default role, usually pointing at `Public/favicon.svg`.
- Legal image templates already have bundled neutral images in the external OG repo. Leave `og_Legal_Split_Image_Roles` and `og_Legal_Background_Image_Roles` empty to use those defaults. Only add legal image assets here when this project deliberately wants to override every image in the relevant legal set.
- Page-owned OG hero images belong in MDX `seo.og_Img.image`, paired with an explicit image template override when the collection default is not already image-led.
- Static/list routes may declare one route-level hero path in `og_Page_Hero_Image_Paths` only when that image genuinely cannot live in content frontmatter.
- Static/list routes that use `zet_General_Split_Icon` must map to an icon in `og_Icon_Config.ts`. Use `{ source: 'builtin', name: 'career' }` for renderer-owned icons, or `{ source: 'asset', asset_Id: '...' }` for project SVG icons registered as `kind: 'svg'` in `og_Asset_Config.ts`. Missing icon choices are errors, not defaults.
- Custom SVG icons may use `currentColor` in `stroke` or `fill`; the external renderer replaces it with the template foreground color. Keep icon SVGs self-contained: no embedded images, scripts, event handlers, or external references.
- Blog or service images should normally come from the page/content data when the template requires a cover or hero image, with validation that the referenced file exists.

`src/Scripts/OG_Images/` validates these paths before the manifest is accepted. Missing assets should fail loudly with the role name and path.

## Output paths

Keep output paths in `og_Output_Config.js`, not in Scripts:

- `rel_Path_Json` is the moved manifest cache location, currently `Cache/Og_Gen/og_Manifest.json`.
- `rel_Path_Og_Fold` is where the external renderer writes cards, currently `dist/Og_Gen`.
- `og_Img_Format` and `og_Img_Ext` must stay consistent because the renderer encodes the file and the site emits public `<meta property="og:image">` URLs.

After changing output format or paths, run `bun run build` and the local OG generation step before declaring the change done.

## Add or rename checklist

1. Decide whether the value is an editable project input. If yes, place it under `Data/Og_Img/`; if it computes or validates, place it under `Scripts/OG_Images/`.
2. When adding a template, update `og_Template_Names`, then add collection defaults for every affected collection.
3. When a collection uses `zet_General_Split_Icon`, add its icon choice to `og_Icon_Config.ts`; for custom SVGs, register the SVG path first in `og_Asset_Config.ts`.
4. When adding an asset role, use a project-root-relative path and keep the role name semantic, not tied to one current file name.
5. When changing theme tokens, update `og_Theme_Contract.js` from `src/Styles/Base/theme.css`.
6. Grep for old paths after renames, especially string paths consumed by the external renderer.
7. Run `bun run build` and the OG image script.
