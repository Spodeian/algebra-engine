//! # `algebra-advanced`
//!
//! Advanced, structural, and applied extensions for the Universal Rust Algebra Engine (URAE).
//!
//! Includes formal verification & theorem proving (Lean 4 / Coq export),
//! category morphisms & structural equivalence classification, quantum mechanics & bra-kets,
//! Galois theory, statistical mechanics, cryptography over finite fields, stochastic Itô calculus,
//! GPU acceleration, code generation, multi-provider AI copilot, Conway Surreal numbers $\mathbf{No}$,
//! $p$-adic fields $\mathbb{Q}_p/\mathbb{Z}_p$, Adele rings $\mathbb{A}_K$, Idele groups $\mathbb{I}_K$,
//! Painlevé transcendents I–VI, Heun differential equations, Braid groups $B_n$, and Jones knot polynomials.

pub mod adele;
#[cfg(feature = "agent")]
pub mod agent;
pub mod clifford;
pub mod codegen;
pub mod crypto;
pub mod finite;
pub mod galois;
pub mod gpu;
pub mod knot;
pub mod morphism;
pub mod padic;
pub mod proof;
pub mod quantum;
pub mod statmech;
pub mod stochastic;
pub mod structure_tensor;
pub mod surreal;
pub mod transcendents;

pub use adele::{Adele, ArtinProduct, Idele, Place};
#[cfg(feature = "agent")]
pub use agent::{
    AgentToolDefinition, AiConfig, AiProvider, AlgebraAgentInterface, ExplanationGenerator,
};
pub use clifford::{BladeMask, CliffordMultivector, CliffordSignature, FreeTensor};
pub use codegen::{CodeGenerator, TargetLanguage};
pub use crypto::{EllipticCurve, EllipticPoint, InformationTheory, ReedSolomonCode};
pub use finite::{GaloisField, ModuloInt};
pub use galois::{FieldExtension, GaloisGroupType};
pub use gpu::{GpuComputeKernel, SimdPolyVector};
pub use knot::{BraidGenerator, BraidWord, KnotInvariants, LaurentPoly};
pub use morphism::{
    DomainMorphism, EquivalenceClassifier, EquivalenceKind, Functor, MonoidalCategory,
};
pub use padic::{HenselLifter, PadicNumber};
pub use proof::{ProofStep, ProofTrace, UnificationEngine};
pub use quantum::{BraKet, DysonSeries, GaugeTheory, QuantumOperator};
pub use statmech::CanonicalPartitionFunction;
pub use stochastic::{ItoProcess, WienerProcess};
pub use structure_tensor::StructureTensorAlgebra;
pub use surreal::SurrealNo;
pub use transcendents::{HeunEquation, PainleveEquation, PainleveKind};
