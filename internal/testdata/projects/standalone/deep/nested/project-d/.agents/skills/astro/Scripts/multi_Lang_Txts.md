# Scripts/Multi_Lang_Txts

Before reaching for this folder, check whether the string actually belongs here at all. Repeated UI labels (buttons, errors, footer link copy) belong here. Page-bound content (blog posts, page headings, hero copy) belongs in `src/Content/` as MDX. Site-wide config belongs in `src/Data/Common/`. The rule and the gray-zone cases are spelled out in [Content vs Config vs Labels](../Content/content_Vs_Config_Vs_Labels.md).

`src/Scripts/Multi_Lang_Txts/` holds **translation tables** — objects keyed by language code, each value the localized version of a UI string. Footer labels, nav items, structural route labels, tooltips, page section copy, CTA button text. Build-time validation ensures every translation covers every supported language; a missing key fails the build.

This is where the actual *words shown to users* in each language live. It is distinct from `Utils/Common/` language *metadata* (which languages are supported, their text directions, their codes) — both are needed, both belong in different places.

## The distinction from `Utils/Common/`

| Goes in `Scripts/Multi_Lang_Txts/` | Goes in `Utils/Common/` |
|---|---|
| The German word for "Read more" | The fact that German uses LTR text direction |
| 12 translations of a CTA button label | The list of supported language codes |
| Tooltip copy in 5 languages | The mapping from short codes to local language names |
| Section headers per page, per language | The detect-lang-from-URL helper |

Strings shown to a user → here. Facts *about* languages → `Utils/Common/`. See [Utils/Common](../Utils/common.md) for the other side of this split.

## Internal structure

Splits **by page or feature** into PascalCase subfolders, with a top-level `assert_Translation.ts` for validation:

```
Multi_Lang_Txts/
├── assert_Translation.ts         # build-time validation helper
├── Shared/                       # cross-page UI labels — header nav, footer, CTAs
│   ├── chrome_Txt.ts             # visual chrome labels
│   └── route_Txt.ts              # structural route labels only
├── Pages/                        # per-page top-level copy
│   ├── home_Txt.ts
│   ├── contact_Txt.ts
│   └── blog_Txt.ts
├── Sections/                     # named page sections (FAQ, awards, services bundle)
│   ├── faq_Txt.ts
│   └── awards_Txt.ts
├── Services/, Team/, Legal/, …   # per-feature folders as the site grows
└── Tippy_Txts/                   # tooltip copy if the site uses a tooltip system
```

The list of subfolders is **per project**. A small site might only have `Shared/` and `Pages/`. A larger site adds folders as new feature areas (services, careers, team, legal) accumulate their own translation tables.

## Structural route labels

When route-aware surfaces need human-readable section names, put those labels in `Shared/route_Txt.ts`:

```ts
// Shared/route_Txt.ts
import { assert_All_Langs } from "../assert_Translation";

export const t_Route_Blog = {
  de: "Beiträge",
  en: "Articles",
  es: "Artículos",
};
assert_All_Langs(t_Route_Blog, "t_Route_Blog");
```

Use route labels for breadcrumbs, sitemap-adjacent UI, and any chrome that names a URL section as a structural route. Do **not** put routing logic in this file. It must only export text tables. URL builders, slug maps, route-key lookup, and breadcrumb assembly stay in `Scripts/Astro_Frontmatter/Common/Routing/` or a `Common/Breadcrumb/` helper.

Do not reuse footer-specific, nav-specific, or page-title strings for route structure just because the wording happens to match today. Prefer a route label when the consumer is naming a route section:

| Consumer | Preferred label source |
|---|---|
| Breadcrumb `Home > Blog > Post` | `Shared/route_Txt.ts` |
| Header nav item | `Shared/chrome_Txt.ts` or a nav-specific table |
| Footer column link | `Shared/chrome_Txt.ts` footer labels |
| Page `<h1>` / SEO title | `Pages/<page>_Txt.ts` or content frontmatter |

## File shape

Each file exports one or more language-keyed objects:

```ts
// Pages/contact_Txt.ts
import { assert_All_Langs } from "../assert_Translation";

export const contact_Form_Labels = {
  en: { name: "Name", email: "Email", message: "Message", submit: "Send" },
  de: { name: "Name", email: "E-Mail", message: "Nachricht", submit: "Senden" },
};

assert_All_Langs(contact_Form_Labels, "contact_Form_Labels");
```

`assert_All_Langs` compares the object's keys to `SUPPORTED_LANGS` from `Utils/Common/` and throws if any language is missing. Catching this at build time is the whole point of the folder — otherwise a missing translation only shows up when a user lands on that page in that language.

Translation objects use **English keys** (`name`, `email`, `submit`) even when the strings inside are not English. The keys are programmer-facing identifiers; the values are user-facing copy.

## Naming

- Folders: `PascalCase` named after the page, section, or feature in **English** (`Pages/`, `Services/`, `Tippy_Txts/`).
- Files: `snake_Case.ext`, typically ending in `_Txt.ts` or `_Texts.ts` to signal that this is a translation file. The `_Txt` suffix is shorthand for "this file is a string table", not a function or feature module.
- Translation keys (inside objects): `snake_Case` or `camelCase` — pick one per project and stick to it. The org convention is `snake_Case` for consistency with the rest of the codebase.

## What does NOT belong here

| Pattern | Where it goes |
|---|---|
| The list of supported languages, text directions, locale codes | `Utils/Common/` |
| A function that detects which language a URL is in | `Utils/Common/` |
| A function that maps a slug from one language to another | `Scripts/Astro_Frontmatter/Common/Routing/` |
| A function that builds breadcrumbs or JSON-LD | `Scripts/Astro_Frontmatter/Common/Breadcrumb/` or `JsonLd/` |
| Long-form content (blog posts, MDX pages) | `src/Content/<Collection>/` |
| Image alt text per language | Same file as the rest of the page's labels (`Pages/<page>_Txt.ts`) — don't split alt text into its own file |

## Importing

From `.astro` frontmatter, use bare `src/` paths:

```astro
---
import { contact_Form_Labels } from "src/Scripts/Multi_Lang_Txts/Pages/contact_Txt";
import { detect_Lang }          from "src/Utils/Common/lang";

const lang  = detect_Lang(Astro.url.pathname);
const labels = contact_Form_Labels[lang];
---
```

Translation files may import `SUPPORTED_LANGS` from `Utils/Common/` (for the assert call) and the local `assert_Translation.ts`. They should not import from pages or components.

## Build-time validation

`assert_Translation.ts` exports `assert_All_Langs(obj, name)`:

```ts
import { SUPPORTED_LANGS } from "src/Utils/Common/lang_Tables";

export function assert_All_Langs(obj: Record<string, unknown>, name: string): void {
  const missing = SUPPORTED_LANGS.filter(l => !(l in obj));
  if (missing.length > 0) {
    throw new Error(`[Translation] "${name}" is missing langs: ${missing.join(", ")}`);
  }
}
```

Call it at the bottom of every translation file. When a new language is added to `SUPPORTED_LANGS`, the next build fails on every translation that hasn't been updated yet — fast, exhaustive coverage.

## Add or rename — checklist

1. Translation strings → here. Language metadata → `Utils/Common/`. Two halves of the same concern, two different folders.
2. Pick the subfolder by what consumes the strings — `Shared/` for site chrome, `Pages/` for per-page copy, `Sections/` for reusable named sections, feature folders (`Services/`, `Team/`) as the site grows.
3. File name ends in `_Txt.ts` (or `_Texts.ts`) — signals "this file is a string table".
4. Object keys in `snake_Case`; values are user-facing copy in each language.
5. Add `assert_All_Langs(obj, "name")` at the bottom of every exported translation object.
6. On rename, grep `.astro` files and any other translation files that re-export.
