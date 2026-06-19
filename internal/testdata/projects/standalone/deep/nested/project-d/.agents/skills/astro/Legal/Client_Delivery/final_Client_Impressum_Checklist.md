# Final Client Impressum Checklist

Checklist for implementing a real German dentist client's `Impressum`. This is a technical/client-intake checklist, not legal advice. The final legal text should be supplied or approved by the client or their legal counsel.

## Before Writing Copy

Confirm the practice/entity shape first. Do not assume the fictional demo defaults apply to a real client.

- Einzelpraxis
- Berufsausübungsgemeinschaft
- MVZ
- GmbH or another registered entity
- Privatpraxis, Vertragszahnarztpraxis, or mixed GKV/PKV setup
- Bundesland and Kammerbezirk
- whether the site has blog/editorial content
- whether the site uses real practice photos, team photos, stock images, generated images, icons, maps, videos, booking widgets, or third-party embeds

## Core DDG Fields

Render these as concrete fields when they apply:

- provider name
- postal address
- phone number
- e-mail address
- legal form for legal persons or partnerships
- full name of the representative person, such as Geschäftsführer, Vorstand, or partners
- register court and register number when registered
- VAT ID or Wirtschafts-ID when assigned
- capital/liquidation information where legally relevant

For a simple sole-practice page, owner name and address may be enough for the entity section. For a GmbH, MVZ, BAG, or other registered setup, do not hide the legal-form/register fields in a generic note.

## Dentist-Specific Fields

Confirm and render the client-specific version of:

- statutory professional title
- awarding state for the professional title
- responsible chamber
- responsible supervisory or approbation authority
- KZV where relevant, especially for contract dental activity
- professional rules and direct access links
- state-specific Heilberufe-/Kammer law

These values vary by Bundesland, chamber district, practice form, and activity type. Do not hardcode Berlin values into final client projects.

## Professional Rules

The professional-rules section should name the rules and make access obvious. Prefer direct links to the named rules or a clearly navigable official page.

Typical dentist rules to verify:

- Gesetz über die Ausübung der Zahnheilkunde (ZHG)
- Gebührenordnung für Zahnärzte (GOZ)
- Berufsordnung of the responsible chamber
- Weiterbildungsordnung if relevant
- state-specific Heilberufekammergesetz or equivalent professional-law act

Avoid vague labels such as "Berufsrecht" when the target page is hard to navigate or buries the actual rules.

## Editorial Responsibility

If the site contains journalistic-editorial content such as blog posts, guides, news, or articles, check whether a section for responsibility under § 18 Abs. 2 MStV belongs in the Impressum.

Typical field:

- responsible person for journalistic-editorial content
- address, often "Anschrift wie oben" when accurate

## Image And Media Rights

Treat media rights as a normal final-client intake item. Add a compact section when images, graphics, videos, generated media, icons, or third-party assets are used and the client can state the licensing truthfully.

Ask for:

- photographer or studio names
- stock provider names and license scope
- whether team/practice images are client-owned or licensed
- whether Zetunweb created any media for the client
- whether AI-generated media is used
- whether attribution is required by the license
- whether social-media reuse is included in the license

Use factual wording. Do not claim exclusive rights for the client unless the rights chain actually supports that.

## Optional Or Lawyer-Check Sections

Do not add long boilerplate automatically. Collect the information and use concise sections only when appropriate or lawyer-approved.

- Haftungsbegrenzung: not a default long block; use only if counsel wants it.
- External-link disclaimer: avoid broad boilerplate unless reviewed.
- Medical/information disclaimer: often more useful than generic liability text; can state that website information does not replace personal diagnosis or treatment advice.
- Berufshaftpflichtversicherung: collect provider and territorial scope when required or client/lawyer wants it shown.
- Schlichtungsstelle / VSBG: check case by case.
- Copyright/image-rights notice: include when useful and factually supported.

## Client Intake Questions

Ask the client or their counsel for:

- exact legal practice/entity name
- legal form
- full postal address
- phone and e-mail for quick contact
- owner, partners, or representatives
- register court and register number, if any
- VAT ID and Wirtschafts-ID, if any
- chamber and membership details
- supervisory/approbation authority
- KZV membership/status, if relevant
- professional title and awarding state
- professional-rule URLs approved for their Bundesland
- MStV responsible person, if editorial content exists
- media-license/source list
- professional-liability insurance details, if to be shown
- preferred lawyer-approved wording for liability, external links, dispute resolution, and medical information disclaimers

## Implementation Notes

- Keep final client pages concise. A real client Impressum should not read like a demo explanation.
- Do not use fictional safe data in final client pages.
- Do not leave TODO markers visible.
- Use variables for repeated legal/practice data when the values appear in multiple pages.
- Put long legal text in `src/Content/`; put reusable structured client data in project config.
- Verify all outbound legal links before launch.
- Run the build after changing MDX legal pages or legal config.

## Source Orientation

Use the official §5 DDG text as the federal baseline. Then check the responsible chamber, Bundesland, KZV, and practice form. Existing public dentist Impressums can be useful examples, but they are not authority and often contain omissions, duplicates, stale TMG wording, or vague links.
