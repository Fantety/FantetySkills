"""Behavioral tests for the independent renderer; all artifacts stay temporary."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

from PIL import Image

SKILL_ROOT = Path(__file__).resolve().parents[2] / "skills" / "oy-pixel"
sys.path.insert(0, str(SKILL_ROOT / "scripts"))
import render


class RenderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        render.ensure_engine()

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="oy-pixel-test-")
        self.root = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def job(self, code, **kwargs):
        job = {"width": 8, "height": 8, "durations": [100], "palette": ["#FF0000", "#00FF00"], "seed": 42, "layers": [{"name": "test", "code": code}]}
        job.update(kwargs)
        return job

    def output(self, result, formats, **kwargs):
        args = argparse.Namespace(scale=1, columns=2, padding=0, formats=formats, name="test", matte=None, out=str(self.root), overwrite=False)
        for k, v in kwargs.items():
            setattr(args, k, v)
        return render.export(result, kwargs.pop("loop", True), args)

    def test_exact_pixels_clipping_and_nearest_scaling(self):
        result = render.run_engine(self.job("canvas.rectfill(-1, 0, 3, 2, pal(1))"))
        self.output(result, "png", scale=3)
        with Image.open(self.root / "test.png") as im:
            self.assertEqual(im.size, (24, 24))
            self.assertEqual(im.getpixel((5, 5)), (255, 0, 0, 255))
            self.assertEqual(im.getpixel((6, 5)), (0, 0, 0, 0))
            self.assertEqual(sum(a > 0 for *_, a in render.pixels(im)), 36)

    def test_layers_opacity_and_hidden_layers(self):
        job = self.job("")
        job["layers"] = [{"name": "base", "code": "canvas.pset(1,1,'#FF0000')"}, {"name": "top", "code": "canvas.pset(1,1,'#0000FF')", "opacity": 0.5}, {"name": "hidden", "code": "canvas.clear('#FFFFFF')", "visible": False}]
        result = render.run_engine(job)
        self.assertEqual(result["frames"][0][36:40], [128, 0, 128, 255])

    def test_transparent_animation_disposal_and_timing(self):
        job = self.job("canvas.pset(frame_index, 2, pal(1))", durations=[83] * 4)
        result = render.run_engine(job)
        self.output(result, "gif,sheet,frames", padding=1)
        durations = []
        with Image.open(self.root / "test.gif") as im:
            self.assertEqual(im.n_frames, 4)
            self.assertEqual(im.info["loop"], 0)
            for i in range(im.n_frames):
                im.seek(i)
                durations.append(im.info["duration"])
                frame = im.convert("RGBA")
                self.assertEqual(frame.getpixel((i, 2)), (255, 0, 0, 255))
                self.assertEqual(sum(a > 0 for *_, a in render.pixels(frame)), 1, "previous positions must be transparent")
        self.assertEqual(durations, [80, 90, 80, 80])
        with Image.open(self.root / "test-sheet.png") as im:
            self.assertEqual(im.size, (19, 19))
            self.assertEqual(im.getpixel((1, 3)), (255, 0, 0, 255))
            self.assertEqual(im.getpixel((11, 3)), (255, 0, 0, 255))

    def test_nonuniform_timeline_and_once(self):
        job = self.job("canvas.pset(math.floor(phase * 4), 0, pal(1))", durations=[100, 300])
        result = render.run_engine(job)
        self.assertEqual(result["frames"][1][4:8], [255, 0, 0, 255])
        self.output(result, "gif", loop=False)
        with Image.open(self.root / "test.gif") as im:
            self.assertNotIn("loop", im.info)

    def test_reproducible_randomness(self):
        job = self.job("for i=1,12 do canvas.pset(rand(0,7),rand(0,7),pal(1)) end", durations=[100] * 3)
        a = render.run_engine(job)
        self.assertEqual(a, render.run_engine(job))
        self.assertNotEqual(a["frames"][0], a["frames"][1])

    def test_base_image_patch_and_stamp_transparency(self):
        image = Image.new("RGBA", (8, 8), "#123456")
        image.save(self.root / "base.png")
        spec = {"width": 8, "height": 8, "layers": [{"baseImage": "base.png", "code": "canvas.stamp({'R.'},{R='#FF0000'},1,1)"}]}
        path = self.root / "recipe.json"
        path.write_text(json.dumps(spec), encoding="utf-8")
        job, _ = render.load_recipe(path)
        result = render.run_engine(job)
        self.assertEqual(result["frames"][0][36:40], [255, 0, 0, 255])
        self.assertEqual(result["frames"][0][40:44], [18, 52, 86, 255])
        self.assertEqual(result["frames"][0][:4], [18, 52, 86, 255])

    def test_errors_do_not_publish_or_overwrite(self):
        for code in ["canvas.pset(8,0,pal(1))", "canvas.pset(1.5,0,pal(1))", "canvas.pset(1,0,'red')", "while true do end", "require('os')"]:
            with self.subTest(code=code), self.assertRaises(RuntimeError):
                render.run_engine(self.job(code))
        self.assertEqual(list(self.root.iterdir()), [])
        result = render.run_engine(self.job("canvas.pset(1,1,pal(1))"))
        self.output(result, "png")
        before = (self.root / "test.png").read_bytes()
        with self.assertRaises(FileExistsError):
            self.output(result, "png,sheet")
        self.assertEqual((self.root / "test.png").read_bytes(), before)
        self.assertFalse((self.root / "test-sheet.png").exists())

    def test_gif_partial_alpha_and_matte(self):
        result = render.run_engine(self.job("canvas.pset(1,1,'#FF000080')"))
        report = self.output(result, "gif,png")
        self.assertTrue(any("1-bit" in w for w in report["warnings"]))
        with Image.open(self.root / "test.png") as im:
            self.assertEqual(im.getpixel((1, 1)), (255, 0, 0, 128))
        report = self.output(result, "gif", name="matte", matte="#FFFFFF")
        self.assertFalse(any("1-bit" in w for w in report["warnings"]))
        with Image.open(self.root / "matte.gif") as im:
            self.assertEqual(im.convert("RGBA").getpixel((1, 1)), (255, 127, 127, 255))

    def test_quantization_and_all_transparent_gif(self):
        result = render.run_engine(self.job("for y=0,31 do for x=0,31 do canvas.pset(x,y,hsv(x*11,y/32,1)) end end", width=32, height=32))
        report = self.output(result, "gif")
        self.assertTrue(any("255-color" in w for w in report["warnings"]))
        with Image.open(self.root / "test.gif") as im:
            self.assertTrue(all(a == 255 for *_, a in render.pixels(im.convert("RGBA"))))
        result = render.run_engine(self.job("canvas.clear(nil)"))
        self.output(result, "gif", name="empty")
        with Image.open(self.root / "empty.gif") as im:
            self.assertEqual(im.convert("RGBA").getbbox(), None)

    def test_unknown_fields_and_storage_budget(self):
        path = self.root / "bad.json"
        for spec in [{"width": 8, "height": 8, "frame": 4, "layers": [{}]}, {"width": 1024, "height": 1024, "frames": 2000, "layers": [{"code": "canvas.clear(nil)"}]}]:
            path.write_text(json.dumps(spec), encoding="utf-8")
            with self.assertRaises(ValueError):
                render.load_recipe(path)

    def test_cli_works_from_unrelated_directory(self):
        command = [sys.executable, str(render.ROOT / "scripts" / "render.py"), str(render.ROOT / "assets" / "examples" / "still.json"), "--out", str(self.root), "--name", "example"]
        result = subprocess.run(command, cwd=self.root, capture_output=True, text=True, encoding="utf-8")
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["frames"], 1)
        self.assertEqual(len(report["files"]), 1)
        self.assertTrue((self.root / "example.png").is_file())

    def test_portable_copy_without_cargo_or_original_project(self):
        portable = self.root / "portable-skill"
        shutil.copytree(render.ROOT, portable, ignore=shutil.ignore_patterns("target", ".venv", "__pycache__"))
        command = [sys.executable, str(portable / "scripts" / "render.py"), str(portable / "assets" / "examples" / "loop.json"), "--out", str(self.root / "art"), "--formats", "gif,sheet", "--name", "portable"]
        environment = dict(os.environ, PATH="")
        result = subprocess.run(command, cwd=self.root, env=environment, capture_output=True, text=True, encoding="utf-8")
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report["gifDurationMs"], 800)
        self.assertTrue((self.root / "art" / "portable.gif").is_file())
        self.assertTrue((self.root / "art" / "portable-sheet.png").is_file())


if __name__ == "__main__":
    unittest.main(verbosity=2)
