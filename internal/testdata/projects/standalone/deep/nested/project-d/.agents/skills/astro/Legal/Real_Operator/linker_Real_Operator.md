# Real Operator Legal

Reference set for the actual operator of a public Astro website or demo. In a public demo, this is the agency/company/person publishing the page, not the fictional practice shown inside the preview.

Use this folder before editing:

- real operator Impressum links
- real operator Datenschutz links
- footer/operator disclosure copy
- shared operator-link config in `src/Data/Common/`
- repeated legal UI labels in `src/Scripts/Multi_Lang_Txts/Legal/`

## Core rule

Never let fictional practice data be the only legal identity on a publicly reachable demo. A demo may show sample legal pages for a fictional practice, but every public page still needs a route to the real operator's Impressum and privacy policy.

Safe structure:

```text
operator.example/impressum
operator.example/datenschutz

operator.example/demo/zahnarzt-tier-3/
operator.example/demo/zahnarzt-tier-3/muster-impressum/
operator.example/demo/zahnarzt-tier-3/muster-datenschutz/
```

If the preview is hosted elsewhere, adapt the URLs but keep the separation:

- real legal pages identify the actual demo operator
- sample legal pages show what a customer site may receive
- demo pages visibly say that practice name, people, address, phone numbers, and content are fictional

## References

- [Required public pages](required_Public_Pages.md): real operator page responsibilities for public demos.
- [Demo operator links](demo_Operator_Links.md): implementation pattern for persistent links to the real operator's Impressum and Datenschutz.
