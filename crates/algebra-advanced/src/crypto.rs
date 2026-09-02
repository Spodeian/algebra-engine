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
