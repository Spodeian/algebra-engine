//! # `algebra_advanced::statmech`
//!
//! Statistical Mechanics & Quantum Thermodynamics:
//! - Canonical Partition Function $Z(\beta) = \sum_i g_i e^{-\beta E_i}$
//! - Thermodynamic Potentials: Free Energy $F = -k_B T \ln Z$, Entropy $S$, Internal Energy $U$
//! - Quantum Statistics: Fermi-Dirac & Bose-Einstein distributions
//! - Grand Canonical Ensemble $\Xi(T, V, \mu)$

use serde::{Deserialize, Serialize};

/// Canonical Partition Function representation with discrete energy spectrum $\{E_i, g_i\}$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalPartitionFunction {
    pub energy_levels: Vec<(f64, usize)>, // (E_i, degeneracy g_i)
}

impl CanonicalPartitionFunction {
    pub fn new(energy_levels: Vec<(f64, usize)>) -> Self {
        Self { energy_levels }
    }

    /// Evaluate partition function $Z(\beta)$ where $\beta = 1 / (k_B T)$.
    pub fn evaluate_z(&self, beta: f64) -> f64 {
        self.energy_levels
            .iter()
            .map(|&(e, g)| (g as f64) * (-beta * e).exp())
            .sum()
    }

    /// Helmholtz Free Energy $F = - \frac{1}{\beta} \ln Z$.
    pub fn free_energy(&self, beta: f64) -> f64 {
        let z = self.evaluate_z(beta);
        if z <= 0.0 {
            0.0
        } else {
            -z.ln() / beta
        }
    }

    /// Mean Internal Energy $U = \langle E \rangle = \frac{1}{Z} \sum g_i E_i e^{-\beta E_i}$.
    pub fn internal_energy(&self, beta: f64) -> f64 {
        let z = self.evaluate_z(beta);
        if z <= 0.0 {
            return 0.0;
        }
        let numerator: f64 = self
            .energy_levels
            .iter()
            .map(|&(e, g)| (g as f64) * e * (-beta * e).exp())
            .sum();
        numerator / z
    }
}

/// Fermi-Dirac distribution $f(E) = \frac{1}{e^{\beta (E - \mu)} + 1}$ (Fermions / Pauli exclusion).
pub fn fermi_dirac(energy: f64, chemical_potential: f64, beta: f64) -> f64 {
    let exp_term = (beta * (energy - chemical_potential)).exp();
    1.0 / (exp_term + 1.0)
}

/// Bose-Einstein distribution $n(E) = \frac{1}{e^{\beta (E - \mu)} - 1}$ (Bosons / condensation).
pub fn bose_einstein(energy: f64, chemical_potential: f64, beta: f64) -> Option<f64> {
    if energy <= chemical_potential {
        return None;
    }
    let exp_term = (beta * (energy - chemical_potential)).exp();
    Some(1.0 / (exp_term - 1.0))
}
