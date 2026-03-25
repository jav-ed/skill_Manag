# Adding a New Collection

End-to-end steps for adding a new content collection. Uses "Web Dev Tips" blog series as a running example (`503_Web_Tips`).

---

## 1. Pick a number prefix

Consult the [numbering table](content_Collections_Structure.md#numbering-convention) and choose the next free number in the right range. Blog series → `5XX`. Next free after `502` → `503`.

---

## 2. Create the folder structure

```
src/Content/503_Web_Tips/
├── en/
│   └── 0001_First_Post.mdx
└── translation/
    ├── de/
    │   └── 0001_First_Post.mdx
    ├── es/
    │   └── 0001_First_Post.mdx
    ├── fr/
    │   └── 0001_First_Post.mdx
    └── zh/
        └── 0001_First_Post.mdx
```

Only create language folders for languages that actually have translations ready. Missing language folders are fine — `avail_lang_provider` will simply not list them.

---

## 3. Write the MDX files

Follow the schema for the collection type. For a blog entry series, use the [blog entry schema](content_Collections_Schema.md#blog-entry-schema-5xx). Use the new nested SEO fields for all new content:

```yaml
---
mdx_Info:
  title: "your post title here"
  subtitle: "optional subtitle"
  desc: "short description for cards"
  date: "01.01.2025 | 1 Rajab 1446"
  seo:
    meta_Title: "..."
    meta_Content: "..."
    og:
      title: "..."
      description: "..."
overview_finder: "intro"
---
```

---

## 4. Define the schema and register the collection

**4a.** Add a new schema to the closest matching file in `src/Scripts/Content_Schemas/`. For a new `5XX` blog series, add to `schema_Entries_5xx.ts`:

```ts
export const web_Tips_503_Schema = base_Entry_Schema.extend({
  image: z.string().optional(),  // only if needed
});
```

**4b.** Import the schema and register the collection in `src/content.config.ts`:

```ts
import { ..., web_Tips_503_Schema } from 'src/Scripts/Content_Schemas/schema_Entries_5xx';

const web_Tips_503 = defineCollection({
  loader: glob({ pattern: ['**/*.md', '**/*.mdx'], base: './src/Content/503_Web_Tips' }),
  schema: web_Tips_503_Schema,
});

export const collections = {
  // ...existing...
  web_Tips_503,
};
```

---

## 5. Create the page file in `src/pages/`

Decide the URL path (e.g. `/[lang]/pub/coding/web-tips/[...web_tips].astro`) and create the file. Copy any existing blog entry page and change two things:

- `ct_Collec_Reader` → your collection key
- `param_Name` → matches the filename param (e.g. `"web_tips"`)

```ts
import { create_Static_Paths } from 'src/Scripts/Astro_Frontmatter/2_Common/1_Routing/routing_Factory';

export const ct_Collec_Reader = "web_Tips_503";

export async function getStaticPaths() {
  return create_Static_Paths({ collection_Key: ct_Collec_Reader, param_Name: "web_tips" });
}
```

See [Routing doc](content_Collections_Routing.md) for full context.

---

## 6. Link from a parent overview (if applicable)

If the new series belongs under an existing overview page (e.g. `201_Web_Overview`), add its slug key to that overview's `page_Cont` in `content.config.ts` and update the overview MDX files to include the new key.

---

## Checklist

- [ ] Folder created under `src/Content/NNN_Name/`
- [ ] `en/` and `translation/{lang}/` subfolders present
- [ ] MDX files written with correct frontmatter (use nested `og:` SEO fields)
- [ ] Schema added to the right file in `src/Scripts/Content_Schemas/`
- [ ] Collection registered in `src/content.config.ts` and exported
- [ ] Page file created in `src/pages/` using `create_Static_Paths` factory
- [ ] Parent overview updated (if applicable)
