use super::Uint;

use std::ops::{BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign};

use num::{BigUint, ToPrimitive, traits::ConstZero};

impl<U> BitAnd<U> for Uint where U: Into<BigUint> + ToPrimitive {
    type Output = Self;
    fn bitand(self, rhs: U) -> Self {
        if let Uint::Machine(a) = self {
            if let Some(b) = rhs.to_usize() {
                return Uint::Machine(a & b);
            }
        }
        Uint::from(BigUint::from(self) & rhs.into())
    }
}

impl<U> BitOr<U> for Uint where U: Into<BigUint> + ToPrimitive {
    type Output = Self;
    fn bitor(self, rhs: U) -> Self {
        if let Uint::Machine(a) = self {
            if let Some(b) = rhs.to_usize() {
                return Uint::Machine(a | b);
            }
        }
        Uint::from(BigUint::from(self) | rhs.into())
    }
}

impl<U> BitXor<U> for Uint where U: Into<BigUint> + ToPrimitive {
    type Output = Self;
    fn bitxor(self, rhs: U) -> Self {
        if let Uint::Machine(a) = self {
            if let Some(b) = rhs.to_usize() {
                return Uint::Machine(a ^ b);
            }
        }
        Uint::from(BigUint::from(self) ^ rhs.into())
    }
}

impl std::ops::Not for Uint {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Uint::Machine(a) => Uint::Machine(!a),
            Uint::Promoted(a) => {
                // Logical NOT for BigUint usually implies a fixed bit-width.
                // Here we flip all bits within the existing allocated limbs.
                let mut digits = a.to_u32_digits();
                for d in &mut digits { *d = !*d; }
                Uint::from(BigUint::from_slice(&digits))
            }
        }
    }
}

impl<U> BitAndAssign<U> for Uint where Self: BitAnd<U, Output = Self> {
    fn bitand_assign(&mut self, rhs: U) { *self = std::mem::replace(self, Uint::ZERO) & rhs; }
}

impl<U> BitOrAssign<U> for Uint where Self: BitOr<U, Output = Self> {
    fn bitor_assign(&mut self, rhs: U) { *self = std::mem::replace(self, Uint::ZERO) | rhs; }
}

impl<U> BitXorAssign<U> for Uint where Self: BitXor<U, Output = Self> {
    fn bitxor_assign(&mut self, rhs: U) { *self = std::mem::replace(self, Uint::ZERO) ^ rhs; }
}

impl ShlAssign<usize> for Uint {
    fn shl_assign(&mut self, rhs: usize) { *self = std::mem::replace(self, Uint::ZERO) << rhs; }
}

impl ShrAssign<usize> for Uint {
    fn shr_assign(&mut self, rhs: usize) { *self = std::mem::replace(self, Uint::ZERO) >> rhs; }
}
