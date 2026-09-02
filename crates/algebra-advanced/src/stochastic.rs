//! # `algebra-stochastic`
//!
//! Stochastic calculus, Wiener processes ($dW_t$), Stochastic Differential Equations (SDEs),
//! and **Itô's Lemma**.

use algebra_core::{AlgebraResult, ExprGraph, ExprId, SymbolId};
use algebra_engine::calculus::SymbolicCalculus;

/// Stochastic Differential Equation $dX_t = \mu(t, X_t) dt + \sigma(t, X_t) dW_t$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItoProcess {
    pub x: SymbolId,
    pub t: SymbolId,
    pub drift_mu: ExprId,
    pub diffusion_sigma: ExprId,
}

impl ItoProcess {
    pub fn new(x: SymbolId, t: SymbolId, drift_mu: ExprId, diffusion_sigma: ExprId) -> Self {
        Self {
            x,
            t,
            drift_mu,
            diffusion_sigma,
        }
    }

    /// Apply **Itô's Lemma** to compute stochastic differential $df(t, X_t) = A dt + B dW_t$.
    ///
    /// Returns tuple `(dt_coeff, dW_coeff)` where:
    /// $$A = \frac{\partial f}{\partial t} + \mu \frac{\partial f}{\partial x} + \frac{1}{2}\sigma^2 \frac{\partial^2 f}{\partial x^2}$$
    /// $$B = \sigma \frac{\partial f}{\partial x}$$
    pub fn apply_ito_lemma(
        &self,
        graph: &ExprGraph,
        f_expr: ExprId,
    ) -> AlgebraResult<(ExprId, ExprId)> {
        let df_dt = graph.diff(f_expr, self.t);
        let df_dx = graph.diff(f_expr, self.x);
        let d2f_dx2 = graph.diff(df_dx, self.x);

        // Term 1: df/dt
        // Term 2: mu * df/dx
        let mu_df_dx = graph.mul([self.drift_mu, df_dx]);

        // Term 3: 0.5 * sigma^2 * d2f/dx2
        let half = graph.div(graph.integer(1), graph.integer(2));
        let sigma_sq = graph.pow(self.diffusion_sigma, graph.integer(2));
        let half_sigma_sq = graph.mul([half, sigma_sq]);
        let term3 = graph.mul([half_sigma_sq, d2f_dx2]);

        // A = df/dt + mu * df/dx + 0.5 * sigma^2 * d2f/dx2
        let dt_coeff = graph.add([df_dt, mu_df_dx, term3]);

        // B = sigma * df/dx
        let dw_coeff = graph.mul([self.diffusion_sigma, df_dx]);

        Ok((dt_coeff, dw_coeff))
    }

    /// Convert Itô drift $\mu_I$ to Stratonovich drift $\mu_S$:
    /// $$\mu_S = \mu_I - \frac{1}{2} \sigma(x) \frac{\partial \sigma(x)}{\partial x}$$
    pub fn ito_to_stratonovich(&self, graph: &ExprGraph) -> AlgebraResult<ExprId> {
        let dsigma_dx = graph.diff(self.diffusion_sigma, self.x);
        let half = graph.div(graph.integer(1), graph.integer(2));
        let correction = graph.mul([half, self.diffusion_sigma, dsigma_dx]);
        let neg_correction = graph.mul([graph.integer(-1), correction]);
        Ok(graph.add([self.drift_mu, neg_correction]))
    }

    /// Convert Stratonovich drift $\mu_S$ to Itô drift $\mu_I$:
    /// $$\mu_I = \mu_S + \frac{1}{2} \sigma(x) \frac{\partial \sigma(x)}{\partial x}$$
    pub fn stratonovich_to_ito(&self, graph: &ExprGraph) -> AlgebraResult<ExprId> {
        let dsigma_dx = graph.diff(self.diffusion_sigma, self.x);
        let half = graph.div(graph.integer(1), graph.integer(2));
        let correction = graph.mul([half, self.diffusion_sigma, dsigma_dx]);
        Ok(graph.add([self.drift_mu, correction]))
    }
}

/// Representation of a standard Wiener Process $W_t$ with Brownian increments $dW_t$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WienerProcess {
    pub symbol: SymbolId,
    pub time_symbol: SymbolId,
}

impl WienerProcess {
    pub fn new(symbol: SymbolId, time_symbol: SymbolId) -> Self {
        Self {
            symbol,
            time_symbol,
        }
    }
}
