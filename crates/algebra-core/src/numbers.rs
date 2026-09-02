//! # `algebra-numbers`
//!
//! Advanced and esoteric number systems: Quaternions (ℍ), Octonions (𝕆), P-adics (ℚ_p), Adeles (𝔸), Surreal numbers, Dual numbers, and Hyperreals.

use num_rational::BigRational;

/// Quaternion q = w + x*i + y*j + z*k in ℍ where i^2 = j^2 = k^2 = ijk = -1.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Quaternion {
    pub w: BigRational,
    pub x: BigRational,
    pub y: BigRational,
    pub z: BigRational,
}

impl Quaternion {
    /// Create a new Quaternion w + xi + yj + zk.
    pub fn new(w: BigRational, x: BigRational, y: BigRational, z: BigRational) -> Self {
        Self { w, x, y, z }
    }

    /// Conjugate q* = w - xi - yj - zk.
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w.clone(),
            x: -self.x.clone(),
            y: -self.y.clone(),
            z: -self.z.clone(),
        }
    }

    /// Quaternion addition q1 + q2.
    pub fn add(&self, other: &Quaternion) -> Quaternion {
        Self {
            w: &self.w + &other.w,
            x: &self.x + &other.x,
            y: &self.y + &other.y,
            z: &self.z + &other.z,
        }
    }

    /// Non-commutative Hamilton quaternion multiplication q1 * q2.
    pub fn mul(&self, other: &Quaternion) -> Quaternion {
        let w = &self.w * &other.w - &self.x * &other.x - &self.y * &other.y - &self.z * &other.z;
        let x = &self.w * &other.x + &self.x * &other.w + &self.y * &other.z - &self.z * &other.y;
        let y = &self.w * &other.y - &self.x * &other.z + &self.y * &other.w + &self.z * &other.x;
        let z = &self.w * &other.z + &self.x * &other.y - &self.y * &other.x + &self.z * &other.w;
        Self { w, x, y, z }
    }

    /// Norm squared ||q||^2 = w^2 + x^2 + y^2 + z^2.
    pub fn norm_sq(&self) -> BigRational {
        &self.w * &self.w + &self.x * &self.x + &self.y * &self.y + &self.z * &self.z
    }
}

/// Octonion o in 𝕆 (8-dimensional non-associative normed division algebra).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Octonion {
    pub c: [BigRational; 8],
}

impl Octonion {
    pub fn new(c: [BigRational; 8]) -> Self {
        Self { c }
    }
}

/// P-adic Number representation x = p^v * (a_0 + a_1*p + a_2*p^2 + ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PAdicNumber {
    /// Prime base p.
    pub prime: u32,
    /// P-adic valuation exponent v_p(x).
    pub valuation: i32,
    /// p-adic expansion digits [a_0, a_1, a_2, ...].
    pub digits: Vec<u32>,
}

impl PAdicNumber {
    /// Create a p-adic number representation.
    pub fn new(prime: u32, valuation: i32, digits: Vec<u32>) -> Self {
        Self {
            prime,
            valuation,
            digits,
        }
    }
}

/// Surreal Number x = { L | R } recursive set construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurrealNumber {
    /// Left set L (all elements x_L < x).
    pub left: Vec<SurrealNumber>,
    /// Right set R (all elements x_R > x).
    pub right: Vec<SurrealNumber>,
}

impl SurrealNumber {
    /// Zero surreal number 0 = { ∅ | ∅ }.
    pub fn zero() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    /// One surreal number 1 = { {0} | ∅ }.
    pub fn one() -> Self {
        Self {
            left: vec![Self::zero()],
            right: Vec::new(),
        }
    }

    /// Minus one surreal number -1 = { ∅ | {0} }.
    pub fn neg_one() -> Self {
        Self {
            left: Vec::new(),
            right: vec![Self::zero()],
        }
    }
}

/// Dual Number z = a + b*ε where ε^2 = 0.
///
/// Used for $O(1)$ exact forward-mode Automatic Differentiation: $f(a + \epsilon) = f(a) + f'(a)\epsilon$.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DualNumber {
    /// Real component a = f(x).
    pub real: f64,
    /// Dual component b = f'(x) (tangent).
    pub dual: f64,
}

impl DualNumber {
    /// Static Zero constant: `0.0 + 0.0*ε`.
    pub const ZERO: Self = Self {
        real: 0.0,
        dual: 0.0,
    };
    /// Static Unity constant: `1.0 + 0.0*ε`.
    pub const ONE: Self = Self {
        real: 1.0,
        dual: 0.0,
    };
    /// Static Dual Unit constant: `0.0 + 1.0*ε` ($ε^2 = 0$).
    pub const EPSILON: Self = Self {
        real: 0.0,
        dual: 1.0,
    };

    #[inline]
    pub const fn new(real: f64, dual: f64) -> Self {
        Self { real, dual }
    }

    /// Variable input with tangent 1.0 (x + 1*ε).
    #[inline]
    pub const fn variable(val: f64) -> Self {
        Self {
            real: val,
            dual: 1.0,
        }
    }

    /// Constant scalar with tangent 0.0 (c + 0*ε).
    #[inline]
    pub const fn constant(val: f64) -> Self {
        Self {
            real: val,
            dual: 0.0,
        }
    }

    /// Dual addition: (a + b ε) + (c + d ε) = (a + c) + (b + d) ε.
    #[inline]
    pub const fn add(&self, other: &DualNumber) -> DualNumber {
        DualNumber {
            real: self.real + other.real,
            dual: self.dual + other.dual,
        }
    }

    /// Dual multiplication: (a + b ε) * (c + d ε) = (a*c) + (a*d + b*c) ε.
    #[inline]
    pub const fn mul(&self, other: &DualNumber) -> DualNumber {
        DualNumber {
            real: self.real * other.real,
            dual: self.real * other.dual + self.dual * other.real,
        }
    }

    /// Sin evaluation via Taylor: sin(a + b ε) = sin(a) + b cos(a) ε.
    pub fn sin(&self) -> DualNumber {
        DualNumber::new(self.real.sin(), self.dual * self.real.cos())
    }

    /// Cos evaluation via Taylor: cos(a + b ε) = cos(a) - b sin(a) ε.
    pub fn cos(&self) -> DualNumber {
        DualNumber::new(self.real.cos(), -self.dual * self.real.sin())
    }

    /// Exp evaluation via Taylor: exp(a + b ε) = exp(a) + b exp(a) ε.
    pub fn exp(&self) -> DualNumber {
        let exp_a = self.real.exp();
        DualNumber::new(exp_a, self.dual * exp_a)
    }
}

/// Hyperreal Number *ℝ = standard real + infinitesimal component ε + infinite component ω.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HyperrealNumber {
    pub standard: f64,
    pub infinitesimal: f64,
    pub infinite: f64,
}

impl HyperrealNumber {
    /// Zero hyperreal number: `0.0`.
    pub const ZERO: Self = Self {
        standard: 0.0,
        infinitesimal: 0.0,
        infinite: 0.0,
    };
    /// Infinitesimal epsilon: `ε`.
    pub const EPSILON: Self = Self {
        standard: 0.0,
        infinitesimal: 1.0,
        infinite: 0.0,
    };
    /// Infinite omega: `ω`.
    pub const OMEGA: Self = Self {
        standard: 0.0,
        infinitesimal: 0.0,
        infinite: 1.0,
    };

    #[inline]
    pub const fn new(standard: f64, infinitesimal: f64, infinite: f64) -> Self {
        Self {
            standard,
            infinitesimal,
            infinite,
        }
    }

    /// Standard part operator st(x).
    #[inline]
    pub const fn standard_part(&self) -> f64 {
        self.standard
    }
}
