# JSON-LD

Structured data (schema.org markup embedded as JSON-LD) tells search engines and other consumers what each page *is* — a blog post, a person's profile, a local business, a list of FAQs. It powers rich results (rating stars, breadcrumb trails, FAQ accordions, knowledge-graph cards) and is the single highest-leverage SEO investment after the basic head-tag block.

This doc covers the org's JSON-LD pattern: one builder per schema.org type, a shared identity builder reused across types, a dispatcher that picks the default schema set from a single Layout prop, and an `extra_Schemas` escape hatch for per-page additions.

## Where it lives

```
src/Scripts/Astro_Frontmatter/Common/JsonLd/
├── jsonLd_Main.js          # dispatcher: og_Type → default schema set
├── jsonLd_Person.js        # shared identity (or jsonLd_Organization.js for orgs)
├── jsonLd_Article.js       # blog posts
├── jsonLd_Profile.js       # about / CV / team-member pages
├── jsonLd_Website.js       # landing page
├── jsonLd_Breadcrumb.js    # URL-derived BreadcrumbList
├── jsonLd_Faq.js           # FAQPage from frontmatter
└── jsonLd_<Domain>.js      # project-specific (LocalBusiness, MedicalProcedure, Recipe, Product, …)
```

Folder is flat. Each file exports one builder function. Builders are pure: input is a plain object of page facts, output is a plain JS object (or `null` when there's no data) that serialises straight to `JSON.stringify`. No `<script>` tags, no string concatenation — the Layout owns rendering.

## The pattern (read this first)

Every builder follows the same shape:

```js
// jsonLd_<Type>.js
export function build_<Type>_JsonLd({ url, title, description, lang, image, /* …type-specific */ }) {
  // Return null when the inputs don't justify emitting the schema.
  // The Layout filters nulls so callers can pass conditionals inline.
  if (/* nothing to say */) return null;

  return {
    "@context": "https://schema.org",
    "@type": "<SchemaOrgType>",
    // ...fields
  };
}
```

The Layout collects all schemas (defaults from the dispatcher + `extra_Schemas` from the page), filters nulls, and emits one `<script type="application/ld+json">` per object:

```astro
{[...jsonLd_Schemas, ...extra_Schemas].filter(Boolean).map((schema) => (
  <script type="application/ld+json" set:html={JSON.stringify(schema)} />
))}
```

`set:html` is required — Astro otherwise HTML-escapes the JSON and crawlers can't parse it.

**Why one builder per type, not one mega-builder.** Each schema.org type has its own required and recommended fields, its own conditional logic (date present → emit `datePublished`; image absent → omit `image`), and its own validation rules in Google's Rich Results Test. Co-locating the rules for one type in one file keeps each builder under 50 lines and the diff for "add `author` to Article" small.

## The dispatcher: `og_Type`

`jsonLd_Main.js` reads the Layout's `og_Type` prop and returns the default schema set for that page kind:

```js
// jsonLd_Main.js
import { build_Website_JsonLd } from "./jsonLd_Website.js";
import { build_Article_JsonLd } from "./jsonLd_Article.js";
import { build_Profile_JsonLd } from "./jsonLd_Profile.js";
import { build_Breadcrumb_JsonLd } from "./jsonLd_Breadcrumb.js";

export function build_JsonLd({ og_Type, mdx_Info, lang, url, og_Img_Url }) {
  const shared = {
    url,
    title:       mdx_Info.seo.title,
    description: mdx_Info.seo.description,
    lang:        bcp_From_Url[lang] ?? lang,
    image:       og_Img_Url,
  };

  let schemas;
  switch (og_Type) {
    case "article":
      schemas = build_Article_JsonLd({ ...shared, date: mdx_Info.date, categories: mdx_Info.categories });
      break;
    case "profile":
      schemas = build_Profile_JsonLd(shared);
      break;
    case "website":
    default:
      schemas = build_Website_JsonLd(shared);
      break;
  }

  const breadcrumb = build_Breadcrumb_JsonLd({ url, lang, title: shared.title });
  if (breadcrumb) schemas.push(breadcrumb);

  return schemas;
}
```

The Layout calls `build_JsonLd({...})` once. One prop on the page (`og_Type="article"`) drives the whole default schema set. Pages that need additional schemas pass them via `extra_Schemas`.

**What goes in the dispatcher vs `extra_Schemas`:**

- **Dispatcher.** Schemas that *every* page of this type should emit — Article on every blog post, ProfilePage on every about/CV page. If the schema can be built from the props the Layout already gets, it belongs in the dispatcher.
- **`extra_Schemas`.** Schemas that depend on per-page data (FAQ items from MDX frontmatter, MedicalProcedure fields from a service collection entry) or appear on only some pages (LocalBusiness on the homepage but not other website pages). Builds in the page's frontmatter, where the data is in scope.

## The shared identity builder

The single most important piece. Every project has one entity that owns the site (a person, an organisation). Build it *once* in `jsonLd_Person.js` or `jsonLd_Organization.js` and import from every other builder that needs to reference it:

```js
// jsonLd_Person.js
import { t_Job_Title } from "@code/Multi_Lang_Txts/Shared/jsonLd_Txt.js";

export function build_Person(lang) {
  const short = lang?.split("-")[0] ?? "en";
  return {
    "@type": "Person",
    name: "Full Name",
    url: "https://example.com",
    jobTitle: t_Job_Title[short] ?? t_Job_Title["en"],
    sameAs: ["https://github.com/...", "https://www.linkedin.com/in/..."],
  };
}
```

```js
// jsonLd_Article.js
import { build_Person } from "./jsonLd_Person.js";

export function build_Article_JsonLd({ url, title, description, lang, image, date, categories }) {
  const person = build_Person(lang);
  return [{
    "@context": "https://schema.org",
    "@type":    "Article",
    headline:   title,
    author:     person,
    publisher:  person,
    // …
  }];
}
```

If the author identity changes (new role, new social profile, name spelling fix), you change one file — every Article, Profile, Website, ProfilePage, etc. picks it up. Without this discipline, identity facts drift across the JSON-LD corpus and Google's knowledge graph merges duplicates incorrectly.

**Translated fields.** `jobTitle`, `description`, `knowsAbout` — anything the identity exposes in human language — should be a translation table keyed by short lang code, not a hardcoded string. Google requires structured data to *reflect the visible page content*; a German page can't say `jobTitle: "Engineer"` while showing "Ingenieur" in the visible markup.

## Catalogue: which schema for which page

| Page kind | `og_Type` | Dispatcher emits | Page may inject via `extra_Schemas` |
|---|---|---|---|
| Landing page | `website` | `WebSite` + identity | `LocalBusiness` (single-location business), `Organization` (multi-location org), `FAQPage` (homepage FAQ section) |
| Blog post | `article` | `Article` + `BreadcrumbList` | `FAQPage` (when post declares `faq.items` in frontmatter), `HowTo` (step-by-step posts) |
| About / CV / team member | `profile` | `ProfilePage` + identity | `FAQPage`, `Course` / `EducationalOccupationalCredential` (CV-style detail) |
| Service / product detail | `website` | `WebSite` + identity | `MedicalProcedure` / `Service` / `Product` (per-page domain schema), `FAQPage`, `Offer` |
| Overview / category page | `website` | `WebSite` + identity | `CollectionPage`, `ItemList` of the collection's entries |
| Legal (imprint, privacy) | `website` | `WebSite` + identity | usually nothing |
| Contact | `website` | `WebSite` + identity | `LocalBusiness` (if separate from homepage), `ContactPoint` |

Don't pre-build schemas for page kinds the project doesn't have. The right list of schemas is "the minimum set Google's Rich Results Test endorses for this page's content type".

## Schema-type reference

### `WebSite` — landing page

The landing dispatcher emits `WebSite` plus the identity entity side-by-side (one schema each, not nested). Tells search engines "this is the site, and this is who runs it":

```js
export function build_Website_JsonLd({ url, title, description, lang, image }) {
  const person = build_Person(lang);
  return [
    {
      "@context": "https://schema.org",
      "@type": "WebSite",
      name: "Site Name",
      url,
      description,
      inLanguage: lang,
      author: person,
    },
    {
      "@context": "https://schema.org",
      ...person,
      description,
      ...(image && { image }),
    },
  ];
}
```

For sites with internal search, add `potentialAction: { "@type": "SearchAction", target: "https://example.com/search?q={search_term_string}", "query-input": "required name=search_term_string" }` to the `WebSite` block — Google may render a search box directly in the SERP.

### `Article` — blog post

```js
export function build_Article_JsonLd({ url, title, description, lang, image, date, categories }) {
  const person = build_Person(lang);
  const iso_Date = to_Iso_Date(date);  // DD.MM.YYYY → YYYY-MM-DD

  return [{
    "@context": "https://schema.org",
    "@type": "Article",
    headline: title,
    description,
    url,
    inLanguage: lang,
    author: person,
    publisher: person,
    mainEntityOfPage: { "@type": "WebPage", "@id": url },
    ...(iso_Date && { datePublished: iso_Date }),
    ...(image && {
      image: { "@type": "ImageObject", url: image, width: 1200, height: 630, caption: title },
    }),
    ...(categories?.length && { articleSection: categories.join(", ") }),
  }];
}
```

Notes:

- **Date format.** schema.org expects ISO 8601 (`YYYY-MM-DD` or full datetime). If the project stores dates in any other format in frontmatter, convert at the boundary in the builder, not at the data source — the source format is a content concern, not a schema concern.
- **`image` as `ImageObject`.** Google rewards explicit `width`/`height` in the structured data. A plain URL works but a fully-typed `ImageObject` is the recommended form.
- **`headline` ≤ 110 chars.** Google clips longer headlines in rich results.
- **`author` vs `publisher`.** For solo sites both point to the same `Person`. For team sites, `author` is the post writer (`Person`), `publisher` is the parent `Organization`.
- **`dateModified`.** Add when the project tracks update dates. Google ranks freshness, so this matters for evergreen posts.

### `ProfilePage` — about / CV / team-member pages

```js
export function build_Profile_JsonLd({ url, title, description, lang, image }) {
  const person = build_Person(lang);
  const short = lang?.split("-")[0] ?? "en";
  return [{
    "@context": "https://schema.org",
    "@type": "ProfilePage",
    name: title,
    description,
    url,
    inLanguage: lang,
    mainEntity: {
      ...person,
      description,
      knowsAbout: t_Knows_About[short] ?? t_Knows_About["en"],
      ...(image && { image }),
    },
  }];
}
```

`ProfilePage` wraps the `Person` (or `Organization`) as `mainEntity`. `knowsAbout` is an array of strings or `Thing` references — keep it short and accurate; padding it with everything-and-the-kitchen-sink hurts more than helps.

### `BreadcrumbList` — every non-flat page

Derived from `Astro.url.pathname` — don't hand-author it per page. Builds on the shared `breadcrumb_Chain.ts` utility in `Common/Routing/` (single source of truth for visible breadcrumbs, single back-link components, and this JSON-LD output). The builder calls `build_Breadcrumb_Chain({ pathname, lang, leaf_Title: title })`, then maps the `{ label, href? }[]` output to `ListItem` shape — so visible chrome and structured data can never drift apart, and a new section registered in `route_Slugs.ts` flows everywhere. Output shape:

```js
{
  "@context": "https://schema.org",
  "@type": "BreadcrumbList",
  itemListElement: [
    { "@type": "ListItem", position: 1, name: "Home",       item: "https://example.com/" },
    { "@type": "ListItem", position: 2, name: "Blog",       item: "https://example.com/blog/" },
    { "@type": "ListItem", position: 3, name: "Post Title", item: "https://example.com/blog/post-slug/" },
  ],
}
```

Return `null` for pages where breadcrumbs don't make sense (landing page, single-segment flat pages).

### `FAQPage` — frontmatter-driven

```js
export function build_Faq_JsonLd(faq_Items) {
  if (!faq_Items?.length) return null;
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faq_Items.map(({ q, a }) => ({
      "@type": "Question",
      name: q,
      acceptedAnswer: { "@type": "Answer", text: a },
    })),
  };
}
```

Pages with an `faq.items` block in frontmatter call this in their layout's frontmatter and pass the result via `extra_Schemas`:

```astro
const faq_Schema    = build_Faq_JsonLd(frontmatter.faq?.items);
const extra_Schemas = faq_Schema ? [faq_Schema] : [];
```

The null-return + `Boolean` filter convention means callers don't have to guard the array shape.

**Critical rule:** the FAQ items in the JSON-LD must also appear visibly in the rendered page. Google strips structured-data FAQs that aren't backed by visible content (and may issue a manual penalty for repeated violations). Partner's pattern: `Post_Layout.astro` reads `frontmatter.faq?.items` and builds the JSON-LD; the author writes the matching visible FAQ section directly in the MDX body (headings + paragraphs, or `<Definitions>` blocks) so the visible content mirrors the frontmatter items. There is no `<FAQ>` MDX component that does both — visible-vs-structured alignment is the author's responsibility.

### Domain schemas — `LocalBusiness`, `MedicalProcedure`, `Product`, `Recipe`, etc.

When the project's content has a domain shape Google recognises, add the matching schema type. Examples:

- **Dentist / dental practice** → `Dentist` (subtype of `LocalBusiness`) on the homepage, `MedicalProcedure` on each service detail page.
- **Restaurant** → `Restaurant` on the homepage, `Menu` / `MenuItem` for menus.
- **E-commerce** → `Product` per product, `Offer` with price.
- **Recipe blog** → `Recipe` per recipe post, with `nutrition`, `cookTime`, `recipeIngredient`.

Same builder pattern — one file per type, pure function returning a plain object, importing the shared identity where it makes sense (e.g. `LocalBusiness.founder` may point to the shared `Person`).

```js
// jsonLd_Local_Business.js — sketched, project-specific
export function build_Local_Business_JsonLd() {
  return {
    "@context": "https://schema.org",
    "@type": "Dentist",
    name: site.practiceName,
    url:   new URL(`${langs_Config.main_Lang}/`, site_Domain).toString(),
    image: new URL("/og-default.jpg", site_Domain).toString(),
    telephone: site.phoneHref,
    address: {
      "@type": "PostalAddress",
      streetAddress:   site.address.street,
      postalCode:      site.address.plz,
      addressLocality: site.address.city,
      addressCountry:  "DE",
    },
    geo: { "@type": "GeoCoordinates", latitude: site.coordinates.lat, longitude: site.coordinates.lng },
    openingHoursSpecification: /* expanded from a human-friendly hours table */,
    areaServed: { "@type": "City", name: site.address.city },
  };
}
```

Source the data from the project's site-config facade (Partner: `src/Data/Site_Config/site_Config.ts`) or shared infrastructure data so the JSON-LD and the visible footer/contact page stay in sync.

## Validation

- **Google Rich Results Test** (`https://search.google.com/test/rich-results`) — paste the page URL, get warnings/errors per schema. Run it after every JSON-LD change.
- **Schema.org Validator** (`https://validator.schema.org/`) — second opinion; catches stricter spec violations Google may ignore.
- **`bun run build && bun run preview`** then view-source on the rendered page — confirms the `<script type="application/ld+json">` block actually emits and the JSON parses.

Common warnings and what they mean:

- "Missing field `image`" — Google wants ImageObject with width/height. Add it.
- "Date is not a valid ISO 8601" — frontmatter date isn't being normalised. Fix the builder.
- "Author missing `url`" — identity builder skipped a field. Fix `build_Person` once; every consumer benefits.
- "Multiple `@type` for same item" — usually a bug; one entity per schema.

## Add a new schema type — checklist

1. Identify the schema.org type (`https://schema.org/<Type>`). Read the required + recommended properties.
2. Run a representative page through Google's Rich Results Test *before* writing anything — confirms Google supports this type for rich results and tells you exactly which properties it wants.
3. Create `jsonLd_<Type>.js`. Pure function, named `build_<Type>_JsonLd`, returning a plain object or `null`.
4. Decide: does *every* page of some kind emit this schema, or only some?
   - **Every page of a kind** → add to the dispatcher (`jsonLd_Main.js`) under a new or existing `og_Type` branch.
   - **Per-page conditional** → leave out of the dispatcher; build in page frontmatter and pass via `extra_Schemas`.
5. If the new schema references identity (author, founder, owner), import `build_Person` / `build_Organization` rather than inlining.
6. If the schema renders human-readable strings (`jobTitle`, `description`, `name` for category labels), pull them from `Multi_Lang_Txts/` keyed by lang.
7. Validate with the Rich Results Test against a real built page.
8. If the schema needs frontmatter data, extend the relevant `schema_<Collection>.ts` to capture that data (see [`../Scripts/content_Schemas.md`](../Scripts/content_Schemas.md)).

## Anti-patterns

- **JSON-LD that contradicts the visible page.** Schema says `priceCurrency: "EUR"`, page says `$`. Google flags this as deceptive and removes rich results.
- **Hand-authoring JSON-LD in MDX or layouts.** Lose the type safety, lose the shared identity, drift between pages.
- **Inlining identity in every builder.** Pulls 6 files out of sync the moment you change a social-media URL. Always extract to one shared builder.
- **Emitting `null` or `undefined` into the JSON-LD object.** `JSON.stringify` keeps `null`, drops `undefined`. Be intentional: use spread-conditional `...(value && { key: value })` for "omit if missing", or explicit `null` only when the spec says null is meaningful (rare).
- **One mega-script with multiple top-level schemas joined into an array.** Most crawlers accept it, but it's harder to debug — one bad schema breaks the whole block. Emit one `<script>` per object.
- **Using `application/json` instead of `application/ld+json`.** Wrong MIME, crawlers skip it. Always `application/ld+json`.
- **Hardcoding the production URL inside builders.** Use `Astro.url`, the `site:` config, or `src/Data/Common/site_Domain`. Hardcoded URLs make local dev advertise production URLs.
