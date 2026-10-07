---
name: create-posters
description: Create or revise professional single-page posters from copy, images, logos, QR codes, and delivery constraints, including fixed-canvas HTML posters and export-ready PNG/PDF layouts. Use when Codex needs to make a commission sheet, event poster, promotional graphic, information poster, social-media poster, key visual, or similar one-page visual; improve poster hierarchy, readability, image presentation, contact placement, or prepare HTML for image export. Do not use for ordinary responsive websites or multi-slide presentations.
---

# Create Posters

Create one cohesive visual composition whose final artifact is a poster, not a webpage disguised as one. Derive the aesthetic from the brief, audience, content, and supplied artwork; never impose a preset color palette, genre, grid, or visual style.

## Workflow

1. Inspect workspace instructions and source material.
   - Read applicable `AGENTS.md` files before editing.
   - Read text files explicitly as UTF-8 when the environment permits.
   - Inventory copy, images, logos, QR codes, brand rules, dimensions, and intended distribution channel.
   - Inspect image dimensions and compositions before assigning them to frames.

2. Define the final artifact before laying it out.
   - Establish the exact canvas size or aspect ratio, output format, and primary viewing context.
   - If the poster will become a PNG or PDF, use a fixed canvas with a deliberate edge and no content that depends on scrolling, hover, animation, navigation, accordions, or modals.
   - If dimensions are unspecified, choose a defensible standard for the intended channel and state the assumption.

3. Build the information hierarchy.
   - Identify the identity/title, primary message, key facts, call to action, supporting details, and mandatory fine print.
   - Make the primary message understandable at thumbnail size.
   - Condense repetitive copy while preserving meaning and tone. Do not silently remove prices, exclusions, conditions, rights, deadlines, or contact details.
   - Move lower-priority detail into concise notes instead of shrinking all text to make it fit.

4. Choose a project-specific visual direction.
   - Infer typography, color, density, image treatment, and composition from the subject and assets.
   - Commit to a coherent direction, but keep it distinct from prior posters unless the user requests continuity.
   - Do not default to industrial red/black, editorial grids, futuristic overlays, or any other previously successful treatment.

5. Compose the fixed canvas.
   - Treat the canvas as one image: establish focal point, reading path, balance, rhythm, and edge behavior.
   - Use a small number of clear visual zones rather than website sections or repeated UI cards.
   - Prefer strong alignment and intentional negative space. Use overlap only when it improves the composition and does not hide essential artwork or text.
   - For HTML delivery, keep poster styles scoped to one fixed-size root element and include print/export-safe styling.

6. Protect legibility and asset integrity.
   - Judge text at the final exported size and at a realistic reduced preview, not only while zoomed in.
   - Use `object-fit: contain` when the whole artwork must remain visible. Use cropping only when it is intentional and preserves the subject.
   - Keep labels outside critical image regions when possible.
   - Give QR codes a quiet zone, high contrast, and enough physical size to scan from the final output.
   - Read `references/poster-constraints.md` whenever setting type scale, arranging multiple images, placing contact information, or exporting the poster.

7. Export and verify.
   - Wait for local fonts and images to load before capture.
   - Export at the intended pixel dimensions rather than relying on an arbitrary browser viewport.
   - Check the full artifact for overflow, accidental crop, hidden content, tiny text, low contrast, uneven margins, broken assets, and QR scanability.
   - Preserve editable source files alongside the exported artifact unless the user asks otherwise.

8. Iterate from the underlying cause.
   - Translate feedback such as “too web-like,” “too long,” “text too small,” “images are blocked,” or “contact is too small” into layout causes before editing.
   - Rebalance hierarchy and space instead of applying isolated patches that create new crowding elsewhere.

## Deliverables

- Keep source content and supplied assets intact unless modification is requested.
- Use a new source file when the user requests an alternate direction rather than overwriting an approved version.
- Report the canvas dimensions, source file, exported file if produced, and any assumptions that affect reuse.
