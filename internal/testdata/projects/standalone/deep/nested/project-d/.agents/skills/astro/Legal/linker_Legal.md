# Legal

Legal-page implementation references for Astro projects in this org. This folder is for source routing, page-structure rules, and implementation checklists. It is not legal advice and does not replace client counsel.

The common task is editing a public demo or client legal page without mixing up the fictional site shown to visitors with the real operator of the public website. Start by deciding which layer you are touching:

- [Real Operator Legal](Real_Operator/linker_Real_Operator.md): the actual publisher/operator of the public website.
- [Fictional Demo Legal](Fictional_Demo/linker_Fictional_Demo.md): sample pages for the fictional practice/customer shown inside the demo.
- [Client Delivery Legal](Client_Delivery/linker_Client_Delivery.md): final legal-page implementation checklist for real customer websites.

## Routing rules

| Question | Where to go |
|---|---|
| A public demo needs real operator Impressum/Datenschutz links | [Demo operator links](Real_Operator/demo_Operator_Links.md) |
| The actual demo operator's public legal-page duties are unclear | [Required public pages](Real_Operator/required_Public_Pages.md) |
| A fictional sample legal page needs wording that says sample, fictional, or not legally operative | [Fictional demo legal](Fictional_Demo/linker_Fictional_Demo.md) |
| A fictional dentist legal-page edit needs source orientation before changing copy | [Fictional demo source register](Fictional_Demo/source_Register.md) |
| A German fictional dentist Impressum needs a field-by-field §5 DDG check | [§5 DDG Impressum checklist](Fictional_Demo/ddg_Impressum_Checklist.md) |
| A final dentist client Impressum needs concrete intake fields and optional-section checks | [Final client Impressum checklist](Client_Delivery/final_Client_Impressum_Checklist.md) |
| A previous legal-page mistake may repeat | [Implementation footguns](Fictional_Demo/implementation_Footguns.md) |
| A page needs long-form authored legal text | Use `src/Content/` and the legal content collections; see [Content Collections](../Content/linker_Content.md) |
| A repeated legal UI label or footer label needs translation | Use `src/Scripts/Multi_Lang_Txts/`; see [Multi_Lang_Txts](../Scripts/multi_Lang_Txts.md) |
| A site-wide operator URL or demo metadata value is reused across pages | Use `src/Data/Common/`; see [Data](../Data/linker_Data.md) |

## Current reference sets

- [Real Operator Legal](Real_Operator/linker_Real_Operator.md): actual public-site operator pages and persistent operator links. Use before changing footer/demo-disclosure links to the real operator's Impressum or Datenschutz.
- [Fictional Demo Legal](Fictional_Demo/linker_Fictional_Demo.md): sample legal pages for fictional customer/practice demos. Use before changing `990_Legal_Notice`, `991_Privacy_Policy`, `992_Accessibility`, sample legal-page labels, or fictional safe data.
- [Client Delivery Legal](Client_Delivery/linker_Client_Delivery.md): real customer legal-page delivery checklist. Use before finalizing a client Impressum or collecting client legal intake fields.
