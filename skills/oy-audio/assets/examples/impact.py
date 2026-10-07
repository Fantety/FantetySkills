"""Layered stylized impact: pitched body, filtered contact, resonant material."""
import numpy as np
from sfx import fade, filter_audio


def build(ctx):
    weight = float(ctx.params.get("weight", 1))
    brightness = float(ctx.params.get("brightness", 2200))
    body = ctx.tone(135 / weight, 0.85, end_frequency=48 / weight, sweep="exponential")
    t = np.arange(len(body)) / ctx.rate
    ctx.add(fade(body * np.exp(-t / 0.14), ctx.rate, 0.002, 0.03), gain=0.65)

    contact = filter_audio(ctx.noise(0.12), ctx.rate, brightness)
    t = np.arange(len(contact)) / ctx.rate
    ctx.add(fade(contact * np.exp(-t / 0.018), ctx.rate, 0.001, 0.02), gain=0.32)

    # Inharmonic partials suggest a hard resonant surface.
    for ratio, level in ((1, 0.12), (1.47, 0.08), (2.09, 0.04)):
        ring = ctx.tone(430 * ratio / weight, 1.1)
        t = np.arange(len(ring)) / ctx.rate
        ctx.add(fade(ring * np.exp(-t / 0.18), ctx.rate, 0.002, 0.03), gain=level)
