#!/usr/bin/env python3
"""Render pixel art and animation from JSON/Lua recipes."""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
ENGINE = SCRIPTS / "engine"
BIN = SCRIPTS / "bin"
MAX_PIXELS = 16_777_216
MAX_EXPORT_PIXELS = 67_108_864
DEFAULT_PALETTE = ["#1D2B53", "#7E2553", "#008751", "#AB5236", "#5F574F", "#C2C3C7", "#FFF1E8", "#FF004D", "#FFA300", "#FFEC27", "#00E436", "#29ADFF", "#83769C", "#FF77A8", "#FFCCAA", "#000000"]


def integer(value, label, low, high):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"{label} must be an integer in {low}..{high}")
    return value


def keys(value, allowed, label):
    if not isinstance(value, dict):
        raise ValueError(f"{label} must be an object")
    extra = set(value) - set(allowed)
    if extra:
        raise ValueError(f"{label}: unknown fields {sorted(extra)}")


def read_json(path):
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError(f"{path}: JSON exceeds 64 MiB")
    return json.loads(path.read_text(encoding="utf-8-sig"))


def engine_hash():
    paths = [ENGINE / "Cargo.toml", ENGINE / "Cargo.lock", *sorted((ENGINE / "src").glob("*.rs"))]
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.name.encode("utf-8"))
        digest.update(path.read_bytes())
    return digest.hexdigest()


def ensure_engine():
    name = "oy-pixel-render.exe" if os.name == "nt" else "oy-pixel-render"
    binary = BIN / name
    stamp = BIN / "source.sha256"
    fingerprint = engine_hash()
    if binary.is_file() and stamp.is_file() and stamp.read_text(encoding="utf-8").strip() == fingerprint:
        return binary
    if not shutil.which("cargo"):
        raise RuntimeError("No built engine for this installation. Build with Rust/Cargo and a C toolchain (vendored Lua); see references/workflow.md.")
    build_dir = ENGINE
    subprocess.run(["cargo", "build", "--release", "--locked", "--manifest-path", str(build_dir / "Cargo.toml")], cwd=build_dir, check=True, stdout=sys.stderr)
    binary.parent.mkdir(parents=True, exist_ok=True)
    # CARGO_TARGET_DIR may be relative to the isolated engine working directory.
    target = Path(os.environ.get("CARGO_TARGET_DIR", ENGINE / "target"))
    if not target.is_absolute():
        target = build_dir / target
    shutil.copy2(target / "release" / name, binary)
    stamp.write_text(fingerprint + "\n", encoding="utf-8")
    return binary


def pillow():
    try:
        from PIL import Image
    except ImportError as error:
        raise RuntimeError("Pillow is missing. Use a Python environment with Pillow (Codex bundled Python works), or install scripts/requirements.txt into a local venv.") from error
    return Image


def pixels(image):
    # Pillow 12.3 renamed getdata; retain compatibility with supported 10.x.
    if hasattr(image, "get_flattened_data"):
        return image.get_flattened_data()
    return image.getdata()


def base_pixels(path, width, height):
    Image = pillow()
    with Image.open(path) as image:
        if image.size != (width, height):
            raise ValueError(f"{path}: expected {width}x{height}, found {image.size}; resize/crop deliberately before editing")
        if getattr(image, "n_frames", 1) != 1:
            raise ValueError(f"{path}: base images must be stills; extract animation frames and use baseImages")
        return [f"#{r:02X}{g:02X}{b:02X}{a:02X}" if a else None for r, g, b, a in pixels(image.convert("RGBA"))]


def load_recipe(path):
    spec = read_json(path)
    keys(spec, ["width", "height", "frames", "durationMs", "durations", "palette", "seed", "loop", "layers"], "recipe")
    width = integer(spec.get("width"), "width", 1, 1024)
    height = integer(spec.get("height"), "height", 1, 1024)
    if "durations" in spec:
        if "frames" in spec or "durationMs" in spec:
            raise ValueError("durations cannot be combined with frames/durationMs")
        durations = spec["durations"]
        if not isinstance(durations, list) or not 1 <= len(durations) <= 2000:
            raise ValueError("durations must be a list of 1..2000 milliseconds values")
        durations = [integer(d, "duration", 1, 60000) for d in durations]
    else:
        count = integer(spec.get("frames", 1), "frames", 1, 2000)
        durations = [integer(spec.get("durationMs", 100), "durationMs", 1, 60000)] * count
    layers = spec.get("layers")
    if not isinstance(layers, list) or not 1 <= len(layers) <= 128:
        raise ValueError("layers must contain 1..128 objects, ordered bottom to top")
    if width * height * len(durations) * len(layers) > MAX_PIXELS:
        raise ValueError("width*height*frames*layers exceeds 16777216 pixels; split the task")
    palette = spec.get("palette", DEFAULT_PALETTE)
    if not isinstance(palette, list) or not 1 <= len(palette) <= 1024 or not all(isinstance(c, str) for c in palette):
        raise ValueError("palette must be a list of 1..1024 hex color strings")
    loop = spec.get("loop", True)
    if type(loop) is not bool:
        raise ValueError("loop must be true or false")
    job = {"width": width, "height": height, "durations": durations, "palette": palette, "seed": integer(spec.get("seed", 42), "seed", 0, 2**64 - 1), "layers": []}
    for i, item in enumerate(layers):
        keys(item, ["name", "script", "code", "opacity", "visible", "baseImage", "baseImages"], f"layer {i}")
        if "code" in item and "script" in item:
            raise ValueError(f"layer {i}: choose code or script")
        code = (path.parent / item["script"]).read_text(encoding="utf-8-sig") if "script" in item else item.get("code", "")
        if not isinstance(code, str) or len(code.encode("utf-8")) > 65536:
            raise ValueError(f"layer {i}: code must be a string of at most 65536 UTF-8 bytes")
        if not code.strip() and "baseImage" not in item and "baseImages" not in item:
            raise ValueError(f"layer {i}: provide script/code or a base image")
        opacity = item.get("opacity", 1)
        if type(opacity) not in (int, float) or not math.isfinite(opacity) or not 0 <= opacity <= 1:
            raise ValueError(f"layer {i}: opacity must be 0..1")
        visible = item.get("visible", True)
        if type(visible) is not bool:
            raise ValueError(f"layer {i}: visible must be boolean")
        layer_name = item.get("name", f"Layer {i + 1}")
        if not isinstance(layer_name, str):
            raise ValueError(f"layer {i}: name must be a string")
        layer = {"name": layer_name, "code": code, "opacity": opacity, "visible": visible}
        if "baseImage" in item and "baseImages" in item:
            raise ValueError(f"layer {i}: choose baseImage or baseImages")
        sources = [item["baseImage"]] if "baseImage" in item else item.get("baseImages", [])
        if not isinstance(sources, list) or len(sources) not in (0, 1, len(durations)) or not all(isinstance(p, str) for p in sources):
            raise ValueError(f"layer {i}: baseImages must contain one or frame-count paths")
        if sources:
            layer["baseFrames"] = [base_pixels(path.parent / p, width, height) for p in sources]
        job["layers"].append(layer)
    return job, loop


def run_engine(job):
    binary = ensure_engine()
    encoded = json.dumps(job, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    if len(encoded) > 64 * 1024 * 1024:
        raise ValueError("expanded recipe exceeds engine's 64 MiB input limit")
    result = subprocess.run([str(binary)], input=encoded, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=15 + 12 * len(job["layers"]))
    if result.returncode:
        raise RuntimeError(result.stderr.decode("utf-8", errors="replace").strip())
    return json.loads(result.stdout)


def gif_frames(images):
    """Shared palette, index 255 transparent, no dithering, full-frame disposal."""
    Image = pillow()
    colors = set()
    samples = []
    partial = False
    stride = max(1, sum(im.width * im.height for im in images) // 1_000_000)
    for im in images:
        for i, (r, g, b, a) in enumerate(pixels(im)):
            partial |= a not in (0, 255)
            if a >= 128:
                if len(colors) <= 255:
                    colors.add((r, g, b))
                if i % stride == 0:
                    samples.append((r, g, b))
    exact = len(colors) <= 255
    if exact:
        rgb = sorted(colors) or [(0, 0, 0)]
    else:
        sample = Image.new("RGB", (max(1, len(samples)), 1))
        sample.putdata(samples or [(0, 0, 0)])
        quantized = sample.quantize(colors=255, dither=Image.Dither.NONE)
        raw = quantized.getpalette()[:765]
        rgb = [tuple(raw[i:i + 3]) for i in range(0, len(raw), 3)]
    rgb += [rgb[-1]] * (255 - len(rgb))
    palette = [v for color in rgb for v in color] + [0, 0, 0]
    palette_image = Image.new("P", (1, 1))
    palette_image.putpalette(palette)
    lookup = {color: i for i, color in reversed(list(enumerate(rgb)))}
    encoded = []
    for im in images:
        if exact:
            indices = [lookup[(r, g, b)] if a >= 128 else 255 for r, g, b, a in pixels(im)]
        else:
            # The 256th palette entry is reserved: give it the same RGB as 254
            # during quantization, then remap it before adding transparency.
            palette_image.putpalette(palette[:765] + list(rgb[254]))
            indices = list(pixels(im.convert("RGB").quantize(palette=palette_image, dither=Image.Dither.NONE)))
            indices = [min(i, 254) if a >= 128 else 255 for i, a in zip(indices, pixels(im.getchannel("A")))]
        frame = Image.new("P", im.size)
        frame.putpalette(palette)
        frame.putdata(indices)
        frame.info["transparency"] = 255
        encoded.append(frame)
    warnings = []
    if partial:
        warnings.append("GIF uses 1-bit alpha (threshold 128); use PNG frames or --matte for soft transparency.")
    if not exact:
        warnings.append("GIF quantized to a shared 255-color palette; PNG preserves original RGBA.")
    return encoded, warnings


def gif_durations(durations):
    # Round cumulative timestamps, avoiding 83 ms -> 80 ms drift per frame.
    elapsed = 0
    encoded = 0
    output = []
    for duration in durations:
        elapsed += duration
        next_time = max(encoded + 10, int(math.floor(elapsed / 10 + 0.5)) * 10)
        output.append(next_time - encoded)
        encoded = next_time
    return output


def export(result, loop, args):
    Image = pillow()
    count = len(result["frames"])
    width, height = result["width"], result["height"]
    scale = integer(args.scale, "scale", 1, 64)
    columns = min(count, integer(args.columns, "columns", 1, 2000))
    padding = integer(args.padding, "padding", 0, 128)
    formats = args.formats.split(",") if args.formats else (["png"] if count == 1 else ["gif"])
    if not formats or set(formats) - {"png", "gif", "sheet", "frames"}:
        raise ValueError("--formats supports png,gif,sheet,frames")
    if max(width * scale, height * scale) > 16384 or width * height * count * scale**2 > MAX_EXPORT_PIXELS:
        raise ValueError("export exceeds 16384 pixels per side or 67108864 total scaled frame pixels; reduce --scale")
    sheet_size = ((columns * width + (columns + 1) * padding) * scale, (math.ceil(count / columns) * height + (math.ceil(count / columns) + 1) * padding) * scale)
    if "sheet" in formats and (max(sheet_size) > 16384 or math.prod(sheet_size) > MAX_EXPORT_PIXELS):
        raise ValueError("sprite sheet exceeds export budget")
    if not args.name or args.name in (".", "..") or any(c in args.name for c in '/\\:*?"<>|'):
        raise ValueError("--name must be a plain filename stem")
    images = [Image.frombytes("RGBA", (width, height), bytes(frame)) for frame in result["frames"]]
    if args.matte:
        import re
        if not re.fullmatch(r"#[0-9a-fA-F]{6}", args.matte):
            raise ValueError("--matte must be #RRGGBB")
        images = [Image.alpha_composite(Image.new("RGBA", im.size, args.matte), im) for im in images]
    images = [im.resize((width * scale, height * scale), Image.Resampling.NEAREST) for im in images]
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    warnings = []
    gif_info = {}
    with tempfile.TemporaryDirectory(prefix=".oy-pixel-", dir=out) as temp:
        stage = Path(temp)
        if "png" in formats:
            images[0].save(stage / f"{args.name}.png")
            if count > 1:
                warnings.append("png exports frame 0; use frames for every frame.")
        if "frames" in formats:
            for i, im in enumerate(images):
                im.save(stage / f"{args.name}-{i:04d}.png")
        if "sheet" in formats:
            sheet = Image.new("RGBA", sheet_size)
            for i, im in enumerate(images):
                sheet.paste(im, ((padding + (i % columns) * (width + padding)) * scale, (padding + (i // columns) * (height + padding)) * scale))
            sheet.save(stage / f"{args.name}-sheet.png")
        if "gif" in formats:
            frames, messages = gif_frames(images)
            warnings.extend(messages)
            delays = gif_durations(result["durations"])
            if delays != result["durations"]:
                warnings.append(f"GIF timings rounded to 10 ms units: {delays}; total {sum(delays)} ms.")
            options = {"save_all": True, "append_images": frames[1:], "duration": delays, "disposal": 2, "transparency": 255, "background": 255, "optimize": False}
            if loop:
                options["loop"] = 0
            frames[0].save(stage / f"{args.name}.gif", **options)
            with Image.open(stage / f"{args.name}.gif") as encoded:
                gif_info["gifFrames"] = encoded.n_frames
                gif_info["gifDurationMs"] = 0
                for i in range(encoded.n_frames):
                    encoded.seek(i)
                    gif_info["gifDurationMs"] += encoded.info.get("duration", 0)
            if gif_info["gifFrames"] != count:
                warnings.append(f"GIF combined identical adjacent frames: {count} logical frames -> {gif_info['gifFrames']} encoded frames, preserving their total delay.")
        files = sorted(stage.iterdir())
        if not args.overwrite:
            conflicts = [str(out / p.name) for p in files if (out / p.name).exists()]
            if conflicts:
                raise FileExistsError("outputs exist; choose another name/directory or --overwrite: " + ", ".join(conflicts))
        # Verify encoded output can be reopened before publishing any files.
        for path in files:
            with Image.open(path) as check:
                check.verify()
        for path in files:
            os.replace(path, out / path.name)
        return {"files": [str(out / p.name) for p in files], "width": width, "height": height, "frames": count, "durationMs": sum(result["durations"]), "scale": scale, **gif_info, "warnings": warnings}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("recipe", nargs="?", type=Path)
    parser.add_argument("--out", help="directory for finished image files")
    parser.add_argument("--name", default="pixel-art")
    parser.add_argument("--formats", help="comma-separated png,gif,sheet,frames; default png for stills, gif for animations")
    parser.add_argument("--scale", type=int, default=1)
    parser.add_argument("--columns", type=int, default=4)
    parser.add_argument("--padding", type=int, default=0)
    parser.add_argument("--matte", help="opaque #RRGGBB background before export")
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--check", action="store_true", help="check Python/Pillow and build or verify the engine")
    args = parser.parse_args()
    try:
        pillow()
        if args.check:
            print(json.dumps({"python": sys.executable, "engine": str(ensure_engine()), "status": "ready"}, ensure_ascii=False))
            return
        if args.recipe is None or args.out is None:
            parser.error("recipe and --out are required unless using --check")
        job, loop = load_recipe(args.recipe.resolve())
        result = run_engine(job)
        print(json.dumps(export(result, loop, args), ensure_ascii=False, indent=2))
    except (ValueError, KeyError, TypeError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"oy-pixel: {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
