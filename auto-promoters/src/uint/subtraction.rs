use super::Uint;

use std::ops::{Sub, SubAssign};

use num::{BigUint, ToPrimitive, traits::{ConstZero, CheckedSub}};

impl<U> Sub<U> for Uint
where
    U: Into<BigUint> + ToPrimitive,
{
    type Output = Self;

    fn sub(self, rhs: U) -> Self {
        match self {
            Uint::Machine(a) => {
                let b_val = rhs.to_usize().unwrap_or(usize::MAX);
                if let Some(result) = a.checked_sub(b_val) {
                    Uint::Machine(result)
                } else {
                    panic!("Uint subtraction underflow");
                }
            }
            Uint::Promoted(a) => {
                Uint::from(a - rhs.into())
            }
        }
    }
}

impl<U> SubAssign<U> for Uint
where Self: Sub<U, Output = Self>
{
    fn sub_assign(&mut self, rhs: U) {
        *self = std::mem::replace(self, Uint::ZERO) - rhs;
    }
}

impl CheckedSub for Uint {
    fn checked_sub(&self, v: &Self) -> Option<Self> {
        match (self, v) {
            (Uint::Machine(a), Uint::Machine(b)) => a.checked_sub(b).map(Uint::Machine),
            _ => {
                let a_big: BigUint = self.clone().into();
                let b_big: BigUint = v.clone().into();
                if a_big >= b_big {
                    Some(Uint::from(a_big - b_big))
                } else {
                    None
                }
            }
        }
    }
}
