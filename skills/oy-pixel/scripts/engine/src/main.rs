use pixel_shader::{run_script, run_script_timeline, ShaderCanvas, ShaderEnv, ShaderLimits};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::Duration;

const MAX_PIXELS: usize = 16_777_216;
fn yes() -> bool {
    true
}
fn opaque() -> f64 {
    1.0
}
fn seed() -> u64 {
    42
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Job {
    width: usize,
    height: usize,
    durations: Vec<u64>,
    palette: Vec<String>,
    #[serde(default = "seed")]
    seed: u64,
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Layer {
    name: String,
    code: String,
    #[serde(default = "opaque")]
    opacity: f64,
    #[serde(default = "yes")]
    visible: bool,
    #[serde(default)]
    base_frames: Vec<Vec<Option<String>>>,
}

#[derive(Serialize)]
struct Output {
    width: usize,
    height: usize,
    durations: Vec<u64>,
    frames: Vec<Vec<u8>>,
}

fn rgba(s: &str) -> Result<[u8; 4], String> {
    if ![7, 9].contains(&s.len())
        || !s.starts_with('#')
        || !s.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
    {
        return Err(format!("invalid color {s:?}; use #RRGGBB or #RRGGBBAA"));
    }
    let byte = |i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string());
    Ok([
        byte(1)?,
        byte(3)?,
        byte(5)?,
        if s.len() == 9 { byte(7)? } else { 255 },
    ])
}

fn render(job: Job) -> Result<Output, String> {
    if !(1..=1024).contains(&job.width) || !(1..=1024).contains(&job.height) {
        return Err("width/height must be 1..1024".into());
    }
    if job.durations.is_empty()
        || job.durations.len() > 2000
        || job.durations.iter().any(|d| !(1..=60000).contains(d))
    {
        return Err("use 1..2000 frame durations, each 1..60000 milliseconds".into());
    }
    let area = job.width * job.height;
    if job.layers.is_empty()
        || job.layers.len() > 128
        || area
            .saturating_mul(job.durations.len())
            .saturating_mul(job.layers.len())
            > MAX_PIXELS
    {
        return Err("use 1..128 layers and at most 16777216 width*height*frames*layers".into());
    }
    if job.palette.is_empty() || job.palette.len() > 1024 {
        return Err("palette must have 1..1024 colors".into());
    }
    for color in &job.palette {
        rgba(color)?;
    }
    let total: u64 = job.durations.iter().sum();
    let mut elapsed = 0;
    let times: Vec<_> = job
        .durations
        .iter()
        .map(|d| {
            let pair = (elapsed as f64 / 1000.0, elapsed as f64 / total as f64);
            elapsed += d;
            pair
        })
        .collect();
    let env = ShaderEnv::new(vec![job.palette], 0, job.seed);
    let mut frames = vec![vec![0u8; area * 4]; times.len()];
    for layer in job.layers {
        if !layer.opacity.is_finite() || !(0.0..=1.0).contains(&layer.opacity) {
            return Err(format!("{}: opacity must be 0..1", layer.name));
        }
        if layer.code.len() > 65536 {
            return Err(format!("{}: Lua code exceeds 65536 bytes", layer.name));
        }
        if !layer.base_frames.is_empty()
            && layer.base_frames.len() != 1
            && layer.base_frames.len() != times.len()
        {
            return Err(format!(
                "{}: baseFrames must contain 1 or frame-count buffers",
                layer.name
            ));
        }
        for pixels in &layer.base_frames {
            if pixels.len() != area {
                return Err(format!(
                    "{}: base frame size does not match canvas",
                    layer.name
                ));
            }
            for color in pixels.iter().flatten() {
                rgba(color)?;
            }
        }
        if !layer.visible || layer.opacity == 0.0 {
            continue;
        }
        let canvases: Vec<_> = (0..times.len())
            .map(|i| {
                let pixels = if layer.base_frames.is_empty() {
                    vec![None; area]
                } else {
                    layer.base_frames[if layer.base_frames.len() == 1 { 0 } else { i }].clone()
                };
                ShaderCanvas::from_pixels(job.width, job.height, pixels)
            })
            .collect();
        // Keep the original runtime budgets and deterministic per-frame seeds.
        let limits = ShaderLimits {
            max_duration: Duration::from_secs(if times.len() == 1 { 5 } else { 10 }),
            ..Default::default()
        };
        let rendered = if times.len() == 1 {
            vec![
                run_script(&layer.code, &canvases[0], &env, &limits)
                    .map_err(|e| format!("{}: {e}", layer.name))?
                    .pixels,
            ]
        } else {
            run_script_timeline(&layer.code, &canvases, &env, &limits, &times)
                .map_err(|e| format!("{}: {e}", layer.name))?
                .into_iter()
                .map(|v| v.pixels)
                .collect()
        };
        for (target, source) in frames.iter_mut().zip(rendered) {
            for (dest, color) in target.chunks_exact_mut(4).zip(source) {
                let Some(color) = color else { continue };
                let c = rgba(&color)?;
                let sa = c[3] as f64 / 255.0 * layer.opacity;
                let da = dest[3] as f64 / 255.0;
                let a = sa + da * (1.0 - sa);
                if a > 0.0 {
                    for k in 0..3 {
                        dest[k] = ((c[k] as f64 * sa + dest[k] as f64 * da * (1.0 - sa)) / a)
                            .round_ties_even() as u8;
                    }
                    dest[3] = (a * 255.0).round_ties_even() as u8;
                }
            }
        }
    }
    Ok(Output {
        width: job.width,
        height: job.height,
        durations: job.durations,
        frames,
    })
}

fn main() {
    let result = (|| -> Result<(), String> {
        let mut input = String::new();
        std::io::stdin()
            .take(64 * 1024 * 1024 + 1)
            .read_to_string(&mut input)
            .map_err(|e| e.to_string())?;
        if input.len() > 64 * 1024 * 1024 {
            return Err("input exceeds 64 MiB".into());
        }
        let job = serde_json::from_str(&input).map_err(|e| format!("invalid render job: {e}"))?;
        let output = render(job)?;
        serde_json::to_writer(std::io::stdout().lock(), &output).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = writeln!(std::io::stderr(), "{error}");
        std::process::exit(1);
    }
}
