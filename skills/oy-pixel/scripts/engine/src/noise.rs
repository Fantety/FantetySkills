use crate::prng::SplitMix64;

/// Deterministic 2D value noise in `[0, 1)`. Lattice values come from a
/// seeded integer hash; bilinear interpolation with a smoothstep gives smooth
/// blobs. `scale` controls frequency (higher = smaller features).
pub fn value_noise(seed: u64, x: f64, y: f64, scale: f64) -> f64 {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let fx = x * scale;
    let fy = y * scale;
    let x0 = fx.floor() as i64;
    let y0 = fy.floor() as i64;
    let tx = smoothstep(fx - fx.floor());
    let ty = smoothstep(fy - fy.floor());
    let (v00, v10, v01, v11) = (
        lattice(seed, x0, y0),
        lattice(seed, x0 + 1, y0),
        lattice(seed, x0, y0 + 1),
        lattice(seed, x0 + 1, y0 + 1),
    );
    let top = v00 + (v10 - v00) * tx;
    let bottom = v01 + (v11 - v01) * tx;
    top + (bottom - top) * ty
}

fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lattice(seed: u64, x: i64, y: i64) -> f64 {
    let mut hash = SplitMix64::new(seed);
    // Mixing the coordinates through two SplitMix rounds avoids visible
    // axis-aligned correlation in the lattice.
    let mixed = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).rotate_left(32);
    hash.state = mixed ^ seed;
    hash.next_f64()
}

#[cfg(test)]
mod tests {
    use super::value_noise;

    #[test]
    fn deterministic_and_bounded() {
        for i in 0..500 {
            let x = (i % 37) as f64 * 0.7;
            let y = (i % 23) as f64 * 1.3;
            let a = value_noise(99, x, y, 0.25);
            let b = value_noise(99, x, y, 0.25);
            assert_eq!(a, b);
            assert!((0.0..1.0).contains(&a));
        }
    }

    #[test]
    fn varies_with_inputs() {
        assert_ne!(value_noise(1, 0.0, 0.0, 1.0), value_noise(2, 0.0, 0.0, 1.0));
        assert_ne!(value_noise(1, 0.0, 0.0, 1.0), value_noise(1, 9.0, 9.0, 1.0));
    }
}
