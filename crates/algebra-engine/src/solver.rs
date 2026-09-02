//! # `algebra-solver`
//!
//! Algebraic equation root finding (`solveset`), Ordinary Differential Equation (ODE) solvers,
//! Partial Differential Equation (PDE) solvers, and integer Diophantine equation solvers.

use crate::calculus::SymbolicCalculus;
use crate::numeric::NumericalEval;
use crate::simplify::Simplifier;
use algebra_core::{
    AlgebraError, AlgebraResult, ExprGraph, ExprId, ExprKind, MathStep, StepByStepResult, SymbolId,
};

/// Diophantine Equation Solver for integer solutions over $\mathbb{Z}$.
pub struct DiophantineSolver;

impl DiophantineSolver {
    /// Solve Linear Diophantine Equation $a x + b y = c$ for $x, y \in \mathbb{Z}$.
    /// Returns particular solution `(x0, y0)` if an integer solution exists.
    pub fn solve_linear_2var(a: i64, b: i64, c: i64) -> Option<(i64, i64)> {
        fn ext_gcd(a: i64, b: i64) -> (i64, i64, i64) {
            if b == 0 {
                (a, 1, 0)
            } else {
                let (g, x1, y1) = ext_gcd(b, a % b);
                (g, y1, x1 - (a / b) * y1)
            }
        }

        let (g, x0, y0) = ext_gcd(a.abs(), b.abs());
        if g == 0 || c % g != 0 {
            return None;
        }

        let k = c / g;
        let x_sol = x0 * k * (if a < 0 { -1 } else { 1 });
        let y_sol = y0 * k * (if b < 0 { -1 } else { 1 });

        Some((x_sol, y_sol))
    }

    /// Solve Pell's Equation $x^2 - D y^2 = 1$ for fundamental positive integer solution $(x_1, y_1)$.
    pub fn solve_pell(d: i64) -> Option<(i64, i64)> {
        let sqrt_d = (d as f64).sqrt();
        if sqrt_d.fract() == 0.0 || d <= 0 {
            return None;
        }

        let m0 = 0i64;
        let d0 = 1i64;
        let a0 = sqrt_d as i64;

        let mut m = m0;
        let mut denom = d0;
        let mut a = a0;

        let mut p_prev2;
        let mut p_prev1 = 1i64;
        let mut q_prev2;
        let mut q_prev1 = 0i64;

        let mut p = a0;
        let mut q = 1i64;

        for _ in 0..100 {
            if p * p - d * q * q == 1 {
                return Some((p, q));
            }

            m = denom * a - m;
            denom = (d - m * m) / denom;
            if denom == 0 {
                break;
            }
            a = (a0 + m) / denom;

            p_prev2 = p_prev1;
            p_prev1 = p;
            q_prev2 = q_prev1;
            q_prev1 = q;

            p = a * p_prev1 + p_prev2;
            q = a * q_prev1 + q_prev2;
        }

        None
    }
}

/// Equation solver trait for algebraic equations, ODEs, and PDEs.
pub trait SymbolicSolver {
    /// Solve algebraic equation $f(x) = 0$ for target variable `target` (Exact symbolic pathway).
    fn solveset(&self, eq: ExprId, target: SymbolId) -> AlgebraResult<Vec<ExprId>>;

    /// Solve algebraic equation $f(x) = 0$ for target variable `target` with verification certificate.
    fn solveset_certified(
        &self,
        eq: ExprId,
        target: SymbolId,
        target_epsilon: f64,
    ) -> AlgebraResult<algebra_core::probabilistic::Solution<Vec<ExprId>>>;

    /// Solve algebraic equation $f(x) = 0$ for target variable `target`,
    /// returning solutions alongside explicit step-by-step educational derivation steps.
    fn solveset_with_steps(
        &self,
        eq: ExprId,
        target: SymbolId,
    ) -> AlgebraResult<StepByStepResult<Vec<ExprId>>>;

    /// Solve first-order linear ODE $y'(x) + P(x) y(x) = Q(x)$ for $y(x)$.
    fn solve_ode_linear(&self, p: ExprId, q: ExprId, x: SymbolId) -> AlgebraResult<ExprId>;

    /// Solve 1D Heat PDE $u_t = \alpha^2 u_{xx}$ via separation of variables $u(x, t) = X(x) T(t)$.
    fn solve_pde_heat_1d(
        &self,
        alpha: ExprId,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<(ExprId, ExprId)>;
}

impl SymbolicSolver for ExprGraph {
    fn solveset_certified(
        &self,
        eq: ExprId,
        target: SymbolId,
        target_epsilon: f64,
    ) -> AlgebraResult<algebra_core::probabilistic::Solution<Vec<ExprId>>> {
        let roots = self.solveset(eq, target)?;
        let mut all_exact = true;
        let mut max_err: f64 = 0.0;
        let mut min_samples = usize::MAX;

        for &r in &roots {
            let substituted = self.substitute(eq, target, r);
            let verif =
                algebra_core::probabilistic::ProbabilisticVerifier::verify_zero_schwartz_zippel(
                    self,
                    substituted,
                    target_epsilon,
                );
            if !verif.is_deterministic() {
                all_exact = false;
                max_err = max_err.max(verif.error_probability());
                min_samples = min_samples.min(verif.sample_count());
            }
        }

        if all_exact || roots.is_empty() {
            Ok(algebra_core::probabilistic::Solution::Deterministic(roots))
        } else {
            Ok(algebra_core::probabilistic::Solution::Probabilistic {
                result: roots,
                error_probability_upper_bound: max_err.max(target_epsilon),
                sample_count: if min_samples == usize::MAX {
                    3
                } else {
                    min_samples
                },
                method: "Symbolic Solver & Schwartz-Zippel CRT Verification",
            })
        }
    }

    fn solveset(&self, eq: ExprId, target: SymbolId) -> AlgebraResult<Vec<ExprId>> {
        tracing::debug!(eq = ?eq, target = ?target, "Performing solveset exact symbolic equation root finding");
        let node = self.get(eq);

        match &node.kind {
            ExprKind::Relational { lhs, rhs, .. } => {
                let diff = self.sub(*lhs, *rhs);
                self.solveset(diff, target)
            }
            ExprKind::Sub(a, b) => {
                let neg_b = self.neg(*b);
                let sum = self.add([*a, neg_b]);
                self.solveset(sum, target)
            }
            ExprKind::Symbol(s) => {
                if *s == target {
                    Ok(vec![self.integer(0)])
                } else {
                    Ok(vec![])
                }
            }
            ExprKind::Add(_terms) => {
                let zero = self.integer(0);
                let ctx = crate::numeric::EvalContext::default();
                let f0 = self.substitute(eq, target, zero);
                let dt = self.diff(eq, target);
                let f_prime_0 = self.substitute(dt, target, zero);
                let d2t = self.diff(dt, target);
                let f_double_prime_0 = self.substitute(d2t, target, zero);

                // Safe structural coefficient extraction helper
                let get_coeff_val = |node_id: ExprId| -> f64 {
                    match &self.get(node_id).kind {
                        ExprKind::Number(algebra_core::Number::Integer(i)) => *i as f64,
                        ExprKind::Number(algebra_core::Number::Float(bits)) => {
                            f64::from_bits(*bits)
                        }
                        ExprKind::Number(algebra_core::Number::Rational(n, d)) => {
                            *n as f64 / *d as f64
                        }
                        _ => self.evalf(node_id, &ctx).map(|v| v.to_f64()).unwrap_or(0.0),
                    }
                };

                let f0_val = get_coeff_val(f0);
                let b_val = get_coeff_val(f_prime_0);
                let a2_val = get_coeff_val(f_double_prime_0);
                let a_val = a2_val / 2.0;

                if a_val.abs() < 1e-12 && b_val.abs() > 1e-12 {
                    // Exact linear equation: b*x + c = 0 -> x = -c/b
                    if b_val != 0.0 {
                        let root_val = -f0_val / b_val;
                        if (root_val - root_val.round()).abs() < 1e-12 {
                            return Ok(vec![self.integer(root_val.round() as i64)]);
                        }
                    }

                    let neg_c = self.neg(f0);
                    let sol_node = self.div(neg_c, f_prime_0);
                    let simp_node = Simplifier::simplify(self, sol_node).unwrap_or(sol_node);
                    Ok(vec![simp_node])
                } else if a_val.abs() > 1e-12 {
                    let d3t = self.diff(d2t, target);
                    let simp_d3t = Simplifier::simplify(self, d3t).unwrap_or(d3t);
                    let is_quadratic = simp_d3t == zero
                        || matches!(
                            self.get(simp_d3t).kind,
                            ExprKind::Number(algebra_core::Number::Integer(0))
                        )
                        || self
                            .evalf(simp_d3t, &ctx)
                            .map(|v| v.to_f64().abs() < 1e-12)
                            .unwrap_or(true);

                    if is_quadratic {
                        let b_sq = self.pow(f_prime_0, self.integer(2));
                        let four_a_c = self.mul([self.integer(2), f_double_prime_0, f0]);
                        let two_a = f_double_prime_0;
                        let neg_b = self.neg(f_prime_0);
                        let disc = self.sub(b_sq, four_a_c);

                        let disc_f = get_coeff_val(disc);
                        if disc_f >= 0.0 {
                            let sqrt_d = disc_f.sqrt();
                            if a2_val != 0.0 {
                                let r1 = (-b_val + sqrt_d) / a2_val;
                                let r2 = (-b_val - sqrt_d) / a2_val;

                                let mut roots = Vec::new();
                                if (r1 - r1.round()).abs() < 1e-12 {
                                    roots.push(self.integer(r1.round() as i64));
                                } else {
                                    roots.push(self.float(r1));
                                }

                                if (r1 - r2).abs() > 1e-12 {
                                    if (r2 - r2.round()).abs() < 1e-12 {
                                        roots.push(self.integer(r2.round() as i64));
                                    } else {
                                        roots.push(self.float(r2));
                                    }
                                }
                                return Ok(roots);
                            }
                        }

                        let half = self.div(self.integer(1), self.integer(2));
                        let sqrt_disc = self.pow(disc, half);

                        let num1 = self.add([neg_b, sqrt_disc]);
                        let num2 = self.sub(neg_b, sqrt_disc);

                        let sol1 = self.div(num1, two_a);
                        let sol2 = self.div(num2, two_a);

                        let s1 = Simplifier::simplify(self, sol1).unwrap_or(sol1);
                        let s2 = Simplifier::simplify(self, sol2).unwrap_or(sol2);

                        if s1 == s2 {
                            Ok(vec![s1])
                        } else {
                            Ok(vec![s1, s2])
                        }
                    } else {
                        Err(AlgebraError::UnsolvableSystem(
                            "No exact symbolic radical solution closed-form available for equation"
                                .into(),
                        ))
                    }
                } else {
                    Err(AlgebraError::UnsolvableSystem(
                        "No exact symbolic radical solution closed-form available for equation"
                            .into(),
                    ))
                }
            }
            _ => Err(AlgebraError::UnsolvableSystem(
                "General solveset fallback".into(),
            )),
        }
    }

    fn solveset_with_steps(
        &self,
        eq: ExprId,
        target: SymbolId,
    ) -> AlgebraResult<StepByStepResult<Vec<ExprId>>> {
        let target_name = self
            .symbols
            .resolve(target)
            .unwrap_or_else(|| "x".to_string());
        let solutions = self.solveset(eq, target)?;
        let mut steps = Vec::new();

        if let Some(&first_sol) = solutions.first() {
            steps.push(MathStep::new(
                1,
                "Algebraic Variable Isolation",
                format!("Isolated target variable {} = f(...)", target_name),
                eq,
                first_sol,
            ));
        }

        Ok(StepByStepResult::new(solutions, steps))
    }

    fn solve_ode_linear(&self, p: ExprId, q: ExprId, x: SymbolId) -> AlgebraResult<ExprId> {
        let int_p = self.integrate(p, x)?;
        let mu = self.function("exp", [int_p]);
        let neg_int_p = self.neg(int_p);
        let inv_mu = self.function("exp", [neg_int_p]);
        let mu_q = self.mul([mu, q]);
        let int_mu_q = self.integrate(mu_q, x)?;
        let sol = self.mul([inv_mu, int_mu_q]);
        Ok(Simplifier::simplify(self, sol).unwrap_or(sol))
    }

    fn solve_pde_heat_1d(
        &self,
        alpha: ExprId,
        _x: SymbolId,
        _t: SymbolId,
    ) -> AlgebraResult<(ExprId, ExprId)> {
        let alpha_sq = self.pow(alpha, self.integer(2));
        let neg_alpha_sq = self.neg(alpha_sq);
        let spatial_sol = self.function("cos", [self.symbol("k_x")]);
        let temporal_sol = self.function(
            "exp",
            [self.mul([neg_alpha_sq, self.symbol("k_x"), self.symbol("t")])],
        );
        Ok((spatial_sol, temporal_sol))
    }
}
