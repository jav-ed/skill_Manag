# Demo Operator Links

Use persistent links to the real operator's legal pages on every public demo page. A footer disclosure can be enough for clearly labelled demos; add a top banner when the preview is easy to mistake for a live practice.

Recommended German substance:

```text
Demo-Website: Diese Website ist eine fiktive Musterseite von [Betreiber]. Praxisname, Personen, Adresse, Telefonnummern und Inhalte sind frei erfunden. Es werden keine zahnärztlichen Leistungen angeboten. Betreiber dieser Demo ist [Betreiber]. Die rechtlich maßgeblichen Anbieterinformationen der Demo finden Sie im Impressum von [Betreiber]; die Datenschutzinformationen finden Sie in der Datenschutzerklärung von [Betreiber].
```

Implementation pattern:

- Put repeated operator URLs and names in `src/Data/Common/`.
- Put repeated labels in `src/Scripts/Multi_Lang_Txts/Legal/`.
- For MDX pages, use an authored notice component instead of copy-pasting disclosure paragraphs across every legal page.
- Verify the rendered text around links; punctuation and spacing can drift when sentence fragments are assembled in components.

## Client delivery boundary

For offers and delivery docs, use a boundary like:

```text
Die technische Einbindung rechtlicher Pflichtseiten ist Teil des Angebots. Die inhaltliche Prüfung und Freigabe erfolgt durch die Praxis bzw. deren Rechtsberatung.
```

This keeps the implementation promise separate from a legal-review promise.
