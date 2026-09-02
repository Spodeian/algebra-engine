//! # `algebra_advanced::padic`
//!
//! $p$-adic Numbers $\mathbb{Q}_p$, Ring of $p$-adic Integers $\mathbb{Z}_p$ & Hensel's Lemma Root Lifter.
//!
//! Encompasses:
//! - **$p$-adic Valuation**: $v_p(x) \in \mathbb{Z} \cup \{\infty\}$.
//! - **$p$-adic Norm & Ultrametric Inequality**: $|x|_p = p^{-v_p(x)}$, $|x + y|_p \le \max(|x|_p, |y|_p)$.
//! - **Canonical $p$-adic Expansion**: $x = \sum_{k=v_p(x)}^\infty d_k p^k$ with digits $d_k \in \{0, \dots, p-1\}$.
//! - **Hensel's Lemma**: Quadratic and linear polynomial root lifting $f(x) \equiv 0 \pmod{p^k} \implies f(\tilde{x}) \equiv 0 \pmod{p^{k+1}}$.

use algebra_core::error::{AlgebraError, AlgebraResult};
use std::fmt;

/// Representation of a $p$-adic number $x \in \mathbb{Q}_p$ with finite precision $N$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PadicNumber {
    /// Prime base $p \ge 2$.
    pub p: u64,
    /// $p$-adic valuation $v_p(x) \in \mathbb{Z}$.
    pub valuation: i64,
    /// Digits $d_0, d_1, \dots, d_{N-1}$ where $d_k \in [0, p-1]$.
    /// Number represents $p^{\text{valuation}} \sum_{k=0}^{N-1} d_k p^k$.
    pub digits: Vec<u64>,
}

impl PadicNumber {
    /// Creates a zero $p$-adic number.
    pub fn zero(p: u64) -> Self {
        Self {
            p,
            valuation: 100, // infinity representation
            digits: vec![0],
        }
    }

    /// Checks if the $p$-adic number is zero.
    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0) || self.valuation > 50
    }

    /// Creates a $p$-adic number from a rational fraction $n / d$.
    pub fn from_rational(mut n: i64, mut d: i64, p: u64, precision: usize) -> AlgebraResult<Self> {
        if d == 0 {
            return Err(AlgebraError::DivisionByZero {
                domain: format!("Q_{p}"),
            });
        }
        if n == 0 {
            return Ok(Self::zero(p));
        }

        let p_i = p as i64;
        let mut val_n = 0i64;
        while n % p_i == 0 {
            n /= p_i;
            val_n += 1;
        }

        let mut val_d = 0i64;
        while d % p_i == 0 {
            d /= p_i;
            val_d += 1;
        }

        let valuation = val_n - val_d;

        // Invert d modulo p^precision
        let mut digits = Vec::with_capacity(precision);
        let mut current_n = n;
        let mut current_d = d;

        // Simplify signs
        if current_d < 0 {
            current_n = -current_n;
            current_d = -current_d;
        }

        for _ in 0..precision {
            // Find d_k in [0, p-1] such that current_n - d_k * current_d is divisible by p
            // d_k = (current_n * inv(current_d)) mod p
            let d_mod_p = ((current_d % p_i) + p_i) % p_i;
            let inv_d = Self::mod_inverse(d_mod_p as u64, p)?;
            let n_mod_p = ((current_n % p_i) + p_i) % p_i;
            let digit = (n_mod_p as u64 * inv_d) % p;

            digits.push(digit);

            // (current_n - digit * current_d) / p
            current_n = (current_n - digit as i64 * current_d) / p_i;
        }

        Ok(Self {
            p,
            valuation,
            digits,
        })
    }

    /// Computes modular inverse using extended Euclidean algorithm.
    fn mod_inverse(a: u64, m: u64) -> AlgebraResult<u64> {
        let (g, x, _) = Self::ext_gcd(a as i64, m as i64);
        if g != 1 {
            return Err(AlgebraError::EvaluationError(format!(
                "No modular inverse for {a} mod {m}"
            )));
        }
        let inv = (x % m as i64 + m as i64) % m as i64;
        Ok(inv as u64)
    }

    fn ext_gcd(a: i64, b: i64) -> (i64, i64, i64) {
        if b == 0 {
            (a, 1, 0)
        } else {
            let (g, x1, y1) = Self::ext_gcd(b, a % b);
            (g, y1, x1 - (a / b) * y1)
        }
    }

    /// Computes the $p$-adic norm $|x|_p = p^{-v_p(x)}$.
    pub fn norm(&self) -> f64 {
        if self.is_zero() {
            0.0
        } else {
            (self.p as f64).powf(-(self.valuation as f64))
        }
    }

    /// Verifies the Ultrametric Inequality: $|x + y|_p \le \max(|x|_p, |y|_p)$.
    pub fn verify_ultrametric(&self, other: &Self) -> bool {
        if self.p != other.p {
            return false;
        }
        let sum = self.add(other);
        let norm_sum = sum.norm();
        let max_norm = self.norm().max(other.norm());
        norm_sum <= max_norm + 1e-12
    }

    /// Addition in $\mathbb{Q}_p$.
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.p, other.p);
        if self.is_zero() {
            return other.clone();
        }
        if other.is_zero() {
            return self.clone();
        }

        let min_val = self.valuation.min(other.valuation);
        let prec = self.digits.len().min(other.digits.len());

        let mut aligned_a = vec![0u64; prec];
        let mut aligned_b = vec![0u64; prec];

        let offset_a = (self.valuation - min_val) as usize;
        let offset_b = (other.valuation - min_val) as usize;

        for (i, &d) in self.digits.iter().enumerate() {
            if i + offset_a < prec {
                aligned_a[i + offset_a] = d;
            }
        }
        for (i, &d) in other.digits.iter().enumerate() {
            if i + offset_b < prec {
                aligned_b[i + offset_b] = d;
            }
        }

        let mut res_digits = vec![0u64; prec];
        let mut carry = 0u64;
        let p = self.p;

        for i in 0..prec {
            let total = aligned_a[i] + aligned_b[i] + carry;
            res_digits[i] = total % p;
            carry = total / p;
        }

        // Adjust valuation if leading digits are 0
        let mut actual_val = min_val;
        let mut shift = 0;
        while shift < res_digits.len() && res_digits[shift] == 0 {
            shift += 1;
            actual_val += 1;
        }

        if shift == res_digits.len() {
            Self::zero(p)
        } else {
            let final_digits = res_digits[shift..].to_vec();
            Self {
                p,
                valuation: actual_val,
                digits: final_digits,
            }
        }
    }
}

impl fmt::Display for PadicNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            write!(f, "0 (in Q_{})", self.p)
        } else {
            write!(f, "p^{} * (", self.valuation)?;
            for (i, &d) in self.digits.iter().enumerate() {
                if i > 0 {
                    write!(f, " + ")?;
                }
                write!(f, "{d}*p^{i}")?;
            }
            write!(f, " + O(p^{})) in Q_{}", self.digits.len(), self.p)
        }
    }
}

/// Hensel's Lemma Root Lifter for polynomial congruences in $\mathbb{Z}_p$.
pub struct HenselLifter;

impl HenselLifter {
    /// Lifts a simple root $r_0$ of polynomial $P(x) \equiv 0 \pmod p$ with $P'(r_0) \not\equiv 0 \pmod p$
    /// to precision $p^k$ in $\mathbb{Z}_p$.
    pub fn lift_root(
        coeffs: &[i64], // highest to lowest degree
        r0: u64,
        p: u64,
        precision: usize,
    ) -> AlgebraResult<PadicNumber> {
        let deg = coeffs.len() - 1;
        let mut current_r = r0;
        let mut current_mod = p;

        // Evaluate P'(r0) mod p
        let mut deriv_val = 0i64;
        for (i, &c) in coeffs.iter().enumerate().take(deg) {
            let power = deg - i;
            let term = c * power as i64 * (r0 as i64).pow((power - 1) as u32);
            deriv_val += term;
        }
        let f_prime_mod_p = ((deriv_val % p as i64) + p as i64) % p as i64;
        if f_prime_mod_p == 0 {
            return Err(AlgebraError::EvaluationError(
                "Hensel condition failed: derivative is 0 mod p (multiple root)".into(),
            ));
        }

        let inv_f_prime = PadicNumber::mod_inverse(f_prime_mod_p as u64, p)?;

        let mut digits = vec![r0];

        for _ in 1..precision {
            // Evaluate P(current_r)
            let mut f_val = 0i128;
            for (i, &c) in coeffs.iter().enumerate() {
                let power = deg - i;
                let term = (c as i128) * (current_r as i128).pow(power as u32);
                f_val += term;
            }

            // delta = -P(current_r) / p^k mod p
            let f_val_scaled = (f_val / (current_mod as i128)) % (p as i128);
            let f_val_pos = ((f_val_scaled % p as i128) + p as i128) % p as i128;
            let next_digit =
                ((-(f_val_pos as i64) * inv_f_prime as i64) % p as i64 + p as i64) as u64 % p;

            digits.push(next_digit);
            current_r += next_digit * current_mod;
            current_mod *= p;
        }

        Ok(PadicNumber {
            p,
            valuation: 0,
            digits,
        })
    }
}
