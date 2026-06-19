# §5 DDG Impressum Checklist

Source: [§5 DDG, gesetze-im-internet.de](https://www.gesetze-im-internet.de/ddg/__5.html). Use this as a technical/source checklist when editing a German dentist-preview sample Impressum. DDG is the federal baseline; dentist pages also need state/chamber/practice-form checks. For the broader source map, start with the [source register](source_Register.md). This is not a legal approval for any real practice.

## Current sample coverage

| §5 DDG item | Current sample coverage | Status |
|---|---|---|
| (1) Nr. 1: name and address | `site.practiceName`, `site.address`, and `site.ownerNames` are rendered. | covered for a fictional sole-practice sample |
| (1) Nr. 1: legal form, representative, capital details for legal persons | Not rendered as data fields yet. Closing note says these details must be checked case by case. | conditional / missing for GmbH, MVZ, BAG, etc. |
| (1) Nr. 2: fast electronic contact and direct communication, including e-mail | Phone and e-mail are rendered. | covered |
| (1) Nr. 3: supervisory authority where approval is required | `site.aufsichtsbehoerde` is rendered. | covered |
| (1) Nr. 4: commercial or similar register and register number | No register fields are rendered. | conditional / missing if the practice entity is registered |
| (1) Nr. 5a: professional chamber | `site.zahnaerztekammer` is rendered. KZV data is rendered separately when applicable, not under the chamber heading. | covered |
| (1) Nr. 5b: statutory professional title and awarding state | `site.berufsbezeichnung` and `site.verleihungsstaat` are rendered. | covered |
| (1) Nr. 5c: professional rules and how to access them | `site.berufsrecht` list is rendered with direct links to the named rules. | covered structurally; use exact chamber/legal URLs for real clients; avoid vague overview pages if the rules are buried or hard to find |
| (1) Nr. 6: VAT ID or economic ID if assigned | Not rendered as concrete fields yet. Closing note says these details must be checked case by case. | conditional / missing when assigned |
| (1) Nr. 7: liquidation note for AG, KGaA, GmbH | Not rendered. | conditional / usually not relevant for a sole dental practice |
| (1) Nr. 8: audiovisual media services | Not rendered. | not relevant for the dentist sample |
| (2): other legal duties remain unaffected | Privacy, accessibility, professional-law, BFSG, HWG, and form-data issues must be checked separately. | tracked separately |

## Implementation guidance

The sample page can stay conservative if it is visibly labelled as a fictional Muster page. For final client websites, do not rely on the sample defaults. Confirm the actual practice/entity shape first:

- sole practice, Berufsausübungsgemeinschaft, MVZ, GmbH, or another legal form
- exact owner/representative names
- real postal address and electronic contact channels
- supervisory authority, chamber, and KZV where applicable
- professional title and awarding state
- exact professional-rule names and URLs for the relevant Bundesland
- VAT ID or economic ID if assigned
- register and register number if applicable
- liquidation status for relevant company forms if applicable
- whether §18 Abs. 2 MStV applies because the site contains journalistic-editorial content

## Source references

See the [source register](source_Register.md) for the maintained list of source URLs and what each one contributes.

## Risk note from the Berlin dentist guidance

The Zahnärztekammer Berlin guidance notes that DDG violations can be administrative offences under §33 DDG with fines up to 50,000 EUR, and that competition-law warnings by other providers or organisations can also be a practical risk. This note belongs in internal implementation/audit guidance, not in the public sample Impressum copy.

Individual projects may have additional legal research files. Consult them during project work when present, but keep this reusable Astro skill independent from any single repository.
