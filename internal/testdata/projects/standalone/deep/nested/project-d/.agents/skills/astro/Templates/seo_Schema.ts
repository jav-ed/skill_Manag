/*
  CANONICAL TEMPLATE - astro skill.

  Copy this file to `src/Scripts/Content_Schemas/schema_Seo.ts` in a project
  that uses content collections. Copy `og_Img_Schema.ts` next to it, then
  compose `seo_Schema` into every collection.

  Why this exists:
  - One schema definition means every collection has the same SEO surface.
  - Missing `seo.title`, `seo.description`, or `seo.og_Img` fails at build time.
  - OG image copy stays separate from normal SEO copy. Do not reuse page title,
    nav labels, excerpts, or meta descriptions as OG layout content unless the
    operator explicitly approves that coupling.
*/

import { z } from "astro/zod";
import { og_Img_Schema } from "./schema_Og_Img";

export const seo_Schema = z.object({

  /*
    Page title. Shown in <title> and used by the head layer for normal
    metadata. Keep it fit for browser tabs and search results.
  */
  title: z.string(),

  /*
    Page description. Shown in <meta name="description"> and the normal OG
    description tag. Keep it fit for search/social metadata.
  */
  description: z.string(),

  /*
    Required because every generated OG card needs its own stable contract.
    Template choice is normally owned by Data/Og_Img defaults; use
    `template_Override` only for true one-page exceptions. For image-led
    one-page exceptions, pair that override with `image` in the same object.
  */
  og_Img: og_Img_Schema,

}).strict();

export type Seo_Data = z.infer<typeof seo_Schema>;
