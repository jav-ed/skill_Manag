# Icons Usage

## Import aliases

Defined in `tsconfig.json`:

| Alias | Resolves to | Library |
|---|---|---|
| `@lucid/*` | `node_modules/lucide-static/icons/*` | Lucide (ISC/MIT) |
| `@heroicons/*` | `node_modules/heroicons/*` | Heroicons (MIT) |
| `@phos_T/*` | `node_modules/@phosphor-icons/core/assets/thin/*` | Phosphor Thin |
| `@phos_L/*` | `node_modules/@phosphor-icons/core/assets/light/*` | Phosphor Light |
| `@phos_R/*` | `node_modules/@phosphor-icons/core/assets/regular/*` | Phosphor Regular |
| `@phos_B/*` | `node_modules/@phosphor-icons/core/assets/bold/*` | Phosphor Bold |

Lucide is the primary library. Phosphor and Heroicons are available for icons Lucide does not cover (brand logos, weight variants).

## Import and render

```astro
---
import Lucid_Sun from "@lucid/sun.svg"
import Lucid_Moon from "@lucid/moon.svg"
import Phos_Whatsapp from "@phos_R/whatsapp-logo.svg"
---

<Lucid_Sun class="size-5" />
<Lucid_Moon class="size-5 hidden group-hover:block" />
<Phos_Whatsapp class="size-6 text-secondary" />
```

## Naming convention

Follows the project's `camel_Case_With_Underline` convention:

```
Lucid_<Icon_Name>     — e.g. Lucid_Sun, Lucid_Chevrons_Down, Lucid_File_Text
Phos_<Icon_Name>      — e.g. Phos_Whatsapp
Hero_<Icon_Name>      — e.g. Hero_Arrow_Right
```

The icon file name from the library maps directly: `chevrons-down.svg` becomes `Lucid_Chevrons_Down`.

## Sizing

astro-icon used a `size={N}` prop. Direct SVG imports use Tailwind's `size-*` utility instead:

| Pixel value | Tailwind class |
|---|---|
| 14px | `size-3.5` |
| 16px | `size-4` |
| 20px | `size-5` |
| 24px | `size-6` |
| 28px | `size-7` |
| 32px | `size-8` |
| 36px | `size-9` |
| 40px | `size-10` |

For non-standard sizes, use arbitrary values: `size-[23px]`, `size-[42px]`.

## Color

SVGs from Lucide/Phosphor use `currentColor`. Apply color via Tailwind text utilities:

```html
<Lucid_Sun class="size-5 text-primary" />
```

For dynamic DaisyUI colors, use inline style:

```html
<Lucid_Sun class="size-6" style={`color: var(--color-${text_color})`} />
```

## Dynamic icons from content collections

MDX frontmatter can only hold strings, not imported components. CV content uses icon strings like `"9_About/4_Combi/1_Code"` in frontmatter fields.

These are resolved at render time via an **icon registry**:

```
src/Scripts/Astro_Frontmatter/Icon_Registry/icon_Registry.ts
```

The registry maps every icon string to an imported SVG component. Components use `resolve_Icon()` to look up the component:

```astro
---
import { resolve_Icon } from "@code/Astro_Frontmatter/Icon_Registry/icon_Registry"
---

{items.map((item) => {
  const Item_Icon = resolve_Icon(item.icon)
  return <Item_Icon class="size-6" />
})}
```

`resolve_Icon()` throws if the icon string is not found — no silent fallbacks. When adding a new icon string to MDX content, add a corresponding entry in the registry.

Brand/custom SVGs (Java, Python, Matlab, Android, LinkedIn, Twitter, Mastodon, YouTube, Rumble, Nostr) are imported from `src/icons/` since they have no Lucide equivalent.
