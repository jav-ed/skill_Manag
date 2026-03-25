# Content Collections Structure

## Folder tree

```
src/Content/
├── 000_Landing_Page/
├── 101_Coding_Overview/
├── 102_Justice_Overview/
├── 103_Sadaqa_Overview/
├── 200_Python_Overview/
├── 201_Web_Overview/
├── 202_Palestine_Overview/
├── 500_Python_Series/
├── 501_Web_Entries/
├── 502_Palestine_Entries/
├── 900_CV/
├── 990_Legal_Notice/
└── 991_Privacy_Policy/
```

## Numbering convention

The `NNN_` prefix encodes hierarchy level and role:

| Range | Role | Examples |
|---|---|---|
| `000` | Landing page | `000_Landing_Page` |
| `1XX` | Level-1 category overviews | `101_Coding_Overview`, `102_Justice_Overview` |
| `2XX` | Level-2 topic overviews | `200_Python_Overview`, `201_Web_Overview` |
| `5XX` | Blog entry series | `500_Python_Series`, `501_Web_Entries` |
| `9XX` | Person / legal pages | `900_CV`, `990_Legal_Notice`, `991_Privacy_Policy` |

Numbers within a range are sequential — the next coding blog series would be `503_`.

## Language structure

Every collection uses the same two-folder split:

```
101_Coding_Overview/
├── en/                    ← primary language
│   └── 0_coding.mdx
└── translation/           ← all other languages
    ├── de/
    │   └── 0_coding.mdx
    ├── es/
    │   └── 0_coding.mdx
    ├── fr/
    │   └── 0_coding.mdx
    └── zh/
        └── 0_coding.mdx
```

Supported languages: `en` (primary), `de`, `es`, `fr`, `zh`.

The `translation/` folder prefix is stripped during routing so URLs always reflect `/{lang}/...` not `/translation/{lang}/...`. See [Routing doc](content_Collections_Routing.md).

## File naming

| Collection type | Pattern | Example |
|---|---|---|
| Overview / single-file | `0_{topic}.mdx` | `0_coding.mdx`, `0_python.mdx` |
| Blog entry | `XXXX_{Title}.mdx` | `0001_Coding_Term.mdx` |
| Landing page | `index.mdx` | `index.mdx` |
| CV | `about.mdx` | `about.mdx` |

The leading `XXXX_` number on blog entries controls display order and is stripped from the URL during slug generation.

## Subdirectories inside collections

Some collections contain non-MDX assets alongside their content files:

| Collection | Subdirectory | Contents |
|---|---|---|
| `500_Python_Series` | `en/Code/` | Raw `.py` / `.ts` files imported into MDX via `?raw` |
| `500_Python_Series` | `en/0_Video_Links/` | Astro components for video embeds |
| `502_Palestine_Entries` | `en/1_Plotly/` | Visualization assets |

Code files are imported in MDX as:
```mdx
import importedCode from "@content/500_Python_Series/en/Code/0004_init.py?raw";
```
