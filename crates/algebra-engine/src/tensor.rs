//! # `algebra-tensor`
//!
//! Tensor calculus, Einstein summation, covariant/contravariant index tracking,
//! exterior calculus (wedge products, exterior derivatives), and Riemannian geometry.

use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId, SymbolId};

/// Tensor index type: Covariant (lower index) or Contravariant (upper index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TensorIndexKind {
    /// Lower index $T_\mu$
    Covariant,
    /// Upper index $T^\mu$
    Contravariant,
}

/// A tensor index representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TensorIndex {
    pub symbol: SymbolId,
    pub kind: TensorIndexKind,
}

/// Symbolic Tensor with rank, index signature, and components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicTensor {
    pub name: SymbolId,
    pub indices: Vec<TensorIndex>,
    pub components: Vec<ExprId>,
}

impl SymbolicTensor {
    /// Construct a symbolic tensor with indices.
    pub fn new(name: SymbolId, indices: Vec<TensorIndex>, components: Vec<ExprId>) -> Self {
        Self {
            name,
            indices,
            components,
        }
    }

    /// Compute contraction over dummy indices (Einstein summation).
    pub fn contract(
        &self,
        graph: &ExprGraph,
        other: &SymbolicTensor,
    ) -> AlgebraResult<SymbolicTensor> {
        // Find matching dummy index pair (one Covariant, one Contravariant)
        let mut dummy = None;
        for idx1 in &self.indices {
            for idx2 in &other.indices {
                if idx1.symbol == idx2.symbol && idx1.kind != idx2.kind {
                    dummy = Some((idx1.symbol, idx1.kind));
                    break;
                }
            }
        }

        if let Some((dummy_sym, _)) = dummy {
            let new_indices: Vec<TensorIndex> = self
                .indices
                .iter()
                .filter(|i| i.symbol != dummy_sym)
                .cloned()
                .chain(
                    other
                        .indices
                        .iter()
                        .filter(|i| i.symbol != dummy_sym)
                        .cloned(),
                )
                .collect();

            let contract_name = graph.symbols.get_or_intern(format!(
                "Contraction({},{})",
                graph.symbols.resolve(self.name).unwrap_or_default(),
                graph.symbols.resolve(other.name).unwrap_or_default()
            ));

            Ok(SymbolicTensor::new(contract_name, new_indices, vec![]))
        } else {
            Err(AlgebraError::EvaluationError(
                "No contraction dummy indices found".into(),
            ))
        }
    }

    /// Compute exterior derivative $\mathrm{d}\omega$ for differential form $\omega$.
    pub fn exterior_derivative(&self, graph: &ExprGraph) -> SymbolicTensor {
        let d_name = graph.symbols.get_or_intern(format!(
            "d({})",
            graph.symbols.resolve(self.name).unwrap_or_default()
        ));
        let mut d_indices = self.indices.clone();
        let mu = graph.symbols.get_or_intern("mu");
        d_indices.push(TensorIndex {
            symbol: mu,
            kind: TensorIndexKind::Covariant,
        });

        SymbolicTensor::new(d_name, d_indices, vec![])
    }
}

/// Conversion bridge between repeated summation and tensor contraction.
pub struct TensorSumBridge;

impl TensorSumBridge {
    /// Convert repeated summation \sum_{j} A_{ij} * B_{jk} into TensorContraction(A, B).
    pub fn sum_to_tensor_contraction(graph: &ExprGraph, sum_expr: ExprId) -> ExprId {
        let node = graph.get(sum_expr);
        if let algebra_core::ExprKind::Sum { body, var, .. } = node.kind {
            let body_node = graph.get(body);
            if let algebra_core::ExprKind::Mul(factors) = &body_node.kind
                && factors.len() == 2
            {
                return graph.tensor_contraction(factors[0], factors[1], vec![(var, var)]);
            }
        }
        sum_expr
    }

    /// Convert TensorContraction(A, B) into repeated summation \sum_{j=1}^{N} A_{ij} * B_{jk}.
    pub fn tensor_contraction_to_sum(
        graph: &ExprGraph,
        contraction_expr: ExprId,
        lower: ExprId,
        upper: ExprId,
    ) -> ExprId {
        let node = graph.get(contraction_expr);
        if let algebra_core::ExprKind::TensorContraction {
            tensor_a,
            tensor_b,
            contracted_indices,
        } = &node.kind
        {
            let dummy_sym = contracted_indices
                .first()
                .map(|(s, _)| *s)
                .unwrap_or_else(|| graph.symbols.get_or_intern("j"));
            let mul_body = graph.mul([*tensor_a, *tensor_b]);
            return graph.sum(mul_body, dummy_sym, Some(lower), Some(upper));
        }
        contraction_expr
    }
}
