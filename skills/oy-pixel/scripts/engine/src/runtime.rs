use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use mlua::{
    Error as LuaError, Function, HookTriggers, Lua, LuaOptions, StdLib, Table, Value, VmState,
};

use crate::color::{hsv_color, mix_colors, normalize_color, paint, set_alpha};
use crate::noise::value_noise;
use crate::prng::SplitMix64;
use crate::{RunOutcome, ShaderCanvas, ShaderEnv, ShaderError, ShaderLimits};

/// Distinctive prefix for errors raised by this host (budget aborts); lets
/// classification tell host aborts apart from script errors.
const HOST_ERROR: &str = "pixel-shader-budget: ";
const HOOK_INTERVAL: u32 = 100_000;

/// State shared by all `canvas.*` closures.
struct Draw {
    width: usize,
    height: usize,
    buffer: Rc<RefCell<Vec<Option<String>>>>,
    changed: Rc<Cell<usize>>,
    max_changed: usize,
}

impl Draw {
    fn set(&self, x: usize, y: usize, value: Option<String>, what: &str) -> Result<(), LuaError> {
        if x >= self.width || y >= self.height {
            return Err(runtime_error(format!(
                "{what}: ({x}, {y}) is outside the {}x{} canvas",
                self.width, self.height
            )));
        }
        let index = y * self.width + x;
        let mut buffer = self.buffer.borrow_mut();
        // Bounds-checked on purpose: a broken length invariant must surface
        // as a script error, not as a panic unwinding through the sandbox.
        let Some(slot) = buffer.get_mut(index) else {
            return Err(runtime_error(format!(
                "{what}: pixel ({x}, {y}) is outside the canvas buffer"
            )));
        };
        if *slot != value {
            if self.changed.get() >= self.max_changed {
                return Err(budget_error(format!(
                    "changed-pixel budget of {} exceeded",
                    self.max_changed
                )));
            }
            self.changed.set(self.changed.get() + 1);
            *slot = value;
        }
        Ok(())
    }

    fn get(&self, x: usize, y: usize) -> Option<String> {
        self.buffer
            .borrow()
            .get(y * self.width + x)
            .cloned()
            .flatten()
    }
}

fn runtime_error(message: impl Into<String>) -> LuaError {
    LuaError::RuntimeError(message.into())
}

fn budget_error(message: String) -> LuaError {
    LuaError::RuntimeError(format!("{HOST_ERROR}{message}"))
}

fn integer(value: f64, name: &str) -> Result<i64, LuaError> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(runtime_error(format!(
            "{name} must be an integer, got {value}"
        )));
    }
    Ok(value as i64)
}

/// In-bounds coordinate (pset, pget, stamp origin, flood seed).
fn coordinate(value: f64, name: &str) -> Result<usize, LuaError> {
    let raw = integer(value, name)?;
    if raw < 0 {
        return Err(runtime_error(format!("{name} must be >= 0, got {raw}")));
    }
    Ok(raw as usize)
}

/// `nil` stays transparent; a color string is validated and alpha-00 becomes
/// transparent, mirroring the host document semantics.
fn color_value(color: &Option<String>, what: &str) -> Result<Option<String>, LuaError> {
    match color {
        None => Ok(None),
        Some(text) => paint(text).map_err(|error| runtime_error(format!("{what}: {error}"))),
    }
}

pub(super) fn execute(
    code: &str,
    canvas: &ShaderCanvas,
    env: &ShaderEnv,
    limits: &ShaderLimits,
) -> Result<RunOutcome, ShaderError> {
    // Hard panic barrier. mlua deliberately re-raises a panic raised inside
    // one of our host callbacks (canvas functions, budgets, …) as
    // `resume_unwind` at the Rust↔Lua boundary once the script hands the
    // wrapped payload back to the host (chunk return, pcall reuse, gc …).
    // That unwind must never escape this crate: crossing the embedded Lua C
    // stack aborts the whole process with "panic in a function that cannot
    // unwind" (observed live in v0.1.5 during an animate=true run). Convert
    // every surviving panic into an ordinary script error the model can act
    // on instead.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_in_sandbox(code, canvas, env, limits).map_err(|error| classify(&error, limits))
    }))
    .unwrap_or_else(|payload| Err(panic_error(payload)))
}

/// Builds the script error for a panic that survived the sandbox (host bug):
/// the message carries the panic payload so logs stay diagnosable.
fn panic_error(payload: Box<dyn std::any::Any + Send>) -> ShaderError {
    let detail = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown cause".to_string());
    ShaderError::Runtime {
        line: 0,
        message: format!("script triggered a shader host error: {detail}"),
    }
}

fn run_in_sandbox(
    code: &str,
    canvas: &ShaderCanvas,
    env: &ShaderEnv,
    limits: &ShaderLimits,
) -> Result<RunOutcome, LuaError> {
    let started = Instant::now();
    let buffer = Rc::new(RefCell::new(canvas.pixels.clone()));
    let changed = Rc::new(Cell::new(0usize));
    let prng = Rc::new(RefCell::new(SplitMix64::new(env.seed)));

    // ALL_SAFE is misleading (it includes io/os/package — everything but
    // ffi/debug); load the genuinely safe subset explicitly. The base library
    // is always opened by mlua.
    let lua = Lua::new_with(
        StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH,
        LuaOptions::new().catch_rust_panics(true),
    )?;
    lua.set_memory_limit(limits.max_memory)?;

    // Instruction + wall-clock budget, checked every HOOK_INTERVAL VM
    // instructions. Returning Err from the hook aborts the chunk.
    {
        let instruction_total = Rc::new(Cell::new(0u64));
        let max_instructions = limits.max_instructions;
        let max_duration = limits.max_duration;
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(HOOK_INTERVAL),
            move |_, _| {
                instruction_total.set(instruction_total.get().saturating_add(HOOK_INTERVAL as u64));
                if instruction_total.get() >= max_instructions {
                    return Err(budget_error(format!(
                        "instruction budget of {max_instructions} exceeded"
                    )));
                }
                if started.elapsed() >= max_duration {
                    return Err(budget_error(format!(
                        "wall-clock budget of {max_duration:?} exceeded"
                    )));
                }
                Ok(VmState::Continue)
            },
        )?;
    }

    // Strip the compilation and module-loading escapes that survive the safe
    // stdlib selection.
    for name in ["load", "loadstring", "dofile", "require"] {
        lua.globals().set(name, Value::Nil)?;
    }
    // math.random seeds itself from wall-clock entropy in Lua 5.4; scripts may
    // only produce randomness through the deterministic rand()/noise().
    let math_table: Table = lua.globals().get("math")?;
    math_table.set("random", Value::Nil)?;
    math_table.set("randomseed", Value::Nil)?;

    let draw = Rc::new(Draw {
        width: canvas.width,
        height: canvas.height,
        buffer: Rc::clone(&buffer),
        changed: Rc::clone(&changed),
        max_changed: limits.max_changed_pixels,
    });

    lua.globals().set("width", canvas.width as i64)?;
    lua.globals().set("height", canvas.height as i64)?;
    // Shader-style time constants: identical in single-frame runs (0/0/0/1),
    // driven per frame by run_script_timeline.
    lua.globals().set("time", env.time)?;
    lua.globals().set("phase", env.phase)?;
    lua.globals().set("frame_index", env.frame_index as i64)?;
    lua.globals()
        .set("frame_count", env.frame_count.max(1) as i64)?;

    let hex_fn = lua.create_function(|_, value: String| {
        paint(&value).map_err(|error| runtime_error(format!("hex: {error}")))
    })?;
    lua.globals().set("hex", hex_fn)?;

    let mix_fn = lua.create_function(|_, (from, to, t): (String, String, f64)| {
        mix_colors(&from, &to, t).map_err(|error| runtime_error(format!("mix: {error}")))
    })?;
    lua.globals().set("mix", mix_fn)?;

    let hsv_fn = lua.create_function(|_, (h, s, v, a): (f64, f64, f64, Option<f64>)| {
        hsv_color(h, s, v, a).map_err(|error| runtime_error(format!("hsv: {error}")))
    })?;
    lua.globals().set("hsv", hsv_fn)?;

    let alpha_fn = lua.create_function(|_, (color, alpha): (String, f64)| {
        set_alpha(&color, alpha).map_err(|error| runtime_error(format!("alpha: {error}")))
    })?;
    lua.globals().set("alpha", alpha_fn)?;

    let active_palette = env
        .palettes
        .get(env.active_palette)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|color| normalize_color(&color).unwrap_or(color))
        .collect::<Vec<_>>();
    let palette_count = active_palette.len();
    let pal_fn = lua.create_function(move |_, index: f64| {
        let raw = integer(index, "pal index")?;
        if raw < 1 {
            return Err(runtime_error(format!("pal: index must be >= 1, got {raw}")));
        }
        active_palette
            .get(raw as usize - 1)
            .cloned()
            .ok_or_else(|| {
                runtime_error(format!(
                    "pal: index {raw} is outside the active palette ({palette_count} colors)"
                ))
            })
    })?;
    lua.globals().set("pal", pal_fn)?;

    let rand_fn = {
        let prng = Rc::clone(&prng);
        lua.create_function(
            move |_, (from, to): (Option<f64>, Option<f64>)| match (from, to) {
                (None, None) => Ok(Value::Number(prng.borrow_mut().next_f64())),
                (Some(from), Some(to)) => {
                    let a = integer(from, "rand lower bound")?;
                    let b = integer(to, "rand upper bound")?;
                    Ok(Value::Integer(prng.borrow_mut().next_range(a, b)))
                }
                _ => Err(runtime_error("rand expects either () or (a, b)")),
            },
        )?
    };
    lua.globals().set("rand", rand_fn)?;

    let noise_seed = env.seed;
    let noise_fn = lua.create_function(move |_, (x, y, scale): (f64, f64, Option<f64>)| {
        Ok(value_noise(noise_seed, x, y, scale.unwrap_or(1.0)))
    })?;
    lua.globals().set("noise", noise_fn)?;

    let canvas_table = lua.create_table()?;
    register_canvas_functions(&lua, &canvas_table, &draw)?;
    lua.globals().set("canvas", canvas_table.clone())?;

    // Models keep writing canvas functions as bare globals ("pset(...)"),
    // which Lua reports as an opaque "attempt to call a nil value". Intercept
    // global misses: when the name is a canvas function, raise an error that
    // states the correct spelling.
    let globals_metatable = lua.create_table()?;
    globals_metatable.set(
        "__index",
        lua.create_function(move |_, (_, key): (Value, Value)| {
            if let Value::String(key) = &key {
                let key = key.to_str()?.to_string();
                if !canvas_table.get::<Value>(key.as_str())?.is_nil() {
                    return Err(runtime_error(format!(
                        "{key} is not a global — canvas functions live in the canvas table: canvas.{key}(...)"
                    )));
                }
            }
            Ok(Value::Nil)
        })?,
    )?;
    lua.globals().set_metatable(Some(globals_metatable))?;

    lua.load(code).set_name("script").exec()?;

    let final_pixels = buffer.borrow().clone();
    let changed_pixels = canvas
        .pixels
        .iter()
        .zip(final_pixels.iter())
        .filter(|(before, after)| before != after)
        .count();
    Ok(RunOutcome {
        changed_pixels,
        pixels: final_pixels,
    })
}

fn register_canvas_functions(lua: &Lua, table: &Table, draw: &Rc<Draw>) -> Result<(), LuaError> {
    let pset = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, (x, y, color): (f64, f64, Option<String>)| {
            let value = color_value(&color, "pset")?;
            let x = coordinate(x, "pset x")?;
            let y = coordinate(y, "pset y")?;
            draw.set(x, y, value, "pset")
        })?
    };
    table.set("pset", pset)?;

    let pget = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, (x, y): (f64, f64)| {
            let x = coordinate(x, "pget x")?;
            let y = coordinate(y, "pget y")?;
            if x >= draw.width || y >= draw.height {
                return Err(runtime_error(format!(
                    "pget: ({x}, {y}) is outside the {}x{} canvas",
                    draw.width, draw.height
                )));
            }
            Ok(draw.get(x, y))
        })?
    };
    table.set("pget", pget)?;

    let line = {
        let draw = Rc::clone(draw);
        lua.create_function(
            move |_, (x0, y0, x1, y1, color): (f64, f64, f64, f64, Option<String>)| {
                let value = color_value(&color, "line")?;
                let (x0, y0) = (integer(x0, "line x0")?, integer(y0, "line y0")?);
                let (x1, y1) = (integer(x1, "line x1")?, integer(y1, "line y1")?);
                // A segment ending far outside the canvas must not translate
                // into an effectively unbounded bresenham walk: clip it to a
                // one-pixel border around the canvas first. Endpoints that
                // already fit the border are returned unchanged.
                let Some((x0, y0, x1, y1)) = clip_segment(
                    (x0, y0),
                    (x1, y1),
                    -1,
                    -1,
                    draw.width as i64,
                    draw.height as i64,
                ) else {
                    return Ok(());
                };
                for (x, y) in bresenham(x0, y0, x1, y1) {
                    if x < 0 || y < 0 || x >= draw.width as i64 || y >= draw.height as i64 {
                        continue;
                    }
                    draw.set(x as usize, y as usize, value.clone(), "line")?;
                }
                Ok(())
            },
        )?
    };
    table.set("line", line)?;

    table.set("rect", rect_function(lua, draw, false)?)?;
    table.set("rectfill", rect_function(lua, draw, true)?)?;
    table.set("circle", circle_function(lua, draw, false)?)?;
    table.set("circfill", circle_function(lua, draw, true)?)?;
    table.set("ellipse", ellipse_function(lua, draw, false)?)?;
    table.set("ellipsefill", ellipse_function(lua, draw, true)?)?;

    let flood = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, (x, y, color): (f64, f64, Option<String>)| {
            let value = color_value(&color, "flood")?;
            let x = coordinate(x, "flood x")?;
            let y = coordinate(y, "flood y")?;
            let target = draw.get(x, y);
            if target == value {
                return Ok(());
            }
            let mut stack = vec![(x, y)];
            while let Some((px, py)) = stack.pop() {
                if draw.get(px, py) != target {
                    continue;
                }
                draw.set(px, py, value.clone(), "flood")?;
                if px + 1 < draw.width {
                    stack.push((px + 1, py));
                }
                if px > 0 {
                    stack.push((px - 1, py));
                }
                if py + 1 < draw.height {
                    stack.push((px, py + 1));
                }
                if py > 0 {
                    stack.push((px, py - 1));
                }
            }
            Ok(())
        })?
    };
    table.set("flood", flood)?;

    let replace = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, (old, new): (String, String)| {
            let from = normalize_color(&old)
                .map_err(|error| runtime_error(format!("replace: {error}")))?;
            let to = normalize_color(&new)
                .map_err(|error| runtime_error(format!("replace: {error}")))?;
            if from == to {
                return Ok(());
            }
            let mut buffer = draw.buffer.borrow_mut();
            for pixel in buffer.iter_mut() {
                if pixel.as_deref() == Some(from.as_str()) {
                    if draw.changed.get() >= draw.max_changed {
                        return Err(budget_error(format!(
                            "changed-pixel budget of {} exceeded",
                            draw.max_changed
                        )));
                    }
                    draw.changed.set(draw.changed.get() + 1);
                    *pixel = Some(to.clone());
                }
            }
            Ok(())
        })?
    };
    table.set("replace", replace)?;

    let outline = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, color: Option<String>| {
            let value = color_value(&color, "outline")?;
            let mut marks = Vec::new();
            for y in 0..draw.height {
                for x in 0..draw.width {
                    if draw.get(x, y).is_none() {
                        continue;
                    }
                    for (nx, ny) in [
                        (x + 1, y),
                        (x.wrapping_sub(1), y),
                        (x, y + 1),
                        (x, y.wrapping_sub(1)),
                    ] {
                        if nx < draw.width && ny < draw.height && draw.get(nx, ny).is_none() {
                            marks.push((nx, ny));
                        }
                    }
                }
            }
            for (x, y) in marks {
                draw.set(x, y, value.clone(), "outline")?;
            }
            Ok(())
        })?
    };
    table.set("outline", outline)?;

    let clear = {
        let draw = Rc::clone(draw);
        lua.create_function(move |_, color: Option<String>| {
            let value = color_value(&color, "clear")?;
            for y in 0..draw.height {
                for x in 0..draw.width {
                    draw.set(x, y, value.clone(), "clear")?;
                }
            }
            Ok(())
        })?
    };
    table.set("clear", clear)?;

    let stamp = {
        let draw = Rc::clone(draw);
        lua.create_function(
            move |_, (rows, legend, x, y): (Vec<String>, HashMap<String, String>, f64, f64)| {
                if rows.is_empty() {
                    return Err(runtime_error("stamp: rows must not be empty"));
                }
                let columns = rows[0].chars().count();
                if columns == 0 {
                    return Err(runtime_error("stamp: row 1 is empty"));
                }
                for (index, row) in rows.iter().enumerate() {
                    if row.chars().count() != columns {
                        return Err(runtime_error(format!(
                            "stamp: row {} has {} characters but row 1 has {columns}; all rows must share one length",
                            index + 1,
                            row.chars().count()
                        )));
                    }
                }
                let mut palette = HashMap::new();
                for (symbol, color) in &legend {
                    let mut chars = symbol.chars();
                    match (chars.next(), chars.next()) {
                        (Some(single), None) => {
                            let value = paint(color).map_err(|error| {
                                runtime_error(format!("stamp legend {symbol:?}: {error}"))
                            })?;
                            palette.insert(single, value);
                        }
                        _ => {
                            return Err(runtime_error(format!(
                                "stamp: legend keys must be single characters, got {symbol:?}"
                            )));
                        }
                    }
                }
                let origin_x = coordinate(x, "stamp x")?;
                let origin_y = coordinate(y, "stamp y")?;
                if origin_x + columns > draw.width || origin_y + rows.len() > draw.height {
                    return Err(runtime_error(format!(
                        "stamp: {}x{} grid at ({origin_x}, {origin_y}) does not fit the {}x{} canvas",
                        columns,
                        rows.len(),
                        draw.width,
                        draw.height
                    )));
                }
                for (row_index, row) in rows.iter().enumerate() {
                    for (column_index, symbol) in row.chars().enumerate() {
                        if symbol == '.' {
                            continue;
                        }
                        let Some(value) = palette.get(&symbol) else {
                            return Err(runtime_error(format!(
                                "stamp: symbol {symbol:?} (row {}, column {}) is missing from the legend",
                                row_index + 1,
                                column_index + 1
                            )));
                        };
                        draw.set(origin_x + column_index, origin_y + row_index, value.clone(), "stamp")?;
                    }
                }
                Ok(())
            },
        )?
    };
    table.set("stamp", stamp)?;

    Ok(())
}

fn rect_function(lua: &Lua, draw: &Rc<Draw>, filled: bool) -> Result<Function, LuaError> {
    let draw = Rc::clone(draw);
    lua.create_function(
        move |_, (x, y, w, h, color): (f64, f64, f64, f64, Option<String>)| {
            let value = color_value(&color, "rect")?;
            let (x, y) = (integer(x, "rect x")?, integer(y, "rect y")?);
            let (w, h) = (integer(w, "rect width")?, integer(h, "rect height")?);
            if w <= 0 || h <= 0 {
                return Err(runtime_error(format!(
                    "rect: width/height must be >= 1, got {w}x{h}"
                )));
            }
            let right = x.saturating_add(w).min(draw.width as i64);
            let bottom = y.saturating_add(h).min(draw.height as i64);
            for yy in y.max(0)..bottom.max(0) {
                for xx in x.max(0)..right.max(0) {
                    let on_edge = xx == x || yy == y || xx == right - 1 || yy == bottom - 1;
                    if filled || on_edge {
                        draw.set(xx as usize, yy as usize, value.clone(), "rect")?;
                    }
                }
            }
            Ok(())
        },
    )
}

fn circle_function(lua: &Lua, draw: &Rc<Draw>, filled: bool) -> Result<Function, LuaError> {
    let draw = Rc::clone(draw);
    lua.create_function(
        move |_, (cx, cy, r, color): (f64, f64, f64, Option<String>)| {
            let value = color_value(&color, "circle")?;
            let (cx, cy) = (integer(cx, "circle cx")?, integer(cy, "circle cy")?);
            let r = integer(r, "circle radius")?;
            if r < 0 {
                return Err(runtime_error(format!(
                    "circle: radius must be >= 0, got {r}"
                )));
            }
            let radius = r as f64 + 0.5;
            let radius2 = radius * radius;
            plot_shape(
                &draw,
                ShapePlot {
                    center: (cx, cy),
                    // The +1 must saturate: a 2^63 radius arrives here as
                    // i64::MAX, and `r + 1` used to overflow-panic inside the
                    // host callback (dev: the panic escaped the sandbox as a
                    // host unwind; release: instant process abort).
                    radius: r.saturating_add(1),
                    // f64 subtraction: x - cx in i64 overflows for centers
                    // near ±i64::MAX, which scripts may pass.
                    filled_at: Box::new(move |x: i64, y: i64| {
                        let (dx, dy) = (x as f64 - cx as f64, y as f64 - cy as f64);
                        dx * dx + dy * dy <= radius2
                    }),
                    filled,
                    value,
                    what: "circle",
                },
            )
        },
    )
}

fn ellipse_function(lua: &Lua, draw: &Rc<Draw>, filled: bool) -> Result<Function, LuaError> {
    let draw = Rc::clone(draw);
    lua.create_function(
        move |_, (cx, cy, rx, ry, color): (f64, f64, f64, f64, Option<String>)| {
            let value = color_value(&color, "ellipse")?;
            let (cx, cy) = (integer(cx, "ellipse cx")?, integer(cy, "ellipse cy")?);
            let (rx, ry) = (integer(rx, "ellipse rx")?, integer(ry, "ellipse ry")?);
            if rx < 0 || ry < 0 {
                return Err(runtime_error(format!(
                    "ellipse: radii must be >= 0, got {rx}, {ry}"
                )));
            }
            let norm_x = rx as f64 + 0.5;
            let norm_y = ry as f64 + 0.5;
            plot_shape(
                &draw,
                ShapePlot {
                    center: (cx, cy),
                    // Saturating like the circle radius: 2^63 radii must be
                    // rejected by the cap, not overflow-panic here.
                    radius: rx.max(ry).saturating_add(1),
                    filled_at: Box::new(move |x: i64, y: i64| {
                        let (dx, dy) = (x as f64 - cx as f64, y as f64 - cy as f64);
                        (dx * dx) / (norm_x * norm_x) + (dy * dy) / (norm_y * norm_y) <= 1.0
                    }),
                    filled,
                    value,
                    what: "ellipse",
                },
            )
        },
    )
}

/// A shape to plot: membership test plus paint style.
struct ShapePlot<'a> {
    center: (i64, i64),
    radius: i64,
    filled_at: Box<dyn Fn(i64, i64) -> bool + 'a>,
    filled: bool,
    value: Option<String>,
    what: &'a str,
}

/// Plots a shape whose membership test is `filled_at`; outline mode keeps
/// only filled pixels touching an unfilled neighbor. The scan is intersected
/// with the canvas and painted inline: iterating the raw radius box would
/// allocate and loop in proportion to a script-controlled radius, and the
/// per-run budgets cannot interrupt a single host call mid-scan. Off-canvas
/// pixels were never painted, so clamping the box changes nothing.
fn plot_shape(draw: &Draw, shape: ShapePlot<'_>) -> Result<(), LuaError> {
    let ShapePlot {
        center: (cx, cy),
        radius,
        filled_at,
        filled,
        value,
        what,
    } = shape;
    // A legitimate fill needs at most the canvas diagonal; the cap keeps a
    // hostile (or wrapped) radius from turning into an unbounded scan. The
    // plotted radius is the script's radius plus one, so the script-facing
    // bound is one lower.
    let radius_cap = (draw.width.max(draw.height) * 4) as i64;
    if radius < 0 || radius > radius_cap {
        return Err(runtime_error(format!(
            "{what}: radius must be within 0..={script_cap} (canvas {}x{}), got {radius}; draw inside the canvas instead",
            draw.width,
            draw.height,
            script_cap = radius_cap.saturating_sub(1)
        )));
    }
    let left = cx.saturating_sub(radius).max(0);
    let right = cx.saturating_add(radius).min(draw.width as i64 - 1);
    let top = cy.saturating_sub(radius).max(0);
    let bottom = cy.saturating_add(radius).min(draw.height as i64 - 1);
    for y in top..=bottom {
        for x in left..=right {
            if !filled_at(x, y) {
                continue;
            }
            let exposed = ![(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
                .iter()
                .all(|&(neighbor_x, neighbor_y)| filled_at(neighbor_x, neighbor_y));
            if filled || exposed {
                draw.set(x as usize, y as usize, value.clone(), what)?;
            }
        }
    }
    Ok(())
}

/// Liang–Barsky clip of an integer segment to the inclusive box
/// `x_min..=x_max × y_min..=y_max`. Returns the clipped endpoints, or `None`
/// when the segment misses the box entirely. All comparisons and products run
/// in `i128`: script-controlled endpoints may sit near ±i64::MAX where naive
/// `i64` math would overflow. Endpoints already inside the box pass through
/// unchanged (t0 = 0, t1 = 1).
fn clip_segment(
    (x0, y0): (i64, i64),
    (x1, y1): (i64, i64),
    x_min: i64,
    y_min: i64,
    x_max: i64,
    y_max: i64,
) -> Option<(i64, i64, i64, i64)> {
    // Keep the cross-products far from the i128 ceiling for script-supplied
    // ±i64::MAX endpoints. Coordinates this far outside the canvas are
    // garbage anyway; clamping them leaves every realistic segment exact
    // while the clipped result stays near the canvas.
    const COORD_CLAMP: i64 = 1 << 40;
    let clamp_coord = |value: i64| value.clamp(-COORD_CLAMP, COORD_CLAMP);
    let (x0, y0, x1, y1) = (
        clamp_coord(x0),
        clamp_coord(y0),
        clamp_coord(x1),
        clamp_coord(y1),
    );
    let dx = (x1 as i128) - (x0 as i128);
    let dy = (y1 as i128) - (y0 as i128);
    // Edge inequalities in parametric form p·t <= q over t in [0, 1].
    let edges = [
        (-(dx), (x0 as i128) - (x_min as i128)), // x >= x_min
        (dx, (x_max as i128) - (x0 as i128)),    // x <= x_max
        (-(dy), (y0 as i128) - (y_min as i128)), // y >= y_min
        (dy, (y_max as i128) - (y0 as i128)),    // y <= y_max
    ];
    // t0/t1 as exact rationals (numerator, positive denominator).
    let (mut t0, mut t0d) = (0i128, 1i128);
    let (mut t1, mut t1d) = (1i128, 1i128);
    for (p, q) in edges {
        if p == 0 {
            if q < 0 {
                return None;
            }
        } else if p > 0 {
            // Leaving boundary: t <= q/p.
            if q * t1d < t1 * p {
                t1 = q;
                t1d = p;
            }
        } else {
            // Entering boundary: t >= q/p (denominator made positive).
            let (p, q) = (-p, -q);
            if q * t0d > t0 * p {
                t0 = q;
                t0d = p;
            }
        }
    }
    if t0 * t1d > t1 * t0d {
        return None;
    }
    let endpoint = |start: i64, delta: i128, (tn, td): (i128, i128), low: i64, high: i64| -> i64 {
        let value = (start as i128) + delta * tn / td;
        value.clamp(low as i128, high as i128) as i64
    };
    Some((
        endpoint(x0, dx, (t0, t0d), x_min, x_max),
        endpoint(y0, dy, (t0, t0d), y_min, y_max),
        endpoint(x0, dx, (t1, t1d), x_min, x_max),
        endpoint(y0, dy, (t1, t1d), y_min, y_max),
    ))
}

fn bresenham(x0: i64, y0: i64, x1: i64, y1: i64) -> impl Iterator<Item = (i64, i64)> {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    let mut x = x0;
    let mut y = y0;
    let mut done = false;
    std::iter::from_fn(move || {
        // The endpoint must be yielded: a missing end pixel leaves gaps that
        // a flood fill can leak through.
        if done {
            return None;
        }
        let current = (x, y);
        if x == x1 && y == y1 {
            done = true;
        } else {
            let doubled = 2 * error;
            if doubled >= dy {
                error += dy;
                x += step_x;
            }
            if doubled <= dx {
                error += dx;
                y += step_y;
            }
        }
        Some(current)
    })
}

fn classify(error: &LuaError, limits: &ShaderLimits) -> ShaderError {
    match error {
        // Errors raised inside Rust callbacks (canvas functions, budgets) are
        // wrapped with a traceback; recurse into the cause.
        LuaError::CallbackError { cause, .. } => classify(cause, limits),
        LuaError::SyntaxError { message, .. } => {
            let (line, text) = extract_line(message);
            ShaderError::Syntax {
                line,
                message: text,
            }
        }
        LuaError::MemoryError(_) => ShaderError::MemoryBudget(limits.max_memory),
        LuaError::RuntimeError(message) => {
            // Hook-raised errors can come back wrapped ("runtime error: <msg>"
            // plus a trailing "stack traceback:" section).
            let normalized = message
                .strip_prefix("runtime error: ")
                .unwrap_or(message.as_str())
                .split("\nstack traceback")
                .next()
                .unwrap_or_default()
                .trim_end()
                .to_string();
            if let Some(detail) = normalized.strip_prefix(HOST_ERROR) {
                if let Some(rest) = detail.strip_prefix("instruction budget of ") {
                    let number = rest
                        .split(' ')
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(limits.max_instructions);
                    return ShaderError::InstructionBudget(number);
                }
                if detail.starts_with("wall-clock budget") {
                    return ShaderError::WallClockBudget(limits.max_duration);
                }
                if let Some(rest) = detail.strip_prefix("changed-pixel budget of ") {
                    let number = rest
                        .split(' ')
                        .next()
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(limits.max_changed_pixels);
                    return ShaderError::ChangedPixelBudget(number);
                }
            }
            let (line, text) = extract_line(&normalized);
            ShaderError::Runtime {
                line,
                message: text,
            }
        }
        other => ShaderError::Runtime {
            line: 0,
            message: other.to_string(),
        },
    }
}

/// Extracts a `script:N:` / `[string "script"]:N:` line prefix from a Lua
/// error message, returning `(line, message-without-prefix)`.
fn extract_line(message: &str) -> (u32, String) {
    if let Some(index) = message.find("script:") {
        if let Some(parsed) = parse_line_prefix(&message[index + "script:".len()..]) {
            return parsed;
        }
    }
    if let Some(index) = message.find("\"]:") {
        if let Some(parsed) = parse_line_prefix(&message[index + "\"]:".len()..]) {
            return parsed;
        }
    }
    (0, message.to_string())
}

fn parse_line_prefix(rest: &str) -> Option<(u32, String)> {
    let digits_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits_end == 0 || !rest[digits_end..].starts_with(':') {
        return None;
    }
    let line = rest[..digits_end].parse::<u32>().ok()?;
    Some((line, rest[digits_end + 1..].trim_start().to_string()))
}

#[cfg(test)]
mod tests {
    use super::extract_line;

    #[test]
    fn extracts_lua_line_prefixes() {
        assert_eq!(
            extract_line("script:12: attempt to index a nil value"),
            (12, "attempt to index a nil value".to_string())
        );
        assert_eq!(
            extract_line("[string \"script\"]:3: unexpected symbol near ')'",),
            (3, "unexpected symbol near ')'".to_string())
        );
        assert_eq!(
            extract_line("no position here"),
            (0, "no position here".to_string())
        );
    }
}

#[cfg(test)]
mod function_tests {
    use crate::{run_script, ShaderCanvas, ShaderEnv, ShaderError, ShaderLimits};
    use std::time::Duration;

    fn env(seed: u64) -> ShaderEnv {
        ShaderEnv::new(
            vec![vec!["#FF0000".into(), "#00FF00".into(), "#0000FF80".into()]],
            0,
            seed,
        )
    }

    fn canvas() -> ShaderCanvas {
        ShaderCanvas::new(16, 16)
    }

    fn run(code: &str) -> Result<ShaderCanvas, ShaderError> {
        run_script(code, &canvas(), &env(7), &ShaderLimits::default()).map(|outcome| {
            let mut canvas = canvas();
            canvas.pixels = outcome.pixels;
            canvas
        })
    }

    #[test]
    fn pset_pget_and_transparent_alpha() {
        let canvas = run("canvas.pset(1, 2, '#ff0000')\n\
             canvas.pset(3, 4, hex('#00ff0000'))\n\
             canvas.pset(5, 6, nil)")
        .unwrap();
        assert_eq!(canvas.pget(1, 2), Some("#FF0000FF"));
        // alpha-00 erases; canvas starts transparent.
        assert_eq!(canvas.pget(3, 4), None);
        assert_eq!(canvas.pget(5, 6), None);
    }

    #[test]
    fn pset_out_of_bounds_is_an_error() {
        let error = run("canvas.pset(16, 0, '#FFFFFF')").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("pset: (16, 0) is outside the 16x16 canvas"),
            "{error}"
        );
        let error = run("canvas.pset(-1, 0, '#FFFFFF')").unwrap_err();
        assert!(error.to_string().contains("pset x must be >= 0"), "{error}");
    }

    #[test]
    fn hostile_shape_radius_fails_fast() {
        // A huge radius used to scan (and allocate marks for) the whole
        // radius box before any clip or budget check, which aborted the
        // process. It must now surface as an ordinary script error.
        let error = run("canvas.circfill(8, 8, 10000000, pal(1))").unwrap_err();
        assert!(
            error.to_string().contains("radius must be within"),
            "{error}"
        );
        let error = run("canvas.ellipsefill(8, 8, 10000000, 3, pal(1))").unwrap_err();
        assert!(
            error.to_string().contains("radius must be within"),
            "{error}"
        );
        // Legitimate fills larger than the canvas keep working.
        run("canvas.circfill(8, 8, 32, pal(1))").unwrap();
    }

    #[test]
    fn hostile_2_63_radii_are_script_errors_not_panics() {
        // The plotted radius used to be computed as `r + 1`, which overflows
        // i64 for radii at 2^63 (or math.maxinteger) and panicked inside the
        // host callback: dev builds escaped the panic into a host unwind and
        // release builds aborted. These must fail fast with a message.
        for call in [
            "canvas.circle(8, 8, 2^63, pal(1))",
            "canvas.circfill(8, 8, 2^63, pal(1))",
            "canvas.circle(8, 8, math.maxinteger, pal(1))",
            "canvas.circfill(8, 8, math.maxinteger, pal(1))",
            "canvas.ellipse(8, 8, 2^63, 1, pal(1))",
            "canvas.ellipsefill(8, 8, 1, 2^63, pal(1))",
            "canvas.ellipse(8, 8, math.maxinteger, math.maxinteger, pal(1))",
        ] {
            let error = run(call).unwrap_err();
            assert!(
                error.to_string().contains("radius must be within"),
                "{call}: {error}"
            );
        }
    }

    #[test]
    fn callback_panics_cannot_escape_the_sandbox() {
        // mlua wraps a host-callback panic as an opaque Lua value that a
        // script pcall can capture and hand back to the host (chunk return,
        // rethrow, callback argument, gc finalizer). The host used to resume
        // that panic outside the crate, aborting the app with "panic in a
        // function that cannot unwind". Every variant must now surface as an
        // ordinary (or successful-but-contained) sandbox result — never an
        // unwinding panic.
        for code in [
            "canvas.circfill(8, 8, 2^63, pal(1))",
            "local ok, err = pcall(canvas.circfill, 8, 8, 2^63, pal(1))\nreturn err",
            "local ok, err = pcall(canvas.circfill, 8, 8, 2^63, pal(1))\nerror(err, 0)",
            "local ok, err = pcall(canvas.circfill, 8, 8, 2^63, pal(1))\ncanvas.pset(0, 0, err)",
            "local ok, err = pcall(canvas.circfill, 8, 8, 2^63, pal(1))\nlocal s = tostring(err)\ncanvas.pset(0, 0, s)",
            "local trap = setmetatable({}, {__gc = function() canvas.circfill(8, 8, 2^63, pal(1)) end})\n_G.keep = trap\nlocal ok, err = pcall(canvas.circfill, 8, 8, 2^63, pal(1))\nerror(err, 0)",
        ] {
            // The returned Result is deliberately unused: this test asserts
            // the call neither panics nor aborts the process.
            let _ = run_script(code, &canvas(), &env(1), &ShaderLimits::default());
        }
        // The direct (un-pcall'd) radius overflow reports the radius cap.
        let error = run("canvas.circfill(8, 8, 2^63, pal(1))").unwrap_err();
        assert!(
            error.to_string().contains("radius must be within"),
            "{error}"
        );
    }

    #[test]
    fn far_line_clips_to_the_canvas() {
        // A diagonal into far-off space must draw its on-canvas part and
        // stop instead of walking the staircase for eons.
        let canvas = run("canvas.line(0, 0, 1e15, 1e15, pal(1))").unwrap();
        assert_eq!(canvas.pget(7, 7), Some("#FF0000FF"));
        assert_eq!(canvas.pget(0, 3), None);
        // A segment that misses the canvas entirely draws nothing.
        let canvas = run("canvas.line(-1e15, -1e15, 1e15, -1e15, pal(1))").unwrap();
        assert!(canvas.pixels.iter().all(|p| p.is_none()));
    }

    #[test]
    fn rand_extreme_bounds_do_not_panic() {
        // The span computation used to overflow i64 for near-full-range
        // bounds; a wrapped zero span divided by zero and aborted.
        run("local r = rand(-9e18, 9e18)\nassert(r >= -9e18 and r <= 9e18)").unwrap();
    }

    #[test]
    fn line_draws_and_clips() {
        let canvas = run("canvas.line(0, 0, 20, 0, pal(1))").unwrap();
        for x in 0..16 {
            assert_eq!(canvas.pget(x, 0), Some("#FF0000FF"), "x={x}");
        }
        assert_eq!(canvas.pget(0, 1), None);
    }

    #[test]
    fn rectfill_and_rect() {
        let canvas = run("canvas.rectfill(2, 2, 3, 3, pal(2))").unwrap();
        for y in 2..5 {
            for x in 2..5 {
                assert_eq!(canvas.pget(x, y), Some("#00FF00FF"), "({x},{y})");
            }
        }
        assert_eq!(canvas.pget(5, 5), None);
        // Outline rect: corners and edges only.
        let canvas = run("canvas.rect(2, 2, 3, 3, pal(1))").unwrap();
        assert_eq!(canvas.pget(2, 2), Some("#FF0000FF"));
        assert_eq!(canvas.pget(3, 3), None);
        assert_eq!(canvas.pget(4, 2), Some("#FF0000FF"));
    }

    #[test]
    fn circfill_matches_the_documented_pixel_count() {
        let canvas = run("canvas.circfill(8, 8, 3, pal(1))").unwrap();
        let filled = canvas.pixels.iter().filter(|p| p.is_some()).count();
        assert_eq!(filled, 37);
        // Outline keeps the edge only.
        let outlined = run("canvas.circle(8, 8, 3, pal(1))").unwrap();
        let ring = outlined.pixels.iter().filter(|p| p.is_some()).count();
        assert!(ring < filled && ring > 0, "ring={ring} filled={filled}");
        assert_eq!(outlined.pget(8, 8), None);
    }

    #[test]
    fn ellipsefill_produces_a_symmetric_disc() {
        let canvas = run("canvas.ellipsefill(8, 8, 6, 3, pal(1))").unwrap();
        assert_eq!(canvas.pget(8, 8), Some("#FF0000FF"));
        assert_eq!(canvas.pget(2, 8), Some("#FF0000FF"));
        assert_eq!(canvas.pget(14, 8), Some("#FF0000FF"));
        assert_eq!(canvas.pget(8, 11), Some("#FF0000FF"));
        assert_eq!(canvas.pget(2, 11), None);
    }

    #[test]
    fn flood_fills_connected_region_only() {
        // Fill everything green, draw a 4-connected diagonal wall (two lines),
        // flood one side.
        let canvas = run("canvas.rectfill(0, 0, 16, 16, pal(2))\n\
             canvas.line(0, 0, 15, 15, pal(1))\n\
             canvas.line(1, 0, 15, 14, pal(1))\n\
             canvas.flood(0, 15, pal(3))")
        .unwrap();
        // The lower-left triangle (below the wall) turned translucent blue;
        // the upper-right triangle stays green.
        assert_eq!(canvas.pget(0, 15), Some("#0000FF80"));
        assert_eq!(canvas.pget(0, 14), Some("#0000FF80"));
        assert_eq!(canvas.pget(15, 0), Some("#00FF00FF"));
        assert_eq!(canvas.pget(3, 0), Some("#00FF00FF"));
    }

    #[test]
    fn replace_and_outline() {
        let canvas = run("canvas.circfill(8, 8, 4, pal(1))\n\
             canvas.replace('#FF0000FF', '#00FF00FF')\n\
             canvas.outline(pal(3))")
        .unwrap();
        assert_eq!(canvas.pget(8, 8), Some("#00FF00FF"));
        // A pixel adjacent to the disc's edge is outlined blue.
        let outlined = (0..16).any(|x| {
            canvas.pget(x, 3) == Some("#0000FF80") || canvas.pget(x, 13) == Some("#0000FF80")
        });
        assert!(outlined, "outline must touch the disc boundary");
    }

    #[test]
    fn clear_uses_budget_like_any_draw() {
        let canvas = run("canvas.clear(pal(1))").unwrap();
        assert_eq!(canvas.pget(0, 0), Some("#FF0000FF"));
        assert_eq!(canvas.pget(15, 15), Some("#FF0000FF"));
    }

    #[test]
    fn stamp_overlays_and_reports_row_problems() {
        let canvas = run("canvas.stamp({'..X.', '.XX.', '..X.'}, {X = '#FF0000'}, 6, 6)").unwrap();
        assert_eq!(canvas.pget(8, 6), Some("#FF0000FF"));
        assert_eq!(canvas.pget(6, 6), None);

        let error = run("canvas.stamp({'XX', 'X'}, {X = '#FF0000'}, 0, 0)").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("row 2 has 1 characters but row 1 has 2"),
            "{error}"
        );
        let error = run("canvas.stamp({'YY'}, {X = '#FF0000'}, 0, 0)").unwrap_err();
        assert!(
            error.to_string().contains("missing from the legend"),
            "{error}"
        );
    }

    #[test]
    fn sandbox_hides_dangerous_globals() {
        let error = run("io.write('x')").unwrap_err();
        assert!(
            error.to_string().contains("attempt to index a nil value"),
            "{error}"
        );
        let error = run("os.exit(1)").unwrap_err();
        assert!(
            error.to_string().contains("attempt to index a nil value"),
            "{error}"
        );
        let error = run("load('return 1')").unwrap_err();
        assert!(
            error.to_string().contains("attempt to call a nil value"),
            "{error}"
        );
        let error = run("require('io')").unwrap_err();
        assert!(
            error.to_string().contains("attempt to call a nil value"),
            "{error}"
        );
    }

    #[test]
    fn bare_canvas_calls_get_a_corrective_error() {
        // The model keeps writing canvas functions as bare globals; the error
        // must state the correct spelling instead of a raw nil-value call.
        let error = run("pset(0, 0, '#FF0000FF')").unwrap_err();
        assert!(error.to_string().contains("canvas.pset(...)"), "{error}");
        // Unknown names keep the ordinary nil-call error; existing globals
        // (including nil'ed math.random) behave as before.
        let error = run("unknown_global_fn()").unwrap_err();
        assert!(
            error.to_string().contains("attempt to call a nil value"),
            "{error}"
        );
        run("assert(math.random == nil)\nassert(canvas ~= nil)").unwrap();
    }

    #[test]
    fn instruction_budget_aborts_infinite_loops() {
        let limits = ShaderLimits {
            max_instructions: 1_000_000,
            ..ShaderLimits::default()
        };
        let error = run_script("while true do end", &canvas(), &env(1), &limits).unwrap_err();
        assert!(
            matches!(error, ShaderError::InstructionBudget(1_000_000)),
            "{error}"
        );
    }

    #[test]
    fn memory_budget_aborts_allocation_bombs() {
        let limits = ShaderLimits {
            max_memory: 512 * 1024,
            ..ShaderLimits::default()
        };
        let error = run_script(
            "local t = {}\nwhile true do t[#t + 1] = string.rep('x', 4096) end",
            &canvas(),
            &env(1),
            &limits,
        )
        .unwrap_err();
        assert!(matches!(error, ShaderError::MemoryBudget(_)), "{error}");
    }

    #[test]
    fn changed_pixel_budget_aborts_huge_draws() {
        let limits = ShaderLimits {
            max_changed_pixels: 10,
            ..ShaderLimits::default()
        };
        let error = run_script(
            "for y = 0, 15 do for x = 0, 15 do canvas.pset(x, y, pal(1)) end end",
            &canvas(),
            &env(1),
            &limits,
        )
        .unwrap_err();
        assert!(
            matches!(error, ShaderError::ChangedPixelBudget(10)),
            "{error}"
        );
    }

    #[test]
    fn syntax_errors_carry_line_numbers() {
        let error = run("canvas.pset(0, 0").unwrap_err();
        assert!(
            matches!(error, ShaderError::Syntax { line: 1, .. }),
            "{error}"
        );
    }

    #[test]
    fn same_seed_same_output_different_seed_different_output() {
        let script = "for y = 0, 15 do for x = 0, 15 do\n\
            if rand() > 0.5 then canvas.pset(x, y, pal(1)) end\nend end";
        let outcome_a = run_script(script, &canvas(), &env(42), &ShaderLimits::default()).unwrap();
        let outcome_b = run_script(script, &canvas(), &env(42), &ShaderLimits::default()).unwrap();
        assert_eq!(outcome_a.pixels, outcome_b.pixels);
        let outcome_c = run_script(script, &canvas(), &env(43), &ShaderLimits::default()).unwrap();
        assert_ne!(outcome_a.pixels, outcome_c.pixels);
    }

    #[test]
    fn noise_is_stable_and_pal_is_one_based() {
        let script = "assert(pal(1) == '#FF0000FF')\nassert(pal(3) == '#0000FF80')\nlocal v = noise(0.5, 0.5)\nassert(v >= 0 and v < 1)\nassert(noise(0.5, 0.5) == v)";
        run(script).unwrap();
        let error = run("pal(4)").unwrap_err();
        assert!(
            error.to_string().contains("outside the active palette"),
            "{error}"
        );
    }

    #[test]
    fn math_library_works_and_random_is_disabled() {
        // Full math library, including string-built colors from computed channels.
        run("assert(math.floor(math.sin(0.5) * 100) >= 0)\n\
             assert(math.abs(-3) == 3)\n\
             assert(math.pi > 3.14 and math.pi < 3.15)\n\
             assert(math.random == nil, 'math.random must be disabled')\n\
             assert(math.randomseed == nil, 'math.randomseed must be disabled')\n\
             local r = math.floor(255 * 0.5)\n\
             local c = string.format('#%02X%02X%02X', r, r, r)\n\
             canvas.pset(0, 0, c)\n\
             assert(canvas.pget(0, 0) == '#7F7F7FFF')")
        .unwrap();
    }

    #[test]
    fn color_helpers_enable_gradients_and_glow() {
        // A horizontal gradient row proves mix works inside loops with
        // variable colors.
        let canvas = run("for x = 0, 15 do\n\
               canvas.pset(x, 0, mix('#000000', '#FFFFFF', x / 15))\n\
             end\n\
             assert(canvas.pget(0, 0) == '#000000FF')\n\
             assert(canvas.pget(15, 0) == '#FFFFFFFF')\n\
             assert(canvas.pget(8, 0) == '#888888FF')\n\
             -- HSV + alpha for glow-style work.\n\
             canvas.pset(1, 1, hsv(120, 1, 1))\n\
             assert(canvas.pget(1, 1) == '#00FF00FF')\n\
             canvas.pset(2, 2, alpha('#FF0000', 0.25))\n\
             assert(canvas.pget(2, 2) == '#FF000040')")
        .unwrap();
        assert_eq!(canvas.pget(8, 0), Some("#888888FF"));
    }

    #[test]
    fn wall_clock_budget_aborts_spin() {
        // A tight loop with a tiny instruction budget also trips quickly;
        // here we force the wall clock instead with a long-running loop that
        // stays under the instruction budget.
        let limits = ShaderLimits {
            max_instructions: u64::MAX / 2,
            max_duration: Duration::ZERO,
            ..ShaderLimits::default()
        };
        let error = run_script(
            "local s = 0\nfor i = 1, 2000000 do s = s + i end",
            &canvas(),
            &env(1),
            &limits,
        )
        .unwrap_err();
        assert!(matches!(error, ShaderError::WallClockBudget(_)), "{error}");
    }

    #[test]
    fn single_frame_run_has_shader_default_time_constants() {
        run(
            "assert(time == 0)\nassert(phase == 0)\nassert(frame_index == 0)\nassert(frame_count == 1)",
        )
        .unwrap();
    }

    #[test]
    fn timeline_renders_every_sample_with_its_time_constants() {
        use crate::run_script_timeline;
        let env = ShaderEnv::new(vec![vec!["#FF0000".into()]], 0, 7);
        // The script paints a marker pixel whose position encodes the frame
        // index, and records the constants it observed.
        let script = "canvas.pset(frame_index * 2, 0, pal(1))\n\
             canvas.pset(0, 1 + frame_index, '#00FF00FF')\n\
             assert(math.abs(time - (frame_index * 0.5)) < 1e-9)\n\
             assert(math.abs(phase - (frame_index * 0.25)) < 1e-9)\n\
             assert(frame_count == 4)";
        let canvases = (0..4).map(|_| ShaderCanvas::new(8, 8)).collect::<Vec<_>>();
        let frames = run_script_timeline(
            script,
            &canvases,
            &env,
            &ShaderLimits::default(),
            &[(0.0, 0.0), (0.5, 0.25), (1.0, 0.5), (1.5, 0.75)],
        )
        .unwrap();
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0].time, 0.0);
        assert_eq!(frames[3].phase, 0.75);
        for (index, frame) in frames.iter().enumerate() {
            let view = ShaderCanvas {
                width: 8,
                height: 8,
                pixels: frame.pixels.clone(),
            };
            assert_eq!(view.pget(index * 2, 0), Some("#FF0000FF"), "frame {index}");
            assert_eq!(view.pget(0, 1 + index), Some("#00FF00FF"), "frame {index}");
        }
        // Frames are independent renders of the same base canvas: frame 3
        // must not contain frame 0's marker.
        let view3 = ShaderCanvas {
            width: 8,
            height: 8,
            pixels: frames[3].pixels.clone(),
        };
        assert_eq!(view3.pget(0, 1), None);
    }

    #[test]
    fn timeline_is_deterministic_and_rand_varies_per_frame() {
        use crate::{run_script_timeline, ShaderLimits};
        let script = "for x = 0, 7 do\n\
               if rand() > 0.5 then canvas.pset(x, 0, pal(1)) end\n\
             end";
        let render = |seed: u64| {
            let env = ShaderEnv::new(vec![vec!["#FF0000".into()]], 0, seed);
            let canvases = (0..3).map(|_| ShaderCanvas::new(8, 8)).collect::<Vec<_>>();
            run_script_timeline(
                script,
                &canvases,
                &env,
                &ShaderLimits::default(),
                &[(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)],
            )
            .unwrap()
        };
        let a = render(11);
        let b = render(11);
        for (frame_a, frame_b) in a.iter().zip(b.iter()) {
            assert_eq!(frame_a.pixels, frame_b.pixels, "same seed must replay");
        }
        // Different frames get different seeds: grain must differ per frame
        // (with overwhelming probability across 8 pixels x 2 pairs).
        let differs = a[0].pixels != a[1].pixels || a[1].pixels != a[2].pixels;
        assert!(differs, "rand() must vary between timeline frames");
    }

    #[test]
    fn timeline_shares_the_wall_clock_deadline() {
        use crate::{run_script_timeline, ShaderLimits};
        let limits = ShaderLimits {
            max_instructions: u64::MAX / 2,
            max_duration: Duration::ZERO,
            ..ShaderLimits::default()
        };
        let canvases = [ShaderCanvas::new(8, 8), ShaderCanvas::new(8, 8)];
        let error = run_script_timeline(
            "local s = 0\nfor i = 1, 2000000 do s = s + i end",
            &canvases,
            &env(1),
            &limits,
            &[(0.0, 0.0), (0.5, 0.5)],
        )
        .unwrap_err();
        assert!(matches!(error, ShaderError::WallClockBudget(_)), "{error}");
    }

    #[test]
    fn timeline_requires_one_canvas_per_sample() {
        use crate::run_script_timeline;
        let error = run_script_timeline(
            "return",
            &[ShaderCanvas::new(8, 8)],
            &env(1),
            &ShaderLimits::default(),
            &[(0.0, 0.0), (0.5, 0.5)],
        )
        .unwrap_err();
        assert!(matches!(error, ShaderError::InvalidArgument(_)), "{error}");
    }
}
