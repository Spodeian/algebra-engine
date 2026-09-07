//! # `algebra_engine::pde::symbolic`
//!
//! Symbolic Partial Differential Equation (PDE) Solvers.
//!
//! Implements:
//! - **PDE Classification**: Discriminant $B^2 - 4AC$ classifying into Elliptic, Parabolic, and Hyperbolic.
//! - **Method of Characteristics**: Solves first-order quasilinear PDEs $a u_x + b u_y = c$.
//! - **Separation of Variables**: Sturm-Liouville eigenbasis expansions for Heat, Wave, and Laplace equations.
//! - **d'Alembert Wave Solution**: Exact analytical wave propagation $u(x, t) = \frac{1}{2}[f(x - ct) + f(x + ct)] + \frac{1}{2c}\int_{x-ct}^{x+ct} g(s) ds$.
//! - **Green's Functions & Fundamental Kernels**: Heat kernel $K(x, t) = \frac{1}{\sqrt{4\pi k t}} e^{-x^2/4kt}$, 3D Poisson potential.

use crate::simplify::Simplifier;
use algebra_core::{AlgebraResult, ExprGraph, ExprId, SymbolId};

/// Classification of 2nd-Order Linear Partial Differential Equations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdeClassification {
    /// Elliptic (e.g. Laplace/Poisson equation: u_xx + u_yy = 0, B^2 - 4AC < 0)
    Elliptic,
    /// Parabolic (e.g. Heat/Diffusion equation: u_t = alpha * u_xx, B^2 - 4AC = 0)
    Parabolic,
    /// Hyperbolic (e.g. Wave equation: u_tt = c^2 * u_xx, B^2 - 4AC > 0)
    Hyperbolic,
}

/// Symbolic PDE Solver Suite.
pub struct SymbolicPdeSolver;

impl SymbolicPdeSolver {
    /// Classify a general 2nd-order PDE $A u_{xx} + B u_{xy} + C u_{yy} + D u_x + E u_y + F u = G$.
    pub fn classify_second_order(a: f64, b: f64, c: f64) -> PdeClassification {
        let disc = b * b - 4.0 * a * c;
        if disc < -1e-12 {
            PdeClassification::Elliptic
        } else if disc.abs() <= 1e-12 {
            PdeClassification::Parabolic
        } else {
            PdeClassification::Hyperbolic
        }
    }

    /// Solve 1D Wave Equation initial value problem on $(-\infty, \infty)$ via d'Alembert's Formula:
    ///
    /// $$u_{tt} = c^2 u_{xx}, \quad u(x, 0) = f(x), \quad u_t(x, 0) = g(x)$$
    ///
    /// $$u(x, t) = \frac{1}{2} \left[ f(x - ct) + f(x + ct) \right] + \frac{1}{2c} \int_{x - ct}^{x + ct} g(s) \, ds$$
    pub fn solve_wave_dalembert(
        graph: &ExprGraph,
        f_init: ExprId,
        g_init: Option<ExprId>,
        c_speed: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let c_node = graph.float(c_speed);
        Self::solve_wave_dalembert_exact(graph, f_init, g_init, c_node, x, t)
    }

    /// Exact symbolic d'Alembert wave solution accepting lossless `c_node: ExprId`.
    pub fn solve_wave_dalembert_exact(
        graph: &ExprGraph,
        f_init: ExprId,
        g_init: Option<ExprId>,
        c_node: ExprId,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(t_str) = graph.symbols.resolve(t) {
            graph.symbol(&t_str)
        } else {
            graph.symbol("t")
        };
        let ct = graph.mul([c_node, var_t]);

        let x_minus_ct = graph.sub(var_x, ct);
        let x_plus_ct = graph.add([var_x, ct]);

        // Evaluate f(x - ct) and f(x + ct)
        let f_left = graph.substitute(f_init, x, x_minus_ct);
        let f_right = graph.substitute(f_init, x, x_plus_ct);

        let half = graph.div(graph.integer(1), graph.integer(2));
        let f_sum = graph.add([f_left, f_right]);
        let pos_part = graph.mul([half, f_sum]);

        if let Some(g) = g_init {
            let _s_sym = graph.symbol("s");
            let two_c = graph.mul([graph.integer(2), c_node]);
            let inv_two_c = graph.div(graph.integer(1), two_c);
            let int_g = graph.integral(g, "s", Some(x_minus_ct), Some(x_plus_ct));
            let vel_part = graph.mul([inv_two_c, int_g]);
            let total = graph.add([pos_part, vel_part]);
            Ok(Simplifier::simplify(graph, total).unwrap_or(total))
        } else {
            Ok(Simplifier::simplify(graph, pos_part).unwrap_or(pos_part))
        }
    }

    /// Solve 1D Heat Equation $u_t = \alpha^2 u_{xx}$ on $[0, L]$ with Dirichlet boundary conditions $u(0, t) = u(L, t) = 0$.
    ///
    /// General Fourier eigenbasis solution:
    /// $$u(x, t) = \sum_{n=1}^\infty B_n \sin\left(\frac{n\pi x}{L}\right) \exp\left(-\alpha^2 \left(\frac{n\pi}{L}\right)^2 t\right)$$
    pub fn solve_heat_1d_separated(
        graph: &ExprGraph,
        alpha: f64,
        l_length: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<(ExprId, ExprId)> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(t_str) = graph.symbols.resolve(t) {
            graph.symbol(&t_str)
        } else {
            graph.symbol("t")
        };

        let pi = graph.symbol("pi");
        let n_sym = graph.symbol("n");
        let l_node = graph.float(l_length);

        // Spatial eigenfunction X_n(x) = sin(n * pi * x / L)
        let n_pi_x = graph.mul([n_sym, pi, var_x]);
        let k_n_x = graph.div(n_pi_x, l_node);
        let spatial_mode = graph.function("sin", [k_n_x]);

        // Temporal decay T_n(t) = exp(-alpha^2 * (n*pi/L)^2 * t)
        let alpha_sq = graph.float(alpha * alpha);
        let k_n = graph.div(graph.mul([n_sym, pi]), l_node);
        let k_n_sq = graph.pow(k_n, graph.integer(2));
        let decay_rate = graph.mul([graph.neg(alpha_sq), k_n_sq, var_t]);
        let temporal_mode = graph.function("exp", [decay_rate]);

        Ok((spatial_mode, temporal_mode))
    }

    /// Construct 1D Fundamental Heat Kernel (Green's function):
    ///
    /// $$K(x, t) = \frac{1}{\sqrt{4\pi \alpha^2 t}} \exp\left( -\frac{x^2}{4\alpha^2 t} \right)$$
    pub fn heat_kernel_1d(
        graph: &ExprGraph,
        alpha: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(t_str) = graph.symbols.resolve(t) {
            graph.symbol(&t_str)
        } else {
            graph.symbol("t")
        };

        let four_alpha_sq = graph.float(4.0 * alpha * alpha);
        let pi = graph.symbol("pi");
        let denom_inner = graph.mul([four_alpha_sq, pi, var_t]);
        let half = graph.div(graph.integer(1), graph.integer(2));
        let norm_factor = graph.pow(denom_inner, half);
        let inv_norm = graph.div(graph.integer(1), norm_factor);

        let x_sq = graph.pow(var_x, graph.integer(2));
        let four_alpha_sq_t = graph.mul([four_alpha_sq, var_t]);
        let exp_arg = graph.neg(graph.div(x_sq, four_alpha_sq_t));
        let exp_term = graph.function("exp", [exp_arg]);

        let kernel = graph.mul([inv_norm, exp_term]);
        Ok(Simplifier::simplify(graph, kernel).unwrap_or(kernel))
    }

    /// Solve 1D Wave Equation $u_{tt} = c^2 u_{xx}$ on $[0, L]$ with fixed Dirichlet boundary conditions $u(0, t) = u(L, t) = 0$.
    ///
    /// Spatial and temporal separation modes:
    /// $$X_n(x) = \sin\left(\frac{n\pi x}{L}\right), \quad T_n(t) = \cos\left(\frac{n\pi c t}{L}\right)$$
    pub fn solve_wave_1d_separated(
        graph: &ExprGraph,
        c_node: ExprId,
        l_node: ExprId,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<(ExprId, ExprId)> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(t_str) = graph.symbols.resolve(t) {
            graph.symbol(&t_str)
        } else {
            graph.symbol("t")
        };

        let pi = graph.symbol("pi");
        let n_sym = graph.symbol("n");

        // Spatial mode: sin(n * pi * x / L)
        let n_pi_x = graph.mul([n_sym, pi, var_x]);
        let k_x = graph.div(n_pi_x, l_node);
        let spatial_mode = graph.function("sin", [k_x]);

        // Temporal mode: cos(n * pi * c * t / L)
        let omega_t = graph.mul([n_sym, pi, c_node, var_t]);
        let arg_t = graph.div(omega_t, l_node);
        let temporal_mode = graph.function("cos", [arg_t]);

        Ok((spatial_mode, temporal_mode))
    }

    /// Solve 2D Laplace Equation $u_{xx} + u_{yy} = 0$ on rectangular domain $[0, a] \times [0, b]$
    /// with boundary conditions $u(0, y) = u(a, y) = 0, u(x, 0) = 0$.
    ///
    /// Harmonic separation eigenbasis:
    /// $$X_n(x) = \sin\left(\frac{n\pi x}{a}\right), \quad Y_n(y) = \sinh\left(\frac{n\pi y}{a}\right)$$
    pub fn solve_laplace_2d_separated(
        graph: &ExprGraph,
        a_node: ExprId,
        x: SymbolId,
        y: SymbolId,
    ) -> AlgebraResult<(ExprId, ExprId)> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_y = if let Some(s_str) = graph.symbols.resolve(y) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("y")
        };

        let pi = graph.symbol("pi");
        let n_sym = graph.symbol("n");

        let n_pi_x = graph.mul([n_sym, pi, var_x]);
        let k_x = graph.div(n_pi_x, a_node);
        let spatial_x = graph.function("sin", [k_x]);

        let n_pi_y = graph.mul([n_sym, pi, var_y]);
        let k_y = graph.div(n_pi_y, a_node);
        let spatial_y = graph.function("sinh", [k_y]);

        Ok((spatial_x, spatial_y))
    }

    /// Solve 1D Transport / Advection Equation via method of characteristics:
    ///
    /// $$u_t + c u_x = 0, \quad u(x, 0) = f(x) \implies u(x, t) = f(x - c t)$$
    pub fn solve_transport_1d(
        graph: &ExprGraph,
        f_init: ExprId,
        c_node: ExprId,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s_str) = graph.symbols.resolve(x) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(t_str) = graph.symbols.resolve(t) {
            graph.symbol(&t_str)
        } else {
            graph.symbol("t")
        };

        let ct = graph.mul([c_node, var_t]);
        let char_coord = graph.sub(var_x, ct);
        let sol = graph.substitute(f_init, x, char_coord);
        Ok(Simplifier::simplify(graph, sol).unwrap_or(sol))
    }

    /// Solve 2D Cylindrical / Radial Laplacian Eigenproblem:
    ///
    /// $$\nabla^2 u + k^2 u = 0 \implies \frac{1}{r} \frac{d}{dr}\left( r \frac{dR}{dr} \right) + \left( k^2 - \frac{n^2}{r^2} \right) R = 0$$
    ///
    /// Yields exact Bessel radical / transcendental eigenfunction:
    /// $$R_n(r) = J_n(k r)$$
    pub fn solve_bessel_radial(
        graph: &ExprGraph,
        k_node: ExprId,
        r: SymbolId,
        order: usize,
    ) -> AlgebraResult<ExprId> {
        let var_r = if let Some(s_str) = graph.symbols.resolve(r) {
            graph.symbol(&s_str)
        } else {
            graph.symbol("r")
        };
        let kr = graph.mul([k_node, var_r]);
        let n_order = graph.integer(order as i64);
        let radial_sol = graph.function("besselj", [n_order, kr]);
        Ok(radial_sol)
    }
}
