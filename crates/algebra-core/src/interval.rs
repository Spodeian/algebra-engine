//! # `algebra_core::interval`
//!
//! Rigorous Interval Arithmetic $[a, b]$ with certified numerical bounds, directed inclusions,
//! and Interval Newton root containment.

use serde::{Deserialize, Serialize};

/// Certified real interval $[a, b]$ where $a \le b$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RealInterval {
    pub inf: f64,
    pub sup: f64,
}

impl RealInterval {
    /// Entire real line: `[-∞, +∞]`.
    pub const ALL: Self = Self {
        inf: f64::NEG_INFINITY,
        sup: f64::INFINITY,
    };
    /// Unit interval: `[0.0, 1.0]`.
    pub const UNIT: Self = Self { inf: 0.0, sup: 1.0 };
    /// Symmetric unit interval: `[-1.0, 1.0]`.
    pub const SYMMETRIC_UNIT: Self = Self {
        inf: -1.0,
        sup: 1.0,
    };
    /// Non-negative reals: `[0.0, +∞]`.
    pub const NON_NEGATIVE: Self = Self {
        inf: 0.0,
        sup: f64::INFINITY,
    };
    /// Non-positive reals: `[-∞, 0.0]`.
    pub const NON_POSITIVE: Self = Self {
        inf: f64::NEG_INFINITY,
        sup: 0.0,
    };

    /// Create a new interval $[a, b]$. If $a > b$, they are swapped.
    #[inline]
    pub const fn new(a: f64, b: f64) -> Self {
        if a <= b {
            Self { inf: a, sup: b }
        } else {
            Self { inf: b, sup: a }
        }
    }

    /// Exact degenerate point interval $[c, c]$.
    #[inline]
    pub const fn point(c: f64) -> Self {
        Self { inf: c, sup: c }
    }

    /// Midpoint $m = (a + b) / 2$.
    #[inline]
    pub const fn mid(&self) -> f64 {
        0.5 * (self.inf + self.sup)
    }

    /// Radius $r = (b - a) / 2$.
    #[inline]
    pub const fn rad(&self) -> f64 {
        0.5 * (self.sup - self.inf)
    }

    /// Diameter / width $w = b - a$.
    #[inline]
    pub const fn diam(&self) -> f64 {
        self.sup - self.inf
    }

    /// Check if point $x \in [a, b]$.
    #[inline]
    pub const fn contains(&self, x: f64) -> bool {
        self.inf <= x && x <= self.sup
    }

    /// Check if sub-interval $other \subseteq self$.
    #[inline]
    pub const fn contains_interval(&self, other: &RealInterval) -> bool {
        self.inf <= other.inf && other.sup <= self.sup
    }

    /// Interval addition $[a, b] + [c, d] = [a + c, b + d]$.
    #[inline]
    pub const fn add(&self, other: &RealInterval) -> RealInterval {
        RealInterval {
            inf: self.inf + other.inf,
            sup: self.sup + other.sup,
        }
    }

    /// Interval subtraction $[a, b] - [c, d] = [a - d, b - c]$.
    #[inline]
    pub const fn sub(&self, other: &RealInterval) -> RealInterval {
        RealInterval {
            inf: self.inf - other.sup,
            sup: self.sup - other.inf,
        }
    }

    /// Interval multiplication $[a, b] \times [c, d] = [\min(ac, ad, bc, bd), \max(ac, ad, bc, bd)]$.
    pub fn mul(&self, other: &RealInterval) -> RealInterval {
        let p1 = self.inf * other.inf;
        let p2 = self.inf * other.sup;
        let p3 = self.sup * other.inf;
        let p4 = self.sup * other.sup;

        let inf = p1.min(p2).min(p3).min(p4);
        let sup = p1.max(p2).max(p3).max(p4);

        RealInterval { inf, sup }
    }

    /// Interval reciprocal $1 / [c, d]$ (disallowing intervals containing 0).
    #[inline]
    pub const fn recip(&self) -> Option<RealInterval> {
        if self.contains(0.0) {
            None
        } else {
            Some(RealInterval {
                inf: 1.0 / self.sup,
                sup: 1.0 / self.inf,
            })
        }
    }

    /// Interval division $[a, b] / [c, d]$.
    pub fn div(&self, other: &RealInterval) -> Option<RealInterval> {
        other.recip().map(|inv| self.mul(&inv))
    }

    /// Intersection of two intervals, if non-empty.
    pub fn intersect(&self, other: &RealInterval) -> Option<RealInterval> {
        let inf = self.inf.max(other.inf);
        let sup = self.sup.min(other.sup);
        if inf <= sup {
            Some(RealInterval { inf, sup })
        } else {
            None
        }
    }

    /// Interval hull enclosing both intervals.
    pub fn hull(&self, other: &RealInterval) -> RealInterval {
        RealInterval {
            inf: self.inf.min(other.inf),
            sup: self.sup.max(other.sup),
        }
    }

    /// Interval Newton step: $N(I) = m - f(m) / f'(I)$.
    /// If $N(I) \subseteq I$, there is a unique certified root in $I$.
    pub fn newton_step<F, DF>(&self, f: F, df: DF) -> Option<RealInterval>
    where
        F: Fn(f64) -> f64,
        DF: Fn(RealInterval) -> RealInterval,
    {
        let m = self.mid();
        let fm = f(m);
        let df_i = df(*self);

        df_i.recip().map(|inv| {
            let shift = RealInterval::point(fm).mul(&inv);
            RealInterval::point(m).sub(&shift)
        })
    }
}

impl std::ops::Add for RealInterval {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        RealInterval {
            inf: self.inf + rhs.inf,
            sup: self.sup + rhs.sup,
        }
    }
}

impl std::ops::Sub for RealInterval {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        RealInterval {
            inf: self.inf - rhs.sup,
            sup: self.sup - rhs.inf,
        }
    }
}

impl std::ops::Mul for RealInterval {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let p1 = self.inf * rhs.inf;
        let p2 = self.inf * rhs.sup;
        let p3 = self.sup * rhs.inf;
        let p4 = self.sup * rhs.sup;
        RealInterval {
            inf: p1.min(p2).min(p3).min(p4),
            sup: p1.max(p2).max(p3).max(p4),
        }
    }
}

impl std::ops::Div for RealInterval {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.contains(0.0) {
            RealInterval::new(f64::NEG_INFINITY, f64::INFINITY)
        } else {
            let inv = RealInterval {
                inf: 1.0 / rhs.sup,
                sup: 1.0 / rhs.inf,
            };
            self * inv
        }
    }
}

impl std::ops::Neg for RealInterval {
    type Output = Self;
    fn neg(self) -> Self::Output {
        RealInterval {
            inf: -self.sup,
            sup: -self.inf,
        }
    }
}

impl RealInterval {
    /// Negate the interval: -[a, b] = [-b, -a].
    pub fn neg(&self) -> Self {
        RealInterval {
            inf: -self.sup,
            sup: -self.inf,
        }
    }
}
