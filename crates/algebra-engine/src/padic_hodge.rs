//! # `algebra_engine::padic_hodge`
//!
//! p-Adic Hodge Theory, Fontaine Filtered (\Phi, N)-Modules & Galois Representations.
//!
//! Features:
//! - **Fontaine Filtered $(\Phi, N)$-Modules**: Semilinear Frobenius $\Phi$, nilpotent monodromy $N$ ($N \Phi = p \Phi N$), and Hodge filtration $\operatorname{Fil}^\bullet D$.
//! - **Galois Representation Admissibility**: Crystalline, Semistable, and de Rham representation classification with Hodge-Tate weights.
//! - **Tate Twists ($\mathbb{Q}_p(r)$)**: Cyclotomic character twists shifting Hodge-Tate weights by $-r$ and scaling Frobenius eigenvalues by $p^{-r}$.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::manual_is_multiple_of)]

use serde::{Deserialize, Serialize};

/// Fontaine Filtered $(\Phi, N)$-Module $(D, \Phi, N, \operatorname{Fil}^\bullet D)$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontaineModule {
    pub dimension: usize,
    pub prime_p: u64,
    pub frobenius_matrix: Vec<Vec<f64>>,
    pub monodromy_matrix: Vec<Vec<f64>>,
    pub hodge_tate_weights: Vec<i64>,
}

impl FontaineModule {
    /// Create new Fontaine module.
    pub fn new(
        dimension: usize,
        prime_p: u64,
        frobenius_matrix: Vec<Vec<f64>>,
        monodromy_matrix: Vec<Vec<f64>>,
        hodge_tate_weights: Vec<i64>,
    ) -> Self {
        assert_eq!(frobenius_matrix.len(), dimension);
        assert_eq!(monodromy_matrix.len(), dimension);
        assert_eq!(hodge_tate_weights.len(), dimension);
        Self {
            dimension,
            prime_p,
            frobenius_matrix,
            monodromy_matrix,
            hodge_tate_weights,
        }
    }

    /// Verification of the fundamental Fontaine relation: $N \Phi = p \Phi N$.
    pub fn verify_monodromy_frobenius_commutation(&self) -> bool {
        let n = self.dimension;
        let p = self.prime_p as f64;

        // Compute N * \Phi
        let mut n_phi = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    n_phi[i][j] += self.monodromy_matrix[i][k] * self.frobenius_matrix[k][j];
                }
            }
        }

        // Compute p * \Phi * N
        let mut p_phi_n = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    p_phi_n[i][j] += p * self.frobenius_matrix[i][k] * self.monodromy_matrix[k][j];
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                if (n_phi[i][j] - p_phi_n[i][j]).abs() > 1e-9 {
                    return false;
                }
            }
        }
        true
    }

    /// Is the representation Crystalline ($N = 0$).
    pub fn is_crystalline(&self) -> bool {
        self.monodromy_matrix
            .iter()
            .all(|row| row.iter().all(|&val| val.abs() < 1e-12))
    }

    /// Is the representation Semistable (Monodromy $N$ is nilpotent).
    pub fn is_semistable(&self) -> bool {
        let n = self.dimension;
        // Compute N^n
        let mut power = self.monodromy_matrix.clone();
        for _ in 1..n {
            let mut next = vec![vec![0.0; n]; n];
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        next[i][j] += power[i][k] * self.monodromy_matrix[k][j];
                    }
                }
            }
            power = next;
        }
        power
            .iter()
            .all(|row| row.iter().all(|&val| val.abs() < 1e-12))
    }
}

/// Galois Representation Classification in p-Adic Hodge Theory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PadicGaloisRepresentation {
    pub dimension: usize,
    pub prime_p: u64,
    pub hodge_tate_weights: Vec<i64>,
    pub d_cris_dimension: usize,
    pub d_st_dimension: usize,
    pub d_dr_dimension: usize,
}

impl PadicGaloisRepresentation {
    pub fn from_fontaine_module(module: &FontaineModule) -> Self {
        let dim = module.dimension;
        let is_cris = module.is_crystalline();
        let is_st = module.is_semistable();

        let d_cris_dim = if is_cris { dim } else { 0 };
        let d_st_dim = if is_st { dim } else { d_cris_dim };
        let d_dr_dim = dim; // Filtered (\Phi, N)-modules always induce de Rham representations

        Self {
            dimension: dim,
            prime_p: module.prime_p,
            hodge_tate_weights: module.hodge_tate_weights.clone(),
            d_cris_dimension: d_cris_dim,
            d_st_dimension: d_st_dim,
            d_dr_dimension: d_dr_dim,
        }
    }

    pub fn is_crystalline(&self) -> bool {
        self.d_cris_dimension == self.dimension
    }

    pub fn is_semistable(&self) -> bool {
        self.d_st_dimension == self.dimension
    }

    pub fn is_de_rham(&self) -> bool {
        self.d_dr_dimension == self.dimension
    }
}

/// Cyclotomic Tate Twists $V(r) = V \otimes \mathbb{Q}_p(r)$.
pub struct TateTwist;

impl TateTwist {
    /// Apply Tate twist $\mathbb{Q}_p(r)$ to Hodge-Tate weights: $k_i \mapsto k_i - r$.
    pub fn twist_hodge_tate_weights(weights: &[i64], r: i64) -> Vec<i64> {
        weights.iter().map(|&k| k - r).collect()
    }

    /// Apply Tate twist $\mathbb{Q}_p(r)$ to Frobenius eigenvalues: $\lambda_i \mapsto \lambda_i \cdot p^{-r}$.
    pub fn twist_frobenius_eigenvalues(eigenvalues: &[f64], p: u64, r: i64) -> Vec<f64> {
        let p_factor = (p as f64).powi(-r as i32);
        eigenvalues
            .iter()
            .map(|&lambda| lambda * p_factor)
            .collect()
    }
}
