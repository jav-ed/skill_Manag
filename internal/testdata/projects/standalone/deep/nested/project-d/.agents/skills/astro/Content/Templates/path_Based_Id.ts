// Force content-collection entry IDs to be the file path (without extension),
// decoupling internal identity from the frontmatter `slug:` field.
//
// Why this matters: Astro's default `generateId` uses the frontmatter `slug:`
// as the entry ID. With per-language translated slugs (the org-standard
// pattern), two posts can legitimately have the same slug — e.g. both the
// German and English glossary entries for "caries" might use `slug: karies`.
// Their IDs would collide. Forcing the ID to be the file path keeps each
// language's file uniquely identified, while `data.slug` is reserved for the
// public URL slug.
//
// Where to put this in a new project:
//   - Inline at the top of `src/content.config.ts` if it's only used there.
//   - Lift to `src/Utils/Common/path_Based_Id.ts` if multiple files import it.
//
// See ../slug_And_Id.md for the full pattern and the translation_key concept.

export const path_Based_Id = ({ entry }: { entry: string }) =>
  entry.replace(/\.mdx?$/, "");
