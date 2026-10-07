"""Small, deterministic sound-design toolkit. All time values are seconds."""
from pathlib import Path
import math
import wave

import numpy as np


def finite_number(value, name, low=None, high=None):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{name} must be a number")
    value = float(value)
    if not math.isfinite(value):
        raise ValueError(f"{name} must be finite")
    if low is not None and value < low or high is not None and value > high:
        raise ValueError(f"{name} must be in [{low}, {high}]")
    return value


def audio_array(value):
    audio = np.asarray(value, dtype=np.float64)
    if audio.ndim not in (1, 2) or audio.ndim == 2 and audio.shape[1] not in (1, 2):
        raise ValueError("Audio must have shape (frames,), (frames, 1), or (frames, 2)")
    if not audio.size or not np.isfinite(audio).all():
        raise ValueError("Audio must be nonempty and contain only finite samples")
    return audio


def db_to_gain(db):
    return 10.0 ** (finite_number(db, "dB", -160, 60) / 20.0)


def envelope(frames, rate, points):
    """Piecewise linear amplitude envelope; points are (seconds, amplitude)."""
    points = np.asarray(points, dtype=np.float64)
    if points.ndim != 2 or points.shape[1] != 2 or len(points) < 2:
        raise ValueError("envelope needs at least two [time, value] points")
    if not np.isfinite(points).all() or points[0, 0] < 0 or np.any(np.diff(points[:, 0]) <= 0):
        raise ValueError("Envelope times must be finite, nonnegative and strictly increasing")
    return np.interp(np.arange(frames) / rate, points[:, 0], points[:, 1], left=0, right=0)


def fade(audio, rate, attack=0.003, release=0.01):
    """Apply independent linear fades, preserving length and channel count."""
    result = audio_array(audio).copy()
    attack = finite_number(attack, "attack", 0)
    release = finite_number(release, "release", 0)
    for seconds, tail in ((attack, False), (release, True)):
        count = min(len(result), round(seconds * rate))
        if count:
            ramp = np.linspace(0, 1, count) if count > 1 else np.zeros(1)
            if tail:
                ramp = ramp[::-1]
            if result.ndim == 2:
                ramp = ramp[:, None]
            if tail:
                result[-count:] *= ramp
            else:
                result[:count] *= ramp
    return result


def pan(audio, position=0):
    """Equal-power pan for mono input; -1 left, +1 right."""
    audio = audio_array(audio)
    if audio.ndim == 2:
        if audio.shape[1] != 1:
            raise ValueError("Pan a mono source before mixing; stereo input already has placement")
        audio = audio[:, 0]
    position = finite_number(position, "pan", -1, 1)
    angle = (position + 1) * np.pi / 4
    return audio[:, None] * np.array([np.cos(angle), np.sin(angle)])


def filter_audio(audio, rate, cutoff, kind="lowpass", taps=513):
    """Windowed-sinc FIR via FFT convolution; same length, compensated group delay.

    Static cutoff only. Use more taps for low cutoffs. Allow envelope/tail space
    around transients because a linear-phase filter can ring on either side.
    """
    audio = audio_array(audio)
    if kind not in ("lowpass", "highpass", "bandpass"):
        raise ValueError("Filter kind must be lowpass, highpass, or bandpass")
    if isinstance(taps, bool) or not isinstance(taps, int) or not 3 <= taps <= 8191 or taps % 2 == 0:
        raise ValueError("taps must be an odd integer from 3 to 8191")
    center = (taps - 1) // 2
    grid = np.arange(taps) - center

    def lowpass(hz):
        hz = finite_number(hz, "cutoff", 1, rate * 0.49)
        kernel = 2 * hz / rate * np.sinc(2 * hz / rate * grid) * np.hamming(taps)
        return kernel / kernel.sum()

    if kind == "bandpass":
        if not isinstance(cutoff, (list, tuple)) or len(cutoff) != 2:
            raise ValueError("Bandpass cutoff must be [low_hz, high_hz]")
        lower, upper = cutoff
        if lower >= upper:
            raise ValueError("Bandpass low cutoff must be less than high cutoff")
        kernel = lowpass(upper) - lowpass(lower)
    else:
        kernel = lowpass(cutoff)
        if kind == "highpass":
            kernel = -kernel
            kernel[center] += 1
    size = 1 << (len(audio) + taps - 2).bit_length()
    response = np.fft.rfft(kernel, n=size)
    if audio.ndim == 2:
        response = response[:, None]
    result = np.fft.irfft(np.fft.rfft(audio, n=size, axis=0) * response, n=size, axis=0)
    return result[center:center + len(audio)]


def delay(audio, rate, seconds=0.12, feedback=0.3, repeats=3, wet=0.25):
    """Finite echo train; returns input plus the entire echo tail."""
    audio = audio_array(audio)
    seconds = finite_number(seconds, "delay seconds", 1 / rate, 10)
    feedback = finite_number(feedback, "feedback", 0, 0.99)
    wet = finite_number(wet, "wet", 0, 1)
    if isinstance(repeats, bool) or not isinstance(repeats, int) or not 1 <= repeats <= 32:
        raise ValueError("repeats must be an integer from 1 to 32")
    offset = max(1, round(seconds * rate))
    result = np.zeros((len(audio) + offset * repeats,) + audio.shape[1:])
    result[:len(audio)] = audio
    for index in range(1, repeats + 1):
        start = index * offset
        result[start:start + len(audio)] += audio * wet * feedback ** (index - 1)
    return result


def read_wav(path):
    """Read mono/stereo uncompressed 8/16/24/32-bit integer PCM WAV."""
    with wave.open(str(path), "rb") as stream:
        channels, width, rate, frames, compression, _ = stream.getparams()
        if compression != "NONE" or channels not in (1, 2) or width not in (1, 2, 3, 4):
            raise ValueError("Use mono/stereo integer PCM WAV (8/16/24/32 bit)")
        raw = stream.readframes(frames)
    if not frames or len(raw) != frames * channels * width:
        raise ValueError("WAV is empty or truncated")
    if width == 1:
        values = (np.frombuffer(raw, dtype=np.uint8).astype(np.float64) - 128) / 128
    elif width == 3:
        b = np.frombuffer(raw, dtype=np.uint8).reshape(-1, 3).astype(np.int32)
        values = b[:, 0] | b[:, 1] << 8 | b[:, 2] << 16
        values = ((values ^ 0x800000) - 0x800000) / 8388608.0
    else:
        values = np.frombuffer(raw, dtype=f"<i{width}").astype(np.float64) / 2 ** (8 * width - 1)
    return values.reshape(-1, channels), rate


class Context:
    """One render's sample clock, RNG, parameters, source paths, and stereo bus."""
    def __init__(self, rate, duration, seed, params, base_dir, extra_frames=0):
        self.rate = rate
        self.output_duration = duration
        self.output_frames = round(duration * rate)
        self.frames = self.output_frames + extra_frames
        self.duration = self.frames / rate
        self.t = np.arange(self.frames) / rate
        self.rng = np.random.Generator(np.random.PCG64(seed))
        self.params = params
        self.base_dir = Path(base_dir)
        self.buffer = np.zeros((self.frames, 2), dtype=np.float64)
        self.sources = []

    def count(self, seconds=None):
        if seconds is None:
            return self.frames
        count = round(finite_number(seconds, "source duration", 1 / self.rate, 300) * self.rate)
        return max(1, count)

    def tone(self, frequency=440, seconds=None, end_frequency=None, sweep="linear", waveform="sine"):
        """Oscillator with integrated frequency; PolyBLEP saw/square."""
        count = self.count(seconds)
        frequency = finite_number(frequency, "frequency", 1, self.rate * 0.45)
        if end_frequency is None:
            frequencies = np.full(count, frequency)
        else:
            end_frequency = finite_number(end_frequency, "end_frequency", 1, self.rate * 0.45)
            if sweep == "linear":
                frequencies = np.linspace(frequency, end_frequency, count)
            elif sweep == "exponential":
                frequencies = np.geomspace(frequency, end_frequency, count)
            else:
                raise ValueError("sweep must be linear or exponential")
        step = frequencies / self.rate
        phase = np.remainder(np.cumsum(step) - step, 1)

        def blep(p):
            result = np.zeros(count)
            mask = p < step
            x = p[mask] / step[mask]
            result[mask] = 2 * x - x * x - 1
            mask = p > 1 - step
            x = (p[mask] - 1) / step[mask]
            result[mask] = x * x + 2 * x + 1
            return result

        if waveform == "sine":
            return np.sin(2 * np.pi * phase)
        if waveform == "saw":
            return 2 * phase - 1 - blep(phase)
        if waveform == "square":
            return np.where(phase < 0.5, 1.0, -1.0) + blep(phase) - blep((phase + 0.5) % 1)
        raise ValueError("waveform must be sine, saw, or square")

    def noise(self, seconds=None, color="white"):
        """Seeded white/pink/brown noise, RMS-normalized; may exceed +/-1."""
        count = self.count(seconds)
        signal = self.rng.standard_normal(count)
        if color not in ("white", "pink", "brown"):
            raise ValueError("noise color must be white, pink, or brown")
        if color != "white":
            hz = np.fft.rfftfreq(count, 1 / self.rate)
            weights = np.maximum(hz, 20) ** (-0.5 if color == "pink" else -1)
            weights[0] = 0
            signal = np.fft.irfft(np.fft.rfft(signal) * weights, n=count)
        rms = np.sqrt(np.mean(signal * signal))
        return signal / max(rms, 1e-12)

    def add(self, audio, at=0, gain=1, position=0):
        """Place mono or stereo audio. Reject overflow instead of truncating tails."""
        audio = audio_array(audio)
        at = finite_number(at, "at", 0)
        gain = finite_number(gain, "gain", -100, 100)
        start = round(at * self.rate)
        end = start + len(audio)
        if end > self.frames:
            raise ValueError(f"Layer ends at {end / self.rate:.6f}s, beyond render {self.duration:.6f}s; extend duration or shorten the layer explicitly")
        if audio.ndim == 1 or audio.shape[1] == 1:
            audio = pan(audio, position)
        elif position != 0:
            raise ValueError("position only applies to mono input")
        self.buffer[start:end] += audio * gain

    def load_wav(self, path):
        path = Path(path)
        if not path.is_absolute():
            path = self.base_dir / path
        path = path.resolve()
        audio, rate = read_wav(path)
        if rate != self.rate:
            raise ValueError(f"Source rate {rate} differs from recipe rate {self.rate}; resample a copy first")
        self.sources.append(path)
        return audio
