use super::Uint;

use std::{
    iter::Product,
    ops::{Mul, MulAssign},
};

use num::{
    BigUint, ToPrimitive,
    traits::{CheckedMul, ConstOne, ConstZero},
};

impl<U> Mul<U> for Uint
where
    U: Into<BigUint> + ToPrimitive,
{
    type Output = Self;

    fn mul(self, rhs: U) -> Self {
        // We check to_usize on a reference to avoid moving rhs early
        if let Uint::Machine(a) = self
            && let Some(b) = rhs.to_usize()
            && let Some(result) = a.checked_mul(b)
        {
            return Uint::Machine(result);
        }

        // If machine math is impossible or overflows,
        // move self and rhs into BigUint logic.
        Uint::Promoted(BigUint::from(self) * rhs.into())
    }
}

impl Product for Uint {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Uint::ONE, |a, b| a * b)
    }
}

impl<U> MulAssign<U> for Uint
where
    Self: Mul<U, Output = Self>,
{
    fn mul_assign(&mut self, rhs: U) {
        *self = std::mem::replace(self, Uint::ZERO) * rhs;
    }
}

impl CheckedMul for Uint {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        match (self, v) {
            (Uint::Machine(a), Uint::Machine(b)) => match a.checked_mul(b) {
                Some(res) => Some(Uint::Machine(res)),
                None => Some(Uint::from(BigUint::from(*a) * BigUint::from(*b))),
            },
            _ => Some(Uint::from(
                BigUint::from(self.clone()) * BigUint::from(v.clone()),
            )),
        }
    }
}
