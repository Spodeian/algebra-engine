use super::Uint;

use std::ops::{Div, Rem, DivAssign, RemAssign};

use num::{BigUint, ToPrimitive, Zero, traits::{ConstZero, Euclid, CheckedDiv, CheckedRem}};

/// Internal helper to ensure we don't duplicate panic logic.
#[inline]
fn check_zero<U: Zero>(rhs: &U) {
    if rhs.is_zero() {
        panic!("attempt to divide by zero");
    }
}

impl<U> Div<U> for Uint
where
    U: Into<BigUint> + ToPrimitive + Zero,
{
    type Output = Self;

    fn div(self, rhs: U) -> Self {
        check_zero(&rhs);

        match self {
            Uint::Machine(a) => {
                if let Some(b_val) = rhs.to_usize() {
                    Uint::Machine(a / b_val)
                } else {
                    // divisor > dividend (since dividend is at most usize::MAX)
                    Uint::ZERO
                }
            }
            Uint::Promoted(a) => {
                Uint::from(a / Into::<BigUint>::into(rhs))
            }
        }
    }
}

impl<U> Rem<U> for Uint
where
    U: Into<BigUint> + ToPrimitive + Zero,
{
    type Output = Self;

    fn rem(self, rhs: U) -> Self {
        check_zero(&rhs);

        match self {
            Uint::Machine(a) => {
                if let Some(b_val) = rhs.to_usize() {
                    Uint::Machine(a % b_val)
                } else {
                    // b > a, so a % b == a
                    Uint::Machine(a)
                }
            }
            Uint::Promoted(a) => {
                Uint::from(a % Into::<BigUint>::into(rhs))
            }
        }
    }
}

impl Euclid for Uint {
    fn div_euclid(&self, v: &Self) -> Self {
        // Since Uint is unsigned, Euclidean division is standard division.
        // We use references to avoid unnecessary moves/clones of BigUint.
        match (self, v) {
            (Uint::Machine(a), Uint::Machine(b)) => {
                if *b == 0 { panic!("attempt to divide by zero"); }
                Uint::Machine(a / b)
            }
            _ => {
                let a_big = BigUint::from(self.clone());
                let b_big = BigUint::from(v.clone());
                Uint::from(a_big.div_euclid(&b_big))
            }
        }
    }

    fn rem_euclid(&self, v: &Self) -> Self {
        match (self, v) {
            (Uint::Machine(a), Uint::Machine(b)) => {
                if *b == 0 { panic!("attempt to calculate remainder with a divisor of zero"); }
                Uint::Machine(a % b)
            }
            _ => {
                let a_big = BigUint::from(self.clone());
                let b_big = BigUint::from(v.clone());
                Uint::from(a_big.rem_euclid(&b_big))
            }
        }
    }
}

impl<U> DivAssign<U> for Uint where Self: Div<U, Output = Self> {
    fn div_assign(&mut self, rhs: U) { *self = std::mem::replace(self, Uint::ZERO) / rhs; }
}

impl<U> RemAssign<U> for Uint where Self: Rem<U, Output = Self> {
    fn rem_assign(&mut self, rhs: U) { *self = std::mem::replace(self, Uint::ZERO) % rhs; }
}

impl CheckedDiv for Uint {
    fn checked_div(&self, v: &Self) -> Option<Self> {
        if v.is_zero() { return None; }
        Some(self.clone() / v.clone())
    }
}

impl CheckedRem for Uint {
    fn checked_rem(&self, v: &Self) -> Option<Self> {
        if v.is_zero() { return None; }
        Some(self.clone() % v.clone())
    }
}
