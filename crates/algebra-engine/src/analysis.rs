//! # `algebra_engine::analysis`
//!
//! Mathematical Function Analysis: Critical Points, Extrema, Inflection Points, & Zero Classification.
//!
//! Provides comprehensive analysis of scalar and multi-variate functions:
//! - **Zero Classification**: Determines the exact multiplicity $m$ and geometric character of roots:
//!   - Simple Transversal Crossing ($m = 1$).
//!   - Tangential Extremum ($m = 2k$, touches axis without sign change).
//!   - Stationary Inflection Crossing ($m = 2k+1, k \ge 1$).
//! - **Univariate Critical Points & Extrema**:
//!   - Solves $f'(x) = 0$ and performs higher-order derivative tests ($f''(x_0), f^{(3)}(x_0), \dots$).
//!   - Identifies local minima, local maxima, stationary points, and inflection points ($f''(x) = 0$).
//! - **Multi-Variate Critical Points & Hessian Classification**:
//!   - Solves stationary gradient system $\nabla f(\mathbf{x}) = \mathbf{0}$.
//!   - Evaluates symbolic Hessian matrix $\mathbf{H}_f(\mathbf{x}) = \left[ \frac{\partial^2 f}{\partial x_i \partial x_j} \right]$.
//!   - Classifies critical points via Hessian eigenvalue spectrum / Sylvester's criterion into
//!     Local Minima (positive-definite), Local Maxima (negative-definite), Saddle Points (indefinite), and Degenerate manifolds.

use crate::calculus::SymbolicCalculus;
use crate::numeric::{EvalContext, NumericalEval};
use crate::poly::roots::{CardanoSolver, ComplexRoot, DurandKernerSolver, FerrariSolver};
use crate::systems::numerical::{NumericalSystemConfig, NumericalSystemSolver};
use algebra_core::error::AlgebraResult;
use algebra_core::{ExprGraph, ExprId, SymbolId};

/// Classification of the nature of a zero / root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroType {
    /// Simple root ($m = 1$): crosses axis with non-zero slope $f'(x_0) \ne 0$.
    Simple,
    /// Tangential extremum ($m$ even $\ge 2$): touches axis with horizontal tangent and no sign change.
    TangentialExtremum,
    /// Inflection crossing ($m$ odd $\ge 3$): crosses axis with horizontal inflection tangent.
    InflectionCrossing,
}

/// Detailed classification and characterization of a function zero (root).
#[derive(Debug, Clone, PartialEq)]
pub struct ClassifiedZero {
    /// Value of the root.
    pub root: ComplexRoot,
    /// Multiplicity $m \ge 1$.
    pub multiplicity: usize,
    /// Geometric type of the zero.
    pub zero_type: ZeroType,
    /// Leading non-vanishing derivative value $f^{(m)}(x_0)$.
    pub leading_derivative_val: f64,
}

/// Geometric nature of a critical point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalPointKind {
    /// Local minimum ($f''(x_0) > 0$ or Hessian $\mathbf{H} \succ 0$).
    LocalMinimum,
    /// Local maximum ($f''(x_0) < 0$ or Hessian $\mathbf{H} \prec 0$).
    LocalMaximum,
    /// Stationary inflection point in 1D ($f'(x_0) = 0$ and $f''(x_0) = 0$, $f^{(3)}(x_0) \ne 0$).
    StationaryInflection,
    /// Non-stationary inflection point in 1D ($f''(x_0) = 0, f'(x_0) \ne 0$).
    Inflection,
    /// Multi-variate Saddle Point with mixed positive and negative curvature.
    SaddlePoint {
        /// Number of positive eigenvalues (directions of upward curvature).
        positive_inertia: usize,
        /// Number of negative eigenvalues (directions of downward curvature).
        negative_inertia: usize,
    },
    /// Degenerate critical point ($\det(\mathbf{H}) = 0$, monkey saddle or valley).
    Degenerate,
}

/// Univariate critical point $x_0 \in \mathbb{R}$.
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalPoint1D {
    /// Coordinate $x_0$.
    pub x: f64,
    /// Value $y_0 = f(x_0)$.
    pub y: f64,
    /// First derivative $f'(x_0)$.
    pub f_prime: f64,
    /// Second derivative $f''(x_0)$.
    pub f_double_prime: f64,
    /// Classified geometric nature.
    pub kind: CriticalPointKind,
}

/// Multi-variate critical point $\mathbf{x}_0 \in \mathbb{R}^n$.
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalPointND {
    /// Vector coordinate $\mathbf{x}_0 = (x_1, \dots, x_n)$.
    pub coords: Vec<f64>,
    /// Function value $f(\mathbf{x}_0)$.
    pub value: f64,
    /// Gradient vector $\nabla f(\mathbf{x}_0)$.
    pub gradient: Vec<f64>,
    /// Hessian matrix $\mathbf{H}_f(\mathbf{x}_0)$ ($n \times n$).
    pub hessian: Vec<Vec<f64>>,
    /// Eigenvalues of the Hessian matrix.
    pub eigenvalues: Vec<f64>,
    /// Classified geometric nature.
    pub kind: CriticalPointKind,
}

/// Comprehensive Function Analyzer.
pub struct FunctionAnalyzer;

#[allow(clippy::needless_range_loop)]
impl FunctionAnalyzer {
    /// Analyzes a set of known roots $r_1, \dots, r_k$ of a univariate function $f(x)$,
    /// computing their exact multiplicity and geometric type.
    pub fn classify_zeroes_1d(
        graph: &ExprGraph,
        expr: ExprId,
        var: SymbolId,
        roots: &[ComplexRoot],
    ) -> AlgebraResult<Vec<ClassifiedZero>> {
        let var_name = graph
            .symbols
            .resolve(var)
            .unwrap_or_else(|| "x".to_string());
        let mut classified = Vec::with_capacity(roots.len());

        // Compute first 6 successive derivatives
        let mut derivatives = Vec::with_capacity(6);
        let mut current_d = expr;
        for _ in 0..6 {
            current_d = graph.diff(current_d, var);
            derivatives.push(current_d);
        }

        for r in roots {
            let mut ctx = EvalContext::new(20);
            ctx.bindings.insert(var_name.clone(), r.re);

            let mut mult = 1;
            let mut leading_val = 0.0;

            for (d_idx, &d_expr) in derivatives.iter().enumerate() {
                let d_val = graph.evalf(d_expr, &ctx)?.to_f64();
                if d_val.abs() > 1e-6 {
                    mult = d_idx + 1;
                    leading_val = d_val;
                    break;
                }
            }

            let zero_type = if mult == 1 {
                ZeroType::Simple
            } else if mult % 2 == 0 {
                ZeroType::TangentialExtremum
            } else {
                ZeroType::InflectionCrossing
            };

            classified.push(ClassifiedZero {
                root: r.clone(),
                multiplicity: mult,
                zero_type,
                leading_derivative_val: leading_val,
            });
        }

        Ok(classified)
    }

    /// Computes all critical points, local extrema, and inflection points of a univariate function $f(x)$.
    pub fn analyze_critical_points_1d(
        graph: &ExprGraph,
        expr: ExprId,
        var: SymbolId,
        search_points: &[f64],
    ) -> AlgebraResult<Vec<CriticalPoint1D>> {
        let var_name = graph
            .symbols
            .resolve(var)
            .unwrap_or_else(|| "x".to_string());

        let df = graph.diff(expr, var);
        let d2f = graph.diff(df, var);
        let d3f = graph.diff(d2f, var);

        let mut critical_points = Vec::new();
        let config = NumericalSystemConfig {
            tol: 1e-7,
            max_iterations: 100,
            ..Default::default()
        };

        // Find roots of f'(x) = 0 using search points
        let mut found_x: Vec<f64> = Vec::new();
        for &x0 in search_points {
            if let Ok(sol) =
                NumericalSystemSolver::solve_newton_raphson(graph, &[df], &[var], &[x0], &config)
                && !sol.is_empty()
            {
                let root_x = sol[0];
                let mut ctx = EvalContext::new(20);
                ctx.bindings.insert(var_name.clone(), root_x);
                if let Ok(df_v) = graph.evalf(df, &ctx)
                    && df_v.to_f64().abs() < 1e-4
                    && !found_x.iter().any(|&x| (x - root_x).abs() < 1e-4)
                {
                    found_x.push(root_x);
                }
            }
        }

        // Also check if df is a polynomial and solve directly via analytical solvers
        if let Ok(poly_roots) = Self::try_solve_poly_derivative(graph, df, &var_name) {
            for pr in poly_roots {
                if pr.im.abs() < 1e-10 && !found_x.iter().any(|&x| (x - pr.re).abs() < 1e-4) {
                    found_x.push(pr.re);
                }
            }
        }

        // Classify each found critical point x
        for &x_val in &found_x {
            let mut ctx = EvalContext::new(20);
            ctx.bindings.insert(var_name.clone(), x_val);

            let y_val = graph.evalf(expr, &ctx)?.to_f64();
            let df_val = graph.evalf(df, &ctx)?.to_f64();
            let d2f_val = graph.evalf(d2f, &ctx)?.to_f64();
            let d3f_val = graph.evalf(d3f, &ctx)?.to_f64();

            let kind = if d2f_val > 1e-6 {
                CriticalPointKind::LocalMinimum
            } else if d2f_val < -1e-6 {
                CriticalPointKind::LocalMaximum
            } else if d3f_val.abs() > 1e-6 {
                CriticalPointKind::StationaryInflection
            } else {
                CriticalPointKind::Degenerate
            };

            critical_points.push(CriticalPoint1D {
                x: x_val,
                y: y_val,
                f_prime: df_val,
                f_double_prime: d2f_val,
                kind,
            });
        }

        // Find Inflection Points where f''(x) = 0 and f'(x) != 0
        let mut found_inflection_x: Vec<f64> = Vec::new();
        for &x0 in search_points {
            if let Ok(sol) =
                NumericalSystemSolver::solve_newton_raphson(graph, &[d2f], &[var], &[x0], &config)
                && !sol.is_empty()
            {
                let root_x = sol[0];
                let mut ctx = EvalContext::new(20);
                ctx.bindings.insert(var_name.clone(), root_x);
                if let Ok(d2f_v) = graph.evalf(d2f, &ctx)
                    && d2f_v.to_f64().abs() < 1e-4
                    && !found_x.iter().any(|&x| (x - root_x).abs() < 1e-4)
                    && !found_inflection_x
                        .iter()
                        .any(|&x| (x - root_x).abs() < 1e-4)
                {
                    found_inflection_x.push(root_x);
                }
            }
        }

        if let Ok(poly_roots) = Self::try_solve_poly_derivative(graph, d2f, &var_name) {
            for pr in poly_roots {
                if pr.im.abs() < 1e-10
                    && !found_x.iter().any(|&x| (x - pr.re).abs() < 1e-4)
                    && !found_inflection_x.iter().any(|&x| (x - pr.re).abs() < 1e-4)
                {
                    found_inflection_x.push(pr.re);
                }
            }
        }

        for &x_val in &found_inflection_x {
            let mut ctx = EvalContext::new(20);
            ctx.bindings.insert(var_name.clone(), x_val);

            let y_val = graph.evalf(expr, &ctx)?.to_f64();
            let df_val = graph.evalf(df, &ctx)?.to_f64();
            let d2f_val = graph.evalf(d2f, &ctx)?.to_f64();

            critical_points.push(CriticalPoint1D {
                x: x_val,
                y: y_val,
                f_prime: df_val,
                f_double_prime: d2f_val,
                kind: CriticalPointKind::Inflection,
            });
        }

        critical_points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        Ok(critical_points)
    }

    /// Analyzes multi-variate critical points for $f(x_1, \dots, x_n)$ by solving $\nabla f = \mathbf{0}$
    /// and performing spectral decomposition on the symbolic Hessian matrix $\mathbf{H}_f$.
    pub fn analyze_critical_points_nd(
        graph: &ExprGraph,
        expr: ExprId,
        variables: &[SymbolId],
        initial_guesses: &[Vec<f64>],
    ) -> AlgebraResult<Vec<CriticalPointND>> {
        let n = variables.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut var_names = Vec::with_capacity(n);
        for &v in variables {
            let name = graph.symbols.resolve(v).unwrap_or_else(|| "x".to_string());
            var_names.push(name);
        }

        // 1. Symbolic Gradient: g_i = d f / d x_i
        let mut gradient_exprs = Vec::with_capacity(n);
        for &v in variables {
            let df = graph.diff(expr, v);
            gradient_exprs.push(df);
        }

        // 2. Symbolic Hessian Matrix: H_{i,j} = d^2 f / (d x_i d x_j)
        let mut hessian_exprs = vec![vec![expr; n]; n];
        for i in 0..n {
            for j in 0..n {
                let d2f = graph.diff(gradient_exprs[i], variables[j]);
                hessian_exprs[i][j] = d2f;
            }
        }

        let config = NumericalSystemConfig {
            tol: 1e-7,
            max_iterations: 150,
            ..Default::default()
        };

        let mut critical_points = Vec::new();
        let mut found_coords: Vec<Vec<f64>> = Vec::new();

        // 3. Solve grad f = 0 from initial guesses
        for guess in initial_guesses {
            if guess.len() != n {
                continue;
            }
            if let Ok(sol) = NumericalSystemSolver::solve_newton_raphson(
                graph,
                &gradient_exprs,
                variables,
                guess,
                &config,
            ) {
                // Check if already found
                let is_duplicate = found_coords.iter().any(|c| {
                    c.iter()
                        .zip(&sol)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                        .sqrt()
                        < 1e-4
                });

                if !is_duplicate {
                    found_coords.push(sol.clone());

                    // 4. Evaluate function, gradient, and Hessian at solution point
                    let mut ctx = EvalContext::new(20);
                    for (i, name) in var_names.iter().enumerate() {
                        ctx.bindings.insert(name.clone(), sol[i]);
                    }

                    let val = graph.evalf(expr, &ctx)?.to_f64();

                    let mut grad_vals = Vec::with_capacity(n);
                    for &g_expr in &gradient_exprs {
                        grad_vals.push(graph.evalf(g_expr, &ctx)?.to_f64());
                    }

                    let mut h_mat = vec![vec![0.0; n]; n];
                    for i in 0..n {
                        for j in 0..n {
                            h_mat[i][j] = graph.evalf(hessian_exprs[i][j], &ctx)?.to_f64();
                        }
                    }

                    // 5. Compute Hessian Eigenvalues for classification
                    let eigenvalues = Self::compute_symmetric_eigenvalues(&h_mat);

                    let pos_count = eigenvalues.iter().filter(|&&e| e > 1e-6).count();
                    let neg_count = eigenvalues.iter().filter(|&&e| e < -1e-6).count();
                    let zero_count = eigenvalues.iter().filter(|&&e| e.abs() <= 1e-6).count();

                    let kind = if zero_count > 0 {
                        CriticalPointKind::Degenerate
                    } else if pos_count == n {
                        CriticalPointKind::LocalMinimum
                    } else if neg_count == n {
                        CriticalPointKind::LocalMaximum
                    } else {
                        CriticalPointKind::SaddlePoint {
                            positive_inertia: pos_count,
                            negative_inertia: neg_count,
                        }
                    };

                    critical_points.push(CriticalPointND {
                        coords: sol,
                        value: val,
                        gradient: grad_vals,
                        hessian: h_mat,
                        eigenvalues,
                        kind,
                    });
                }
            }
        }

        Ok(critical_points)
    }

    /// Computes eigenvalues of an $n \times n$ symmetric matrix.
    fn compute_symmetric_eigenvalues(matrix: &[Vec<f64>]) -> Vec<f64> {
        let n = matrix.len();
        if n == 0 {
            return Vec::new();
        }
        if n == 1 {
            return vec![matrix[0][0]];
        }
        if n == 2 {
            // 2x2 exact characteristic polynomial eigenvalues:
            // det(H - lambda*I) = lambda^2 - tr(H)*lambda + det(H) = 0
            let tr = matrix[0][0] + matrix[1][1];
            let det = matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0];
            let disc = (tr * tr - 4.0 * det).max(0.0);
            let lam1 = (tr + disc.sqrt()) / 2.0;
            let lam2 = (tr - disc.sqrt()) / 2.0;
            return vec![lam1, lam2];
        }

        // For n >= 3, use Jacobi eigenvalue method for symmetric matrices
        let mut a = matrix.to_vec();
        let max_rotations = 50;

        for _ in 0..max_rotations {
            // Find largest off-diagonal element
            let mut p = 0;
            let mut q = 1;
            let mut max_val = a[0][1].abs();

            for i in 0..n {
                for j in (i + 1)..n {
                    if a[i][j].abs() > max_val {
                        max_val = a[i][j].abs();
                        p = i;
                        q = j;
                    }
                }
            }

            if max_val < 1e-10 {
                break;
            }

            // Compute Jacobi rotation angle
            let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let t = if theta >= 0.0 {
                1.0 / (theta + (theta * theta + 1.0).sqrt())
            } else {
                -1.0 / (-theta + (theta * theta + 1.0).sqrt())
            };
            let c = 1.0 / (t * t + 1.0).sqrt();
            let s = t * c;

            // Apply similarity transformation A' = J^T A J
            let app = a[p][p];
            let aqq = a[q][q];
            let apq = a[p][q];

            a[p][p] = app - t * apq;
            a[q][q] = aqq + t * apq;
            a[p][q] = 0.0;
            a[q][p] = 0.0;

            for r in 0..n {
                if r != p && r != q {
                    let arp = a[r][p];
                    let arq = a[r][q];
                    a[r][p] = c * arp - s * arq;
                    a[p][r] = a[r][p];
                    a[r][q] = s * arp + c * arq;
                    a[q][r] = a[r][q];
                }
            }
        }

        let mut eigenvalues: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
        eigenvalues.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        eigenvalues
    }

    /// Attempts to extract dense polynomial coefficients and solve roots analytically.
    fn try_solve_poly_derivative(
        graph: &ExprGraph,
        df_expr: ExprId,
        var_name: &str,
    ) -> AlgebraResult<Vec<ComplexRoot>> {
        use crate::poly::multivariate::{MonomialOrder, MultiPoly};
        let vars = vec![var_name.to_string()];
        let p = MultiPoly::from_expr(graph, df_expr, &vars, MonomialOrder::Lex)?;
        if p.is_zero() {
            return Ok(Vec::new());
        }

        let deg = p
            .terms
            .first()
            .map(|t| t.monomial.exponents[0])
            .unwrap_or(0);
        let mut coeffs = vec![0.0; deg + 1];
        for t in &p.terms {
            let exp = t.monomial.exponents[0];
            let idx = deg - exp;
            coeffs[idx] = t.coeff;
        }

        match deg {
            1 => {
                let a = coeffs[0];
                let b = coeffs[1];
                if a.abs() > 1e-14 {
                    Ok(vec![ComplexRoot::real(-b / a)])
                } else {
                    Ok(Vec::new())
                }
            }
            2 => {
                let a = coeffs[0];
                let b = coeffs[1];
                let c = coeffs[2];
                let disc = b * b - 4.0 * a * c;
                if disc >= 0.0 {
                    Ok(vec![
                        ComplexRoot::real((-b + disc.sqrt()) / (2.0 * a)),
                        ComplexRoot::real((-b - disc.sqrt()) / (2.0 * a)),
                    ])
                } else {
                    Ok(vec![
                        ComplexRoot::complex(-b / (2.0 * a), (-disc).sqrt() / (2.0 * a)),
                        ComplexRoot::complex(-b / (2.0 * a), -(-disc).sqrt() / (2.0 * a)),
                    ])
                }
            }
            3 => CardanoSolver::solve(coeffs[0], coeffs[1], coeffs[2], coeffs[3]),
            4 => FerrariSolver::solve(coeffs[0], coeffs[1], coeffs[2], coeffs[3], coeffs[4]),
            _ => {
                let lc = coeffs[0];
                let monic: Vec<f64> = coeffs.iter().map(|&c| c / lc).collect();
                Ok(DurandKernerSolver::solve(&monic, 100, 1e-10))
            }
        }
    }
}
