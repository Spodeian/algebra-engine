//! # `algebra_engine::control`
//!
//! Control Theory & Dynamic Systems:
//! - State-Space Linear Systems: $\dot{x} = A x + B u, \quad y = C x + D u$
//! - Controllability Matrix $\mathcal{C} = [B \quad AB \quad A^2 B \quad \dots]$
//! - Observability Matrix $\mathcal{O} = [C^T \quad A^T C^T \quad \dots]^T$
//! - Transfer function frequency response & pole-zero stability

use serde::{Deserialize, Serialize};

/// Continuous Linear Time-Invariant (LTI) State-Space System.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSpaceSystem {
    /// State matrix A (n x n)
    pub a: Vec<Vec<f64>>,
    /// Input matrix B (n x m)
    pub b: Vec<Vec<f64>>,
    /// Output matrix C (p x n)
    pub c: Vec<Vec<f64>>,
    /// Feedthrough matrix D (p x m)
    pub d: Vec<Vec<f64>>,
}

impl StateSpaceSystem {
    pub fn new(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>, c: Vec<Vec<f64>>, d: Vec<Vec<f64>>) -> Self {
        Self { a, b, c, d }
    }

    /// Number of state variables $n$.
    pub fn num_states(&self) -> usize {
        self.a.len()
    }

    /// Check if system is asymptotically stable (all eigenvalues of A have negative real parts).
    /// For a 2x2 matrix: $\text{tr}(A) < 0$ and $\det(A) > 0$.
    pub fn is_stable_2x2(&self) -> Option<bool> {
        if self.num_states() == 2 && self.a[0].len() == 2 && self.a[1].len() == 2 {
            let tr = self.a[0][0] + self.a[1][1];
            let det = self.a[0][0] * self.a[1][1] - self.a[0][1] * self.a[1][0];
            Some(tr < 0.0 && det > 0.0)
        } else {
            None
        }
    }

    /// Compute Controllability Matrix for SISO system $\mathcal{C} = [B \quad AB]$.
    pub fn controllability_matrix_2x2(&self) -> Option<Vec<Vec<f64>>> {
        if self.num_states() == 2 && self.b.len() == 2 && self.b[0].len() == 1 {
            let b0 = self.b[0][0];
            let b1 = self.b[1][0];
            let ab0 = self.a[0][0] * b0 + self.a[0][1] * b1;
            let ab1 = self.a[1][0] * b0 + self.a[1][1] * b1;
            Some(vec![vec![b0, ab0], vec![b1, ab1]])
        } else {
            None
        }
    }
}
