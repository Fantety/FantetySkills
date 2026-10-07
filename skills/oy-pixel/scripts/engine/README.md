# pixel_shader

A sandboxed, deterministic Lua drawing environment for pixel art. A script
paints onto a `ShaderCanvas` (a row-major framebuffer of optional `#RRGGBBAA`
colors) through a small pico-8-style API; the host decides budgets (Lua
instructions, memory, wall clock, changed pixels) and receives the modified
buffer back.

```rust
use pixel_shader::{run_script, ShaderCanvas, ShaderEnv, ShaderLimits};
use std::time::Duration;

let mut canvas = ShaderCanvas::new(8, 8);
let env = ShaderEnv::new(vec![vec!["#D1495B".into(), "#F4F1DE".into()]], 0, 42);
let limits = ShaderLimits::default();

let outcome = run_script(
    "canvas.circfill(4, 4, 3, pal(1))\ncanvas.pset(0, 0, nil)",
    &canvas,
    &env,
    &limits,
)
.unwrap();

assert_eq!(outcome.changed_pixels, 37);
canvas.pixels = outcome.pixels;
assert_eq!(canvas.pget(4, 4).as_deref(), Some("#D1495BFF"));
assert_eq!(canvas.pget(0, 0), None);
```

Guarantees:

- **Sandboxed** — only safe Lua standard libraries are loaded (`math` and
  `string` included; `math.random` is removed so randomness can only come from
  the seeded `rand()`); `load`, `dofile`, and `require` are removed.
  Instruction, memory, wall-clock, and changed-pixel budgets abort runaway
  scripts.
- **Deterministic** — `rand` and `noise` are seeded by the caller, so the same
  script, canvas, and seed always produce the same output.
- **Standalone** — no platform or host dependencies beyond `mlua`; the
  framebuffer uses plain `#RRGGBBAA` strings so it is trivial to bridge into
  any document model.

Beyond the drawing primitives, scripts can compute freely: colors are ordinary
Lua values (store them in variables, build them with `string.format`), and the
helpers `mix(c1, c2, t)` (linear blend), `hsv(h, s, v[, a])`, and
`alpha(c, a)` make gradients, glows, and palette-free color math one-liners.
