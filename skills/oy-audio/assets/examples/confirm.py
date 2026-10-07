"""An understated two-part interaction chime; tune its family via parameters."""
import numpy as np
from sfx import fade


def build(ctx):
    root = float(ctx.params.get("frequency", 660))
    interval = float(ctx.params.get("interval", 4 / 3))
    for frequency, at, level in ((root, 0, 0.4), (root * interval, 0.09, 0.45)):
        tone = ctx.tone(frequency, 0.36)
        t = np.arange(len(tone)) / ctx.rate
        body = tone * np.exp(-t / 0.065)
        ctx.add(fade(body, ctx.rate, 0.002, 0.025), at=at, gain=level)
