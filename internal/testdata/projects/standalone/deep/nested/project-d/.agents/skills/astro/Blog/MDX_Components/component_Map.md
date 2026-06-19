# Component Map

The component map is the shared registry that lets blog authors use helpers in MDX without importing them in every file. The post route renders the content entry, then passes `mdx_Components` into Astro's `<Content components={...} />`.

The value is author velocity: no duplicated imports, no broken authoring when a component moves, and one place to audit what helpers the blog supports. The cost is one indirection, which is worth it for long-form content.

## Map Pattern

```ts
// src/Components/Mdx/mdx_Components.ts
import Custom_Header_Lvl_1 from "@components/Mdx/Default/Headers/Header_Lvl_1.astro";
import Custom_Header_Lvl_2 from "@components/Mdx/Default/Headers/Header_Lvl_2.astro";
import Custom_Header_Lvl_3 from "@components/Mdx/Default/Headers/Header_Lvl_3.astro";

import Table_Wrapper from "@components/Mdx/Default/Blocks/Table_Wrapper.astro";
import Inline_Link  from "@components/Mdx/Default/Inline/Inline_Link.astro";
import Fixed_Lang_Link from "@components/Mdx/Default/Inline/Fixed_Lang_Link.astro";
import Inline_Code  from "@components/Mdx/Default/Inline/Inline_Code.astro";
import Cite         from "@components/Mdx/Default/Inline/Cite.astro";
import Blockquote   from "@components/Mdx/Default/Blocks/Blockquote.astro";
import Post_Img     from "@components/Mdx/Default/Media/Post_Img.astro";
import E_Link_Card  from "@components/Mdx/Default/Cards/E_Link_Card.astro";
import I_Link_Card  from "@components/Mdx/Default/Cards/I_Link_Card.astro";
import Definitions  from "@components/Mdx/Default/Lists/Definitions/Definitions.astro";
import Def_Item     from "@components/Mdx/Default/Lists/Definitions/Def_Item.astro";
import Ref_List     from "@components/Mdx/Default/Lists/Ref_List/Ref_List.astro";
import AI_Disclosure from "@components/Mdx/Default/Notifications/AI_Disclosure.astro";
import { Code } from "astro-expressive-code/components";

export const mdx_Components = {
  h1:    Custom_Header_Lvl_1,
  h2:    Custom_Header_Lvl_2,
  h3:    Custom_Header_Lvl_3,
  table: Table_Wrapper,
  a:     Inline_Link,
  code:        Inline_Code,
  blockquote:  Blockquote,

  Post_Img,
  Fixed_Lang_Link,
  E_Link_Card,
  I_Link_Card,
  Cite,
  Definitions,
  Def_Item,
  Ref_List,
  AI_Disclosure,
  Code,
};
```

The page template passes the map down:

```astro
---
import { mdx_Components } from "@components/Mdx/mdx_Components";
const { Content } = await entry.render();
---
<Post_Layout>
  <Content components={mdx_Components} slot="main_text" />
</Post_Layout>
```

## Key Shapes

Lowercase keys override markdown-produced HTML elements. `a`, `table`, `code`, `blockquote`, and headings apply automatically wherever markdown emits those tags.

PascalCase keys are explicit authoring components. Authors write `<Post_Img />`, `<E_Link_Card />`, or `<Definitions>` only when the content calls for that helper.

Keep component names and map keys identical. `Post_Img` should be registered as `Post_Img`, not aliased to a different tag name.

## Per-Category Extras

When one post category needs components the others do not, import them in that route and spread them over the shared map:

```astro
---
import { mdx_Components } from "@components/Mdx/mdx_Components";
import Yus  from "@components/Mdx/Blog_Series/Python_Series/Yus.astro";
import Alex from "@components/Mdx/Blog_Series/Python_Series/Alex.astro";
import Jav  from "@components/Mdx/Blog_Series/Python_Series/Jav.astro";

const components = { ...mdx_Components, Yus, Alex, Jav };
---
<Content components={components} slot="main_text" />
```

The universal map stays conceptually clean. Astro tree-shakes unused registered components per page, so the issue is not runtime cost; the issue is whether the map implies a helper is universal when it is category-specific.

## Add a Universal Component

1. Build the component in `src/Components/Mdx/<Subfolder>/<Name>.astro`.
2. Add the import and map entry in `src/Components/Mdx/mdx_Components.ts`.
3. Validate the prop shape with TypeScript types on the component's `Props`.
4. Document the component in this folder or one of its leaf docs.
5. Keep data wiring in the route/layout when a component needs frontmatter. The map itself stays a pure registry.

## Add a Category-Specific Component

1. Build the component.
2. Do not edit `mdx_Components.ts`.
3. Import it in the category route file and spread it over the shared map.
4. Pass the resulting `components` object to `<Content />`.
5. Document it near the route or in the relevant category docs.

## Anti-Patterns

- Inline imports at the top of every `.mdx` file for universal components.
- Map keys that do not match component names.
- HTML-element overrides for elements markdown never emits.
- One mega-component exposing many unrelated helpers through props.
- DOM access at component top level. MDX components render during SSR, so browser-bound behavior belongs in a client-side `<script>` or `Browser_Client/` file.
