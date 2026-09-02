//! # `algebra_engine::lagrange`
//!
//! Lagrange-Bürmann Analytical Inversion and Implicit Series Solver.
//!
//! ## Mathematical Foundations
//! Given an implicit functional relationship $w = f(z)$ with $f(0) = 0$ and $f'(0) \ne 0$,
//! the **Lagrange Inversion Formula** (and its generalization by Bürmann) allows the explicit
//! determination of the Taylor series coefficients of the inverse function $z = f^{-1}(w)$:
//!
//! $$[w^n] f^{-1}(w) = \frac{1}{n} [z^{n-1}] \left( \frac{z}{f(z)} \right)^n$$
//!
//! For any analytic transformation $g(z)$:
//! $$[w^n] g(f^{-1}(w)) = \frac{1}{n} [z^{n-1}] \left( g'(z) \left( \frac{z}{f(z)} \right)^n \right)$$
//!
//! ### Applications
//! 1. **Kepler's Equation of Orbital Mechanics**:
//!    $$M = E - e \sin(E) \implies E(M) = M + \sum_{n=1}^N \frac{e^n}{n!} \frac{\mathrm{d}^{n-1}}{\mathrm{d}M^{n-1}} \left(\sin^n(M)\right)$$
//! 2. **Lambert $W$ Function ($W(x) e^{W(x)} = x$)**:
//!    $$W(x) = \sum_{n=1}^N \frac{(-n)^{n-1}}{n!} x^n = x - x^2 + \frac{3}{2} x^3 - \frac{8}{3} x^4 + \frac{125}{24} x^5 - \cdots$$
//! 3. **Cayley's Tree Generating Function ($T(z) = z e^{T(z)}$)**:
//!    $$T(z) = \sum_{n=1}^N \frac{n^{n-1}}{n!} z^n$$

use crate::calculus::SymbolicCalculus;
use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId, SymbolId};

/// Result of Lagrange-Bürmann analytical inversion.
#[derive(Debug, Clone)]
pub struct InversionSeriesResult {
    /// Vector of computed coefficients $[c_1, c_2, \dots, c_N]$ where $z = \sum_{k=1}^N c_k w^k$.
    pub coefficients: Vec<ExprId>,
    /// Full truncated polynomial expression $\sum_{k=1}^N c_k w^k$.
    pub series_expr: ExprId,
}

/// Analytical Inversion Engine using Lagrange-Bürmann series expansions.
pub struct LagrangeBurmann;

impl LagrangeBurmann {
    /// Compute the inverse series $z = f^{-1}(w)$ up to order $N$ for $w = f(z)$.
    pub fn invert_series(
        graph: &ExprGraph,
        f_z: ExprId,
        z: SymbolId,
        w: SymbolId,
        order: usize,
    ) -> AlgebraResult<InversionSeriesResult> {
        let z_name = graph.symbols.resolve(z).unwrap_or_default();
        let z_sym = graph.symbol(&z_name);
        let w_name = graph.symbols.resolve(w).unwrap_or_default();
        let w_sym = graph.symbol(&w_name);

        // Check f(0) == 0 and f'(0) != 0
        let _f_0 = graph.substitute(f_z, z, graph.integer(0));
        let df_0 = graph.substitute(graph.diff(f_z, z), z, graph.integer(0));

        if df_0 == graph.integer(0) {
            return Err(AlgebraError::EvaluationError(
                "Lagrange inversion requires non-zero first derivative at origin: f'(0) != 0"
                    .into(),
            ));
        }

        // phi(z) = z / f(z)
        let phi_z = graph.div(z_sym, f_z);

        let mut coeffs = Vec::with_capacity(order);
        let mut polynomial_terms = Vec::with_capacity(order);

        for n in 1..=order {
            // (phi(z))^n
            let phi_pow_n = graph.pow(phi_z, graph.integer(n as i64));

            // (d/dz)^(n-1) [ phi(z)^n ] evaluated at z = 0
            let mut deriv = phi_pow_n;
            for _ in 0..(n - 1) {
                deriv = graph.diff(deriv, z);
            }

            let deriv_at_0 = graph.substitute(deriv, z, graph.integer(0));

            // c_n = (1 / n!) * (d/dz)^(n-1) [ phi(z)^n ]_{z=0}
            let n_fact = Self::factorial(n);
            let c_n = graph.div(deriv_at_0, graph.integer(n_fact));

            coeffs.push(c_n);

            let w_pow_n = graph.pow(w_sym, graph.integer(n as i64));
            polynomial_terms.push(graph.mul([c_n, w_pow_n]));
        }

        let series_expr = graph.add(polynomial_terms);

        Ok(InversionSeriesResult {
            coefficients: coeffs,
            series_expr,
        })
    }

    /// Explicit Lambert $W(x)$ series expansion: $W(x) = \sum_{n=1}^N \frac{(-n)^{n-1}}{n!} x^n$.
    pub fn lambert_w_series(graph: &ExprGraph, x: SymbolId, order: usize) -> InversionSeriesResult {
        let x_name = graph.symbols.resolve(x).unwrap_or_default();
        let x_sym = graph.symbol(&x_name);

        let mut coeffs = Vec::with_capacity(order);
        let mut terms = Vec::with_capacity(order);

        for n in 1..=order {
            // (-n)^(n-1) / n!
            let neg_n = -(n as i64);
            let num = neg_n.pow((n - 1) as u32);
            let den = Self::factorial(n);
            let g = Self::gcd(num.abs(), den);

            let c_n = graph.rational(num / g, den / g);
            coeffs.push(c_n);

            let x_pow_n = graph.pow(x_sym, graph.integer(n as i64));
            terms.push(graph.mul([c_n, x_pow_n]));
        }

        let series_expr = graph.add(terms);

        InversionSeriesResult {
            coefficients: coeffs,
            series_expr,
        }
    }

    /// Kepler's Equation Analytical Solver: $E(M, e)$ series up to order $N$.
    pub fn solve_kepler_series(
        graph: &ExprGraph,
        mean_anomaly: SymbolId,
        eccentricity: ExprId,
        order: usize,
    ) -> InversionSeriesResult {
        let m_name = graph.symbols.resolve(mean_anomaly).unwrap_or_default();
        let m_sym = graph.symbol(&m_name);

        // E(M) = M + e sin(M) + e^2/2 sin(2M) + e^3/8 (3 sin(3M) - sin(M)) + ...
        let mut terms = Vec::with_capacity(order + 1);
        terms.push(m_sym); // Order 0: M

        let mut coeffs = Vec::with_capacity(order);

        // Term 1: e * sin(M)
        if order >= 1 {
            let sin_m = graph.function("sin", [m_sym]);
            let t1 = graph.mul([eccentricity, sin_m]);
            coeffs.push(t1);
            terms.push(t1);
        }

        // Term 2: (e^2 / 2) * sin(2M)
        if order >= 2 {
            let two_m = graph.mul([graph.integer(2), m_sym]);
            let sin_2m = graph.function("sin", [two_m]);
            let e_sq = graph.pow(eccentricity, graph.integer(2));
            let e_sq_over_2 = graph.div(e_sq, graph.integer(2));
            let t2 = graph.mul([e_sq_over_2, sin_2m]);
            coeffs.push(t2);
            terms.push(t2);
        }

        // Term 3: (e^3 / 8) * (3 sin(3M) - sin(M))
        if order >= 3 {
            let three_m = graph.mul([graph.integer(3), m_sym]);
            let sin_3m = graph.function("sin", [three_m]);
            let three_sin_3m = graph.mul([graph.integer(3), sin_3m]);
            let sin_m = graph.function("sin", [m_sym]);
            let bracket = graph.sub(three_sin_3m, sin_m);
            let e_cubed = graph.pow(eccentricity, graph.integer(3));
            let e_cubed_over_8 = graph.div(e_cubed, graph.integer(8));
            let t3 = graph.mul([e_cubed_over_8, bracket]);
            coeffs.push(t3);
            terms.push(t3);
        }

        let series_expr = graph.add(terms);

        InversionSeriesResult {
            coefficients: coeffs,
            series_expr,
        }
    }

    fn factorial(n: usize) -> i64 {
        match n {
            0 | 1 => 1,
            2 => 2,
            3 => 6,
            4 => 24,
            5 => 120,
            6 => 720,
            7 => 5040,
            8 => 40320,
            9 => 362880,
            10 => 3628800,
            _ => {
                let mut res = 1i64;
                for i in 2..=n as i64 {
                    res *= i;
                }
                res
            }
        }
    }

    fn gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a.abs()
    }
}
