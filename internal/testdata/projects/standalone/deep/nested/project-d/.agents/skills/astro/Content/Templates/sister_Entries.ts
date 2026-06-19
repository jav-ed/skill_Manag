// Find all language versions of a given content-collection entry, linked by
// their translation_key (the filename stem). Returns the entries — one per
// language that has a translation of the same translation_key — including
// the input entry itself.
//
// This is the canonical lookup for cross-language navigation: given the
// current post, render a language switcher pointing at every translated
// version that exists. Entries that haven't been translated yet simply
// don't appear in the result.
//
// Where to put this in a new project:
//   - `src/Utils/<Feature>/sister_Entries.ts` if it's used in one feature
//     (e.g. only the blog), or
//   - `src/Utils/Common/sister_Entries.ts` if multiple collections use it.
//
// Usage in a page template:
//   import { getCollection } from "astro:content";
//   import { sister_Entries } from "src/Utils/Common/sister_Entries";
//
//   const all_Posts = await getCollection("blog");
//   const sisters   = sister_Entries(post, all_Posts);
//   // sisters: every language version of `post`, identified by filename match.
//
// Assumes the org-standard pattern: flat <lang>/ folders, path_Based_Id, and
// filenames identical across language folders. See ../slug_And_Id.md.

import type { CollectionEntry, CollectionKey } from "astro:content";

export function sister_Entries<C extends CollectionKey>(
  post: CollectionEntry<C>,
  all:  CollectionEntry<C>[],
): CollectionEntry<C>[] {
  // The entry ID is "<lang>/<translation_key>" (e.g. "de/0001_first-visit").
  // Drop the language segment to get the translation_key.
  const translation_Key = post.id.split("/").slice(1).join("/");

  return all.filter(entry => {
    const entry_Key = entry.id.split("/").slice(1).join("/");
    return entry_Key === translation_Key;
  });
}
