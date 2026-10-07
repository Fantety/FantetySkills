# Lua Drawing API

Scripts run in Lua 5.4 with `math`, `string`, tables, and loops. File and network access, `require`, `load`, and `dofile` are unavailable. Use seeded `rand` instead of `math.random`.

## Globals and colors

| Value or function | Meaning |
| --- | --- |
| `width`, `height` | Canvas dimensions |
| `time`, `phase` | Frame start time in seconds and timeline phase in `[0,1)` |
| `frame_index`, `frame_count` | Zero-based frame index and total frame count |
| `pal(i)` | Color at one-based palette index i; out-of-range indices fail |
| `hex('#RRGGBB[AA]')` | Validate and normalize a hex color |
| `mix(a,b,t)` | Linear RGBA interpolation; t is clamped to 0–1 |
| `hsv(h,s,v[,a])` | Hue in degrees; saturation, value, and alpha in 0–1 |
| `alpha(color,a)` | Replace the color's alpha with a value in 0–1 |
| `rand()` / `rand(a,b)` | Float in `[0,1)` / integer with inclusive bounds |
| `noise(x,y[,scale])` | Seeded noise in `[0,1)` |

Drawing colors are `#RRGGBB` or `#RRGGBBAA` strings. Both `nil` and an alpha-zero color erase a pixel. `pal(i)` returns a color string.

## Canvas calls

Use dot syntax:

```lua
canvas.pset(x, y, color)
canvas.pget(x, y)                         -- color or nil
canvas.line(x0, y0, x1, y1, color)
canvas.rect(x, y, w, h, color)            -- width and height, not end coordinates
canvas.rectfill(x, y, w, h, color)
canvas.circle(cx, cy, r, color)
canvas.circfill(cx, cy, r, color)
canvas.ellipse(cx, cy, rx, ry, color)
canvas.ellipsefill(cx, cy, rx, ry, color)
canvas.flood(x, y, color)
canvas.replace(oldColor, newColor)
canvas.outline(color)                    -- add an outer contour
canvas.clear(color)                      -- nil clears the layer
canvas.stamp(rows, legend, x, y)
```

Coordinates and radii must be integers. Round computed positions explicitly, for example with `math.floor(v + 0.5)`. Point reads, point writes, and flood seeds must be within the canvas. Lines, rectangles, circles, and ellipses clip at its edges. A stamp's entire rectangle must fit inside the canvas. Extreme geometry sizes are rejected.

Small character grids work well for reusable sprite parts:

```lua
local rows = {".AA.", "ABBA", ".AA."}
canvas.stamp(rows, {A=pal(1), B=pal(2)}, 2, 3)
```

Rows must have equal lengths, and every non-dot symbol needs a legend entry. A dot **preserves the underlying pixel**. To erase, use `pset(...,nil)` or `rectfill(...,nil)`. Compose larger artwork with shapes and functions.

## Animation and repeatability

```lua
local p = phase * 2 * math.pi
local y = math.floor(height / 2 - 3 * math.sin(p) + 0.5)
canvas.circfill(math.floor(width / 2), y, 3, pal(1))
```

Every frame uses a fresh Lua state. Calculate positions from time, phase, or key-pose parameters. Reusing the recipe and seed reproduces the output.

Timeline rendering mixes the frame index into the seed, so `rand` and `noise` vary between frames. Stable surface details should use fixed patterns or hashes based on object-local coordinates.

Loops sample `[0,1)`. Make the final sampled pose transition naturally into the first. For a one-shot action that must reach its endpoint, use `frame_index / math.max(1, frame_count - 1)` for pose progress and frame durations for timing.

## Budgets and errors

Each script accepts at most 65536 UTF-8 bytes. Per frame, the runtime allows approximately 20 million Lua instructions, 32 MiB of Lua memory, and 65536 writes that change pixel values. **Repeated changes to the same pixel consume the write budget.** A still has a 5-second time limit; all frames of an animated layer share a 10-second limit.

Reduce repeated full-canvas overpainting with separate layers, local drawing regions, or a single calculation of each final color. Errors identify the layer and Lua line. Correct invalid colors, fractional coordinates, out-of-bounds writes, or malformed stamps before retrying. Reduce workload for budget errors.
