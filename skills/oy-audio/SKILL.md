---
name: oy-audio
description: Generate and refine sound effects (SFX), UI feedback, game sounds, stylized foley, ambience, and seamless sound loops with deterministic procedural synthesis. Deliver playable WAV files and editable recipes. Use for sound design, not voice narration or full-song composition.
license: MIT
---

# Sound Effect Generation

Turn sound requirements into editable synthesis scripts and JSON recipes, then render actual audio offline. Use the Python/NumPy tools bundled with this skill; the original oy-audio application, model APIs, and developer-specific paths are not required.

## Workflow

1. Establish the purpose, defining sound characteristics, duration, one-shot or loop behavior, channels, and delivery format. For unspecified short effects, use 48 kHz, 16-bit PCM WAV. Prefer mono for game assets that will be spatialized, and stereo for ambience when appropriate. Choose duration to fit the sound rather than forcing every asset into a UI cue or musical bar.
2. Divide the sound into purposeful layers: attack/transient, body/resonance, texture/motion, and tail/space. Read [Sound design](references/sound-design.md) and apply the construction principles relevant to the request. Use user-provided samples when needed for complex realistic sounds. Describe the realism of synthesized material accurately; do not present layered noise as a field recording.
3. Read [Rendering and script API](references/rendering.md), then write a `build(ctx)` script and recipe in the task directory. Copy a relevant starting point from `assets/examples/` if useful, and adapt it to the request; the examples do not limit the supported sound categories. Expose meaningful parameters and use `ctx.rng` for reproducible variation.
4. Run `<python> <skill-root>/scripts/render.py <recipe.json> --out <output-dir> --name <name>` to produce the WAV. Runtime dependencies are Python 3.10+ and NumPy 2.x. Check with `--check`; install `scripts/requirements.txt` in an appropriate virtual environment if dependencies are missing. Locate all resources relative to the installed skill directory.
5. Inspect the returned `analysis` and `warnings` for duration, audible content, peak headroom, tails, mono compatibility, and loop boundaries. When playback is available, assess recognizability, harshness, timing, tails, and at least three consecutive loop repetitions. Passing numeric checks does not establish perceptual quality; state when only numeric checks were possible.
6. Adjust the layer responsible for an observed issue instead of repeatedly regenerating random variations without a reason. Change one main sound dimension per iteration. Finish one asset first unless the user requests alternatives. Deliver a WAV link or player together with the recipe, script, and analysis report; briefly state duration and useful production parameters.

## Essential constraints

- Use a fixed seed and sample clock so the same parameters reproduce the sound. Variants should retain the same timbral structure, with deliberate changes to timing, pitch, texture, or intensity.
- Integrate frequency to obtain phase for pitch sweeps. A varying `frequency(t) * t` is not a correct substitute. Preserve filter and oscillator state across processing blocks to avoid periodic artifacts from resets.
- Allocate real time for decay, echoes, and reverb. The renderer rejects layers that exceed the render interval; do not fix overflow by cutting off tails. The default 3 ms edge fade prevents clicks but does not replace a complete release.
- Loops need continuous motion and compatible energy at the boundary. `loop_crossfade_ms` requests extra synthesis time while preserving the delivered duration; loop mode does not apply one-shot fades. Do not use arbitrary crossfades to hide timing errors in discrete events such as speech, beats, or impacts.
- Peak normalization defaults to -6 dBFS and can be adjusted for the intended use. Set `peak_dbfs: null` and control layer gains when relative intensity across assets must be preserved; normalizing each variant independently erases level differences. Reported RMS is not LUFS, and peaks are sample peaks rather than true peaks.
- Synthesis scripts are ordinary Python with the current process's permissions, not sandboxed code. Read externally supplied scripts before executing them. Do not assume the original application's Lua sandbox or internal tool names are available here.
- Preserve source samples and existing deliverables; give new versions new filenames. When suitable conversion tools are available, convert WAV to requested formats such as MP3, OGG, or FLAC and check the result. Prefer PCM WAV for strict looping to avoid codec padding at boundaries.
