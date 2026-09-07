use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num::{
    bigint::ParseBigIntError,
    traits::{ConstOne, ConstZero, Signed},
    BigInt, Integer, Num, One, Zero,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Int {
    Machine(i64),
    Promoted(BigInt),
}

impl Int {
    pub const ZERO: Self = Self::Machine(0);
    pub const ONE: Self = Self::Machine(1);

    pub fn to_bigint(&self) -> BigInt {
        match self {
            Self::Machine(v) => BigInt::from(*v),
            Self::Promoted(b) => b.clone(),
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Machine(v) => Some(*v),
            Self::Promoted(b) => {
                use num::ToPrimitive;
                b.to_i64()
            }
        }
    }

    pub fn is_promoted(&self) -> bool {
        matches!(self, Self::Promoted(_))
    }

    pub fn demote_if_possible(self) -> Self {
        match self {
            Self::Promoted(b) => {
                use num::ToPrimitive;
                if let Some(i) = b.to_i64() {
                    Self::Machine(i)
                } else {
                    Self::Promoted(b)
                }
            }
            other => other,
        }
    }
}

impl From<i64> for Int {
    fn from(v: i64) -> Self {
        Self::Machine(v)
    }
}

impl From<i32> for Int {
    fn from(v: i32) -> Self {
        Self::Machine(v as i64)
    }
}

impl From<isize> for Int {
    fn from(v: isize) -> Self {
        Self::Machine(v as i64)
    }
}

impl From<BigInt> for Int {
    fn from(b: BigInt) -> Self {
        Self::Promoted(b).demote_if_possible()
    }
}

impl Ord for Int {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Machine(a), Self::Machine(b)) => a.cmp(b),
            (Self::Machine(a), Self::Promoted(b)) => BigInt::from(*a).cmp(b),
            (Self::Promoted(a), Self::Machine(b)) => a.cmp(&BigInt::from(*b)),
            (Self::Promoted(a), Self::Promoted(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Int {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Int {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Zero for Int {
    fn zero() -> Self {
        Self::ZERO
    }

    fn set_zero(&mut self) {
        *self = Self::ZERO;
    }

    fn is_zero(&self) -> bool {
        match self {
            Self::Machine(v) => *v == 0,
            Self::Promoted(b) => b.is_zero(),
        }
    }
}

impl ConstZero for Int {
    const ZERO: Self = Self::Machine(0);
}

impl One for Int {
    fn one() -> Self {
        Self::ONE
    }

    fn set_one(&mut self) {
        *self = Self::ONE;
    }

    fn is_one(&self) -> bool {
        match self {
            Self::Machine(v) => *v == 1,
            Self::Promoted(b) => b.is_one(),
        }
    }
}

impl ConstOne for Int {
    const ONE: Self = Self::Machine(1);
}

impl Num for Int {
    type FromStrRadixErr = ParseBigIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        BigInt::from_str_radix(str, radix).map(Int::from)
    }
}

impl Signed for Int {
    fn abs(&self) -> Self {
        match self {
            Self::Machine(v) => {
                if let Some(pos) = v.checked_abs() {
                    Self::Machine(pos)
                } else {
                    Self::Promoted(BigInt::from(*v).abs())
                }
            }
            Self::Promoted(b) => Self::Promoted(b.abs()).demote_if_possible(),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Self::ZERO
        } else {
            self.clone() - other.clone()
        }
    }

    fn signum(&self) -> Self {
        match self {
            Self::Machine(v) => Self::Machine(v.signum()),
            Self::Promoted(b) => {
                use num::ToPrimitive;
                Self::Machine(b.signum().to_i64().unwrap_or(0))
            }
        }
    }

    fn is_positive(&self) -> bool {
        match self {
            Self::Machine(v) => *v > 0,
            Self::Promoted(b) => b.is_positive(),
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            Self::Machine(v) => *v < 0,
            Self::Promoted(b) => b.is_negative(),
        }
    }
}

impl Integer for Int {
    fn div_rem(&self, other: &Self) -> (Self, Self) {
        match (self, other) {
            (Self::Machine(a), Self::Machine(b)) if *b != 0 => {
                if let (Some(q), Some(r)) = (a.checked_div(*b), a.checked_rem(*b)) {
                    (Self::Machine(q), Self::Machine(r))
                } else {
                    let (q, r) = self.to_bigint().div_rem(&other.to_bigint());
                    (Self::from(q), Self::from(r))
                }
            }
            _ => {
                let (q, r) = self.to_bigint().div_rem(&other.to_bigint());
                (Self::from(q), Self::from(r))
            }
        }
    }

    fn div_floor(&self, other: &Self) -> Self {
        self.div_rem(other).0
    }

    fn mod_floor(&self, other: &Self) -> Self {
        self.div_rem(other).1
    }

    fn gcd(&self, other: &Self) -> Self {
        let g = self.to_bigint().gcd(&other.to_bigint());
        Self::from(g)
    }

    fn lcm(&self, other: &Self) -> Self {
        let l = self.to_bigint().lcm(&other.to_bigint());
        Self::from(l)
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        if other.is_zero() {
            return self.is_zero();
        }
        (self.clone() % other.clone()).is_zero()
    }

    fn is_even(&self) -> bool {
        match self {
            Self::Machine(v) => v % 2 == 0,
            Self::Promoted(b) => b.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        !self.is_even()
    }
}

impl Neg for Int {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Machine(v) => {
                if let Some(neg) = v.checked_neg() {
                    Self::Machine(neg)
                } else {
                    Self::Promoted(-BigInt::from(v))
                }
            }
            Self::Promoted(b) => Self::from(-b),
        }
    }
}

impl Add for Int {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Machine(a), Self::Machine(b)) => {
                if let Some(c) = a.checked_add(b) {
                    Self::Machine(c)
                } else {
                    Self::from(BigInt::from(a) + BigInt::from(b))
                }
            }
            (a, b) => Self::from(a.to_bigint() + b.to_bigint()),
        }
    }
}

impl Sub for Int {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Machine(a), Self::Machine(b)) => {
                if let Some(c) = a.checked_sub(b) {
                    Self::Machine(c)
                } else {
                    Self::from(BigInt::from(a) - BigInt::from(b))
                }
            }
            (a, b) => Self::from(a.to_bigint() - b.to_bigint()),
        }
    }
}

impl Mul for Int {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Machine(a), Self::Machine(b)) => {
                if let Some(c) = a.checked_mul(b) {
                    Self::Machine(c)
                } else {
                    Self::from(BigInt::from(a) * BigInt::from(b))
                }
            }
            (a, b) => Self::from(a.to_bigint() * b.to_bigint()),
        }
    }
}

impl Div for Int {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Machine(a), Self::Machine(b)) => {
                if let Some(c) = a.checked_div(b) {
                    Self::Machine(c)
                } else {
                    Self::from(BigInt::from(a) / BigInt::from(b))
                }
            }
            (a, b) => Self::from(a.to_bigint() / b.to_bigint()),
        }
    }
}

impl Rem for Int {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Machine(a), Self::Machine(b)) => {
                if let Some(c) = a.checked_rem(b) {
                    Self::Machine(c)
                } else {
                    Self::from(BigInt::from(a) % BigInt::from(b))
                }
            }
            (a, b) => Self::from(a.to_bigint() % b.to_bigint()),
        }
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Machine(v) => write!(f, "{}", v),
            Self::Promoted(b) => write!(f, "{}", b),
        }
    }
}
