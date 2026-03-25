# Adding an OG Image to a Page

Add `image` inside the `seo.og` block in the MDX frontmatter.

## Frontmatter

```yaml
mdx_Info:
  seo:
    meta_Title: "Page title"
    meta_Content: "Page description"
    og:
      title: "Social card title"
      description: "Social card description"
      image:
        variant: "img_Right"
        img_Rel_Path: "path/to/image.webp"   # relative to src/Assets/1_Images/
        theme_Name: "theme_Jav_0"            # optional — defaults to theme_Jav_0
```

## Fields

| Field | Required | Notes |
|---|---|---|
| `variant` | yes | Only `"img_Right"` is supported today |
| `img_Rel_Path` | yes | Path relative to `src/Assets/1_Images/` |
| `theme_Name` | no | `"theme_Jav_0"` or `"theme_Jav_1"` — omit to use generator default |

## What happens

The manifest worker picks up the entry, creates a render task, and the generator produces `dist/Og_Gen/<derived_name>.webp`. The layout references it automatically via `<meta property="og:image">`.

Pages without `seo.og.image` are skipped — no OG image tag is emitted for them.
