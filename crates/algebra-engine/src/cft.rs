//! # `algebra_engine::cft`
//!
//! Conformal Field Theory (CFT), Virasoro Algebra & Affine Kac-Moody Current Algebras.
//!
//! Features:
//! - **Virasoro Lie Algebra ($[L_m, L_n]$)**: Infinite-dimensional central extension with exact Jacobi identity verification.
//! - **Kac Determinant & Minimal Models**: Degenerate conformal weights $h_{r, s}(c)$ for Ising, tricritical Ising, and 3-state Potts models.
//! - **Affine Kac-Moody Lie Algebras ($\hat{\mathfrak{g}}_k$)**: Loop algebra current commutators with level $k$ central extension.
//! - **Sugawara Construction**: Energy-momentum tensor quadratic current composite and central charge $c = \frac{k \dim(\mathfrak{g})}{k + h^\vee}$.
//! - **Operator Product Expansion (OPE)**: Radial contour Laurent pole coefficients for $T(z) T(w)$ and primary fields $T(z) \phi(w)$.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::manual_is_multiple_of)]

use serde::{Deserialize, Serialize};

/// Virasoro Infinite-Dimensional Lie Algebra Engine.
pub struct VirasoroAlgebra;

impl VirasoroAlgebra {
    /// Compute Virasoro Lie bracket $[L_m, L_n] = (m - n) L_{m+n} + \frac{c}{12} (m^3 - m) \delta_{m+n, 0}$.
    /// Returns `(mode_out, coeff_L, central_term)`.
    pub fn bracket(m: i64, n: i64, c: f64) -> (i64, f64, f64) {
        let mode_out = m + n;
        let coeff_l = (m - n) as f64;
        let central_term = if m + n == 0 {
            (c / 12.0) * (m * m * m - m) as f64
        } else {
            0.0
        };
        (mode_out, coeff_l, central_term)
    }

    /// Exact Jacobi identity verification: $[[L_m, L_n], L_k] + [[L_n, L_k], L_m] + [[L_k, L_m], L_n] = 0$.
    pub fn verify_jacobi(m: i64, n: i64, k: i64, c: f64) -> bool {
        // Term 1: [[L_m, L_n], L_k] = (m - n) [L_{m+n}, L_k] + central * 0
        let (_, c1_l, _) = Self::bracket(m, n, c);
        let (_, c1_ll, cent1) = Self::bracket(m + n, k, c);
        let term1_l = c1_l * c1_ll;
        let term1_cent = c1_l * cent1;

        // Term 2: [[L_n, L_k], L_m]
        let (_, c2_l, _) = Self::bracket(n, k, c);
        let (_, c2_ll, cent2) = Self::bracket(n + k, m, c);
        let term2_l = c2_l * c2_ll;
        let term2_cent = c2_l * cent2;

        // Term 3: [[L_k, L_m], L_n]
        let (_, c3_l, _) = Self::bracket(k, m, c);
        let (_, c3_ll, cent3) = Self::bracket(k + m, n, c);
        let term3_l = c3_l * c3_ll;
        let term3_cent = c3_l * cent3;

        let total_l = term1_l + term2_l + term3_l;
        let total_cent = term1_cent + term2_cent + term3_cent;

        total_l.abs() < 1e-10 && total_cent.abs() < 1e-10
    }

    /// Central charge for unitary minimal model $\mathcal{M}(m, m+1)$: $c = 1 - \frac{6}{m(m+1)}$.
    pub fn minimal_model_c(m: usize) -> f64 {
        assert!(m >= 2);
        let m_f = m as f64;
        1.0 - 6.0 / (m_f * (m_f + 1.0))
    }

    /// Degenerate conformal weight $h_{r, s}(m) = \frac{((m+1)r - m s)^2 - 1}{4m(m+1)}$ from the Kac table.
    pub fn kac_conformal_weight(m: usize, r: usize, s: usize) -> f64 {
        assert!(m >= 2);
        assert!(r >= 1 && r < m);
        assert!(s >= 1 && s <= m);

        let m_f = m as f64;
        let num = ((m_f + 1.0) * r as f64 - m_f * s as f64).powi(2) - 1.0;
        let den = 4.0 * m_f * (m_f + 1.0);
        num / den
    }
}

/// Affine Kac-Moody Current Lie Algebra Engine.
pub struct KacMoodyAlgebra;

impl KacMoodyAlgebra {
    /// $\mathfrak{su}(2)$ totally antisymmetric structure constants $\epsilon^{abc}$ with $a, b, c \in \{1, 2, 3\}$.
    pub fn su2_structure_constant(a: usize, b: usize, c: usize) -> f64 {
        if a == b
            || b == c
            || a == c
            || !(1..=3).contains(&a)
            || !(1..=3).contains(&b)
            || !(1..=3).contains(&c)
        {
            0.0
        } else if (a == 1 && b == 2 && c == 3)
            || (a == 2 && b == 3 && c == 1)
            || (a == 3 && b == 1 && c == 2)
        {
            1.0
        } else {
            -1.0
        }
    }

    /// Commutator of affine current modes $[J^a_m, J^b_n] = i f^{abc} J^c_{m+n} + k \, m \, \delta^{ab} \delta_{m+n, 0}$.
    /// Returns `(mode_out, imaginary_structure_constants_by_c, central_term)`.
    pub fn su2_bracket(a: usize, m: i64, b: usize, n: i64, level_k: f64) -> (i64, [f64; 3], f64) {
        let mode_out = m + n;
        let mut f_coeffs = [0.0; 3];
        for c in 1..=3 {
            f_coeffs[c - 1] = Self::su2_structure_constant(a, b, c);
        }
        let central_term = if a == b && m + n == 0 {
            level_k * m as f64
        } else {
            0.0
        };
        (mode_out, f_coeffs, central_term)
    }

    /// Sugawara central charge formula: $c = \frac{k \dim(\mathfrak{g})}{k + h^\vee}$.
    pub fn sugawara_central_charge(dim_g: f64, dual_coxeter: f64, level_k: f64) -> f64 {
        (level_k * dim_g) / (level_k + dual_coxeter)
    }
}

/// Operator Product Expansion (OPE) Pole Analyzer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorProductExpansion;

impl OperatorProductExpansion {
    /// OPE singular pole coefficients for $T(z) T(w) \sim \frac{c/2}{(z-w)^4} + \frac{2 T(w)}{(z-w)^2} + \frac{\partial T(w)}{z-w}$.
    /// Returns `(order_4_c_half, order_2_coeff, order_1_coeff)`.
    pub fn energy_momentum_ope(central_charge: f64) -> (f64, f64, f64) {
        (central_charge / 2.0, 2.0, 1.0)
    }

    /// OPE singular pole coefficients for $T(z) \phi(w) \sim \frac{h \phi(w)}{(z-w)^2} + \frac{\partial \phi(w)}{z-w}$.
    /// Returns `(order_2_weight_h, order_1_coeff)`.
    pub fn primary_field_ope(conformal_weight_h: f64) -> (f64, f64) {
        (conformal_weight_h, 1.0)
    }
}
