---
name: oy-pixel
description: Create and edit pixel art, sprites, icons, tiles, effects, and frame animations with precise pixels and consistent palettes. Export finished artwork as PNG, GIF, sprite sheets, or PNG frame sequences.
license: MIT
---

# Pixel Art and Animation

Draw pixel artwork with Lua scripts, render it at the intended logical resolution, inspect the results, and deliver the requested image or animation files.

## Workflow

1. Establish the subject, use, logical dimensions, palette, background transparency, and output format. Follow the user's specifications; choose suitable defaults for unspecified details. Small sprites often fit 16–64 pixels, while scenes may need a larger canvas.
2. Read [Rendering and recipes](references/workflow.md) to prepare a JSON recipe and [Lua drawing API](references/lua-api.md) when writing scripts. Before drawing, read [Pixel construction](references/craft.md#pixel-construction); for animations, also read [Motion and timing](references/craft.md#motion-and-timing) and its subsections. Apply the guidance appropriate to the subject, resolution, and output format. Use shapes, pixel clusters, loops, and small stamps to express the artwork compactly.
3. Write scripts and recipes in the task's working directory. Arrange layers from background to foreground. Reuse shape functions across frames and drive motion through `phase`, `time`, or explicit key poses. Round drawing coordinates to integers.
4. Render with `<python> <skill-root>/scripts/render.py <recipe.json> --out <output-dir> --name <name>`. The default output is PNG for a still or GIF for an animation. Select additional formats when useful. `--scale` enlarges pixels with integer nearest-neighbor scaling.
5. Inspect the finished files. Check stills at native size and enlarged pixel scale. For animation, inspect the GIF and a contact sheet for readable poses, consistent anchors, clean transparency, timing, and the last-to-first transition. Revise the relevant drawing or timing parameters when an issue appears.
6. Present the artwork and links to the requested files. Include dimensions and useful playback or sheet-layout details. Use the renderer's `files` paths and account for any export `warnings`.

## Drawing rules

- Use reference images to guide silhouette, proportions, colors, and recognizable features. Simplify details to the target resolution.
- Load existing artwork through `baseImage` or `baseImages` for edits. Preserve pixels outside the intended change. `canvas.clear(nil)` clears the current layer.
- Call drawing functions with dot syntax: `canvas.pset(...)`. Coordinates start at the top-left, with x increasing right and y increasing down. Palette indices start at 1.
- Each frame executes independently. Looping `phase` spans `[0,1)`; build motion that wraps smoothly. Random values are reproducible but vary by frame, so stable surface patterns should use fixed patterns or coordinate hashes.
- Use PNG for full RGBA transparency. GIF exports support 255 visible colors, binary transparency, and 10 ms timing units. Use PNG frames or a chosen matte background when an effect needs soft transparency.
- Correct errors using the reported layer and script line. For resource limits, reduce repeated pixel writes, simplify the drawing, or split the work into layers or smaller sections.

For runtime setup, recipe fields, and export options, read [Rendering and recipes](references/workflow.md).
