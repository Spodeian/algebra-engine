//! # `algebra_advanced::adele`
//!
//! Adele Ring $\mathbb{A}_K$, Idele Group $\mathbb{I}_K$ & Global Artin Product Formula.
//!
//! Encompasses:
//! - **Places of $\mathbb{Q}$**: Archimedean place $v = \infty$ ($|\cdot|_\infty$) and non-Archimedean places $v = p$ ($|\cdot|_p$).
//! - **Restricted Direct Product**: $\mathbb{A}_\mathbb{Q} = {\prod'_v} \mathbb{Q}_v$ where $x_v \in \mathbb{Z}_p$ for almost all $v$.
//! - **Idele Group $\mathbb{I}_\mathbb{Q}$**: Invertible adeles $\mathbb{A}_\mathbb{Q}^\times$.
//! - **Artin Product Formula**: For any non-zero rational number $x \in \mathbb{Q}^\times$:
//!   $$\prod_{v \le \infty} |x|_v = |x|_\infty \cdot \prod_{p} |x|_p = 1$$

use crate::padic::PadicNumber;
use algebra_core::error::{AlgebraError, AlgebraResult};
use algebra_core::traits::ValuationProvider;
use std::fmt;

/// Classification of a global field valuation place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// Archimedean place $v = \infty$ corresponding to the standard real absolute value $|\cdot|_\infty$.
    Archimedean,
    /// Non-Archimedean finite place corresponding to prime ideal $p$.
    Finite(u64),
}

/// Element of the Adele Ring $\mathbb{A}_\mathbb{Q}$.
#[derive(Debug, Clone, PartialEq)]
pub struct Adele {
    /// Archimedean real component $x_\infty \in \mathbb{R}$.
    pub archimedean: f64,
    /// Finite non-Archimedean components $(p, x_p \in \mathbb{Q}_p)$.
    pub finite_places: Vec<(u64, PadicNumber)>,
}

impl Adele {
    /// Embeds a global rational number $x = n / d \in \mathbb{Q}$ diagonally into the Adele Ring $\mathbb{A}_\mathbb{Q}$
    /// at specified prime places.
    pub fn from_rational(n: i64, d: i64, primes: &[u64], precision: usize) -> AlgebraResult<Self> {
        if d == 0 {
            return Err(AlgebraError::DivisionByZero {
                domain: "Adele Ring A_Q".into(),
            });
        }
        let archimedean = n as f64 / d as f64;
        let mut finite_places = Vec::with_capacity(primes.len());

        for &p in primes {
            let padic = PadicNumber::from_rational(n, d, p, precision)?;
            finite_places.push((p, padic));
        }

        Ok(Self {
            archimedean,
            finite_places,
        })
    }

    /// Construct an Adele from a GlobalValuationProfile.
    pub fn from_valuation_profile(
        profile: &algebra_engine::numbertheory::GlobalValuationProfile,
        precision: usize,
    ) -> AlgebraResult<Self> {
        ArtinProduct::to_adele_vector(profile, precision)
    }

    /// Addition of adeles.
    pub fn add(&self, other: &Self) -> Self {
        let archimedean = self.archimedean + other.archimedean;
        let mut finite_places = Vec::new();

        for (p, p_num) in &self.finite_places {
            if let Some((_, other_p_num)) = other.finite_places.iter().find(|(op, _)| op == p) {
                finite_places.push((*p, p_num.add(other_p_num)));
            } else {
                finite_places.push((*p, p_num.clone()));
            }
        }

        Self {
            archimedean,
            finite_places,
        }
    }
}

/// Idele Group Element $\mathbf{a} \in \mathbb{I}_\mathbb{Q} = \mathbb{A}_\mathbb{Q}^\times$.
#[derive(Debug, Clone, PartialEq)]
pub struct Idele {
    pub adele: Adele,
}

impl Idele {
    /// Creates an idele from an invertible adele.
    pub fn from_adele(adele: Adele) -> AlgebraResult<Self> {
        if adele.archimedean.abs() < 1e-15 {
            return Err(AlgebraError::EvaluationError(
                "Archimedean component is zero".into(),
            ));
        }
        for (_, p_num) in &adele.finite_places {
            if p_num.is_zero() {
                return Err(AlgebraError::EvaluationError(
                    "Finite component is zero".into(),
                ));
            }
        }
        Ok(Self { adele })
    }

    /// Computes the Idele Norm (Content / Volume) $\operatorname{vol}(\mathbf{a}) = |\alpha_\infty|_\infty \cdot \prod_p |\alpha_p|_p$.
    pub fn idele_norm(&self) -> f64 {
        let mut total_norm = self.adele.archimedean.abs();
        for (_, p_num) in &self.adele.finite_places {
            total_norm *= p_num.norm();
        }
        total_norm
    }
}

pub trait AdeleEmbedding {
    /// Verifies the Global Product Formula $\prod_{v \le \infty} |x|_v = 1$ for a non-zero rational $x = n / d$.
    fn verify_artin_product(profile: &impl ValuationProvider) -> bool;

    /// Embeds a global rational number $x = n / d \in \mathbb{Q}$ diagonally into the Adele Ring $\mathbb{A}_\mathbb{Q}$.
    fn to_adele_vector(
        profile: &algebra_engine::numbertheory::GlobalValuationProfile,
        precision: usize,
    ) -> AlgebraResult<Adele>;
}

/// Global Artin Product Formula Verifier.
pub struct ArtinProduct;

impl ArtinProduct {
    /// Convenience method: verifies the Global Product Formula for $n / d$ by constructing a `GlobalValuationProfile`.
    pub fn verify_product_formula(n: i64, d: i64) -> AlgebraResult<bool> {
        let profile = algebra_engine::numbertheory::GlobalValuationProfile::from_rational(n, d)
            .ok_or_else(|| {
                AlgebraError::EvaluationError("Number must be non-zero rational in Q*".into())
            })?;
        Ok(<Self as AdeleEmbedding>::verify_artin_product(&profile))
    }
}

impl AdeleEmbedding for ArtinProduct {
    fn verify_artin_product(profile: &impl ValuationProvider) -> bool {
        let arch_norm = profile.infinite_place_abs();
        if arch_norm == 0.0 {
            return false;
        }

        let mut finite_product = 1.0;
        for (p, val) in profile.finite_places() {
            // Norm of p in Q_p is p^{-val}
            finite_product *= (p as f64).powi(-val);
        }

        let total_product = arch_norm * finite_product;
        (total_product - 1.0).abs() < 1e-9
    }

    fn to_adele_vector(
        profile: &algebra_engine::numbertheory::GlobalValuationProfile,
        precision: usize,
    ) -> AlgebraResult<Adele> {
        let archimedean = profile.num as f64 / profile.den as f64;
        let mut finite_places = Vec::with_capacity(profile.finite_places.len());

        for &p in profile.finite_places.keys() {
            let padic = PadicNumber::from_rational(profile.num, profile.den, p, precision)?;
            finite_places.push((p, padic));
        }

        Ok(Adele {
            archimedean,
            finite_places,
        })
    }
}

impl fmt::Display for Adele {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4}_inf", self.archimedean)?;
        for (p, p_num) in &self.finite_places {
            write!(f, ", Q_{p}: {}", p_num.valuation)?;
        }
        write!(f, ")")
    }
}
