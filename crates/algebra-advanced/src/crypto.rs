//! # `algebra-crypto`
//!
//! Elliptic Curve Cryptography over finite fields ($E/GF(p^n)$), Error-Correcting Codes
//! (Reed-Solomon, Hamming), and Symbolic Information Theory (Entropy $H(X)$, Mutual Information $I(X; Y)$).

use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId};
use num_bigint::BigInt;
use num_traits::Zero;

/// Point on an Elliptic Curve $y^2 = x^3 + ax + b \pmod p$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EllipticPoint {
    Point { x: BigInt, y: BigInt },
    IdentityAtInfinity,
}

/// Elliptic Curve over prime finite field $GF(p)$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EllipticCurve {
    pub a: BigInt,
    pub b: BigInt,
    pub p: BigInt,
}

impl EllipticCurve {
    pub fn new(a: BigInt, b: BigInt, p: BigInt) -> AlgebraResult<Self> {
        // Verify non-singular discriminant: 4a^3 + 27b^2 != 0 (mod p)
        let a3 = (&a * &a * &a) * 4;
        let b2 = (&b * &b) * 27;
        let disc: BigInt = (a3 + b2) % &p;
        if disc.is_zero() {
            return Err(AlgebraError::DomainViolation {
                domain: "EllipticCurve".to_string(),
                reason: "Elliptic curve is singular (discriminant is 0 mod p)".to_string(),
            });
        }
        Ok(Self { a, b, p })
    }

    /// Check if point $(x, y)$ satisfies $y^2 \equiv x^3 + ax + b \pmod p$.
    pub fn contains_point(&self, point: &EllipticPoint) -> bool {
        match point {
            EllipticPoint::IdentityAtInfinity => true,
            EllipticPoint::Point { x, y } => {
                let lhs = (y * y) % &self.p;
                let rhs = ((x * x * x) + &self.a * x + &self.b) % &self.p;
                let lhs_norm = (lhs + &self.p) % &self.p;
                let rhs_norm = (rhs + &self.p) % &self.p;
                lhs_norm == rhs_norm
            }
        }
    }
}

/// Reduction type of an elliptic curve modulo a prime place $p$.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReductionType {
    /// Non-singular reduction ($p \nmid \Delta$).
    Good,
    /// Multiplicative (nodal) reduction ($p \mid \Delta$ and $p \nmid c_4$).
    Multiplicative { split: bool },
    /// Additive (cuspidal) reduction ($p \mid \Delta$ and $p \mid c_4$).
    Additive,
}

/// Elliptic curve defined over $\mathbb{Q}$ via Weierstrass equation $y^2 = x^3 + ax + b$ with $a, b \in \mathbb{Z}$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RationalEllipticCurve {
    pub a: i64,
    pub b: i64,
}

impl RationalEllipticCurve {
    pub fn new(a: i64, b: i64) -> AlgebraResult<Self> {
        let disc = Self::compute_discriminant(a, b);
        if disc == 0 {
            return Err(AlgebraError::DomainViolation {
                domain: "RationalEllipticCurve".to_string(),
                reason: "Discriminant Delta = -16(4a^3 + 27b^2) is zero (curve is singular over Q)"
                    .to_string(),
            });
        }
        Ok(Self { a, b })
    }

    /// Compute discriminant $\Delta = -16(4a^3 + 27b^2)$.
    pub fn discriminant(&self) -> i128 {
        Self::compute_discriminant(self.a, self.b)
    }

    fn compute_discriminant(a: i64, b: i64) -> i128 {
        let a = a as i128;
        let b = b as i128;
        -16 * (4 * a * a * a + 27 * b * b)
    }

    /// Compute invariants $c_4 = -48a$.
    pub fn c4(&self) -> i128 {
        -48 * (self.a as i128)
    }

    /// Identify the bad reduction places (primes dividing $\Delta$) using prime factorization.
    pub fn bad_reduction_places(&self) -> Vec<u64> {
        let disc_abs = self.discriminant().unsigned_abs();
        if disc_abs <= 1 {
            return Vec::new();
        }
        let factors = algebra_engine::numbertheory::integer_prime_factors(disc_abs as u64);
        factors.into_iter().map(|(p, _)| p).collect()
    }

    /// Determine the reduction type at a given prime place $p$.
    pub fn reduction_at(&self, p: u64) -> ReductionType {
        let disc = self.discriminant();
        let p_i128 = p as i128;
        if disc % p_i128 != 0 {
            ReductionType::Good
        } else {
            let c4 = self.c4();
            if c4 % p_i128 != 0 {
                // Multiplicative reduction: check if split (tangents in F_p)
                // For y^2 = x^3 + ax + b, at nodal singularity (x0, 0),
                // root of 3x0^2 + a in F_p is quadratic residue
                let split =
                    p > 2 && algebra_engine::numbertheory::legendre_symbol(-2 * self.a, p) == 1;
                ReductionType::Multiplicative { split }
            } else {
                ReductionType::Additive
            }
        }
    }

    /// Compute the Frobenius trace $a_p = p + 1 - \#E(\mathbb{F}_p) = -\sum_{x=0}^{p-1} \left(\frac{x^3+ax+b}{p}\right)$
    /// for an odd prime $p$.
    pub fn frobenius_trace(&self, p: u64) -> Option<i64> {
        if p == 2 {
            // Manual count for F_2:
            let mut count = 1i64; // point at infinity
            for x in 0..2 {
                let rhs = (x * x * x + (self.a % 2 + 2) % 2 * x + (self.b % 2 + 2) % 2) % 2;
                if rhs == 0 {
                    count += 1; // (x, 0)
                }
            }
            return Some((p as i64) + 1 - count);
        }

        let mut sum_legendre = 0i64;
        let a_mod = ((self.a % p as i64) + p as i64) % p as i64;
        let b_mod = ((self.b % p as i64) + p as i64) % p as i64;

        for x in 0..p as i64 {
            let x3 = (x * x % p as i64) * x % p as i64;
            let rhs = (x3 + a_mod * x + b_mod) % p as i64;
            let leg = algebra_engine::numbertheory::legendre_symbol(rhs, p);
            sum_legendre += leg as i64;
        }

        Some(-sum_legendre)
    }

    /// Reduce the curve modulo $p$ into an `EllipticCurve` over $\mathbb{F}_p$.
    pub fn reduce_mod_p(&self, p: u64) -> AlgebraResult<EllipticCurve> {
        let p_big = BigInt::from(p);
        let a_mod = (BigInt::from(self.a) % &p_big + &p_big) % &p_big;
        let b_mod = (BigInt::from(self.b) % &p_big + &p_big) % &p_big;
        EllipticCurve::new(a_mod, b_mod, p_big)
    }
}

/// Reed-Solomon Code generator structure over finite field.
#[derive(Debug, Clone)]
pub struct ReedSolomonCode {
    pub n: usize,
    pub k: usize,
}

impl ReedSolomonCode {
    pub fn new(n: usize, k: usize) -> AlgebraResult<Self> {
        if k >= n {
            return Err(AlgebraError::DomainViolation {
                domain: "ReedSolomonCode".to_string(),
                reason: format!(
                    "Reed-Solomon k ({}) must be strictly less than n ({})",
                    k, n
                ),
            });
        }
        Ok(Self { n, k })
    }

    pub fn parity_symbols(&self) -> usize {
        self.n - self.k
    }
}

/// Symbolic Information Theory utilities.
#[derive(Debug, Clone, Default)]
pub struct InformationTheory;

impl InformationTheory {
    /// Symbolic Shannon Entropy $H(X) = -\sum_{i} P(x_i) \log_2 P(x_i)$.
    pub fn entropy(&self, graph: &ExprGraph, probabilities: &[ExprId]) -> ExprId {
        let mut terms = Vec::with_capacity(probabilities.len());

        for &p in probabilities {
            let log_p = graph.function("log2", [p]);
            let p_log_p = graph.mul([p, log_p]);
            terms.push(p_log_p);
        }

        let sum = graph.add(terms);
        graph.mul([graph.integer(-1), sum])
    }

    /// Symbolic Mutual Information $I(X; Y) = H(X) + H(Y) - H(X, Y)$.
    pub fn mutual_information(
        &self,
        graph: &ExprGraph,
        h_x: ExprId,
        h_y: ExprId,
        h_xy: ExprId,
    ) -> ExprId {
        let neg_h_xy = graph.mul([graph.integer(-1), h_xy]);
        graph.add([h_x, h_y, neg_h_xy])
    }

    /// Symbolic Kullback-Leibler Divergence $D_{KL}(P \parallel Q) = \sum P(x) \log \left(\frac{P(x)}{Q(x)}\right)$.
    pub fn kl_divergence(&self, graph: &ExprGraph, p_dist: &[ExprId], q_dist: &[ExprId]) -> ExprId {
        let mut terms = Vec::new();

        for (&p, &q) in p_dist.iter().zip(q_dist) {
            let ratio = graph.div(p, q);
            let log_ratio = graph.function("log", [ratio]);
            let term = graph.mul([p, log_ratio]);
            terms.push(term);
        }

        graph.add(terms)
    }
}
