use super::Uint;

use std::ops::{Shl, Shr, ShlAssign, ShrAssign};

use num::{BigUint, ToPrimitive, traits::ConstZero};

// --- Core Logic Helper ---
// We centralize the shift logic to avoid duplication
impl Uint {
    fn execute_shl(self, rhs: usize) -> Self {
        match self {
            Uint::Machine(a) => {
                if let Some(result) = a.checked_shl(rhs as u32) {
                    Uint::Machine(result)
                } else {
                    Uint::from(BigUint::from(a) << rhs)
                }
            }
            Uint::Promoted(a) => Uint::Promoted(a << rhs),
        }
    }

    fn execute_shr(self, rhs: usize) -> Self {
        match self {
            Uint::Machine(a) => Uint::Machine(a >> rhs),
            Uint::Promoted(a) => Uint::from(a >> rhs),
        }
    }
}

// --- Shl Implementations ---

// 1. Implementation for shifting by another Uint
impl Shl<Uint> for Uint {
    type Output = Self;
    fn shl(self, rhs: Uint) -> Self {
        let s = rhs.to_usize().expect("Shift amount too large");
        self.execute_shl(s)
    }
}

// 2. Implementation for ShlAssign using the same logic
impl ShlAssign<Uint> for Uint {
    fn shl_assign(&mut self, rhs: Uint) {
        let s = rhs.to_usize().expect("Shift amount too large");
        *self = std::mem::replace(self, Uint::ZERO).execute_shl(s);
    }
}

// 3. To handle primitives (u8, u32, usize, etc.) without conflicts,
// we implement for usize specifically, which usually satisfies most needs.
// If you need more, you can use a macro to implement for u8, u32, u64.
impl Shl<usize> for Uint {
    type Output = Self;
    fn shl(self, rhs: usize) -> Self {
        self.execute_shl(rhs)
    }
}

// --- Shr Implementations ---

impl Shr<Uint> for Uint {
    type Output = Self;
    fn shr(self, rhs: Uint) -> Self {
        let s = rhs.to_usize().expect("Shift amount too large");
        self.execute_shr(s)
    }
}

impl ShrAssign<Uint> for Uint {
    fn shr_assign(&mut self, rhs: Uint) {
        let s = rhs.to_usize().expect("Shift amount too large");
        *self = std::mem::replace(self, Uint::ZERO).execute_shr(s);
    }
}

impl Shr<usize> for Uint {
    type Output = Self;
    fn shr(self, rhs: usize) -> Self {
        self.execute_shr(rhs)
    }
}
