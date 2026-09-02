//! # `algebra_engine::ode`
//!
//! Ordinary Differential Equation (ODE) computational subsystem for URAE.
//!
//! Encompasses:
//! - **Symbolic Solvers**: Exact 1st-order linear, separable, Bernoulli, exact, 2nd-order constant coefficients, Cauchy-Euler, systems of linear ODEs.
//! - **Numerical Solvers**: Adaptive DOPRI5, Tsitouras 5(4), Radau IIA (stiff), Symplectic Velocity Verlet, custom Butcher Tableaus.

pub mod numerical;
pub mod symbolic;

pub use numerical::{
    ButcherTableau, NumericalOdeConfig, NumericalOdeMethod, NumericalOdeSolver, OdeTrajectory,
};
pub use symbolic::{OdeType, SymbolicOdeSolver};
