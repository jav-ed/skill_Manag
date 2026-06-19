# Content vs Config vs Labels — where does a value live?

Before reaching for content collections, answer the prior question: **does this even belong in a collection?** The astro stack has three homes for "stuff with text in it" and the choice is not always crisp. This page gives the rule, then admits the gray-zone cases where the rule resolves to "either is fine."

## The three homes

| Format | Best for | Examples |
|---|---|---|
| **MDX (frontmatter + body)** in `src/Content/<Collection>/` | **Page-bound content** — tied 1:1 to a URL | Blog post, glossary term, team member, service detail, landing-page hero copy |
| **JS/TS** in `src/Data/` | **Site-wide configuration the build uses** | Supported languages, site domain, font config, dev flags, OG template/output/asset contracts |
| **JS/TS** in `src/Scripts/Multi_Lang_Txts/` | **Repeated UI labels** across multiple pages/components | "Read more", form labels, aria text, error messages, footer link copy, breadcrumb labels |

That rule covers about 90% of decisions. The other 10% is honest gray zone — see below.

## The deciding question

When you're not sure, ask: **"Where is this value's natural home?"**

- *"On this page, and nowhere else"* → MDX of that page
- *"Many pages need it"* → `Multi_Lang_Txts/` (if it's a string) or `Data/` (if it's structured config)
- *"The build needs it to assemble things"* → `Data/`

## Clear cases — no ambiguity

| Thing | Home | Why |
|---|---|---|
| Blog post body + frontmatter | MDX | Long-form prose, one entity per URL |
| Glossary term + definition | MDX | Body + fields, per-URL |
| Team member bio + role + photo | MDX | Body + fields, per-URL — even when the body is short |
| Service detail page | MDX | Body + fields, per-URL |
| Landing-page hero copy and sections | MDX | Page-bound, one entity per language |
| Per-page `<title>` / `<meta description>` / `og:image` | MDX frontmatter (`seo`) | Per-page |
| Site name, domain, supported languages list | `Data/Common` | Site-wide config |
| Font config | `Data/Common` | Build-side font contract |
| Theme tokens | `Styles/Base/theme.css` | Style contract, not content |
| OG template catalogue, output paths, reusable OG assets, OG theme contract | `Data/Og_Img` | Build-side renderer contract |
| Dev feature flags | `Data/Dev/` | Site-wide config (audit-fenced) |
| "Read more", "Send", form button labels | `Multi_Lang_Txts/` | UI chrome, repeated |
| Aria text, error messages | `Multi_Lang_Txts/` | UI chrome, repeated |
| Footer column link labels | `Multi_Lang_Txts/` | UI chrome, repeated |
| Header nav item labels | `Multi_Lang_Txts/` | UI chrome, repeated |
| Structural route labels (breadcrumb "Blog", "Team") | `Multi_Lang_Txts/Shared/route_Txt.ts` | UI chrome, route-naming |

## The honest gray zone

Some values resolve to "either is fine, pick one." That's not a flaw in the rule — it's real ambiguity in the data itself. Three recurring examples:

### Opening hours

Appears on: kontakt page, landing page hero, footer.

- **As `Data/Common/site_Hours.ts`**: structured array, imported by all three consumers. Easy to update in one place.
- **As MDX frontmatter in `001_Kontakt`**: lives next to the kontakt prose. Landing and footer import via `getEntry('kontakt', …)`.

Either is defensible. Picking the first wins if you anticipate the schedule being re-formatted or computed (timezones, holiday overrides). The second wins if the kontakt page already owns most of the surrounding content and you don't want a structural file just for hours.

### Address block

- **As `Multi_Lang_Txts/Shared/address_Txt.ts`**: it's text shown to users, varies by language (street-name conventions, comma placement).
- **As `Data/Common/site_Address.ts`**: it's part of the site identity, and the same data feeds JSON-LD LocalBusiness schema.

Either is defensible. The first wins if you treat the address as content. The second wins if you also feed schema.org markup from the same source — splitting the address into "the words shown" and "the structured data" creates two sources of truth.

### Per-page hero phrase that happens to repeat words

The string "Praxis Mitte Berlin" appears in four page heroes.

- **Duplicate the string in four MDX files**: cheap, each page is self-contained.
- **Extract to `Multi_Lang_Txts/Shared/practice_Name_Txt.ts`** and reference it: DRY, single source.

This one's a real coin-flip. See the tiebreaker below.

## Tiebreaker

When you genuinely can't decide:

**Default to MDX-per-page.** Duplicating a few words across MDX files is cheap. Fragmenting a page's content into a `Multi_Lang_Txts` key is expensive — anyone reading the page has to chase two files to understand what it renders.

The exception: if the string is part of the **site identity** (name, address, contact email) and a change there must propagate everywhere on the same day, `Data/Common` wins. You don't want to grep MDX for "info@example.com" when the address changes.

## "Should I find it later?" lens

If a future you (or a colleague) reads a page in the browser and wonders *"where is this string defined?"*, the answer should be predictable:

1. Is it a heading, paragraph, or per-page title? → `src/Content/<collection>/<lang>/<entry>.mdx`
2. Is it a button, a label, an error message, or repeated chrome? → `src/Scripts/Multi_Lang_Txts/<area>/...`
3. Is it the site name, a config flag, or a list of supported languages? → `src/Data/`

If that mental model is trained, the per-case rule mostly disappears — the answer pops out.

## Worked example — Partner

Partner has 14 collections after the May 2026 migration. Gut-check applications of the rule:

| Source | Home | Note |
|---|---|---|
| `500_Blogs` (posts) | MDX | Clear case — long-form prose |
| `010_Services`, `020_Team`, `030_Glossary` | MDX | Clear case — body + fields, per-URL |
| `040_Static_Pages` (first-visit, practice-philosophy) | MDX | Clear case — editorial body |
| `000_Landing_Page`, `001_Kontakt`, `002_Praxistour` | MDX | Clear case — page-bound; structural fields land here as .astro pages refactor |
| `003_Blog_List`, `004_Team_List`, `005_Glossary_List` | MDX | Clear case — list-page metadata, per-URL |
| `langs_Config`, `site_Domain` | `Data/Common` | Clear case — site infrastructure |
| `font_Roles` | `Data/Common` | Clear case — build-side font contract |
| `og_Theme_Contract`, `og_Template_Names`, `collection_Template_Defaults`, `og_Assets`, `og_Img_Format` | `Data/Og_Img` | Clear case — OG image renderer contract |
| `t_Route_Blog`, footer link labels, form button labels | `Multi_Lang_Txts/` | Clear case — repeated UI strings |
| **Opening hours** | currently MDX (kontakt frontmatter) | **Gray zone** — would move to `Data/Common` if it grows JSON-LD consumers |
| **Practice address** | currently `Multi_Lang_Txts/Shared/address_Txt.ts` | **Gray zone** — would move to `Data/Common` if it grows JSON-LD consumers |
| **Practice name** ("Praxis Mitte Berlin") | currently `Data/Common/mandatory_Inp.ts` as `site_Name` | Clear, by tiebreaker — site identity wins over per-page duplication |

The gray-zone rows are marked explicitly. That honesty is part of the rule: not all questions have crisp answers, and pretending otherwise misleads the next reader who hits an ambiguous case and concludes the doc is wrong.

## Related

- [Content overview](./linker_Content.md) — content-collection routing entry point.
- [Multi_Lang_Txts](../Scripts/multi_Lang_Txts.md) — what belongs in translation tables.
- [Data overview](../Data/linker_Data.md) — what belongs in `src/Data/`.
- [Utils overview](../Utils/linker_Utils.md) — the other site-wide-imports folder; the Data-vs-Utils split is documented there.
