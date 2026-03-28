use super::Uint;

use std::iter::FromIterator;

use num::{bigint::ParseBigIntError, BigUint, FromPrimitive, ToPrimitive, Num, traits::ConstZero};

impl FromPrimitive for Uint {
    fn from_i64(value: i64) -> Option<Self> {
        if value >= 0 {
            value.to_usize().and_then(|v| Some(Uint::Machine(v))).or_else(|| BigUint::from_i64(value).map(Uint::Promoted))
        } else {
            None
        }
    }

    fn from_u64(value: u64) -> Option<Self> {
        value.to_usize().and_then(|v| Some(Uint::Machine(v))).or_else(|| BigUint::from_u64(value).map(Uint::Promoted))
    }
}

impl ToPrimitive for Uint {
    fn to_i64(&self) -> Option<i64> {
        match self {
            Uint::Machine(v) => v.to_i64(),
            Uint::Promoted(v) => v.to_i64(),
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match self {
            Uint::Machine(v) => v.to_u64(),
            Uint::Promoted(v) => v.to_u64(),
        }
    }
}

impl From<usize> for Uint {
    fn from(value: usize) -> Self {
        Uint::Machine(value)
    }
}

impl From<BigUint> for Uint {
    fn from(value: BigUint) -> Self {
        value.to_usize().and_then(|v| Some(Uint::Machine(v))).unwrap_or(Uint::Promoted(value))
    }
}

impl From<Uint> for BigUint {
    fn from(value: Uint) -> Self {
        match value {
            Uint::Machine(v) => BigUint::from(v),
            Uint::Promoted(v) => v,
        }
    }
}

impl Num for Uint {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigUint::from_str_radix(str, radix).map(Uint::from)
    }
}

impl FromIterator<u64> for Uint {
    fn from_iter<I: IntoIterator<Item = u64>>(iter: I) -> Self {
        let mut it = iter.into_iter();

        // 1. Try to grab the first limb.
        let first = match it.next() {
            Some(v) => v,
            None => return Uint::ZERO,
        };

        // 2. Peek to see if there are more limbs.
        match it.next() {
            None => {
                // Only one limb. Try to stay in Machine space.
                // This correctly handles 32-bit vs 64-bit targets via TryFrom.
                if let Ok(machine_val) = usize::try_from(first) {
                    Uint::Machine(machine_val)
                } else {
                    Uint::Promoted(BigUint::from(first))
                }
            }
            Some(second) => {
                // More than one limb: Must be Promoted.
                // We rebuild using BigUint's native from_slice or similar logic.
                let mut limbs = vec![first, second];
                limbs.extend(it);

                // BigUint::from_slice/from_digits usually expects u32 or u64
                Uint::Promoted(BigUint::from_slice(&limbs_to_u32(&limbs)))
            }
        }
    }
}

/// Helper to convert u64 limbs to u32 limbs for BigUint compatibility
/// if the specific BigUint version prefers u32 slices.
fn limbs_to_u32(limbs: &[u64]) -> Vec<u32> {
    let mut res = Vec::with_capacity(limbs.len() * 2);
    for &l in limbs {
        res.push(l as u32);
        res.push((l >> 32) as u32);
    }
    res
}
