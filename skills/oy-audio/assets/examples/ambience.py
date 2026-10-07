"""Continuous wind-like texture with slow motion and independent side detail."""
import numpy as np
from sfx import filter_audio


def build(ctx):
    motion = float(ctx.params.get("motion_hz", 0.25))
    brightness = float(ctx.params.get("brightness", 1800))
    # No one-shot fades: generate through the entire overlap extension.
    bed = filter_audio(ctx.noise(color="brown"), ctx.rate, 320, taps=2049)
    ctx.add(bed * (0.72 + 0.14 * np.sin(2 * np.pi * motion * ctx.t)), gain=0.22)
    for position, phase in ((-0.7, 0), (0.7, 1.7)):
        air = filter_audio(ctx.noise(color="pink"), ctx.rate, [350, brightness], "bandpass")
        swell = 0.65 + 0.25 * np.sin(2 * np.pi * motion * ctx.t + phase)
        ctx.add(air * swell, gain=0.25, position=position)
