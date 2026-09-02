//! Deterministic Step-by-Step Educational Derivation Structures.

use crate::id::ExprId;

/// Representation of a single deterministic mathematical transformation step.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MathStep {
    pub step_number: usize,
    pub rule_name: String,
    pub description: String,
    pub before_expr: ExprId,
    pub after_expr: ExprId,
}

impl MathStep {
    pub fn new(
        step_number: usize,
        rule_name: impl Into<String>,
        description: impl Into<String>,
        before_expr: ExprId,
        after_expr: ExprId,
    ) -> Self {
        Self {
            step_number,
            rule_name: rule_name.into(),
            description: description.into(),
            before_expr,
            after_expr,
        }
    }
}

/// Result wrapper carrying both the final mathematical result and deterministic educational steps.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StepByStepResult<T> {
    pub result: T,
    pub steps: Vec<MathStep>,
}

impl<T> StepByStepResult<T> {
    pub fn new(result: T, steps: Vec<MathStep>) -> Self {
        Self { result, steps }
    }

    pub fn fast(result: T) -> Self {
        Self {
            result,
            steps: Vec::new(),
        }
    }
}
