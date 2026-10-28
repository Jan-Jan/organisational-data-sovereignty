//! A position's bits. LLR-mm45ry.

/// Bit `index` of `position_bytes`, bit 0 being the most significant bit of byte 0;
/// `true` says right. Callers pass an index below the tree's `DEPTH`
/// (LLR-mm45ry), so a position never reads past its bits; an index past the
/// 256th bit reads `false` all the same, without indexing out of bounds.
pub(crate) fn bit(position_bytes: &[u8; 32], index: usize) -> bool {
    position_bytes
        .get(index / 8)
        .is_some_and(|byte| (byte >> (7 - index % 8)) & 1 == 1)
}

/// The first `depth` bits of `position_bytes`, from the root down: exactly
/// `depth` of them, so a walk cannot run past the tree. LLR-mm45ry.
pub(crate) fn path_bits(
    position_bytes: &[u8; 32],
    depth: usize,
) -> impl DoubleEndedIterator<Item = bool> + ExactSizeIterator + '_ {
    (0..depth).map(move |index| bit(position_bytes, index))
}
