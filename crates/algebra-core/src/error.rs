//! Error types for algebraic operations and domain checks.

use thiserror::Error;

/// Core error enum representing failure modes across algebraic operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AlgebraError {
    #[error("Domain mismatch: expected {expected}, found {found}")]
    DomainMismatch { expected: String, found: String },

    #[error("Operation '{operation}' is undefined for domain '{domain}'")]
    UndefinedOperation {
        operation: &'static str,
        domain: String,
    },

    #[error("Division by zero in domain '{domain}'")]
    DivisionByZero { domain: String },

    #[error("Invalid element for domain '{domain}': {reason}")]
    InvalidElement { domain: String, reason: String },

    #[error("Dimension mismatch: {0}")]
    DimensionMismatch(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),

    #[error("Domain violation in domain '{domain}': {reason}")]
    DomainViolation { domain: String, reason: String },

    #[error("Singularity or branch cut encountered: {0}")]
    SingularityEncountered(String),

    #[error("Algorithm non-convergence: {0}")]
    NonConvergence(String),

    #[error("Unsolvable system of equations: {0}")]
    UnsolvableSystem(String),

    #[error("Parsing error: {0}")]
    ParseError(String),

    #[error("Evaluation failed: {0}")]
    EvaluationError(String),
}

/// Result type alias for algebraic operations.
pub type AlgebraResult<T> = Result<T, AlgebraError>;
