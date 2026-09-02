//! Expression node definitions and structural AST variants.

use crate::domain::Domain;
use crate::id::{ExprId, SymbolId};
use crate::number::Number;
use smallvec::SmallVec;
use std::fmt;

/// Relational operators for comparisons and equations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RelOp {
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

impl fmt::Display for RelOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::LessThan => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
        }
    }
}

/// ZF Set operation variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SetOp {
    Union,
    Intersection,
    Difference,
    SymmetricDifference,
    CartesianProduct,
    In,
    Subset,
}

/// Structural kinds of expression AST nodes in the DAG.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExprKind {
    /// Scalar numbers or mathematical constants.
    Number(Number),

    /// Variable symbol.
    Symbol(SymbolId),

    /// Polyadic addition: term_1 + term_2 + ... + term_n
    Add(SmallVec<[ExprId; 4]>),

    /// Polyadic multiplication: factor_1 * factor_2 * ... * factor_n
    Mul(SmallVec<[ExprId; 4]>),

    /// Exponentiation: base ^ exponent
    Pow(ExprId, ExprId),

    /// Division / Fraction: numerator / denominator
    Div(ExprId, ExprId),

    /// Subtraction: lhs - rhs
    Sub(ExprId, ExprId),

    /// Unary negation: -expr
    Neg(ExprId),

    /// Symbolic function call: f(x_1, x_2, ..., x_n)
    Function {
        name: SymbolId,
        args: SmallVec<[ExprId; 2]>,
    },

    /// Partial / Total derivative: d^order(expr) / d(wrt)^order
    Derivative {
        expr: ExprId,
        wrt: SymbolId,
        order: u32,
    },

    /// Indefinite or Definite Integral: ∫ expr d(wrt)
    Integral {
        expr: ExprId,
        wrt: SymbolId,
        lower: Option<ExprId>,
        upper: Option<ExprId>,
    },

    /// Matrix structure literal node with explicit dimensions.
    Matrix {
        rows: usize,
        cols: usize,
        elements: Vec<ExprId>,
    },

    /// Relational expression (e.g. x == y, a < b).
    Relational { op: RelOp, lhs: ExprId, rhs: ExprId },

    /// ZF Set theory operation (e.g. A ∪ B, x ∈ S).
    SetOperation {
        op: SetOp,
        args: SmallVec<[ExprId; 2]>,
    },

    /// Repeated summation: \sum_{var=lower}^{upper} body
    Sum {
        body: ExprId,
        var: SymbolId,
        lower: Option<ExprId>,
        upper: Option<ExprId>,
    },

    /// Repeated product: \prod_{var=lower}^{upper} body
    Product {
        body: ExprId,
        var: SymbolId,
        lower: Option<ExprId>,
        upper: Option<ExprId>,
    },

    /// Tensor contraction / Einstein summation over dummy indices
    TensorContraction {
        tensor_a: ExprId,
        tensor_b: ExprId,
        contracted_indices: Vec<(SymbolId, SymbolId)>,
    },
}

/// Primary node in the expression DAG, combining domain typing with structural node payload.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExprNode {
    /// Mathematical domain to which this expression belongs.
    pub domain: Domain,
    /// Structural AST payload variant.
    pub kind: ExprKind,
}

impl ExprNode {
    /// Create a new expression node with specified domain and kind.
    #[inline]
    pub fn new(domain: Domain, kind: ExprKind) -> Self {
        Self { domain, kind }
    }

    /// Returns direct child [`ExprId`] handles referenced by this node.
    pub fn children(&self) -> SmallVec<[ExprId; 4]> {
        match &self.kind {
            ExprKind::Number(_) | ExprKind::Symbol(_) => SmallVec::new(),
            ExprKind::Add(terms) => terms.clone(),
            ExprKind::Mul(factors) => factors.clone(),
            ExprKind::Pow(base, exp) => smallvec::smallvec![*base, *exp],
            ExprKind::Div(num, den) => smallvec::smallvec![*num, *den],
            ExprKind::Sub(lhs, rhs) => smallvec::smallvec![*lhs, *rhs],
            ExprKind::Neg(inner) => smallvec::smallvec![*inner],
            ExprKind::Function { args, .. } => SmallVec::from_slice(args),
            ExprKind::Derivative { expr, .. } => smallvec::smallvec![*expr],
            ExprKind::Integral {
                expr, lower, upper, ..
            } => {
                let mut v = smallvec::smallvec![*expr];
                if let Some(l) = lower {
                    v.push(*l);
                }
                if let Some(u) = upper {
                    v.push(*u);
                }
                v
            }
            ExprKind::Matrix { elements, .. } => SmallVec::from_slice(elements),
            ExprKind::Relational { lhs, rhs, .. } => smallvec::smallvec![*lhs, *rhs],
            ExprKind::SetOperation { args, .. } => SmallVec::from_slice(args),
            ExprKind::Sum {
                body, lower, upper, ..
            }
            | ExprKind::Product {
                body, lower, upper, ..
            } => {
                let mut v = smallvec::smallvec![*body];
                if let Some(l) = lower {
                    v.push(*l);
                }
                if let Some(u) = upper {
                    v.push(*u);
                }
                v
            }
            ExprKind::TensorContraction {
                tensor_a, tensor_b, ..
            } => smallvec::smallvec![*tensor_a, *tensor_b],
        }
    }

    /// Returns `true` if this is a leaf node (contains no child expressions).
    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, ExprKind::Number(_) | ExprKind::Symbol(_))
    }
}
