#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use std::time::Duration;

mod color;
mod noise;
mod prng;
mod runtime;

/// A row-major pixel framebuffer. Every pixel is either transparent (`None`)
/// or a canonical `#RRGGBBAA` color string (uppercase).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderCanvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Option<String>>,
}

impl ShaderCanvas {
    /// Creates a fully transparent canvas.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![None; width.saturating_mul(height)],
        }
    }

    /// Builds a canvas from existing pixel data; the buffer length must match
    /// `width * height`.
    pub fn from_pixels(width: usize, height: usize, pixels: Vec<Option<String>>) -> Self {
        assert_eq!(
            pixels.len(),
            width.saturating_mul(height),
            "pixel buffer length must equal width*height"
        );
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Returns the pixel at (`x`, `y`), or `None` for transparent.
    pub fn pget(&self, x: usize, y: usize) -> Option<&str> {
        self.pixels
            .get(y.saturating_mul(self.width).saturating_add(x))
            .and_then(|value| value.as_deref())
    }
}

/// Host-provided environment: palettes available to `pal()` and the seed that
/// makes `rand`/`noise` deterministic. The same seed always produces the same
/// output for the same script and canvas.
///
/// The time fields let a script animate: `time` is the frame's start time in
/// seconds on the timeline, `phase` is `time / total` (in `[0, 1)`, matching
/// loop semantics), and `frame_index`/`frame_count` locate the frame. For a
/// single-frame run they are `0, 0, 0, 1`.
#[derive(Clone, Debug, Default)]
pub struct ShaderEnv {
    /// Palette colors in order; every color must already be canonical
    /// `#RRGGBBAA` (uppercase). Empty palettes make `pal()` unavailable.
    pub palettes: Vec<Vec<String>>,
    /// Index into `palettes` of the palette served by `pal(i)`.
    pub active_palette: usize,
    /// Seed for `rand` and `noise`.
    pub seed: u64,
    /// Frame start time in seconds on the timeline.
    pub time: f64,
    /// `time / total_duration`, in `[0, 1)`.
    pub phase: f64,
    /// Zero-based position of this frame on the timeline.
    pub frame_index: usize,
    /// Total number of frames being rendered.
    pub frame_count: usize,
}

impl ShaderEnv {
    pub fn new(palettes: Vec<Vec<String>>, active_palette: usize, seed: u64) -> Self {
        Self {
            palettes,
            active_palette,
            seed,
            ..Default::default()
        }
    }
}

/// Resource budgets for a script run. All limits are hard aborts that surface
/// as [`ShaderError::InstructionBudget`], [`ShaderError::MemoryBudget`],
/// [`ShaderError::WallClockBudget`], or [`ShaderError::ChangedPixelBudget`].
#[derive(Clone, Debug)]
pub struct ShaderLimits {
    /// Maximum Lua VM instructions.
    pub max_instructions: u64,
    /// Maximum Lua memory in bytes.
    pub max_memory: usize,
    /// Maximum wall-clock time.
    pub max_duration: Duration,
    /// Maximum number of pixels a script may change.
    pub max_changed_pixels: usize,
}

impl Default for ShaderLimits {
    fn default() -> Self {
        Self {
            max_instructions: 20_000_000,
            max_memory: 32 * 1024 * 1024,
            max_duration: Duration::from_secs(5),
            max_changed_pixels: 65_536,
        }
    }
}

/// The framebuffer after a successful run, plus the number of pixels whose
/// value differs from the input canvas.
#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub changed_pixels: usize,
    pub pixels: Vec<Option<String>>,
}

/// Everything that can go wrong in a script run. Messages are written for an
/// LLM to read and act on: syntax and runtime errors carry the script line.
#[derive(Debug, thiserror::Error)]
pub enum ShaderError {
    #[error("shader syntax error at line {line}: {message}")]
    Syntax { line: u32, message: String },
    #[error("shader error at line {line}: {message}")]
    Runtime { line: u32, message: String },
    #[error("shader exceeded its instruction budget ({0} Lua instructions); simplify the script or use fewer per-pixel operations")]
    InstructionBudget(u64),
    #[error("shader exceeded its memory budget ({0} bytes)")]
    MemoryBudget(usize),
    #[error("shader exceeded its wall-clock budget ({0:?})")]
    WallClockBudget(Duration),
    #[error("shader exceeded its changed-pixel budget ({0} pixels)")]
    ChangedPixelBudget(usize),
    #[error("{0}")]
    InvalidColor(String),
    #[error("{0}")]
    InvalidArgument(String),
}

/// Runs a Lua drawing script against a copy of `canvas` and returns the
/// modified buffer. The input canvas is never mutated.
///
/// # Example
///
/// ```
/// use pixel_shader::{run_script, ShaderCanvas, ShaderEnv, ShaderLimits};
///
/// let canvas = ShaderCanvas::new(8, 8);
/// let env = ShaderEnv::new(vec![vec!["#D1495B".into()]], 0, 7);
/// let outcome = run_script("canvas.circfill(4, 4, 3, pal(1))", &canvas, &env, &ShaderLimits::default()).unwrap();
/// assert_eq!(outcome.changed_pixels, 37);
/// ```
pub fn run_script(
    code: &str,
    canvas: &ShaderCanvas,
    env: &ShaderEnv,
    limits: &ShaderLimits,
) -> Result<RunOutcome, ShaderError> {
    if canvas.pixels.len() != canvas.width.saturating_mul(canvas.height) {
        return Err(ShaderError::InvalidArgument(
            "canvas pixel buffer length must equal width*height".to_string(),
        ));
    }
    if env.active_palette >= env.palettes.len() && !env.palettes.is_empty() {
        return Err(ShaderError::InvalidArgument(
            "active_palette is outside the palette list".to_string(),
        ));
    }
    runtime::execute(code, canvas, env, limits)
}

/// One rendered frame of a timeline run.
#[derive(Clone, Debug)]
pub struct FrameOutcome {
    /// The `time` value the frame was rendered with.
    pub time: f64,
    /// The `phase` value the frame was rendered with.
    pub phase: f64,
    pub changed_pixels: usize,
    pub pixels: Vec<Option<String>>,
}

/// Renders the script once per timeline sample into the matching canvas —
/// the shader equivalent of sampling `time`/`phase` at every frame of an
/// animation. `canvases[i]` is the base buffer for sample `times[i]` (the
/// frame's own cel, so existing content is preserved unless the script clears
/// it). Each frame runs in a fresh Lua state (frames have zero execution-order
/// dependence), with independent instruction/memory/changed-pixel budgets and
/// a per-frame seed derived from `env.seed` and the frame index, so `rand`
/// differs between frames yet replays identically. The whole call shares
/// `limits.max_duration` as a wall-clock deadline.
///
/// ```
/// use pixel_shader::{run_script_timeline, ShaderCanvas, ShaderEnv, ShaderLimits};
///
/// let canvases = [ShaderCanvas::new(8, 8), ShaderCanvas::new(8, 8)];
/// let env = ShaderEnv::new(vec![vec!["#D1495B".into()]], 0, 7);
/// let frames = run_script_timeline(
///     "canvas.circfill(4, math.floor(1 + phase * 5), 2, pal(1))",
///     &canvases, &env, &ShaderLimits::default(),
///     &[(0.0, 0.0), (0.5, 0.5)],
/// ).unwrap();
/// assert_eq!(frames.len(), 2);
/// assert_ne!(frames[0].pixels, frames[1].pixels, "the ball must move");
/// ```
pub fn run_script_timeline(
    code: &str,
    canvases: &[ShaderCanvas],
    env: &ShaderEnv,
    limits: &ShaderLimits,
    times: &[(f64, f64)],
) -> Result<Vec<FrameOutcome>, ShaderError> {
    if canvases.len() != times.len() {
        return Err(ShaderError::InvalidArgument(
            "one canvas per timeline sample is required".to_string(),
        ));
    }
    let started = std::time::Instant::now();
    let frame_count = times.len();
    let mut outcomes = Vec::with_capacity(frame_count);
    for (index, (time, phase)) in times.iter().enumerate() {
        // Shared wall-clock deadline: each frame gets the remaining time,
        // clamped by the per-frame limit.
        let remaining = limits
            .max_duration
            .checked_sub(started.elapsed())
            .unwrap_or_default();
        let frame_limits = ShaderLimits {
            max_duration: remaining.min(limits.max_duration),
            ..limits.clone()
        };
        let mut frame_env = env.clone();
        frame_env.time = *time;
        frame_env.phase = *phase;
        frame_env.frame_index = index;
        frame_env.frame_count = frame_count;
        // Per-frame seed: independent between frames, reproducible overall.
        let mut seeder =
            prng::SplitMix64::new(env.seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        frame_env.seed = seeder.next_u64();
        let outcome = run_script(code, &canvases[index], &frame_env, &frame_limits)?;
        outcomes.push(FrameOutcome {
            time: *time,
            phase: *phase,
            changed_pixels: outcome.changed_pixels,
            pixels: outcome.pixels,
        });
    }
    Ok(outcomes)
}
