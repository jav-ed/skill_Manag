/*
  CANONICAL TEMPLATE - astro skill.

  Copy this file to `src/Scripts/Content_Schemas/schema_Og_Img.ts` when a
  project generates OG images from content collections.

  The template catalogue lives in Data/Og_Img because it is a project input.
  The schema imports that list so author mistakes fail during content
  validation before the manifest reaches the external renderer.

  OG title and description are card display copy, not prose paragraphs. Write
  them without terminal full stops; the manifest builder strips one final
  ASCII full stop before render for older entries.
*/

import { z } from "astro/zod";
import { og_Template_Names } from "../../Data/Og_Img/og_Image_Config";

export const og_Img_Schema = z.object({
  title:       z.string().min(1),
  description: z.string().optional(),

  /*
    Optional source image for image-led OG templates. This does not select an
    image template by itself; pair it with template_Override when an entry
    intentionally wants an image layout. If the resolved template is text-only,
    the manifest builder fails instead of silently ignoring the field. Paths
    are relative to the MDX file.
  */
  image: z.string().min(1).optional(),

  /*
    Collection defaults own normal template selection. Set this only when one
    entry intentionally needs a different OG card template than its siblings.
    The manifest may still emit the resolved value as "variant" when an
    external renderer contract uses that field name.
  */
  template_Override: z.enum(og_Template_Names).optional(),
}).strict();

export type Og_Img_Data = z.infer<typeof og_Img_Schema>;
