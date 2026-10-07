# Poster Constraints and QA

Use this reference while designing or revising a poster. Apply the principles proportionally to the chosen canvas and viewing context; the numeric guidance is a starting point, not a fixed style system.

## Poster, not website

- Compose one bounded canvas with a clear overall silhouette.
- Avoid navigation bars, interactive cards, hover-only information, dialogs, carousels, accordions, sticky elements, and long webpage-style section stacking when the final output is a static image.
- Keep the reading path visible at once: identity, offer/message, evidence or imagery, action/contact.
- A tall social poster may still be vertically oriented, but its length must be chosen deliberately rather than emerging from unlimited document flow.

## Canvas and hierarchy

- Decide output dimensions before layout. Keep the root canvas fixed to those dimensions for HTML-based posters.
- Reserve edge safety margins and align major elements to a small set of guides.
- Limit the composition to roughly three hierarchy levels: dominant, supporting, detail.
- Test the poster as a thumbnail. The title, subject, main offer, and action should remain identifiable.
- When content does not fit, shorten secondary copy, combine repeated statements, or restructure zones. Do not solve crowding primarily by shrinking type.

## Typography

- Size type relative to canvas width and final viewing distance.
- As a starting range for raster posters:
  - dominant title: about 7–14% of canvas width;
  - major section or price: about 3–7%;
  - essential body copy: about 1.2–2%;
  - secondary labels: about 0.9–1.3%, only when genuinely secondary.
- Increase these proportions for mobile feeds, distant viewing, or low-resolution delivery.
- Prefer shorter lines, meaningful weight contrast, and stronger spacing over dense paragraphs.
- Do not let decorative English, codes, or metadata compete with the actual user-facing information.

## Images

- Inspect every image’s aspect ratio and subject placement before designing frames.
- Use a regular grid when several samples must be compared equally.
- Use collage, rotation, or overlap only when the user values expressive composition more than complete comparison.
- Use `contain` for design sheets, product views, diagrams, reference turnarounds, or any artwork whose edges matter.
- Use `cover` for atmospheric photography or intentional crops, and verify faces, products, text, signatures, and key details remain visible.
- Put captions in dedicated strips or open space instead of covering important artwork.

## Contact and QR codes

- Treat contact as a primary conversion element, not fine print.
- Give a QR code a high-contrast quiet zone and avoid texture, transparency, distortion, rotation, or overlap.
- Start around 13–16% of canvas width for a prominent QR code, then adjust for viewing distance and expected display size.
- Pair it with a direct label such as “扫码联系,” “报名入口,” or “了解详情.”
- Scan-test the exported file at actual size and at the size users are likely to see in a feed.

## HTML implementation

- Use a fixed-size root such as `.poster { width: ...; height: ...; overflow: hidden; }`.
- Keep images local or reliably embedded when portability matters.
- Avoid runtime-only content and interactions if the deliverable is a static export.
- Add `@media print` rules when PDF or print output is expected.
- Keep the source editable and separate from previously approved variants.

## Export checklist

- Correct pixel dimensions and aspect ratio.
- All assets loaded; no broken paths.
- No unexpected scroll length or clipped canvas edge.
- No essential text below the legibility threshold.
- No unintended image crop, overlap, or label obstruction.
- Prices, conditions, exclusions, and rights match the source copy.
- Contact method is prominent and QR code scans successfully.
- Contrast and margins remain sound after export.
- Exported PNG/PDF and editable source are both retained when appropriate.

## Style freedom

These constraints govern usability and production quality, not aesthetics. Select any suitable direction—minimal, maximal, editorial, illustrative, typographic, luxury, playful, organic, brutalist, retro, futuristic, cultural, photographic, or otherwise—based on the task. Vary layout, palette, type, texture, and composition intentionally between projects.
