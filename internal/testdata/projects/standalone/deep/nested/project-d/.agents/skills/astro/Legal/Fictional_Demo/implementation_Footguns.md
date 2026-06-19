# Implementation Footguns

Checklist of mistakes that are easy to repeat when editing fictional demo legal pages.

## Legal/content mistakes

- Do not let a fictional practice be the only legal identity on a public demo. Link to the real operator's Impressum and Datenschutz.
- Do not put KZV data under `Zuständige Berufskammer`. The chamber and KZV are separate institutions.
- Do not only list professional rules. Also say how the rules are accessible.
- Do not guess `gesetze-im-internet.de` slugs. For example, ZHG is `/zhg/`, not `/z_hg/`; verify rendered external URLs before shipping.
- Do not hardcode Berlin for all dentist customers. Real delivery depends on Bundesland, chamber district, practice form, KZV relevance, and concrete offer.
- Do not copy stale dispute-resolution or ODR text without checking current sources.
- Do not use visible `TODO` lines in polished demo pages. Use conditional wording such as `müsste/müssten geprüft und ergänzt werden`.
- Avoid future-promising legal text such as `wir werden`. Use general requirement wording like `müsste/müssten geprüft werden`.
- When a fictional demo keeps professional SEO and schema.org output enabled, the visible demo disclosure and real-operator legal links need to be especially clear.
- Follow the repo's German copy style: avoid paired gendering forms such as `Zahnärztinnen und Zahnärzte`; prefer normal, idiomatic wording such as `für die zahnärztliche Berufsausübung`.

## Astro/MDX mistakes

- Do not wrap Markdown paragraphs inside raw `<p class="...">` blocks. MDX can emit nested paragraphs. Use `<div class="...">` or keep the whole paragraph as explicit inline HTML.
- Do not trust source text around linked sentence fragments. Build and inspect rendered text for punctuation and spaces, especially before semicolons and full stops.
- Do not assume a component-level legal notice is enough. Footer links, sample page labels, and page intro copy must all support the same real-vs-fictional distinction.
