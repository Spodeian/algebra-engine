//! # `algebra_engine::lifting`
//!
//! Automatic Functorial Domain Lifting & Retraction Clean-Up Engine.
//!
//! Solvers automatically elevate difficult mathematical problems from their base domain $\mathcal{D}_{\text{src}}$
//! into higher or transformed algebraic representations $\mathcal{D}_{\text{lift}}$ (e.g. Euler complex exponentials,
//! Weierstrass rational fields, Laplace operational frequency domain, Clifford rotors, Nilpotent duals),
//! execute the required operations algebraically, and project/retract the results back with full artifact clean-up.

pub mod cleaner;
pub mod router;

pub use cleaner::RetractionCleaner;
pub use router::{
    DomainCompatibilityChecker, DomainLiftRouter, EulerLiftingFunctor, LiftingFunctor,
    WeierstrassLiftingFunctor,
};
