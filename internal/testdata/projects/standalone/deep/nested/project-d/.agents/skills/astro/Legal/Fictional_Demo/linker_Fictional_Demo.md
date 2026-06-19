# Fictional Demo Legal

Reference set for fictional customer/practice legal pages shown inside public Astro demos. The fictional layer can contain sample legal pages, but it must not be the only legal identity available on a public demo.

Use this folder before editing:

- `src/Content/990_Legal_Notice/{de,en,es}/legal-notice.mdx`
- `src/Content/991_Privacy_Policy/{de,en,es}/privacy-policy.mdx`
- `src/Content/992_Accessibility/{de,en,es}/accessibility.mdx`
- sample legal-page labels in the footer
- MDX notice components that identify a page as a fictional sample
- fictional safe contact/practice data in site config

For the actual demo operator's legal pages and persistent real-operator links, use [Real Operator Legal](../Real_Operator/linker_Real_Operator.md).

## Common implementation rule

Sample pages must be visibly labelled as demo/fictitious/sample pages. They can show realistic jurisdiction-specific data, but they must state why those details appear and that they would need to be checked and replaced for a real customer.

## Sample legal page labels

Use visible names such as:

- `Muster-Impressum`
- `Muster-Datenschutz`
- `Freiwillige Hinweise zur Barrierefreiheit`

Avoid labels that read as the operative legal page for the actual demo operator.

## References

- [Dentist sample pages](dentist_Sample_Pages.md): demo-specific content rules for fictional dentist pages.
- [§5 DDG Impressum checklist](ddg_Impressum_Checklist.md): field-by-field source checklist from the official DDG text, with sample coverage and conditional fields to keep explicit.
- [Source register](source_Register.md): source list with short descriptions of what each legal/reference source contributes.
- [Implementation footguns](implementation_Footguns.md): recurring mistakes to check before editing public legal/demo pages.
