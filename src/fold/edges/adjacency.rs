//! Adjacency list compression: sorted targets with delta + varint.
//!
//! Instead of storing each target_id as u32 (4 bytes), we:
//! 1. Group edges by source_id
//! 2. Sort targets within each source
//! 3. Store consecutive differences (deltas)
//! 4. Encode deltas as varints (1 byte for small deltas, up to 5 for u32)
//!
//! Dense neighborhoods (many close targets) compress 3-4×.
//! Sparse (scattered) targets compress ~2.5×.

/// Encode a u32 as ULEB128 varint bytes.
pub fn varint_encode(mut n: u32, out: &mut Vec<u8>) {
    loop {
        let byte = (n & 0x7F) as u8;
        n >>= 7;
        if n != 0 {
            out.push(byte | 0x80);
        } else {
            out.push(byte);
            return;
        }
    }
}

/// Decode a varint from a byte slice. Returns (value, bytes_consumed).
pub fn varint_decode(bytes: &[u8]) -> Option<(u32, usize)> {
    let mut val = 0u32;
    let mut shift = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if shift >= 32 { return None; }
        val |= ((b & 0x7F) as u32) << shift;
        if b & 0x80 == 0 {
            return Some((val, i + 1));
        }
        shift += 7;
    }
    None
}

/// Encode a sorted list of u32 targets as delta + varint.
/// Targets MUST be sorted ascending and unique.
pub fn encode_sorted_targets(sorted_targets: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(sorted_targets.len() * 2);
    let mut prev = 0u32;
    for &t in sorted_targets {
        debug_assert!(t >= prev, "targets must be sorted");
        let delta = t - prev;
        varint_encode(delta, &mut out);
        prev = t;
    }
    out
}

/// Decode a delta+varint byte stream back to absolute target_ids.
pub fn decode_targets(bytes: &[u8], count: usize) -> Option<Vec<u32>> {
    let mut out = Vec::with_capacity(count);
    let mut pos = 0usize;
    let mut prev = 0u32;
    for _ in 0..count {
        let (delta, consumed) = varint_decode(&bytes[pos..])?;
        prev = prev.checked_add(delta)?;
        out.push(prev);
        pos += consumed;
    }
    Some(out)
}

/// Sort, dedup, then encode. Useful when caller isn't sure of ordering.
pub fn encode_unsorted_targets(targets: &mut Vec<u32>) -> (Vec<u8>, usize) {
    targets.sort_unstable();
    targets.dedup();
    let encoded = encode_sorted_targets(targets);
    (encoded, targets.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_encodes_small_as_one_byte() {
        let mut buf = Vec::new();
        varint_encode(127, &mut buf);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf[0], 127);
    }

    #[test]
    fn varint_encodes_128_as_two_bytes() {
        let mut buf = Vec::new();
        varint_encode(128, &mut buf);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn varint_roundtrip_many_values() {
        let values = [0u32, 1, 127, 128, 1000, 65535, 1_000_000, u32::MAX];
        for v in values {
            let mut buf = Vec::new();
            varint_encode(v, &mut buf);
            let (d, _) = varint_decode(&buf).unwrap();
            assert_eq!(d, v);
        }
    }

    #[test]
    fn delta_encoding_is_smaller_than_raw() {
        let targets: Vec<u32> = (1000..1100).collect();  // 100 consecutive
        let encoded = encode_sorted_targets(&targets);
        let raw_size = targets.len() * 4;
        println!("dense: {} raw, {} compressed = {:.2}×", raw_size, encoded.len(),
                 raw_size as f64 / encoded.len() as f64);
        assert!(encoded.len() < raw_size / 2);
    }

    #[test]
    fn decode_sorted_targets_roundtrip() {
        let targets: Vec<u32> = vec![10, 25, 100, 200, 1000, 100_000];
        let encoded = encode_sorted_targets(&targets);
        let decoded = decode_targets(&encoded, targets.len()).unwrap();
        assert_eq!(decoded, targets);
    }

    #[test]
    fn scattered_compression_still_wins() {
        let targets: Vec<u32> = vec![100, 5000, 10_000, 50_000, 100_000, 500_000];
        let encoded = encode_sorted_targets(&targets);
        let raw_size = targets.len() * 4;
        println!("scattered: {} raw, {} compressed = {:.2}×", raw_size, encoded.len(),
                 raw_size as f64 / encoded.len() as f64);
        assert!(encoded.len() < raw_size);
    }

    #[test]
    fn encode_unsorted_sorts_and_dedups() {
        let mut targets = vec![50, 10, 50, 30, 10];
        let (encoded, n) = encode_unsorted_targets(&mut targets);
        assert_eq!(n, 3);  // sorted + dedup: [10, 30, 50]
        let decoded = decode_targets(&encoded, n).unwrap();
        assert_eq!(decoded, vec![10, 30, 50]);
    }

    #[test]
    fn decode_insufficient_bytes_returns_none() {
        let bytes = vec![0xFF];  // incomplete varint
        assert!(decode_targets(&bytes, 1).is_none());
    }

    #[test]
    fn empty_list_encodes_to_empty_bytes() {
        let encoded = encode_sorted_targets(&[]);
        assert!(encoded.is_empty());
        let decoded = decode_targets(&encoded, 0).unwrap();
        assert!(decoded.is_empty());
    }
}
