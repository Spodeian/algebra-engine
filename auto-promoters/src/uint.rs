use std::fmt;

use num::{BigUint, One, Unsigned, Zero, traits::{ConstZero, ConstOne}, Integer};

mod addition;
mod division;
mod multiplication;
mod subtraction;

mod bits;
mod convert;
mod iter;
mod power;
mod serde;
mod shift;
mod roots;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Uint {
    Machine(usize),
    Intermediate([usize; 3]),
    Promoted(BigUint),
    Infinite,
}

// Machine is ALWAYS less than Promoted.
impl Ord for Uint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match mem::discriminant(&t1).cmp(mem::discriminant(&t2)) {
            std::cmp::Ordering::Equal => {
                match (self, other) {
                    (Uint::Machine(a), Uint::Machine(b)) => a.cmp(b),
                    (Uint::Intermediate())
                    (Uint::Promoted(a), Uint::Promoted(b)) => a.cmp(b),
                }
            }
            ordering: Ordering => return ordering,
        }
        match (self, other) {
            (Uint::Machine(a), Uint::Machine(b)) => a.cmp(b),
            (Uint::Promoted(a), Uint::Promoted(b)) => a.cmp(b),
            (Uint::Machine(_), Uint::Promoted(_)) => std::cmp::Ordering::Less,
            (Uint::Promoted(_), Uint::Machine(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Uint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Uint {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Zero for Uint {
    fn zero() -> Self {
        Self::ZERO
    }

    fn set_zero(&mut self) {
        *self = Self::ZERO;
    }

    fn is_zero(&self) -> bool {
        matches!(self, Uint::Machine(0))
    }
}

impl ConstZero for Uint {
    const ZERO: Self = Self::Machine(0);
}

impl One for Uint {
    fn one() -> Self {
        Self::ONE
    }

    fn set_one(&mut self) {
        *self = Self::ONE;
    }

    fn is_one(&self) -> bool {
        matches!(self, Uint::Machine(1))
    }
}

impl ConstOne for Uint {
    const ONE: Self = Self::Machine(1);
}

impl Unsigned for Uint {}

impl Integer for Uint {
    fn div_rem(&self, other: &Self) -> (Self, Self) {
        let a_big = BigUint::from(self.clone());
        let b_big = BigUint::from(other.clone());
        let (q, r) = a_big.div_rem(&b_big);
        (Uint::from(q), Uint::from(r))
    }

    fn div_floor(&self, other: &Self) -> Self {
        self.div_rem(other).0
    }

    fn mod_floor(&self, other: &Self) -> Self {
        self.div_rem(other).1
    }

    fn gcd(&self, other: &Self) -> Self {
        let a_big = BigUint::from(self.clone());
        let b_big = BigUint::from(other.clone());
        Uint::from(a_big.gcd(&b_big))
    }

    fn lcm(&self, other: &Self) -> Self {
        let a_big = BigUint::from(self.clone());
        let b_big = BigUint::from(other.clone());
        Uint::from(a_big.lcm(&b_big))
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if other.is_zero() { return false; }
        let a_big = BigUint::from(self.clone());
        let b_big = BigUint::from(other.clone());
        a_big.is_multiple_of(&b_big)
    }

    fn is_even(&self) -> bool {
        match self {
            Uint::Machine(n) => n % 2 == 0,
            Uint::Promoted(n) => n.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        !self.is_even()
    }
}

impl fmt::Display for Uint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uint::Machine(v) => write!(f, "{}", v),
            Uint::Promoted(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::LowerHex for Uint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uint::Machine(v) => write!(f, "{:x}", v),
            Uint::Promoted(v) => write!(f, "{:x}", v),
        }
    }
}

impl fmt::UpperHex for Uint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uint::Machine(v) => fmt::UpperHex::fmt(v, f),
            Uint::Promoted(v) => fmt::UpperHex::fmt(v, f),
        }
    }
}

impl fmt::Binary for Uint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uint::Machine(v) => fmt::Binary::fmt(v, f),
            Uint::Promoted(v) => fmt::Binary::fmt(v, f),
        }
    }
}

impl fmt::Octal for Uint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Uint::Machine(v) => fmt::Octal::fmt(v, f),
            Uint::Promoted(v) => fmt::Octal::fmt(v, f),
        }
    }
}
