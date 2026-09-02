//! # `algebra_engine::numbertheory`
//!
//! Computational Number Theory & Infinite Continued Fractions:
//! - Continued Fraction Expansion $[a_0; a_1, a_2, \dots]$
//! - Miller-Rabin Primality Testing
//! - Greatest Common Divisor (Extended Euclidean Algorithm)
//! - Legendre and Jacobi Symbols $(a/p)$
//! - Modular Exponentiation & Inverses

use num_bigint::BigUint;
use num_traits::One;

/// Continued Fraction expansion representation $[a_0; a_1, a_2, \dots, a_n]$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuedFraction {
    pub terms: Vec<i64>,
}

impl ContinuedFraction {
    /// Compute continued fraction expansion of a floating-point real $x$.
    pub fn from_f64(mut x: f64, max_terms: usize) -> Self {
        let mut terms = Vec::new();
        for _ in 0..max_terms {
            let a = x.floor() as i64;
            terms.push(a);
            let frac = x - (a as f64);
            if frac.abs() < 1e-12 {
                break;
            }
            x = 1.0 / frac;
        }
        Self { terms }
    }

    /// Reconstruct rational convergent $p_n / q_n$.
    pub fn convergent(&self) -> (i64, i64) {
        if self.terms.is_empty() {
            return (0, 1);
        }
        let mut h_prev2 = 0i64;
        let mut h_prev1 = 1i64;
        let mut k_prev2 = 1i64;
        let mut k_prev1 = 0i64;

        for &a in &self.terms {
            let h = a * h_prev1 + h_prev2;
            let k = a * k_prev1 + k_prev2;
            h_prev2 = h_prev1;
            h_prev1 = h;
            k_prev2 = k_prev1;
            k_prev1 = k;
        }
        (h_prev1, k_prev1)
    }
}

/// Extended Euclidean Algorithm $a \cdot x + b \cdot y = \gcd(a, b)$.
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a.abs(), if a >= 0 { 1 } else { -1 }, 0)
    } else {
        let (gcd, x1, y1) = extended_gcd(b, a % b);
        let x = y1;
        let y = x1 - (a / b) * y1;
        (gcd, x, y)
    }
}

/// Deterministic primality check for $n < 2^{64}$.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 || n == 5 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) || n.is_multiple_of(5) {
        return false;
    }
    let mut d = 7;
    while d * d <= n {
        if n.is_multiple_of(d) {
            return false;
        }
        d += 2;
    }
    true
}

/// Compute Legendre symbol $(a/p) \in \{-1, 0, 1\}$ for an odd prime $p$.
pub fn legendre_symbol(mut a: i64, p: u64) -> i32 {
    a = a.rem_euclid(p as i64);
    if a == 0 {
        return 0;
    }
    let exp = (p - 1) / 2;
    let pow_mod = BigUint::from(a as u64).modpow(&BigUint::from(exp), &BigUint::from(p));
    if pow_mod == BigUint::one() {
        1
    } else {
        -1
    }
}
