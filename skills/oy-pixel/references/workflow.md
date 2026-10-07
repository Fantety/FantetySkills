# Rendering and Recipes

## Runtime setup

`<skill-root>` is the directory containing SKILL.md. Resolve scripts and assets relative to it.

```text
<python> <skill-root>/scripts/render.py --check
<python> <skill-root>/scripts/render.py <work>/recipe.json --out <work>/output --name sprite
```

Use Python 3.10+ with Pillow 10–12. In Codex desktop, `load_workspace_dependencies` can locate an available Python/Pillow environment. If needed, create a local virtual environment and install `scripts/requirements.txt` into it.

The renderer uses the cached executable in `scripts/bin/` when its source fingerprint matches. Otherwise it builds `scripts/engine/Cargo.toml` with Cargo and a platform C compiler, such as MSVC Build Tools on Windows. Lua is compiled from vendored sources, and Cargo.lock pins dependencies. A cached executable must match the host platform. If a required build tool is unavailable, report the missing prerequisite and preserve the recipe.

## JSON recipe

A still image:

```json
{
  "width": 32,
  "height": 32,
  "palette": ["#172038", "#346856", "#62AB46", "#C7D859", "#FFF3CD"],
  "layers": [
    {"name": "Subject", "script": "sprite.lua"}
  ]
}
```

Script and base-image paths are relative to the **JSON file's directory**. Text files use UTF-8; a BOM is accepted. Choose either a script file or inline `code` for each layer. Examples in [assets/examples/](../assets/examples/) demonstrate the interface and timing; adapt their structure to the requested subject.

| Field | Meaning and default |
| --- | --- |
| `width`, `height` | Required logical dimensions, 1–1024 pixels each |
| `frames` | Frame count, default 1, maximum 2000 |
| `durationMs` | Uniform frame duration, default 100, range 1–60000 ms |
| `durations` | Per-frame duration array; mutually exclusive with `frames` and `durationMs` |
| `palette` | 1–1024 colors in `#RRGGBB` or `#RRGGBBAA`; specify explicitly for deliberate color control |
| `seed` | Unsigned 64-bit integer, default 42 |
| `loop` | Default true; false exports a GIF without a looping extension |
| `layers` | 1–128 layers ordered bottom to top |

If omitted, the palette uses the 16 colors listed in `DEFAULT_PALETTE` in render.py.

Each layer accepts `name`, `script` or `code`, `opacity` (default 1), `visible` (default true), and `baseImage` or `baseImages`. Each frame starts from its corresponding base image or a transparent canvas. A single base image can be reused across all frames. `baseImages` must contain one image or exactly the frame count, and every image must match the canvas dimensions.

To edit a GIF, use Pillow's per-frame `seek` and `convert('RGBA')` to extract composited PNG frames, preserving each frame's `duration`. Supply these through `baseImages` and `durations`.

Layers use source-over composition with layer opacity. Drawing calls **replace** pixels within a layer. For overlapping translucent effects, use separate layers or explicitly calculate colors with `pget` and `mix`.

For animation, add timing fields and drive the script with `phase`:

```json
{
  "width": 32,
  "height": 32,
  "frames": 8,
  "durationMs": 100,
  "loop": true,
  "palette": ["#172038", "#346856", "#62AB46", "#C7D859", "#FFF3CD"],
  "layers": [{"name": "Subject", "script": "sprite.lua"}]
}
```

`time` is the sum of preceding frame durations in seconds. `phase` is that elapsed time divided by the total duration. For uneven timing, use these values for motion; `frame_index / frame_count` represents frame position rather than elapsed time.

## Export

```text
<python> <skill-root>/scripts/render.py recipe.json --out output --name sprite --formats gif,sheet --scale 4 --columns 4 --padding 1
```

Choose the formats required for the task:

- `png`: first-frame RGBA image, `sprite.png`.
- `gif`: animation, `sprite.gif`, with a shared palette and disposal=2 to clear previous positions.
- `sheet`: `sprite-sheet.png`, ordered left to right, then top to bottom. Columns default to 4; padding defaults to 0 and applies to both outer edges and gaps.
- `frames`: individual RGBA images, `sprite-0000.png`, `sprite-0001.png`, and so on.

`--scale` accepts integers from 1 to 64 and uses nearest-neighbor scaling. Export dimensions are limited to 16384 pixels per side and 67108864 total scaled frame pixels. Sprite sheets have the same size budget. The recipe's `width * height * frames * layers` must not exceed 16777216.

`--matte '#RRGGBB'` composites an opaque background before export. Output names are plain filename stems. Existing files are preserved unless `--overwrite` is supplied; use it when intentionally replacing an earlier render. Files are encoded and checked in a temporary directory before being moved to the output directory.

GIF timing is rounded to 10 ms units using cumulative timestamps to reduce drift. Frames below 10 ms are raised to 10 ms. Alpha uses a threshold of 128. Images with more than 255 visible colors are quantized to a shared palette with dithering disabled. Draw deliberate dithering patterns in Lua when needed.

Identical adjacent GIF frames may be combined with their delays accumulated. The report's `frames` is the logical count; `gifFrames` and `gifDurationMs` are read back from the encoded file. Use a sprite sheet or PNG frames when each logical frame must remain individually accessible. The `warnings` field reports export conversions.

## Visual inspection

Export a sheet to a preview directory to inspect all poses, then inspect the animation's playback and loop. Render separate native-size and integer-scaled previews as needed. Check clipping, anchors, transparency, palette consistency, and rhythm before delivering the finished artwork.
