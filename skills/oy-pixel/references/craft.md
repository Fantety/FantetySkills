# Art and Animation Craft

## Pixel construction

Compose at the target logical resolution. Establish a recognizable silhouette, major masses, and signature features before adding texture or small details. Check readability at the actual display size.

Build with deliberate pixel clusters and a limited palette. Use a few shades per material, share colors across objects, and keep the light direction consistent. Hue-shift shadows and highlights when useful. `mix` and `hsv` can generate candidate colors; reuse selected colors throughout the picture.

Choose a base color, a shadow, and a highlight for each major material; add deeper shadow or a brighter accent only when the scale needs it. A 3–5 shade ramp is a useful starting point for larger forms, while tiny assets may read better with 2–4 colors overall. Pick discrete steps from `mix(shadow, highlight, t)` or vary hue, saturation, and value with `hsv`; check that adjacent steps remain distinguishable at native size. Cooler shadows and warmer highlights are an option when the lighting supports them. Place shading on the form and cast shadows in the same light direction.

Give diagonals and curves a consistent stepping rhythm. Reserve isolated pixels for intentional details such as eyes, glints, or sparks. Keep the outline strategy consistent. Tiny sprites usually benefit from crisp edges; transparent borders should retain their intended colors.

Use anti-aliasing selectively on a curve against a known background: a single chosen transition color, such as `mix(edge, background, 0.5)`, may soften a harsh step. Keep tiny silhouettes and outlines crisp when an intermediate color would blur their shape. For assets placed on arbitrary backgrounds, avoid baking a background color into the outer edge; use a hard edge or intentional partial alpha in PNG. Judge the exported result at native size, especially when GIF removes partial coverage.

For limited-palette gradients, use deliberate checkerboard or ordered dithering. Apply procedural noise where it helps describe a material, preserving clear shapes. Check tile seams on both axes and keep isometric assets aligned to a consistent grid angle.

## Motion and timing

Choose key poses and timing for the action. Idle animation needs a few readable changes; walking needs distinct contact, support, passing, and lift poses. Fast actions should communicate force and rhythm. Around 10–12 FPS is a useful starting point for small sprites; adjust to the intended motion.

Block out complex motion with simple masses before detailing. Check balance, foot contact, trajectories, and overlap, then reuse the same motion parameters for the final shapes. Animate the body parts responsible for the action.

Match acceleration to the subject. Rotation or conveyor movement may be uniform; jumps, impacts, and attacks need purposeful timing. Use anticipation, short impact compression, or modest overshoot when they improve readability.

Preserve identifying features across frames: proportions, palette, facial details, and accessories. Keep anchors stable and ensure extreme poses fit inside the canvas. Intentional deformation should strengthen the action.

Construct loops with periodic functions or closed pose paths. Check displacement and velocity across the last-to-first transition. One-shot actions can end in a settled pose with a longer hold. Use per-frame durations to express holds efficiently.

For movements of only one or two pixels, control when pixel clusters change. Use designed intermediate poses, local dithering, or blending against a known background when needed. PNG frames retain soft alpha; GIF works well with clear color clusters and binary transparency.

### Parameterized motion

Define a shared trajectory or pose function, then sample it for each frame. Divide an action into meaningful segments such as preparation, travel, impact, and recovery. For a segment starting at `startSeconds` and lasting `segmentSeconds > 0`, compute `u = math.max(0, math.min(1, (time - startSeconds) / segmentSeconds))`. Interpolate a coordinate with `a + (b - a) * u`; use `u*u*(3-2*u)` for a smooth start and stop, `u*u` for acceleration, or `1-(1-u)*(1-u)` for settling. Choose these per segment; a continuous rotation need not slow down every cycle.

For a uniformly timed one-shot movement with at least two frames, set `"loop": false` in the recipe. This example reaches both endpoints; replace the simple shape with the subject's drawing function while keeping the motion calculation:

```lua
local function ease(u)
    return u * u * (3 - 2 * u)
end
local u = frame_index / math.max(1, frame_count - 1)
local x = 2 + (width - 5) * ease(u)
canvas.circfill(math.floor(x + 0.5), math.floor(height / 2), 2, pal(1))
```

For uneven frame durations, use elapsed segment time instead of frame position. Place the endpoint at the start of the final hold so it is actually sampled. For loops, use periodic paths such as `math.sin(2 * math.pi * phase)`; applying a one-way interpolation directly to looping `phase` would jump at the seam.

Attach secondary motion to the main pose's anchor. For a periodic offset, `amplitude * math.sin(2 * math.pi * (phase - lag))` provides a controllable delay; choose smaller amplitude where appropriate. For a one-shot follow-through, evaluate the same motion at `math.max(0, time - lagSeconds)` and clamp its progress instead of wrapping it into a previous cycle.

### Fast motion and fading effects

When displacement between poses hides the action, use a brief smear stretched along the path or a few trailing silhouettes sampled at earlier times. Keep the start, contact, and recovery poses readable; shorten the smear's exposure instead of adding many nearly identical frames. Reconstruct each trail from the trajectory within the current frame, since Lua state does not persist between frames. Wrap earlier samples only for looping motion; omit samples before a one-shot action begins.

For smoke, glows, ghosts, and other decaying effects, define age from elapsed time and fade over a chosen lifetime: `u = math.max(0, math.min(1, age / lifetime))`, then `alpha(color, (1-u)*(1-u))`. Draw only while `0 <= age and age < lifetime`, with `lifetime > 0`. A few readable stages can be enough; combine opacity with changes in size, shape, or color when they describe the effect. Deliberate flashes or hard cuts may remain abrupt.

Overlapping translucent trails need separate recipe layers or explicit compositing because drawing calls replace pixels. PNG frames preserve a soft fade; for a transparent GIF, reduce cluster area or use stable ordered dithering as the effect fades. Against a known opaque background, blend toward that background. A fade expressed only through alpha will cross GIF's transparency threshold abruptly, so check the encoded animation for unintended popping and flicker.
