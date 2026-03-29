use super::Uint;

use std::{ops::{Add, AddAssign}, iter::Sum};

use num::{BigUint, ToPrimitive, traits::{ConstZero, CheckedAdd}};

impl<U> Add<U> for Uint
where
    U: Into<BigUint> + ToPrimitive
{
    type Output = Self;

    fn add(self, rhs: U) -> Self {
        // We check to_usize on a reference to avoid moving rhs early
        if let Uint::Machine(a) = self {
            if let Some(b) = rhs.to_usize() {
                if let Some(result) = a.checked_add(b) {
                    return Uint::Machine(result);
                }
            }
        }

        // If machine math is impossible or overflows,
        // move self and rhs into BigUint logic.
        Uint::Promoted(BigUint::from(self) + rhs.into())
    }
}

impl Sum for Uint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Uint::ZERO, |a, b| a + b)
    }
}

impl<U> AddAssign<U> for Uint
where Self: Add<U, Output = Self>
{
    fn add_assign(&mut self, rhs: U) {
        *self = std::mem::replace(self, Uint::ZERO) + rhs;
    }
}

impl CheckedAdd for Uint {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        match (self, v) {
            (Uint::Machine(a), Uint::Machine(b)) => {
                match a.checked_add(b) {
                    Some(res) => Some(Uint::Machine(res)),
                    None => Some(Uint::Promoted(BigUint::from(*a) + BigUint::from(*b))),
                }
            }
            (Uint::Promoted(a), Uint::Machine(b)) => Some(Uint::from(a + BigUint::from(*b))),
            (Uint::Machine(a), Uint::Promoted(b)) => Some(Uint::from(BigUint::from(*a) + b)),
            (Uint::Promoted(a), Uint::Promoted(b)) => Some(Uint::Promoted(a + b)),
        }
    }
}
