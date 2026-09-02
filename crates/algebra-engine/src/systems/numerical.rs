//! # `algebra_engine::systems::numerical`
//!
//! Multi-Variable Non-Linear Numerical Systems Solvers.
//!
//! Provides:
//! - **Multi-Variable Newton-Raphson**: Solves non-linear vector equation $\mathbf{F}(\mathbf{x}) = \mathbf{0}$
//!   using exact analytical Jacobian matrices $\mathbf{J}_{\mathbf{F}}(\mathbf{x})$ generated automatically via symbolic differentiation.
//! - **Levenberg-Marquardt Damping**: Robust regularization $(\mathbf{J}^T\mathbf{J} + \lambda \mathbf{I})\Delta \mathbf{x} = -\mathbf{J}^T\mathbf{F}$
//!   for ill-conditioned or over-determined systems.

use crate::calculus::SymbolicCalculus;
use crate::numeric::{EvalContext, NumericalEval};
use algebra_core::error::{AlgebraError, AlgebraResult};
use algebra_core::{ExprGraph, ExprId, SymbolId};

/// Configuration options for non-linear multi-variable solver.
#[derive(Debug, Clone)]
pub struct NumericalSystemConfig {
    /// Maximum allowed iterations (default: 100).
    pub max_iterations: usize,
    /// Absolute residual tolerance $\|\mathbf{F}(\mathbf{x})\| < \text{tol}$ (default: 1e-8).
    pub tol: f64,
    /// Levenberg-Marquardt damping factor $\lambda$ (default: 1e-4).
    pub lambda: f64,
}

impl Default for NumericalSystemConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tol: 1e-8,
            lambda: 1e-4,
        }
    }
}

/// Numerical multi-variable system solver.
pub struct NumericalSystemSolver;

impl NumericalSystemSolver {
    /// Solve a non-linear system of equations $\{F_1(\mathbf{x}) = 0, \dots, F_n(\mathbf{x}) = 0\}$
    /// for variable vector $\mathbf{x} = (x_1, \dots, x_n)$ starting from initial guess $\mathbf{x}_0$.
    pub fn solve_newton_raphson(
        graph: &ExprGraph,
        equations: &[ExprId],
        variables: &[SymbolId],
        initial_guess: &[f64],
        config: &NumericalSystemConfig,
    ) -> AlgebraResult<Vec<f64>> {
        let n = equations.len();
        if variables.len() != n || initial_guess.len() != n {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Dimension mismatch: {} equations, {} variables, {} initial values",
                n,
                variables.len(),
                initial_guess.len()
            )));
        }

        if n == 0 {
            return Ok(Vec::new());
        }

        // 1. Generate Symbolic Analytical Jacobian J_{i,j} = dF_i / dx_j
        let mut jacobian_exprs = Vec::with_capacity(n * n);
        for &eq in equations {
            for &var in variables {
                let d_eq = graph.diff(eq, var);
                jacobian_exprs.push(d_eq);
            }
        }

        let mut x = initial_guess.to_vec();

        // 2. Iterative Newton-Raphson loop
        for _iter in 0..config.max_iterations {
            // Build EvalContext from current x
            let mut ctx = EvalContext::new(20);
            for (i, &var) in variables.iter().enumerate() {
                let var_name = graph
                    .symbols
                    .resolve(var)
                    .unwrap_or_else(|| format!("x{i}"));
                ctx.bindings.insert(var_name, x[i]);
            }

            // Evaluate residual vector F(x)
            let mut f_val = Vec::with_capacity(n);
            let mut max_res = 0.0f64;
            for &eq in equations {
                let val = graph.evalf(eq, &ctx)?.to_f64();
                max_res = max_res.max(val.abs());
                f_val.push(val);
            }

            if max_res < config.tol {
                return Ok(x);
            }

            // Evaluate Jacobian matrix J(x)
            let mut j_mat = Vec::with_capacity(n * n);
            for &j_expr in &jacobian_exprs {
                let val = graph.evalf(j_expr, &ctx)?.to_f64();
                j_mat.push(val);
            }

            // Solve linear system J * delta_x = -F (with Levenberg-Marquardt regularization)
            let delta = Self::solve_regularized_linear_step(&j_mat, &f_val, n, config.lambda)?;

            // Update state
            for i in 0..n {
                x[i] += delta[i];
            }
        }

        // Final verification of residual
        let mut ctx = EvalContext::new(20);
        for (i, &var) in variables.iter().enumerate() {
            let var_name = graph
                .symbols
                .resolve(var)
                .unwrap_or_else(|| format!("x{i}"));
            ctx.bindings.insert(var_name, x[i]);
        }
        let mut final_max_res = 0.0f64;
        for &eq in equations {
            let val = graph.evalf(eq, &ctx)?.to_f64();
            final_max_res = final_max_res.max(val.abs());
        }

        if final_max_res < config.tol * 10.0 {
            Ok(x)
        } else {
            Err(AlgebraError::EvaluationError(format!(
                "Newton-Raphson failed to converge within {} iterations (final residual: {final_max_res})",
                config.max_iterations
            )))
        }
    }

    /// Solves linear step $(\mathbf{J}^T\mathbf{J} + \lambda \mathbf{I})\Delta \mathbf{x} = -\mathbf{J}^T\mathbf{F}$.
    fn solve_regularized_linear_step(
        j_mat: &[f64],
        f_val: &[f64],
        n: usize,
        lambda: f64,
    ) -> AlgebraResult<Vec<f64>> {
        // Form A = J^T * J + lambda * I
        let mut a = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += j_mat[k * n + i] * j_mat[k * n + j];
                }
                a[i * n + j] = sum;
            }
            a[i * n + i] += lambda;
        }

        // Form rhs = -J^T * F
        let mut rhs = vec![0.0; n];
        for i in 0..n {
            let mut sum = 0.0;
            for k in 0..n {
                sum += j_mat[k * n + i] * f_val[k];
            }
            rhs[i] = -sum;
        }

        // Solve A * delta = rhs via Gaussian elimination
        for i in 0..n {
            let mut pivot = i;
            let mut max_val = a[i * n + i].abs();
            for r in (i + 1)..n {
                if a[r * n + i].abs() > max_val {
                    max_val = a[r * n + i].abs();
                    pivot = r;
                }
            }

            if pivot != i {
                for c in 0..n {
                    a.swap(i * n + c, pivot * n + c);
                }
                rhs.swap(i, pivot);
            }

            let pivot_elem = a[i * n + i];
            if pivot_elem.abs() < 1e-14 {
                continue;
            }

            for r in (i + 1)..n {
                let factor = a[r * n + i] / pivot_elem;
                for c in i..n {
                    a[r * n + c] -= factor * a[i * n + c];
                }
                rhs[r] -= factor * rhs[i];
            }
        }

        // Back substitution
        let mut delta = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = rhs[i];
            for j in (i + 1)..n {
                sum -= a[i * n + j] * delta[j];
            }
            if a[i * n + i].abs() > 1e-14 {
                delta[i] = sum / a[i * n + i];
            } else {
                delta[i] = 0.0;
            }
        }

        Ok(delta)
    }
}
