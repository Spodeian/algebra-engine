//! # `algebra_engine::pde::reduction`
//!
//! Traveling Wave Reductions and Symmetry Invariant ODE Reductions for Nonlinear PDEs.
//!
//! Converts nonlinear evolution PDEs into autonomous ODE boundary value and soliton problems:
//! - **Korteweg-de Vries (KdV)**: Solitary wave / sech^2 soliton profile
//! - **Viscous Burgers Equation**: Traveling shock front profile
//! - **Fisher-KPP Reaction-Diffusion**: Traveling wave invasion front

use algebra_core::{AlgebraResult, ExprGraph, ExprId, SymbolId};
use crate::simplify::Simplifier;

/// Nonlinear PDE Traveling Wave Reductions.
pub struct PdeTravelingWaveReduction;

impl PdeTravelingWaveReduction {
    /// Construct exact single-soliton analytical solution for the Korteweg-de Vries (KdV) equation:
    ///
    /// $$u_t + 6 u u_x + u_{xxx} = 0 \implies u(x, t) = \frac{c}{2} \operatorname{sech}^2\left( \frac{\sqrt{c}}{2}(x - c t) \right)$$
    pub fn kdv_soliton(
        graph: &ExprGraph,
        c_speed: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s) = graph.symbols.resolve(x) {
            graph.symbol(&s)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(s) = graph.symbols.resolve(t) {
            graph.symbol(&s)
        } else {
            graph.symbol("t")
        };

        let c_node = graph.float(c_speed);
        let half = graph.div(graph.integer(1), graph.integer(2));
        let half_c = graph.mul([half, c_node]);

        let ct = graph.mul([c_node, var_t]);
        let xi = graph.sub(var_x, ct);

        let sqrt_c = graph.float(c_speed.sqrt());
        let k = graph.mul([graph.div(graph.integer(1), graph.integer(2)), sqrt_c]);
        let k_xi = graph.mul([k, xi]);

        // sech(z) = 1 / cosh(z)
        let cosh_k_xi = graph.function("cosh", [k_xi]);
        let sech_k_xi = graph.div(graph.integer(1), cosh_k_xi);
        let sech_sq = graph.pow(sech_k_xi, graph.integer(2));

        let sol = graph.mul([half_c, sech_sq]);
        Ok(Simplifier::simplify(graph, sol).unwrap_or(sol))
    }

    /// Construct exact traveling shock wave solution for the Viscous Burgers Equation:
    ///
    /// $$u_t + u u_x = \nu u_{xx}$$
    ///
    /// $$u(x, t) = \frac{u_L + u_R}{2} - \frac{u_L - u_R}{2} \tanh\left( \frac{(u_L - u_R)(x - c t)}{4\nu} \right)$$
    pub fn burgers_shock(
        graph: &ExprGraph,
        u_left: f64,
        u_right: f64,
        nu_visc: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s) = graph.symbols.resolve(x) {
            graph.symbol(&s)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(s) = graph.symbols.resolve(t) {
            graph.symbol(&s)
        } else {
            graph.symbol("t")
        };

        let c_speed = (u_left + u_right) / 2.0;
        let c_node = graph.float(c_speed);
        let ct = graph.mul([c_node, var_t]);
        let xi = graph.sub(var_x, ct);

        let delta_u = u_left - u_right;
        let width = 4.0 * nu_visc;
        let k_val = delta_u / width;
        let k_node = graph.float(k_val);
        let k_xi = graph.mul([k_node, xi]);

        let tanh_term = graph.function("tanh", [k_xi]);
        let half_delta = graph.float(delta_u / 2.0);
        let mean_u = graph.float(c_speed);

        let sub_part = graph.mul([half_delta, tanh_term]);
        let sol = graph.sub(mean_u, sub_part);

        Ok(Simplifier::simplify(graph, sol).unwrap_or(sol))
    }

    /// Construct traveling wave invasion front for the Fisher-KPP reaction-diffusion equation:
    ///
    /// $$u_t = D u_{xx} + r u (1 - u)$$
    ///
    /// Propagates with minimum speed $c^* = 2\sqrt{r D}$.
    pub fn fisher_kpp_front(
        graph: &ExprGraph,
        d_diff: f64,
        r_rate: f64,
        x: SymbolId,
        t: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let var_x = if let Some(s) = graph.symbols.resolve(x) {
            graph.symbol(&s)
        } else {
            graph.symbol("x")
        };
        let var_t = if let Some(s) = graph.symbols.resolve(t) {
            graph.symbol(&s)
        } else {
            graph.symbol("t")
        };

        let c_speed = 2.0 * (r_rate * d_diff).sqrt();
        let c_node = graph.float(c_speed);
        let ct = graph.mul([c_node, var_t]);
        let xi = graph.sub(var_x, ct);

        let alpha = (r_rate / (6.0 * d_diff)).sqrt();
        let alpha_node = graph.float(alpha);
        let alpha_xi = graph.mul([alpha_node, xi]);

        // 1 / (1 + exp(alpha * xi))^2
        let exp_term = graph.function("exp", [alpha_xi]);
        let denom_inner = graph.add([graph.integer(1), exp_term]);
        let denom = graph.pow(denom_inner, graph.integer(2));
        let sol = graph.div(graph.integer(1), denom);

        Ok(Simplifier::simplify(graph, sol).unwrap_or(sol))
    }
}
