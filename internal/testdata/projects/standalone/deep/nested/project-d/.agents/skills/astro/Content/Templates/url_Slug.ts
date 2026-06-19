// Reusable Zod schema piece for the public URL slug of a content-collection
// entry. Enforces kebab-case: lowercase letters, digits, single hyphens
// between segments, no leading or trailing hyphen, no consecutive hyphens.
//
// Apply this to every per-language `slug:` field across every collection.
// Define `url_Slug` once and reuse it in every schema in
// `src/Scripts/Content_Schemas/` — that way every collection enforces the
// same slug grammar and a bad slug fails the build with a clear error.
//
// Where to put this in a new project:
//   - As a sibling of the schema files, e.g. `src/Scripts/Content_Schemas/url_Slug.ts`,
//     imported by every `schema_<Collection>.ts`.
//
// Usage in a schema:
//   import { url_Slug } from "src/Scripts/Content_Schemas/url_Slug";
//   export const blog_Schema = z.object({
//     slug: url_Slug,
//     // ...other fields
//   });
//
// See ../slug_And_Id.md for the full pattern.

import { z } from "astro/zod";

export const url_Slug = z
  .string()
  .regex(
    /^[a-z0-9]+(-[a-z0-9]+)*$/,
    "slug must be kebab-case (lowercase, hyphens)",
  );
