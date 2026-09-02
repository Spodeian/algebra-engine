//! # `algebra-sparse`
//!
//! Sparse mathematical data structures (Compressed Sparse Row CSR matrices,
//! sparse multivariate polynomials) and multi-tier concurrent memoization caches.

use ahash::AHashMap;
use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId, SymbolId};
use dashmap::DashMap;
use std::sync::Arc;

/// Compressed Sparse Row (CSR) matrix representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrMatrix {
    pub rows: usize,
    pub cols: usize,
    /// Non-zero element values.
    pub values: Vec<ExprId>,
    /// Column indices corresponding to each element in `values`.
    pub col_indices: Vec<usize>,
    /// Row offsets array of size `rows + 1`.
    pub row_offsets: Vec<usize>,
}

impl CsrMatrix {
    /// Create a new CSR matrix from raw vectors.
    pub fn new(
        rows: usize,
        cols: usize,
        values: Vec<ExprId>,
        col_indices: Vec<usize>,
        row_offsets: Vec<usize>,
    ) -> AlgebraResult<Self> {
        if row_offsets.len() != rows + 1 {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Expected row_offsets length {}, got {}",
                rows + 1,
                row_offsets.len()
            )));
        }
        Ok(Self {
            rows,
            cols,
            values,
            col_indices,
            row_offsets,
        })
    }

    /// Compute matrix-vector product $y = A \cdot x$.
    pub fn multiply_vec(&self, graph: &ExprGraph, x: &[ExprId]) -> AlgebraResult<Vec<ExprId>> {
        if x.len() != self.cols {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Vector dimension {} mismatch for CSR matrix cols {}",
                x.len(),
                self.cols
            )));
        }

        let mut y = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            let start = self.row_offsets[r];
            let end = self.row_offsets[r + 1];
            let mut terms = Vec::new();
            for i in start..end {
                let val = self.values[i];
                let col = self.col_indices[i];
                let prod = graph.mul([val, x[col]]);
                terms.push(prod);
            }
            y.push(graph.add(terms));
        }

        Ok(y)
    }

    /// Construct a `CsrMatrix` from a 2D grid of `ExprId`s.
    pub fn from_dense_grid(graph: &ExprGraph, grid: &[Vec<ExprId>]) -> Self {
        let rows = grid.len();
        let cols = if rows > 0 { grid[0].len() } else { 0 };
        let mut values = Vec::new();
        let mut col_indices = Vec::new();
        let mut row_offsets = Vec::with_capacity(rows + 1);
        row_offsets.push(0);

        let zero_id = graph.integer(0);

        for row in grid {
            for (c, &val) in row.iter().enumerate() {
                if val != zero_id {
                    values.push(val);
                    col_indices.push(c);
                }
            }
            row_offsets.push(values.len());
        }

        Self {
            rows,
            cols,
            values,
            col_indices,
            row_offsets,
        }
    }

    /// Determine if a dense matrix grid exceeds the sparsity threshold (e.g. >= 70% zero entries).
    pub fn should_use_csr(graph: &ExprGraph, grid: &[Vec<ExprId>], threshold: f64) -> bool {
        let total = grid.len() * if !grid.is_empty() { grid[0].len() } else { 0 };
        if total == 0 {
            return false;
        }
        let zero_id = graph.integer(0);
        let zero_count = grid
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&val| val == zero_id)
            .count();
        (zero_count as f64 / total as f64) >= threshold
    }
}

/// Sparse Multivariate Polynomial mapping term exponent vectors to coefficient `ExprId` handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparsePoly {
    pub vars: Vec<SymbolId>,
    /// Term exponents -> coefficient `ExprId` map.
    pub terms: AHashMap<Vec<u32>, ExprId>,
}

impl SparsePoly {
    pub fn new(vars: Vec<SymbolId>, terms: AHashMap<Vec<u32>, ExprId>) -> Self {
        Self { vars, terms }
    }

    /// Convert sparse polynomial back to `ExprGraph` AST node.
    pub fn to_expr(&self, graph: &ExprGraph) -> ExprId {
        let mut term_exprs = Vec::new();
        for (exponents, &coeff) in &self.terms {
            let mut factors = vec![coeff];
            for (&var_sym, &exp) in self.vars.iter().zip(exponents) {
                if exp > 0 {
                    let var_name = graph.symbols.resolve(var_sym).unwrap_or_default();
                    let var_node = graph.symbol(&var_name);
                    let exp_node = graph.integer(exp as i64);
                    let pow_node = graph.pow(var_node, exp_node);
                    factors.push(pow_node);
                }
            }
            term_exprs.push(graph.mul(factors));
        }
        graph.add(term_exprs)
    }
}

/// Lock-free concurrent memoization cache for substitution subproblems.
#[derive(Debug, Clone, Default)]
pub struct MemoCache {
    cache: Arc<DashMap<(ExprId, SymbolId, ExprId), ExprId>>,
}

impl MemoCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Retrieve or compute memoized substitution.
    pub fn get_or_insert_with<F>(
        &self,
        expr: ExprId,
        wrt: SymbolId,
        rep: ExprId,
        compute: F,
    ) -> ExprId
    where
        F: FnOnce() -> ExprId,
    {
        let key = (expr, wrt, rep);
        if let Some(val) = self.cache.get(&key) {
            return *val;
        }
        let res = compute();
        self.cache.insert(key, res);
        res
    }
}
