//! # `algebra_engine::systems::diophantine`
//!
//! Multi-Variable Integer Diophantine Systems Solvers.
//!
//! Provides:
//! - **Smith Normal Form (SNF)**: Exact unimodular factorization $\mathbf{U}\mathbf{A}\mathbf{V} = \mathbf{D}$
//!   for integer matrix $\mathbf{A} \in \mathbb{Z}^{m \times n}$ with invariant factors $d_i \mid d_{i+1}$.
//! - **Linear Diophantine System Solver**: Determines integer lattice solutions $\mathbf{A}\mathbf{x} = \mathbf{b}$ over $\mathbb{Z}$.

use algebra_core::error::{AlgebraError, AlgebraResult};

/// Result of Smith Normal Form decomposition $\mathbf{U}\mathbf{A}\mathbf{V} = \mathbf{D}$.
#[derive(Debug, Clone)]
pub struct SmithNormalForm {
    /// Invariant diagonal factors $(d_1, \dots, d_r)$ where $d_i \mid d_{i+1}$.
    pub diagonal: Vec<i64>,
    /// Unimodular left transformation matrix $\mathbf{U} \in \operatorname{GL}_m(\mathbb{Z})$ ($m \times m$).
    pub u: Vec<Vec<i64>>,
    /// Unimodular right transformation matrix $\mathbf{V} \in \operatorname{GL}_n(\mathbb{Z})$ ($n \times n$).
    pub v: Vec<Vec<i64>>,
    /// Rank of the matrix.
    pub rank: usize,
}

/// Integer solution space for Diophantine system $\mathbf{A}\mathbf{x} = \mathbf{b}$ over $\mathbb{Z}$.
#[derive(Debug, Clone)]
pub struct DiophantineSolution {
    /// Particular integer solution $\mathbf{x}_p \in \mathbb{Z}^n$.
    pub particular: Vec<i64>,
    /// Basis vectors spanning the homogeneous integer nullspace lattice $\operatorname{ker}_\mathbb{Z}(\mathbf{A})$.
    pub nullspace_basis: Vec<Vec<i64>>,
}

/// Integer Diophantine systems solver.
pub struct DiophantineSystemSolver;

#[allow(clippy::needless_range_loop)]
impl DiophantineSystemSolver {
    /// Computes the Smith Normal Form (SNF) $\mathbf{U}\mathbf{A}\mathbf{V} = \mathbf{D}$ for integer matrix $\mathbf{A} \in \mathbb{Z}^{m \times n}$.
    pub fn smith_normal_form(matrix: &[Vec<i64>]) -> AlgebraResult<SmithNormalForm> {
        let m = matrix.len();
        if m == 0 {
            return Err(AlgebraError::EvaluationError(
                "Matrix must have at least one row".into(),
            ));
        }
        let n = matrix[0].len();
        if n == 0 {
            return Err(AlgebraError::EvaluationError(
                "Matrix must have at least one column".into(),
            ));
        }

        let mut a = matrix.to_vec();
        // Initialize U as m x m identity
        let mut u = vec![vec![0i64; m]; m];
        for i in 0..m {
            u[i][i] = 1;
        }

        // Initialize V as n x n identity
        let mut v = vec![vec![0i64; n]; n];
        for j in 0..n {
            v[j][j] = 1;
        }

        let min_dim = m.min(n);
        let mut rank = 0;

        for k in 0..min_dim {
            // 1. Pivot selection: find smallest non-zero element in submatrix
            loop {
                let mut pivot = None;
                let mut min_val = i64::MAX;

                for i in k..m {
                    for j in k..n {
                        let val = a[i][j].abs();
                        if val != 0 && val < min_val {
                            min_val = val;
                            pivot = Some((i, j));
                        }
                    }
                }

                let (pi, pj) = match pivot {
                    Some(p) => p,
                    None => break, // Entire remaining submatrix is zero
                };

                // Swap pivot to (k, k)
                if pi != k {
                    a.swap(k, pi);
                    u.swap(k, pi);
                }
                if pj != k {
                    for r in 0..m {
                        a[r].swap(k, pj);
                    }
                    for r in 0..n {
                        v[r].swap(k, pj);
                    }
                }

                if a[k][k] < 0 {
                    for j in 0..n {
                        a[k][j] = -a[k][j];
                    }
                    for j in 0..m {
                        u[k][j] = -u[k][j];
                    }
                }

                // Eliminate row entries in column k
                let mut changed = false;
                for i in (k + 1)..m {
                    let q = a[i][k] / a[k][k];
                    if q != 0 {
                        for j in 0..n {
                            a[i][j] -= q * a[k][j];
                        }
                        for j in 0..m {
                            u[i][j] -= q * u[k][j];
                        }
                        if a[i][k] != 0 {
                            changed = true;
                        }
                    }
                }

                // Eliminate column entries in row k
                for j in (k + 1)..n {
                    let q = a[k][j] / a[k][k];
                    if q != 0 {
                        for i in 0..m {
                            a[i][j] -= q * a[i][k];
                        }
                        for i in 0..n {
                            v[i][j] -= q * v[i][k];
                        }
                        if a[k][j] != 0 {
                            changed = true;
                        }
                    }
                }

                if !changed {
                    rank += 1;
                    break;
                }
            }
        }

        // Extract diagonal entries
        let mut diagonal = Vec::with_capacity(rank);
        for i in 0..rank {
            diagonal.push(a[i][i]);
        }

        Ok(SmithNormalForm {
            diagonal,
            u,
            v,
            rank,
        })
    }

    /// Solves linear integer Diophantine system $\mathbf{A}\mathbf{x} = \mathbf{b}$ over $\mathbb{Z}$.
    pub fn solve(matrix: &[Vec<i64>], b_vector: &[i64]) -> AlgebraResult<DiophantineSolution> {
        let m = matrix.len();
        if m == 0 || b_vector.len() != m {
            return Err(AlgebraError::DimensionMismatch(
                "Matrix rows and RHS vector length mismatch".into(),
            ));
        }
        let n = matrix[0].len();

        let snf = Self::smith_normal_form(matrix)?;

        // Compute c = U * b
        let mut c = vec![0i64; m];
        for i in 0..m {
            let mut sum = 0;
            for j in 0..m {
                sum += snf.u[i][j] * b_vector[j];
            }
            c[i] = sum;
        }

        // Verify divisibility and consistency
        let mut y_part = vec![0i64; n];
        for i in 0..snf.rank {
            let d_i = snf.diagonal[i];
            if d_i == 0 || c[i] % d_i != 0 {
                return Err(AlgebraError::EvaluationError(
                    "No integer solution: RHS element not divisible by invariant factor".into(),
                ));
            }
            y_part[i] = c[i] / d_i;
        }

        for i in snf.rank..m {
            if c[i] != 0 {
                return Err(AlgebraError::EvaluationError(
                    "Inconsistent integer system: zero row has non-zero RHS".into(),
                ));
            }
        }

        // Particular solution x_p = V * y_part
        let mut particular = vec![0i64; n];
        for i in 0..n {
            let mut sum = 0;
            for j in 0..n {
                sum += snf.v[i][j] * y_part[j];
            }
            particular[i] = sum;
        }

        // Nullspace basis: columns of V corresponding to free variables (rank..n)
        let mut nullspace_basis = Vec::new();
        for col in snf.rank..n {
            let mut basis_vec = vec![0i64; n];
            for i in 0..n {
                basis_vec[i] = snf.v[i][col];
            }
            nullspace_basis.push(basis_vec);
        }

        Ok(DiophantineSolution {
            particular,
            nullspace_basis,
        })
    }
}
