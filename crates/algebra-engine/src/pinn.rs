//! # `algebra_engine::pinn`
//!
//! Physics-Informed Neural Network (PINN) PDE Residual Evaluator.
//!
//! Evaluates exact analytical and hyper-dual PDE differential residuals:
//! - **1D Viscous Burgers Equation**: $\mathcal{R}(x, t) = \partial_t u + u \partial_x u - \nu \partial_{xx} u$
//! - **2D Heat Diffusion Equation**: $\mathcal{R}(x, y, t) = \partial_t u - \alpha (\partial_{xx} u + \partial_{yy} u)$
//! - **2D Poisson Equation**: $\mathcal{R}(x, y) = \partial_{xx} u + \partial_{yy} u - f(x, y)$

use crate::autodiff::HyperDual;

/// Physics-Informed Neural Network PDE Residual Loss Evaluator.
pub struct PinnResidual;

impl PinnResidual {
    /// Evaluate 1D Viscous Burgers PDE residual: $\partial_t u + u \partial_x u - \nu \partial_{xx} u$
    pub fn burgers_residual<F>(u_fn: F, x: f64, t: f64, nu: f64) -> f64
    where
        F: Fn(HyperDual<f64>, HyperDual<f64>) -> HyperDual<f64>,
    {
        // Compute u, u_x, u_xx using HyperDual on x
        let hd_x = HyperDual::var_both(x);
        let hd_t_const = HyperDual::constant(t);
        let res_x = u_fn(hd_x, hd_t_const);
        let u_val = res_x.val;
        let u_x = res_x.eps1;
        let u_xx = res_x.eps12;

        // Compute u_t using HyperDual on t
        let hd_x_const = HyperDual::constant(x);
        let hd_t = HyperDual::var_both(t);
        let res_t = u_fn(hd_x_const, hd_t);
        let u_t = res_t.eps1;

        // Residual: u_t + u * u_x - nu * u_xx
        u_t + u_val * u_x - nu * u_xx
    }

    /// Evaluate 2D Heat Diffusion PDE residual: $\partial_t u - \alpha (\partial_{xx} u + \partial_{yy} u)$
    pub fn heat_2d_residual<F>(u_fn: F, x: f64, y: f64, t: f64, alpha: f64) -> f64
    where
        F: Fn(HyperDual<f64>, HyperDual<f64>, HyperDual<f64>) -> HyperDual<f64>,
    {
        // u_xx
        let res_x = u_fn(
            HyperDual::var_both(x),
            HyperDual::constant(y),
            HyperDual::constant(t),
        );
        let u_xx = res_x.eps12;

        // u_yy
        let res_y = u_fn(
            HyperDual::constant(x),
            HyperDual::var_both(y),
            HyperDual::constant(t),
        );
        let u_yy = res_y.eps12;

        // u_t
        let res_t = u_fn(
            HyperDual::constant(x),
            HyperDual::constant(y),
            HyperDual::var_both(t),
        );
        let u_t = res_t.eps1;

        u_t - alpha * (u_xx + u_yy)
    }

    /// Evaluate 2D Poisson PDE residual: $\partial_{xx} u + \partial_{yy} u - f(x, y)$
    pub fn poisson_2d_residual<F>(u_fn: F, x: f64, y: f64, source_term: f64) -> f64
    where
        F: Fn(HyperDual<f64>, HyperDual<f64>) -> HyperDual<f64>,
    {
        let res_x = u_fn(HyperDual::var_both(x), HyperDual::constant(y));
        let u_xx = res_x.eps12;

        let res_y = u_fn(HyperDual::constant(x), HyperDual::var_both(y));
        let u_yy = res_y.eps12;

        (u_xx + u_yy) - source_term
    }

    /// Compute Mean Squared Error (MSE) loss over a set of collocation points.
    pub fn mean_squared_loss(residuals: &[f64]) -> f64 {
        if residuals.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = residuals.iter().map(|&r| r * r).sum();
        sum_sq / (residuals.len() as f64)
    }
}
