# Fonts Usage

## Tailwind utility classes

```html
<p class="font-platypi">Body text</p>
<p class="font-al_qalam">Quranic text</p>
<code class="font-mono">Code</code>
```

## CSS variables

```css
font-family: var(--font-platypi);
font-family: var(--font-al_qalam);
font-family: var(--font-mono);
```

## Default body font

Platypi is set as the default on `html` in `src/Styles/0_Base/fonts_Base.css`:

```css
@layer base {
  html {
    font-family: var(--font-platypi), var(--font-sans);
  }
}
```

All pages inherit Platypi automatically — no need to add `font-platypi` manually.

## Quranic text

Pass `b_Quran_Font={true}` to the layout to preload Al Qalam on that page:

```astro
<Init_Layout b_Quran_Font={true} ...>
```

Then apply the font where needed:

```html
<span class="font-al_qalam">...</span>
```
