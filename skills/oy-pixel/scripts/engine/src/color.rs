/// Validates and normalizes a color: `#RRGGBB` or `#RRGGBBAA`, uppercased,
/// with a missing alpha becoming `FF`. Errors quote the offending input.
pub fn normalize_color(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() != 7 && bytes.len() != 9 {
        return Err(format!(
            "invalid color {value:?}: expected #RRGGBB or #RRGGBBAA"
        ));
    }
    if bytes[0] != b'#' || !bytes[1..].iter().all(u8::is_ascii_hexdigit) {
        return Err(format!(
            "invalid color {value:?}: expected #RRGGBB or #RRGGBBAA"
        ));
    }
    let upper = value.to_ascii_uppercase();
    if upper.len() == 7 {
        return Ok(format!("{upper}FF"));
    }
    Ok(upper)
}

/// A fully transparent color (`alpha == 00`) erases the pixel, mirroring the
/// host document semantics.
pub fn paint(color: &str) -> Result<Option<String>, String> {
    let normalized = normalize_color(color)?;
    Ok(if normalized.ends_with("00") {
        None
    } else {
        Some(normalized)
    })
}

fn hex_pair(value: &str) -> Result<u8, String> {
    u8::from_str_radix(value, 16).map_err(|_| format!("invalid hex color {value:?}"))
}

/// Parses a canonical or shorthand color into RGBA bytes.
pub fn parse_rgba(color: &str) -> Result<[u8; 4], String> {
    let normalized = normalize_color(color)?;
    Ok([
        hex_pair(&normalized[1..3])?,
        hex_pair(&normalized[3..5])?,
        hex_pair(&normalized[5..7])?,
        hex_pair(&normalized[7..9])?,
    ])
}

pub fn format_rgba(rgba: [u8; 4]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        rgba[0], rgba[1], rgba[2], rgba[3]
    )
}

/// Linear blend of two colors; `t` is clamped to [0, 1].
pub fn mix_colors(from: &str, to: &str, t: f64) -> Result<String, String> {
    if !t.is_finite() {
        return Err(format!("mix factor must be finite, got {t}"));
    }
    let t = t.clamp(0.0, 1.0);
    let from = parse_rgba(from)?;
    let to = parse_rgba(to)?;
    let blended = from
        .iter()
        .zip(to.iter())
        .map(|(a, b)| {
            (*a as f64 + (*b as f64 - *a as f64) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        })
        .collect::<Vec<_>>();
    Ok(format_rgba([
        blended[0], blended[1], blended[2], blended[3],
    ]))
}

/// HSV (hue in degrees, saturation/value/alpha in [0, 1]) to color string.
pub fn hsv_color(h: f64, s: f64, v: f64, a: Option<f64>) -> Result<String, String> {
    if !h.is_finite() {
        return Err(format!("hsv hue must be finite, got {h}"));
    }
    for (name, value) in [("saturation", s), ("value", v)] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(format!("hsv {name} must be within [0, 1], got {value}"));
        }
    }
    let alpha = match a {
        None => 1.0,
        Some(alpha) if alpha.is_finite() && (0.0..=1.0).contains(&alpha) => alpha,
        Some(alpha) => return Err(format!("hsv alpha must be within [0, 1], got {alpha}")),
    };
    let hue = h.rem_euclid(360.0);
    let chroma = v * s;
    let sector = hue / 60.0;
    let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = v - chroma;
    Ok(format_rgba([
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
        (alpha * 255.0).round() as u8,
    ]))
}

/// Replaces a color's alpha channel; `alpha` is in [0, 1].
pub fn set_alpha(color: &str, alpha: f64) -> Result<String, String> {
    if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
        return Err(format!("alpha must be within [0, 1], got {alpha}"));
    }
    let mut rgba = parse_rgba(color)?;
    rgba[3] = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
    Ok(format_rgba(rgba))
}

#[cfg(test)]
mod tests {
    use super::{hsv_color, mix_colors, normalize_color, paint, set_alpha};

    #[test]
    fn normalizes_and_appends_alpha() {
        assert_eq!(normalize_color("#d1495b").unwrap(), "#D1495BFF");
        assert_eq!(normalize_color("#d1495bff").unwrap(), "#D1495BFF");
        assert_eq!(normalize_color("#AbCdEf80").unwrap(), "#ABCDEF80");
    }

    #[test]
    fn rejects_malformed_colors() {
        for bad in [
            "", "#", "red", "#12345", "#1234567", "##12345", "#12345G", "#1234 6",
        ] {
            assert!(normalize_color(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn alpha_zero_is_transparent() {
        assert_eq!(paint("#FF000000").unwrap(), None);
        assert_eq!(paint("#FF0000").unwrap().as_deref(), Some("#FF0000FF"));
    }

    #[test]
    fn mix_blends_and_clamps() {
        assert_eq!(mix_colors("#000000", "#FFFFFF", 0.5).unwrap(), "#808080FF");
        assert_eq!(mix_colors("#000000", "#FFFFFF", 0.0).unwrap(), "#000000FF");
        assert_eq!(mix_colors("#000000", "#FFFFFF", 1.0).unwrap(), "#FFFFFFFF");
        // Out-of-range factors clamp.
        assert_eq!(mix_colors("#101010", "#202020", 5.0).unwrap(), "#202020FF");
        assert_eq!(mix_colors("#101010", "#202020", -5.0).unwrap(), "#101010FF");
        // Alpha blends too.
        assert_eq!(
            mix_colors("#FF0000FF", "#FF000000", 0.5).unwrap(),
            "#FF000080"
        );
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(hsv_color(0.0, 1.0, 1.0, None).unwrap(), "#FF0000FF");
        assert_eq!(hsv_color(120.0, 1.0, 1.0, None).unwrap(), "#00FF00FF");
        assert_eq!(hsv_color(240.0, 1.0, 1.0, None).unwrap(), "#0000FFFF");
        // Hue wraps: 360 == 0; grayscale at s=0.
        assert_eq!(hsv_color(360.0, 1.0, 1.0, None).unwrap(), "#FF0000FF");
        assert_eq!(hsv_color(200.0, 0.0, 0.5, None).unwrap(), "#808080FF");
        // Alpha passthrough.
        assert_eq!(hsv_color(0.0, 1.0, 1.0, Some(0.5)).unwrap(), "#FF000080");
    }

    #[test]
    fn set_alpha_overrides_transparency() {
        assert_eq!(set_alpha("#FF0000", 0.5).unwrap(), "#FF000080");
        assert_eq!(set_alpha("#FF000080", 1.0).unwrap(), "#FF0000FF");
        assert!(set_alpha("#FF0000", 1.5).is_err());
    }
}
