//! A stable fingerprint of a frame's pixels.

/// FNV-1a over `bytes`, 64-bit.
///
/// The value depends only on the bytes: it is the same on every platform and in every Rust
/// version (unlike `std`'s `DefaultHasher`), so hashes can be compared across runs,
/// machines and processes. Not for security; two frames with equal hashes are equal for
/// any practical purpose of a determinism check.
pub fn frame_hash(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, &b| (hash ^ u64::from(b)).wrapping_mul(PRIME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_published_fnv1a_values() {
        assert_eq!(frame_hash(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(frame_hash(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(frame_hash(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn one_changed_byte_changes_the_hash() {
        let mut frame = vec![7u8; 4096];
        let before = frame_hash(&frame);
        frame[2048] = 8;
        assert_ne!(frame_hash(&frame), before);
    }
}
