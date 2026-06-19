// Starter `src/content.config.ts` for the org-standard content-collection
// pattern:
//
//   - Flat language siblings under each collection: src/Content/<Collection>/{de,en,es}/
//   - `path_Based_Id` forces entry id = file path (decoupled from slug)
//   - Per-language `slug:` frontmatter field for public URLs
//   - Schemas live in `src/Scripts/Content_Schemas/` (one per collection)
//   - Translations linked by filename stem (translation_key)
//
// COPY THIS FILE to `src/content.config.ts` and adapt:
//   1. Add an import for each collection's schema.
//   2. Add a `defineCollection` block per collection.
//   3. Export them all from the `collections` object.
//
// See ../linker_Content.md for the full pattern, ../add_New.md for the
// add-a-collection workflow, ../folder_Structure.md for folder naming.

import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";

// --- helpers -----------------------------------------------------------------

// Force entry id = file path without extension. See Templates/path_Based_Id.ts
// for the full rationale.
const path_Based_Id = ({ entry }: { entry: string }) =>
  entry.replace(/\.mdx?$/, "");

// --- schema imports ----------------------------------------------------------

// Import each collection's Zod schema from src/Scripts/Content_Schemas/.
// Example:
//   import { blog_Schema }     from "src/Scripts/Content_Schemas/schema_Blog";
//   import { glossary_Schema } from "src/Scripts/Content_Schemas/schema_Glossary";

// --- collections -------------------------------------------------------------

// One defineCollection per collection. The shape is identical across
// collections; only `base` (folder path) and `schema` change.
//
// Example collection (uncomment and adapt):
//
// const blog = defineCollection({
//   loader: glob({
//     pattern: "**/*.mdx",
//     base: "./src/Content/blog",
//     generateId: path_Based_Id,
//   }),
//   schema: blog_Schema,
// });
//
// const glossary = defineCollection({
//   loader: glob({
//     pattern: "**/*.mdx",
//     base: "./src/Content/glossary",
//     generateId: path_Based_Id,
//   }),
//   schema: glossary_Schema,
// });

// --- export ------------------------------------------------------------------

// Astro reads this object to register every collection. Add every
// defineCollection variable here.
export const collections = {
  // blog,
  // glossary,
};
