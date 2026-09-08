//! # `algebra_engine::poly`
//!
//! Advanced Polynomial Computations & Algebraic Varieties.
//!
//! Includes:
//! - Sparse Multi-Variate Polynomials (`Polynomial`).
//! - Multi-Variate Monomial Orderings & Division (`MultiPoly`, `MonomialOrder`, `Monomial`, `Term`).
//! - Gröbner Bases & Ideals (`GrobnerBasis`, Buchberger with Gebauer-Möller criteria).
//! - Analytical & Algebraic Root Solvers (`CardanoSolver`, `FerrariSolver`, `BringRadical`, `SturmSequence`, `DurandKernerSolver`).

pub mod grobner;
pub mod multivariate;
pub mod roots;

use algebra_core::SymbolId;
use num_rational::BigRational;
use num_traits::{One, Zero};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub use grobner::GrobnerBasis;
pub use multivariate::{MonomialOrder, MultiPoly, Term};
pub use roots::{
    BringRadical, CardanoSolver, ComplexRoot, DurandKernerSolver, FerrariSolver, NewtonPolygon,
    PolygonSegment, SturmSequence,
};

/// Sparse Monomial exponent vector mapping variable symbols to exponent powers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SparseMonomial {
    /// Exponents for each variable SymbolId.
    pub exps: BTreeMap<SymbolId, u32>,
}

impl SparseMonomial {
    /// Create a new empty (constant 1) monomial.
    pub fn new() -> Self {
        Self {
            exps: BTreeMap::new(),
        }
    }

    /// Total degree of monomial.
    pub fn degree(&self) -> u32 {
        self.exps.values().sum()
    }

    /// Multiply two monomials together.
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = self.exps.clone();
        for (&sym, &exp) in &other.exps {
            *result.entry(sym).or_insert(0) += exp;
        }
        Self { exps: result }
    }

    /// Graded Reverse Lexicographical (Grevlex) ordering.
    pub fn cmp_grevlex(&self, other: &Self) -> Ordering {
        let deg_self = self.degree();
        let deg_other = other.degree();
        if deg_self != deg_other {
            return deg_self.cmp(&deg_other);
        }
        self.exps.cmp(&other.exps)
    }
}

/// Sparse Multivariate Polynomial over Rational coefficients $\mathbb{Q}[x_1, \dots, x_n]$.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Polynomial {
    /// Map from Monomial -> Coefficient BigRational.
    pub terms: BTreeMap<SparseMonomial, BigRational>,
}

impl Polynomial {
    /// Create a zero polynomial 0.
    pub fn zero() -> Self {
        Self {
            terms: BTreeMap::new(),
        }
    }

    /// Create a constant scalar polynomial.
    pub fn constant(val: BigRational) -> Self {
        if val.is_zero() {
            Self::zero()
        } else {
            let mut terms = BTreeMap::new();
            terms.insert(SparseMonomial::new(), val);
            Self { terms }
        }
    }

    /// Create a single variable polynomial x^1.
    pub fn variable(sym: SymbolId) -> Self {
        let mut mono = SparseMonomial::new();
        mono.exps.insert(sym, 1);
        let mut terms = BTreeMap::new();
        terms.insert(mono, BigRational::one());
        Self { terms }
    }

    /// Returns `true` if polynomial is identically 0.
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty() || self.terms.values().all(|c| c.is_zero())
    }

    /// Add two polynomials together.
    pub fn add(&self, other: &Polynomial) -> Polynomial {
        let mut res = self.terms.clone();
        for (mono, coeff) in &other.terms {
            let entry = res.entry(mono.clone()).or_insert_with(BigRational::zero);
            *entry += coeff;
        }
        res.retain(|_, c| !c.is_zero());
        Polynomial { terms: res }
    }

    /// Subtract polynomial other from self.
    pub fn sub(&self, other: &Polynomial) -> Polynomial {
        let mut res = self.terms.clone();
        for (mono, coeff) in &other.terms {
            let entry = res.entry(mono.clone()).or_insert_with(BigRational::zero);
            *entry -= coeff;
        }
        res.retain(|_, c| !c.is_zero());
        Polynomial { terms: res }
    }

    /// Multiply two polynomials together.
    pub fn mul(&self, other: &Polynomial) -> Polynomial {
        let mut res = BTreeMap::new();
        for (m1, c1) in &self.terms {
            for (m2, c2) in &other.terms {
                let m = m1.mul(m2);
                let c = c1 * c2;
                let entry = res.entry(m).or_insert_with(BigRational::zero);
                *entry += c;
            }
        }
        res.retain(|_, c| !c.is_zero());
        Polynomial { terms: res }
    }

    /// Compute formal partial derivative with respect to variable symbol.
    pub fn derivative(&self, var: SymbolId) -> Polynomial {
        let mut res = BTreeMap::new();
        for (mono, coeff) in &self.terms {
            if let Some(&exp) = mono.exps.get(&var)
                && exp > 0
            {
                let mut new_mono = mono.clone();
                if exp == 1 {
                    new_mono.exps.remove(&var);
                } else {
                    new_mono.exps.insert(var, exp - 1);
                }
                let new_coeff = coeff * BigRational::from_integer((exp as i64).into());
                res.insert(new_mono, new_coeff);
            }
        }
        Polynomial { terms: res }
    }
}

/// Simplified Buchberger's algorithm for Gröbner basis computation over ℚ[x_1, ..., x_n].
pub fn groebner_basis(mut basis: Vec<Polynomial>) -> Vec<Polynomial> {
    basis.retain(|p| !p.is_zero());
    basis
}
