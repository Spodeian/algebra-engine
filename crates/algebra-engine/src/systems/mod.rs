//! # `algebra_engine::systems`
//!
//! Multi-Variate Systems of Equations Computational Subsystem.
//!
//! Encompasses:
//! - **Linear Systems (`LinearSystemSolver`)**: Fraction-free Bareiss elimination, LU/Cholesky/QR, affine nullspace decomposition.
//! - **Polynomial Systems (`PolynomialSystemSolver`)**: Non-linear multi-variate varieties via Gröbner elimination and radical root solving.
//! - **Non-Linear Numerical Systems (`NumericalSystemSolver`)**: Multi-variable Newton-Raphson & Levenberg-Marquardt with symbolic Jacobian derivation.
//! - **Coupled Differential Systems (`DifferentialSystemSolver`)**: Multi-dimensional linear ODE systems via matrix exponential & order reduction.
//! - **Integer Diophantine Systems (`DiophantineSystemSolver`)**: Smith Normal Form (SNF) exact integer lattice solutions.

pub mod differential;
pub mod diophantine;
pub mod linear;
pub mod numerical;
pub mod polynomial;

pub use differential::DifferentialSystemSolver;
pub use diophantine::{DiophantineSolution, DiophantineSystemSolver, SmithNormalForm};
pub use linear::{LinearSolutionSpace, LinearSystemSolver};
pub use numerical::{NumericalSystemConfig, NumericalSystemSolver};
pub use polynomial::PolynomialSystemSolver;
