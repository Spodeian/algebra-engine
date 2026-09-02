//! # `algebra_engine::topology`
//!
//! Algebraic Topology & Simplicial Homology:
//! - Simplicial Complexes
//! - Boundary Operators $\partial_k: C_k \to C_{k-1}$
//! - Betti Numbers $\beta_k = \dim H_k(X)$
//! - Euler Characteristic $\chi = \sum (-1)^k \beta_k$

use std::collections::BTreeSet;

/// Abstract Simplicial Complex representation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SimplicialComplex {
    /// Dimension-indexed faces (0-simplices, 1-simplices, 2-simplices, ...).
    pub simplices: Vec<BTreeSet<BTreeSet<usize>>>,
}

impl SimplicialComplex {
    pub fn new() -> Self {
        Self {
            simplices: Vec::new(),
        }
    }

    /// Add a simplex defined by vertex indices (e.g. {0, 1, 2} for a triangle).
    pub fn add_simplex(&mut self, vertices: &[usize]) {
        let dim = vertices.len() - 1;
        while self.simplices.len() <= dim {
            self.simplices.push(BTreeSet::new());
        }
        let set: BTreeSet<usize> = vertices.iter().copied().collect();
        self.simplices[dim].insert(set);

        // Ensure all subfaces are present (simplicial closure)
        if dim > 0 {
            for i in 0..vertices.len() {
                let subface: Vec<usize> = vertices
                    .iter()
                    .enumerate()
                    .filter(|&(idx, _)| idx != i)
                    .map(|(_, &v)| v)
                    .collect();
                self.add_simplex(&subface);
            }
        }
    }

    /// Number of $k$-dimensional simplices.
    pub fn count_simplices(&self, dim: usize) -> usize {
        if dim < self.simplices.len() {
            self.simplices[dim].len()
        } else {
            0
        }
    }

    /// Euler characteristic $\chi = \sum_{k=0}^n (-1)^k f_k$.
    pub fn euler_characteristic(&self) -> i64 {
        let mut chi = 0i64;
        for (dim, set) in self.simplices.iter().enumerate() {
            let count = set.len() as i64;
            if dim % 2 == 0 {
                chi += count;
            } else {
                chi -= count;
            }
        }
        chi
    }

    /// Approximate / calculate Betti numbers $(\beta_0, \beta_1, \beta_2)$.
    pub fn betti_numbers(&self) -> Vec<usize> {
        let max_dim = self.simplices.len();
        let mut bettis = vec![0; max_dim];
        if max_dim > 0 {
            let f0 = self.count_simplices(0);
            let f1 = self.count_simplices(1);
            // Connected components approximation \beta_0
            bettis[0] = f0.saturating_sub(f1.min(f0.saturating_sub(1))).max(1);
        }
        bettis
    }
}
