# Rendering and Script API

## Running the renderer

Requires Python 3.10+ and NumPy 2.x. No API key, SoundFont, GPU, or Rust installation is required. Replace the placeholders below with the actual Python and installed skill paths. In PowerShell, use `&` to invoke an executable path containing spaces.

```sh
python -m pip install -r <skill-root>/scripts/requirements.txt
python <skill-root>/scripts/render.py --check
python <skill-root>/scripts/render.py /work/impact.json --out /work/output --name impact-v1
python <skill-root>/scripts/render.py --analyze /work/output/impact-v1.wav
```

Script and sample paths are resolved relative to the recipe directory. `--out` is resolved relative to the caller's working directory; results contain absolute output paths. Non-ASCII paths are supported. `--name` accepts 1-80 ASCII letters, digits, underscores, or hyphens, must start with a letter or digit, and must not be a Windows device name. Existing outputs are protected by default; use `--overwrite` only when replacing that specific version is intended.

Outputs are `<name>.wav` and `<name>.json`. The report contains the effective recipe, hashes of the original recipe/script/input samples, the WAV hash, runtime versions, normalization gain, analysis, and warnings. Deliver the original recipe and script for subsequent editing; the report itself is not an executable recipe. Identical sources, parameters, seed, and runtime environment reproduce the same PCM. Byte-identical output across NumPy versions or platforms is not guaranteed.

## JSON recipe

```json
{
  "version": 1,
  "script": "impact.py",
  "duration": 1.4,
  "sample_rate": 48000,
  "channels": 1,
  "seed": 42,
  "params": {"weight": 1.0, "brightness": 2200},
  "peak_dbfs": -6,
  "fade_ms": 3,
  "loop_crossfade_ms": 0
}
```

| Field | Contract |
| --- | --- |
| `version` | Required; integer `1` |
| `script` | Required Python file path, absolute or relative to the recipe directory |
| `duration` | Required output duration, 0.01-300 seconds; frame count is `round(duration * sample_rate)` |
| `sample_rate` | Default 48000; supports 22050, 24000, 44100, 48000, and 96000 |
| `channels` | `1` or `2`, default `2`; mono averages the stereo bus before normalization |
| `seed` | Default `0`; integer from 0 through 2^64-1 |
| `params` | Default `{}`; JSON parameter object passed to the script |
| `peak_dbfs` | Default -6; accepts -60 through -0.1, or `null` to preserve levels; samples outside the PCM range cause an error instead of hard clipping |
| `fade_ms` | Default `3`; click-prevention fades at both ends of a one-shot, each no longer than half the duration |
| `loop_crossfade_ms` | Default `0` for a one-shot; positive values enable loop blending, spanning at least two frames and at most 1/4 of the duration; `fade_ms` is ignored in this mode |

A render may contain at most 14,400,000 output frames. Memory use grows with duration and layer size; split long ambience into sections or shorter loops. Unknown fields, NaN/Infinity, overflowing layers, and silent output cause errors. Correct the specific issue before retrying.

Loop mode renders an additional crossfade interval after the target frame count. It fades that continuation out over the beginning while fading the original beginning in; the exported frame count remains unchanged. Scripts should generate through `ctx.duration` rather than stopping early at `ctx.output_duration`. This improves boundary continuity but does not guarantee compatible event timing, timbre, or loudness across repetitions.

## Python contract

Define `build(ctx)`, fill the audio bus through `ctx.add(...)`, and return `None`. The renderer makes the `sfx` module importable; scripts can use `import numpy as np` and `from sfx import ...` directly. Save scripts as UTF-8. This interface is separate from the original oy-audio Lua runtime and project format.

```python
import numpy as np
from sfx import fade

def build(ctx):
    seconds = 0.22
    tone = ctx.tone(880, seconds, end_frequency=440, sweep="exponential")
    t = np.arange(len(tone)) / ctx.rate
    tone = fade(tone * np.exp(-t / 0.045), ctx.rate, 0.002, 0.015)
    ctx.add(tone, gain=0.35)
```

| Property or method | Behavior |
| --- | --- |
| `ctx.rate`, `ctx.frames`, `ctx.duration`, `ctx.t` | Actual synthesis interval, including any loop extension; `t` starts at zero and is measured in seconds |
| `ctx.output_frames`, `ctx.output_duration` | Final delivery interval |
| `ctx.params` | Recipe parameters; provide sensible defaults in the script for omitted values |
| `ctx.rng` | Independent NumPy PCG64 generator for each render; use methods such as `uniform`, `normal`, and `integers` |
| `ctx.tone(frequency=440, seconds=None, end_frequency=None, sweep="linear", waveform="sine")` | Mono waveform; omitted duration uses the whole synthesis interval; sweeps support `linear`/`exponential`; waveforms support `sine`/`saw`/`square`, with PolyBLEP antialiasing for saw and square |
| `ctx.noise(seconds=None, color="white")` | Mono `white`, `pink`, or `brown` noise normalized to unit RMS; peaks may exceed 1, so reduce gain when mixing |
| `ctx.add(audio, at=0, gain=1, position=0)` | Mix at an offset in seconds; accepts `(frames,)`, `(frames,1)`, or `(frames,2)`; mono uses equal-power pan from -1 left to +1 right, while stereo retains its placement; overflow causes an error |
| `ctx.load_wav(path)` | Returns a `(frames,channels)` floating-point array; accepts mono/stereo 8/16/24/32-bit integer PCM WAV at the recipe's sample rate; records the source path and hash |

For compressed sources, floating-point WAV, or mismatched sample rates, use an available conversion tool to create an integer PCM WAV copy at the required rate. For example, with FFmpeg: `ffmpeg -i source.flac -ar 48000 -acodec pcm_s24le source-48k.wav`. Do not automatically download audio sources or SoundFonts.

### `sfx` helpers

- `fade(audio, rate, attack=0.003, release=0.01)`: Apply linear fades measured in seconds; return a copy with unchanged length.
- `envelope(frames, rate, points)`: Piecewise linear envelope from `[(seconds, amplitude), ...]` with strictly increasing times; values before the first point and after the last point are zero. Multiply two-dimensional audio by `env[:, None]`.
- `pan(mono, position=0)`: Convert mono to stereo using equal-power panning; center places each channel at approximately -3 dB.
- `db_to_gain(db)`: Convert dB to linear gain.
- `filter_audio(audio, rate, cutoff, kind="lowpass", taps=513)`: Windowed FIR with static cutoff; use `highpass` or `bandpass` as needed. Bandpass cutoff is `[low_hz,high_hz]`. `taps` must be an odd integer from 3 through 8191; narrow low-frequency bands often need 2049-4097 taps. Length is preserved and group delay compensated, but ringing can precede and follow transients; allow envelope space on both sides. Implement dynamic filtering in the script rather than passing frequency arrays to this helper.
- `delay(audio, rate, seconds=0.12, feedback=0.3, repeats=3, wet=0.25)`: Finite echo train; returns an array extended by `round(seconds*rate)*repeats` frames. Reserve output time for the entire tail.
- `read_wav(path)`: Return `(audio, sample_rate)`; usually prefer `ctx.load_wav` so source provenance is recorded.

These helpers are building blocks for harmonics, modulation, noise, envelopes, and timed events. Scripts can also implement new DSP. When no example fits the request, construct a suitable sound rather than renaming an unrelated example.

## Analysis and revision

Reports analyze the quantized PCM actually written to disk. `peak_dbfs` and `rms_dbfs` measure sample peak and root-mean-square amplitude; they do not provide LUFS or true peak. Spectral centroid uses average channel power across at most 64 windows, so short transients and sparse events also need listening assessment. `mono_rms_loss_db` measures the level change after mono downmix. `boundary_jump` is the largest per-channel difference between the first and last samples, not a complete perceptual loop test.

- Acceptable peaks but rough sound: inspect source waveforms, excessive high frequencies, filter state, and masking between layers. Normalization does not repair distortion.
- Tail-energy warning: check whether envelopes and echoes finish; extend the render or end events earlier. Click-prevention fades cannot turn an abruptly cut tail into a natural release.
- Finite samples but apparently silent output: inspect active duration and level. Complete silence, complete mono cancellation, and silence after quantization are rejected.
- Loop boundaries: play at least three repetitions and check for clicks, periodic dips in loudness, and timing errors. Increase overlap, change the motion trajectory, or schedule events across periods as needed.
- Mechanical repetition: vary event pitch, spectral shape, intensity, or timing slightly using bounded, reproducible randomness. Keep timing changes within the permitted range when synchronization matters.
