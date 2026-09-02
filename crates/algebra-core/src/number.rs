//! Scalar numbers and mathematical constants.

use std::fmt;

/// Built-in mathematical constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Constant {
    /// Archimedian constant Pi (π ≈ 3.14159...)
    Pi,
    /// Euler's number e (e ≈ 2.71828...)
    E,
    /// Imaginary unit i (i^2 = -1)
    I,
    /// Euler-Mascheroni constant γ (γ ≈ 0.57721...)
    EulerGamma,
    /// Golden ratio φ (φ ≈ 1.61803...)
    GoldenRatio,
    /// Apéry's constant ζ(3) (ζ(3) ≈ 1.20205...)
    Apery,
    /// Catalan's constant G (G ≈ 0.91596...)
    Catalan,
    /// Positive infinity (+∞)
    Infinity,
    /// Negative infinity (-∞)
    NegInfinity,
    /// Undefined / Not-a-Number mathematical result (NaN / Complex infinity)
    Undefined,
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pi => write!(f, "π"),
            Self::E => write!(f, "e"),
            Self::I => write!(f, "i"),
            Self::EulerGamma => write!(f, "γ"),
            Self::GoldenRatio => write!(f, "φ"),
            Self::Apery => write!(f, "ζ(3)"),
            Self::Catalan => write!(f, "G"),
            Self::Infinity => write!(f, "∞"),
            Self::NegInfinity => write!(f, "-∞"),
            Self::Undefined => write!(f, "Undefined"),
        }
    }
}

/// Scalar numbers representation used in expression DAG nodes.
#[derive(Debug, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Number {
    /// Exact 64-bit signed integer.
    Integer(i64),
    /// Exact rational fraction (numerator, denominator).
    Rational(i64, i64),
    /// 64-bit floating point number (ordered for hashing).
    Float(u64),
    /// Mathematical constant.
    Constant(Constant),
}

impl Constant {
    /// Numerical floating-point approximation of constant.
    pub const fn approx_f64(&self) -> Option<f64> {
        match self {
            Self::Pi => Some(std::f64::consts::PI),
            Self::E => Some(std::f64::consts::E),
            Self::EulerGamma => Some(0.5772156649015329),
            Self::GoldenRatio => Some(1.618033988749895),
            Self::Apery => Some(1.202056903159594),
            Self::Catalan => Some(0.915965594177219),
            Self::Infinity => Some(f64::INFINITY),
            Self::NegInfinity => Some(f64::NEG_INFINITY),
            Self::I | Self::Undefined => None,
        }
    }
}

impl Number {
    /// Static Zero constant: `0`.
    pub const ZERO: Self = Self::Integer(0);
    /// Static Unity constant: `1`.
    pub const ONE: Self = Self::Integer(1);
    /// Static Pi constant: `π`.
    pub const PI: Self = Self::Constant(Constant::Pi);
    /// Static Euler's Number constant: `e`.
    pub const E: Self = Self::Constant(Constant::E);
    /// Static Imaginary Unit: `i`.
    pub const I: Self = Self::Constant(Constant::I);
    /// Static Positive Infinity: `+∞`.
    pub const INFINITY: Self = Self::Constant(Constant::Infinity);
    /// Static Negative Infinity: `-∞`.
    pub const NEG_INFINITY: Self = Self::Constant(Constant::NegInfinity);
    /// Static Undefined constant: `Undefined`.
    pub const UNDEFINED: Self = Self::Constant(Constant::Undefined);

    /// Create an exact integer number in const environments.
    #[inline]
    pub const fn integer(val: i64) -> Self {
        Self::Integer(val)
    }

    /// Create an exact rational fraction in const environments.
    #[inline]
    pub const fn rational(num: i64, den: i64) -> Self {
        Self::Rational(num, den)
    }

    /// Create a mathematical constant in const environments.
    #[inline]
    pub const fn constant(c: Constant) -> Self {
        Self::Constant(c)
    }

    /// Create a float number from raw IEEE-754 bit pattern in const environments.
    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self::Float(bits)
    }

    /// Create a float number from `f64`, converting bit pattern for hash consistency.
    #[inline]
    pub const fn float(val: f64) -> Self {
        Self::Float(val.to_bits())
    }

    /// Extract `f64` float value if float node.
    #[inline]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(bits) => Some(f64::from_bits(*bits)),
            Self::Integer(i) => Some(*i as f64),
            Self::Rational(num, den) => Some(*num as f64 / *den as f64),
            Self::Constant(c) => c.approx_f64(),
        }
    }

    /// Returns `true` if number represents zero (integer 0, rational 0/n, or float 0.0).
    #[inline]
    pub const fn is_zero(&self) -> bool {
        match self {
            Self::Integer(i) => *i == 0,
            Self::Rational(num, _) => *num == 0,
            Self::Float(bits) => {
                let f = f64::from_bits(*bits);
                f == 0.0
            }
            _ => false,
        }
    }

    /// Returns `true` if number represents unity (integer 1, rational n/n, float 1.0).
    #[inline]
    pub const fn is_one(&self) -> bool {
        match self {
            Self::Integer(i) => *i == 1,
            Self::Rational(num, den) => *num == *den && *den != 0,
            Self::Float(bits) => {
                let f = f64::from_bits(*bits);
                f == 1.0
            }
            _ => false,
        }
    }
}

impl Eq for Number {}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(i) => write!(f, "{}", i),
            Self::Rational(num, den) => write!(f, "{}/{}", num, den),
            Self::Float(bits) => write!(f, "{}", f64::from_bits(*bits)),
            Self::Constant(c) => write!(f, "{}", c),
        }
    }
}
