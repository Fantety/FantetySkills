#!/usr/bin/env python3
"""Render a JSON recipe + trusted Python synthesis script to PCM16 WAV."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import runpy
import sys
import wave

try:
    import numpy as np
except ImportError:
    raise SystemExit("Missing NumPy. Install with: python -m pip install -r <skill-root>/scripts/requirements.txt")

from sfx import Context, audio_array, db_to_gain, fade, finite_number, read_wav

VERSION = "1.0.0"


def load_recipe(path):
    recipe = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(recipe, dict):
        raise ValueError("Recipe must be a JSON object")
    fields = {"version", "script", "duration", "sample_rate", "channels", "seed", "params", "peak_dbfs", "fade_ms", "loop_crossfade_ms"}
    unknown = recipe.keys() - fields
    if unknown:
        raise ValueError(f"Unknown recipe fields: {', '.join(sorted(unknown))}")
    if type(recipe.get("version")) is not int or recipe["version"] != 1:
        raise ValueError("Recipe version must be 1")
    if not isinstance(recipe.get("script"), str) or not recipe["script"].strip():
        raise ValueError("script must name a Python source file relative to the recipe")
    duration = finite_number(recipe.get("duration"), "duration", 0.01, 300)
    rate = recipe.get("sample_rate", 48000)
    if type(rate) is not int or rate not in (22050, 24000, 44100, 48000, 96000):
        raise ValueError("sample_rate must be 22050, 24000, 44100, 48000, or 96000")
    if round(duration * rate) > 14_400_000:
        raise ValueError("Render exceeds 14,400,000 frames; split long assets into sections")
    channels = recipe.get("channels", 2)
    if type(channels) is not int or channels not in (1, 2):
        raise ValueError("channels must be 1 or 2")
    seed = recipe.get("seed", 0)
    if type(seed) is not int or not 0 <= seed <= 2**64 - 1:
        raise ValueError("seed must be an unsigned 64-bit integer")
    params = recipe.get("params", {})
    if not isinstance(params, dict):
        raise ValueError("params must be an object")
    # JSON's nonstandard NaN/Infinity are rejected, including inside params.
    json.dumps(recipe, allow_nan=False)
    peak = recipe.get("peak_dbfs", -6)
    if peak is not None:
        peak = finite_number(peak, "peak_dbfs", -60, -0.1)
    fade_ms = finite_number(recipe.get("fade_ms", 3), "fade_ms", 0, duration * 500)
    crossfade = finite_number(recipe.get("loop_crossfade_ms", 0), "loop_crossfade_ms", 0, duration * 250)
    if crossfade and round(crossfade * rate / 1000) < 2:
        raise ValueError("loop_crossfade_ms must span at least two samples")
    return dict(version=1, script=recipe["script"], duration=duration, sample_rate=rate,
                channels=channels, seed=seed, params=params, peak_dbfs=peak,
                fade_ms=fade_ms, loop_crossfade_ms=crossfade)


def dbfs(value):
    return max(-120.0, 20 * math.log10(max(float(value), 1e-6)))


def analyze(audio, rate):
    audio = audio_array(audio)
    if audio.ndim == 1:
        audio = audio[:, None]
    peak = float(np.max(np.abs(audio)))
    rms = float(np.sqrt(np.mean(audio * audio)))
    # Average channel powers so anti-phase stereo does not disappear in analysis.
    size = min(2048, len(audio))
    starts = np.unique(np.linspace(0, len(audio) - size, min(64, max(1, len(audio) // size)), dtype=int))
    power = np.zeros(size // 2 + 1)
    window = np.hanning(size)
    for start in starts:
        spectrum = np.fft.rfft(audio[start:start + size] * window[:, None], axis=0)
        power += np.mean(np.abs(spectrum) ** 2, axis=1)
    hz = np.fft.rfftfreq(size, 1 / rate)
    centroid = float(np.sum(hz * power) / max(float(power.sum()), 1e-30))
    edge = min(len(audio), max(1, round(rate * 0.01)))
    mono_rms = float(np.sqrt(np.mean(np.mean(audio, axis=1) ** 2)))
    return {
        "frames": len(audio), "duration_seconds": len(audio) / rate,
        "sample_rate": rate, "channels": audio.shape[1],
        "peak_dbfs": dbfs(peak), "rms_dbfs": dbfs(rms),
        "crest_factor_db": dbfs(peak) - dbfs(rms),
        "dc_offset": np.mean(audio, axis=0).tolist(),
        "clipped_samples": int(np.count_nonzero(np.abs(audio) >= 1)),
        "near_full_scale_samples": int(np.count_nonzero(np.abs(audio) >= 0.999)),
        "spectral_centroid_hz": centroid,
        "start_rms_dbfs": dbfs(np.sqrt(np.mean(audio[:edge] ** 2))),
        "end_rms_dbfs": dbfs(np.sqrt(np.mean(audio[-edge:] ** 2))),
        "boundary_jump": float(np.max(np.abs(audio[-1] - audio[0]))),
        "mono_rms_loss_db": dbfs(mono_rms) - dbfs(rms),
    }


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(recipe_path, output_dir, name, overwrite=False):
    recipe_path = recipe_path.resolve()
    recipe = load_recipe(recipe_path)
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,79}", name) or name.upper() in {"CON", "PRN", "AUX", "NUL", *[f"COM{i}" for i in range(1, 10)], *[f"LPT{i}" for i in range(1, 10)]}:
        raise ValueError("name must be 1–80 ASCII letters, digits, underscores or hyphens; start with a letter/digit and avoid device names")
    script = (recipe_path.parent / recipe["script"]).resolve()
    if script.suffix.lower() != ".py" or not script.is_file():
        raise ValueError(f"Python script not found: {script}")
    output_dir = output_dir.resolve()
    wav_path = output_dir / f"{name}.wav"
    report_path = output_dir / f"{name}.json"
    protected = {recipe_path, script}
    if wav_path in protected or report_path in protected:
        raise ValueError("Output would replace the recipe or synthesis script")
    if not overwrite and (wav_path.exists() or report_path.exists()):
        raise FileExistsError("Output already exists; choose a new name or use --overwrite")
    rate = recipe["sample_rate"]
    overlap = round(recipe["loop_crossfade_ms"] * rate / 1000)
    ctx = Context(rate, recipe["duration"], recipe["seed"], recipe["params"], recipe_path.parent, overlap)
    # Source code is normal trusted Python, not a sandbox or a restricted DSL.
    namespace = runpy.run_path(str(script))
    if not callable(namespace.get("build")):
        raise ValueError("Script must define build(ctx) and fill ctx.buffer via ctx.add(...)")
    returned = namespace["build"](ctx)
    if returned is not None:
        raise ValueError("build(ctx) must return None; place audio with ctx.add(...)")
    signal = audio_array(ctx.buffer).copy()
    if signal.shape != (ctx.frames, 2):
        raise ValueError("Do not change ctx.buffer shape")
    if wav_path in ctx.sources or report_path in ctx.sources:
        raise ValueError("Output would replace an input sample")
    signal -= np.mean(signal, axis=0)
    if overlap:
        # Render one period plus overlap; blend its continuation into the head.
        # The exported period stays exactly output_frames long.
        weight = np.linspace(0, 1, overlap)[:, None]
        signal[:overlap] = signal[:overlap] * weight + signal[ctx.output_frames:] * (1 - weight)
        signal = signal[:ctx.output_frames]
    else:
        signal = fade(signal, rate, recipe["fade_ms"] / 1000, recipe["fade_ms"] / 1000)
    if recipe["channels"] == 1:
        signal = np.mean(signal, axis=1, keepdims=True)
    peak = float(np.max(np.abs(signal)))
    if peak < 1e-8:
        raise ValueError("Render is silent or nearly silent; check layers, placement, and mono cancellation")
    gain = 1.0 if recipe["peak_dbfs"] is None else db_to_gain(recipe["peak_dbfs"]) / peak
    signal *= gain
    if np.max(np.abs(signal)) >= 1:
        raise ValueError("Render would clip PCM output; lower gains or set peak_dbfs")
    pcm = np.rint(signal * 32767).astype("<i2")
    if not np.any(pcm):
        raise ValueError("Render becomes silent at 16-bit precision; increase its level")
    quantized = pcm.astype(np.float64) / 32768
    report = analyze(quantized, rate)
    warnings = []
    if report["rms_dbfs"] <= -70:
        warnings.append("Very low RMS; check whether most of the asset is unintended silence.")
    if not overlap and report["end_rms_dbfs"] > -35:
        warnings.append("High energy in the final 10 ms; inspect for an abruptly cut release or effect tail.")
    if overlap and report["boundary_jump"] > 0.02:
        warnings.append("Large loop boundary step; audition repeated playback and revise overlap/source continuity.")
    if report["mono_rms_loss_db"] < -6:
        warnings.append("Substantial mono cancellation; inspect stereo phase and low-frequency placement.")
    if any(abs(dc) > 0.005 for dc in report["dc_offset"]):
        warnings.append("Residual DC offset above 0.005 after finishing; inspect asymmetric waveforms/envelopes.")
    report = {
        "renderer_version": VERSION, "numpy_version": np.__version__, "recipe": recipe,
        "files": {"wav": str(wav_path), "report": str(report_path)},
        "sources": [{"path": str(p), "sha256": sha256(p)} for p in dict.fromkeys([recipe_path, script, *ctx.sources])],
        "normalization_gain_db": dbfs(gain), "pre_normalization_peak_dbfs": dbfs(peak),
        "analysis": report, "warnings": warnings,
    }
    output_dir.mkdir(parents=True, exist_ok=True)
    # All validation and synthesis complete before either output is touched.
    with wav_path.open("wb" if overwrite else "xb") as handle:
        with wave.open(handle, "wb") as stream:
            stream.setnchannels(recipe["channels"])
            stream.setsampwidth(2)
            stream.setframerate(rate)
            stream.writeframes(pcm.tobytes())
    report["wav_sha256"] = sha256(wav_path)
    with report_path.open("w" if overwrite else "x", encoding="utf-8", newline="\n") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2, allow_nan=False)
        handle.write("\n")
    return report


def main():
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("recipe", nargs="?", type=Path)
    parser.add_argument("--out", type=Path, default=Path("output"))
    parser.add_argument("--name", default="sound")
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--check", action="store_true", help="Check runtime without rendering")
    parser.add_argument("--analyze", type=Path, help="Analyze an integer PCM WAV without changing it")
    args = parser.parse_args()
    try:
        if args.check:
            result = {"renderer_version": VERSION, "numpy_version": np.__version__, "ready": True}
        elif args.analyze:
            audio, rate = read_wav(args.analyze)
            result = analyze(audio, rate)
        elif args.recipe:
            result = render(args.recipe, args.out, args.name, args.overwrite)
        else:
            parser.error("Provide a recipe, --analyze <wav>, or --check")
        print(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False))
        return 0
    except Exception as error:
        # A script traceback identifies its file and failing line for repair.
        import traceback
        traceback.print_exc(file=sys.stderr)
        print(f"Render failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
