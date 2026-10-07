"""Behavioral checks for generated audio and a standalone installed skill."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import wave

import numpy as np

SKILL = Path(__file__).resolve().parents[2] / "skills" / "oy-audio"
sys.path.insert(0, str(SKILL / "scripts"))
import render
from sfx import Context, delay, filter_audio, read_wav


class RenderTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="sfx-test-")
        # Match renderer paths, including Windows 8.3 aliases and symlinked temp directories.
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / "source.py"
        self.source.write_text("def build(ctx):\n    ctx.add(ctx.noise() * 0.1)\n", encoding="utf-8")
        self.recipe = dict(version=1, script="source.py", duration=0.15,
                           sample_rate=24000, channels=2, seed=71, peak_dbfs=-6)
        self.recipe_path = self.root / "recipe.json"

    def tearDown(self):
        self.temp.cleanup()

    def run_render(self, name="out", overwrite=False):
        self.recipe_path.write_text(json.dumps(self.recipe), encoding="utf-8")
        return render.render(self.recipe_path, self.root / "output", name, overwrite)

    def test_seed_reproducibility_variants_and_file_metrics(self):
        first = self.run_render("first")
        same = self.run_render("same")
        self.assertEqual(first["wav_sha256"], same["wav_sha256"])
        self.recipe["seed"] += 1
        variant = self.run_render("variant")
        self.assertNotEqual(first["wav_sha256"], variant["wav_sha256"])
        audio, rate = read_wav(first["files"]["wav"])
        self.assertEqual(rate, 24000)
        self.assertEqual(audio.shape, (3600, 2))
        self.assertEqual(first["analysis"], render.analyze(audio, rate))
        self.assertAlmostEqual(first["analysis"]["peak_dbfs"], -6, delta=0.002)
        self.assertEqual(first["analysis"]["clipped_samples"], 0)
        self.assertGreater(first["analysis"]["rms_dbfs"], -30)

    def test_mono_and_no_normalization(self):
        self.source.write_text("def build(ctx):\n    ctx.add(ctx.tone(400), gain=0.1)\n", encoding="utf-8")
        self.recipe.update(channels=1, peak_dbfs=None, fade_ms=0)
        report = self.run_render()
        audio, _ = read_wav(report["files"]["wav"])
        self.assertEqual(audio.shape, (3600, 1))
        self.assertAlmostEqual(float(np.max(np.abs(audio))), 0.1 / np.sqrt(2), delta=0.0001)
        self.assertEqual(report["normalization_gain_db"], 0)
        self.assertAlmostEqual(report["analysis"]["spectral_centroid_hz"], 400, delta=3)

    def test_stereo_position(self):
        ctx = Context(24000, 0.1, 0, {}, self.root)
        ctx.add(ctx.tone(440), position=-1)
        self.assertGreater(float(np.max(np.abs(ctx.buffer[:, 0]))), 0.99)
        self.assertEqual(float(np.max(np.abs(ctx.buffer[:, 1]))), 0)

    def test_loop_preserves_length_and_continuation(self):
        self.source.write_text("import numpy as np\ndef build(ctx):\n    ctx.add(np.cos(2*np.pi*113.37*ctx.t), gain=0.1)\n", encoding="utf-8")
        self.recipe.update(duration=1, loop_crossfade_ms=50, channels=1)
        report = self.run_render()
        audio, _ = read_wav(report["files"]["wav"])
        self.assertEqual(len(audio), 24000)
        self.assertLess(report["analysis"]["boundary_jump"], 0.02)
        self.assertGreater(abs(float(audio[0, 0])), 0.1)

    def test_nonfinite_silent_clipping_and_overflow_fail_before_output(self):
        cases = (
            "def build(ctx):\n    ctx.buffer[:] = float('nan')\n",
            "def build(ctx):\n    pass\n",
            "def build(ctx):\n    ctx.add(ctx.tone(), gain=10)\n",
            "def build(ctx):\n    ctx.add(ctx.tone(440, ctx.duration + 0.1))\n",
            "def build(ctx):\n    ctx.add(ctx.tone(), gain=0.0000001)\n",
        )
        self.recipe["peak_dbfs"] = None
        for source in cases:
            with self.subTest(source=source):
                self.source.write_text(source, encoding="utf-8")
                with self.assertRaises(ValueError):
                    self.run_render()
                self.assertFalse((self.root / "output" / "out.wav").exists())

    def test_invalid_recipe_and_names(self):
        baseline = self.recipe.copy()
        for key, value in (("version", True), ("duration", float("nan")), ("duration", -1),
                           ("sample_rate", 123), ("channels", 3), ("seed", -1),
                           ("peak_dbfs", 0), ("params", []), ("typo", 1),
                           ("loop_crossfade_ms", 0.0001)):
            self.recipe = dict(baseline, **{key: value})
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.run_render()
        self.recipe = baseline
        for name in ("../outside", "CON", "foo/bar", " "):
            with self.subTest(name=name), self.assertRaises(ValueError):
                self.run_render(name)

    def test_existing_output_is_preserved(self):
        report = self.run_render()
        before = Path(report["files"]["wav"]).read_bytes()
        self.recipe["seed"] = 555
        with self.assertRaises(FileExistsError):
            self.run_render()
        self.assertEqual(Path(report["files"]["wav"]).read_bytes(), before)
        replaced = self.run_render(overwrite=True)
        self.assertNotEqual(replaced["wav_sha256"], report["wav_sha256"])

    def test_delay_tail_and_filter_behavior(self):
        impulse = np.zeros(2400)
        impulse[240] = 1
        echoes = delay(impulse, 24000, seconds=0.05, repeats=2, feedback=0.5, wet=0.5)
        self.assertEqual(len(echoes), 4800)
        self.assertEqual(echoes[240], 1)
        self.assertEqual(echoes[1440], 0.5)
        self.assertEqual(echoes[2640], 0.25)
        t = np.arange(24000) / 24000
        mixed = np.sin(2 * np.pi * 100 * t) + np.sin(2 * np.pi * 5000 * t)
        filtered = filter_audio(mixed, 24000, 500, taps=1025)
        spectrum = np.abs(np.fft.rfft(filtered[2400:-2400]))
        self.assertGreater(spectrum[80], spectrum[4000] * 100)

    def test_import_pcm_widths_and_sample_rate_validation(self):
        encodings = {
            1: bytes([0, 128, 192]),
            2: np.array([-32768, 0, 16384], dtype="<i2").tobytes(),
            3: bytes([0, 0, 128, 0, 0, 0, 0, 0, 64]),
            4: np.array([-2147483648, 0, 1073741824], dtype="<i4").tobytes(),
        }
        for width, raw in encodings.items():
            source = self.root / f"pcm{width}.wav"
            with wave.open(str(source), "wb") as stream:
                stream.setnchannels(1)
                stream.setsampwidth(width)
                stream.setframerate(24000)
                stream.writeframes(raw)
            ctx = Context(24000, 0.1, 0, {}, self.root)
            audio = ctx.load_wav(source.name)
            np.testing.assert_array_equal(audio[:, 0], [-1, 0, 0.5])
            self.assertEqual(ctx.sources, [source])
        with self.assertRaisesRegex(ValueError, "resample"):
            Context(48000, 0.1, 0, {}, self.root).load_wav("pcm1.wav")

    def test_sample_import_provenance_and_source_preservation(self):
        report = self.run_render("original")
        sample = Path(report["files"]["wav"])
        before = sample.read_bytes()
        self.source.write_text("def build(ctx):\n    ctx.add(ctx.load_wav('output/original.wav'), gain=0.5)\n", encoding="utf-8")
        changed = self.run_render("modified")
        self.assertEqual(sample.read_bytes(), before)
        self.assertIn(str(sample), [p["path"] for p in changed["sources"]])
        with self.assertRaisesRegex(ValueError, "input sample"):
            self.run_render("original", overwrite=True)
        self.assertEqual(hashlib.sha256(sample.read_bytes()).hexdigest(), report["wav_sha256"])

    def test_shipped_examples_and_standalone_cli_from_unrelated_directory(self):
        installed = self.root / "独立 skill"
        shutil.copytree(SKILL, installed, ignore=shutil.ignore_patterns("__pycache__"))
        unrelated = self.root / "其他 工作目录"
        unrelated.mkdir()
        for recipe in sorted((installed / "assets" / "examples").glob("*.json")):
            result = subprocess.run(
                [sys.executable, "-X", "utf8", str(installed / "scripts" / "render.py"),
                 str(recipe), "--out", str(unrelated), "--name", recipe.stem],
                cwd=unrelated, capture_output=True, text=True, encoding="utf-8", timeout=45,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads(result.stdout)
            self.assertTrue(Path(report["files"]["wav"]).is_file())
            self.assertEqual(report["analysis"]["clipped_samples"], 0)
            self.assertGreater(report["analysis"]["rms_dbfs"], -60)
            analyzed = subprocess.run(
                [sys.executable, str(installed / "scripts" / "render.py"), "--analyze", report["files"]["wav"]],
                cwd=unrelated, capture_output=True, text=True, encoding="utf-8", timeout=15,
            )
            self.assertEqual(analyzed.returncode, 0, analyzed.stderr)
            self.assertEqual(json.loads(analyzed.stdout), report["analysis"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
