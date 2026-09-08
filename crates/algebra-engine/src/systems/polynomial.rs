//! # `algebra_engine::systems::polynomial`
//!
//! Multi-Variate Non-Linear Polynomial Systems Solvers.
//!
//! Provides:
//! - **Gröbner Lexicographic Elimination**: Reduces multi-variate non-linear polynomial systems $\{f_1(\mathbf{x}) = 0, \dots, f_m(\mathbf{x}) = 0\}$
//!   into a triangular univariate system.
//! - **Exact Algebraic Variety Solving**: Combines Gröbner basis triangularization with analytical and numerical root solvers
//!   (Cardano, Ferrari, Durand-Kerner) to compute all exact and numerical solution tuples $\mathbf{x} = (x_1, \dots, x_n)$.

use crate::poly::grobner::GrobnerBasis;
use crate::poly::multivariate::{MonomialOrder, MultiPoly};
use crate::poly::roots::{CardanoSolver, ComplexRoot, DurandKernerSolver, FerrariSolver};
use algebra_core::error::{AlgebraError, AlgebraResult};
use algebra_core::{ExprGraph, ExprId};

/// Multi-Variate Polynomial System Solver.
pub struct PolynomialSystemSolver;

#[allow(clippy::needless_range_loop)]
impl PolynomialSystemSolver {
    /// Solves a multi-variate polynomial system $\{f_1(\mathbf{x}) = 0, \dots, f_m(\mathbf{x}) = 0\}$
    /// for variables $\mathbf{x} = (x_1, \dots, x_n)$.
    pub fn solve(
        graph: &ExprGraph,
        equations: &[ExprId],
        var_names: &[String],
    ) -> AlgebraResult<Vec<Vec<ComplexRoot>>> {
        let num_vars = var_names.len();
        if num_vars == 0 {
            return Ok(Vec::new());
        }

        // 1. Convert expressions to MultiPoly under Lexicographic ordering
        let mut polys = Vec::with_capacity(equations.len());
        for &eq in equations {
            let p = MultiPoly::from_expr(graph, eq, var_names, MonomialOrder::Lex)?;
            polys.push(p);
        }

        // 2. Compute Reduced Gröbner Basis under Lex order
        let gb = GrobnerBasis::buchberger(&polys);
        if gb.polynomials.is_empty() {
            return Ok(Vec::new());
        }

        // Check for inconsistency: if basis contains a non-zero constant (1 = 0)
        for p in &gb.polynomials {
            if p.terms.len() == 1 && p.terms[0].monomial.total_degree == 0 {
                return Err(AlgebraError::EvaluationError(
                    "Inconsistent polynomial system: no common algebraic variety exists".into(),
                ));
            }
        }

        // 3. Find univariate polynomial in the last variable (x_n)
        let last_var_idx = num_vars - 1;
        let mut univariate_poly = None;

        for p in &gb.polynomials {
            let is_univariate_in_last = p.terms.iter().all(|t| {
                t.monomial
                    .exponents
                    .iter()
                    .enumerate()
                    .all(|(idx, &exp)| idx == last_var_idx || exp == 0)
            });

            if is_univariate_in_last && !p.is_zero() {
                univariate_poly = Some(p.clone());
                break;
            }
        }

        let uni = match univariate_poly {
            Some(u) => u,
            None => {
                // If not strictly univariate in last variable, attempt to solve 2-variable system via resultant or first element
                if gb.polynomials.len() == 1 && num_vars == 1 {
                    gb.polynomials[0].clone()
                } else {
                    return Err(AlgebraError::EvaluationError(
                        "Positive-dimensional variety or underdetermined polynomial system".into(),
                    ));
                }
            }
        };

        // Extract dense coefficient array for univariate polynomial: a_d * x^d + ... + a_0
        let deg = uni
            .terms
            .first()
            .map(|t| t.monomial.exponents[last_var_idx])
            .unwrap_or(0);
        let mut coeffs = vec![0.0; deg + 1];
        for t in &uni.terms {
            let exp = t.monomial.exponents[last_var_idx];
            let idx = deg - exp;
            coeffs[idx] = t.coeff;
        }

        // 4. Solve for last variable roots
        let last_var_roots = Self::solve_univariate_roots(&coeffs)?;

        // 5. Back-solve for remaining variables
        let mut solutions = Vec::new();
        for root_n in last_var_roots {
            let mut current_tuple = vec![ComplexRoot::real(0.0); num_vars];
            current_tuple[last_var_idx] = root_n;

            // Back substitution for preceding variables (x_{n-1}, ..., x_1)
            let mut solvable = true;
            for var_idx in (0..last_var_idx).rev() {
                // Find polynomial with leading term in var_idx
                let mut found_linear_sub = false;
                for p in &gb.polynomials {
                    if let Some(lm) = p.leading_monomial()
                        && lm.exponents[var_idx] == 1
                        && lm.exponents[..var_idx].iter().all(|&e| e == 0)
                    {
                        // Substitute known values of variables (var_idx + 1 .. n)
                        let mut val_sum_re = 0.0;
                        let mut val_sum_im = 0.0;
                        let mut lc = 1.0;

                        for t in &p.terms {
                            if t.monomial.exponents[var_idx] == 1 {
                                lc = t.coeff;
                            } else {
                                // Evaluate known terms
                                let mut term_val_re = t.coeff;
                                let mut term_val_im = 0.0;
                                for k in (var_idx + 1)..num_vars {
                                    let exp_k = t.monomial.exponents[k];
                                    if exp_k > 0 {
                                        for _ in 0..exp_k {
                                            let r_re = current_tuple[k].re;
                                            let r_im = current_tuple[k].im;
                                            let new_re = term_val_re * r_re - term_val_im * r_im;
                                            let new_im = term_val_re * r_im + term_val_im * r_re;
                                            term_val_re = new_re;
                                            term_val_im = new_im;
                                        }
                                    }
                                }
                                val_sum_re += term_val_re;
                                val_sum_im += term_val_im;
                            }
                        }

                        let sol_re = -val_sum_re / lc;
                        let sol_im = -val_sum_im / lc;
                        current_tuple[var_idx] = ComplexRoot::complex(sol_re, sol_im);
                        found_linear_sub = true;
                        break;
                    }
                }

                if !found_linear_sub {
                    solvable = false;
                    break;
                }
            }

            if solvable {
                solutions.push(current_tuple);
            }
        }

        Ok(solutions)
    }

    /// Solves dense univariate polynomial using exact radical formulas for degree 1..4 or Durand-Kerner.
    fn solve_univariate_roots(coeffs: &[f64]) -> AlgebraResult<Vec<ComplexRoot>> {
        let deg = coeffs.len().saturating_sub(1);
        match deg {
            0 => Ok(Vec::new()),
            1 => {
                // a*x + b = 0 -> x = -b / a
                let a = coeffs[0];
                let b = coeffs[1];
                if a.abs() < 1e-14 {
                    return Err(AlgebraError::EvaluationError(
                        "Degenerate linear polynomial".into(),
                    ));
                }
                Ok(vec![ComplexRoot::real(-b / a)])
            }
            2 => {
                // a*x^2 + b*x + c = 0
                let a = coeffs[0];
                let b = coeffs[1];
                let c = coeffs[2];
                let disc = b * b - 4.0 * a * c;
                if disc >= 0.0 {
                    let r1 = (-b + disc.sqrt()) / (2.0 * a);
                    let r2 = (-b - disc.sqrt()) / (2.0 * a);
                    Ok(vec![ComplexRoot::real(r1), ComplexRoot::real(r2)])
                } else {
                    let re = -b / (2.0 * a);
                    let im = (-disc).sqrt() / (2.0 * a);
                    Ok(vec![
                        ComplexRoot::complex(re, im),
                        ComplexRoot::complex(re, -im),
                    ])
                }
            }
            3 => {
                // Cubic Cardano
                CardanoSolver::solve(coeffs[0], coeffs[1], coeffs[2], coeffs[3])
            }
            4 => {
                // Quartic Ferrari
                FerrariSolver::solve(coeffs[0], coeffs[1], coeffs[2], coeffs[3], coeffs[4])
            }
            _ => {
                // Degree >= 5: Durand-Kerner simultaneous iteration
                let lc = coeffs[0];
                let monic_coeffs: Vec<f64> = coeffs.iter().map(|&c| c / lc).collect();
                Ok(DurandKernerSolver::solve(&monic_coeffs, 100, 1e-10))
            }
        }
    }
}
