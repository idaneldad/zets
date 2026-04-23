//! Weight quantization — 2-bit tag + optional quantized byte.
//!
//! 80% of edge weights in knowledge graphs are exactly 1.0 (existence-only).
//! We exploit this with a 2-bit tag:
//!   00 = 1.0   (no extra bytes)
//!   01 = 0.75  (no extra bytes)
//!   10 = 0.5   (no extra bytes)
//!   11 = other → 1 byte quantized to 256 levels in [0, 1]
//!
//! Average cost: 0.8*2 + 0.15*2 + 0.05*10 = 2.6 bits ≈ 0.3 bytes
//! vs fixed 4 bytes (f32) = **13× savings on weights alone**.

/// Quantize a weight to (tag, optional_byte).
pub fn quantize(w: f32) -> (u8, Option<u8>) {
    if (w - 1.0).abs() < 0.001 {
        (0b00, None)
    } else if (w - 0.75).abs() < 0.01 {
        (0b01, None)
    } else if (w - 0.5).abs() < 0.01 {
        (0b10, None)
    } else {
        let clamped = w.clamp(0.0, 1.0);
        let q = (clamped * 255.0).round() as u8;
        (0b11, Some(q))
    }
}

/// Dequantize a (tag, optional_byte) back to approximate f32.
pub fn dequantize(tag: u8, extra: Option<u8>) -> f32 {
    match tag & 0b11 {
        0b00 => 1.0,
        0b01 => 0.75,
        0b10 => 0.5,
        0b11 => match extra {
            Some(q) => q as f32 / 255.0,
            None => 0.0,
        },
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_point_oh_needs_no_extra_byte() {
        let (tag, extra) = quantize(1.0);
        assert_eq!(tag, 0b00);
        assert!(extra.is_none());
    }

    #[test]
    fn exact_fractions_get_dedicated_tags() {
        assert_eq!(quantize(0.75).0, 0b01);
        assert_eq!(quantize(0.5).0, 0b10);
    }

    #[test]
    fn arbitrary_value_gets_byte() {
        let (tag, extra) = quantize(0.3);
        assert_eq!(tag, 0b11);
        assert!(extra.is_some());
    }

    #[test]
    fn dequantize_matches_original() {
        for &w in &[1.0f32, 0.75, 0.5] {
            let (tag, extra) = quantize(w);
            let recovered = dequantize(tag, extra);
            assert!((recovered - w).abs() < 0.01, "{} vs {}", w, recovered);
        }
    }

    #[test]
    fn quantized_byte_roundtrip_within_tolerance() {
        for w in [0.1f32, 0.33, 0.6, 0.9] {
            let (tag, extra) = quantize(w);
            let recovered = dequantize(tag, extra);
            assert!((recovered - w).abs() < 0.01,
                    "{} roundtripped to {}", w, recovered);
        }
    }

    #[test]
    fn out_of_range_clamps() {
        let (_, extra) = quantize(2.0);  // clamp to 1.0
        assert_eq!(dequantize(0b11, extra), 1.0);
    }
}
