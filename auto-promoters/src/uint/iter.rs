use super::Uint;

/// An iterator over the 64-bit digits (limbs) of a `Uint`.
/// Yields digits in Little-Endian order (least significant limb first).
pub struct UintDigitsIter {
    inner: DigitInner,
}

enum DigitInner {
    // For Machine, we yield the value once.
    // We use Option to track if it has been consumed.
    Machine(Option<u64>),
    // For Promoted, we wrap the BigUint's native u64 digit iterator.
    Promoted(std::vec::IntoIter<u64>),
}

impl Uint {
    /// Returns a lossless iterator over the 64-bit limbs of the integer.
    /// This is architecture-independent.
    pub fn iter_digits(&self) -> UintDigitsIter {
        match self {
            Uint::Machine(v) => UintDigitsIter {
                inner: DigitInner::Machine(Some(*v as u64)),
            },
            Uint::Promoted(v) => UintDigitsIter {
                inner: DigitInner::Promoted(v.to_u64_digits().into_iter()),
            },
        }
    }
}

impl Iterator for UintDigitsIter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            DigitInner::Machine(opt) => opt.take(),
            DigitInner::Promoted(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            DigitInner::Machine(opt) => {
                let len = if opt.is_some() { 1 } else { 0 };
                (len, Some(len))
            }
            DigitInner::Promoted(iter) => iter.size_hint(),
        }
    }
}

impl ExactSizeIterator for UintDigitsIter {}
