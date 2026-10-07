"""Repeated stylized soft contacts with coherent timing and material variation."""
import numpy as np
from sfx import fade, filter_audio


def build(ctx):
    count = int(ctx.params.get("count", 6))
    spacing = float(ctx.params.get("spacing", 0.42))
    center = float(ctx.params.get("center_hz", 850))
    for index in range(count):
        at = 0.04 + index * spacing + ctx.rng.uniform(-0.008, 0.008)
        cutoff = center * ctx.rng.uniform(0.90, 1.10)
        body = filter_audio(ctx.noise(0.18), ctx.rate, [cutoff * 0.45, cutoff * 1.6], "bandpass")
        t = np.arange(len(body)) / ctx.rate
        decay = ctx.rng.uniform(0.022, 0.032)
        body = fade(body * np.exp(-t / decay), ctx.rate, 0.0015, 0.02)
        ctx.add(body, at=at, gain=ctx.rng.uniform(0.28, 0.36))
        thud = ctx.tone(ctx.rng.uniform(95, 115), 0.13)
        t = np.arange(len(thud)) / ctx.rate
        ctx.add(fade(thud * np.exp(-t / 0.025), ctx.rate), at=at, gain=0.10)
