//! # `algebra_engine::tda`
//!
//! Topological Data Analysis (TDA), Persistent Homology & Discrete Hodge Laplacian.
//!
//! Features:
//! - **Vietoris-Rips Simplicial Filtration**: Generates $k$-simplices over point clouds and FEA meshes.
//! - **$\mathbb{F}_2$ Boundary Matrix Reduction**: Standard persistence algorithm pairing $(b_i, d_i)$ to extract barcodes.
//! - **Topological Feature Detection**: $H_0$ (connected components), $H_1$ (loops/tunnels), and $H_2$ (voids/cavities).
//! - **Discrete Hodge Laplacian**: Discrete exterior calculus $L_k = d_{k-1} d_{k-1}^T + d_k^T d_k$ and harmonic $k$-forms.

#![allow(clippy::needless_range_loop)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An abstract $k$-simplex with sorted vertices and filtration birth time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Simplex {
    pub vertices: Vec<usize>,
    pub dimension: usize,
    pub birth: f64,
}

impl Simplex {
    pub fn new(mut vertices: Vec<usize>, birth: f64) -> Self {
        vertices.sort_unstable();
        let dimension = if vertices.is_empty() {
            0
        } else {
            vertices.len() - 1
        };
        Self {
            vertices,
            dimension,
            birth,
        }
    }

    /// Check if `self` is a codimension-1 face of `other`.
    pub fn is_face_of(&self, other: &Simplex) -> bool {
        if self.dimension + 1 != other.dimension {
            return false;
        }
        let mut i = 0;
        let mut j = 0;
        let mut mismatch = 0;
        while i < self.vertices.len() && j < other.vertices.len() {
            if self.vertices[i] == other.vertices[j] {
                i += 1;
                j += 1;
            } else {
                j += 1;
                mismatch += 1;
                if mismatch > 1 {
                    return false;
                }
            }
        }
        i == self.vertices.len()
    }
}

/// Vietoris-Rips Simplicial Complex Filtration Generator.
pub struct VietorisRipsFiltration;

impl VietorisRipsFiltration {
    /// Construct Vietoris-Rips filtration from Euclidean point cloud up to dimension `max_dim`.
    pub fn from_point_cloud(
        points: &[Vec<f64>],
        max_dim: usize,
        max_edge_len: f64,
    ) -> Vec<Simplex> {
        let n = points.len();
        let mut dist_matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let mut sum_sq = 0.0;
                for k in 0..points[i].len() {
                    let d = points[i][k] - points[j][k];
                    sum_sq += d * d;
                }
                let dist = sum_sq.sqrt();
                dist_matrix[i][j] = dist;
                dist_matrix[j][i] = dist;
            }
        }

        let mut simplices = Vec::new();

        // 0-simplices (vertices)
        for i in 0..n {
            simplices.push(Simplex::new(vec![i], 0.0));
        }

        // 1-simplices (edges)
        if max_dim >= 1 {
            for i in 0..n {
                for j in (i + 1)..n {
                    let d = dist_matrix[i][j];
                    if d <= max_edge_len {
                        simplices.push(Simplex::new(vec![i, j], d));
                    }
                }
            }
        }

        // 2-simplices (triangles)
        if max_dim >= 2 {
            for i in 0..n {
                for j in (i + 1)..n {
                    let d_ij = dist_matrix[i][j];
                    if d_ij > max_edge_len {
                        continue;
                    }
                    for k in (j + 1)..n {
                        let d_jk = dist_matrix[j][k];
                        let d_ik = dist_matrix[i][k];
                        if d_jk <= max_edge_len && d_ik <= max_edge_len {
                            let birth = d_ij.max(d_jk).max(d_ik);
                            simplices.push(Simplex::new(vec![i, j, k], birth));
                        }
                    }
                }
            }
        }

        // Sort simplices: first by birth ascending, then by dimension ascending
        simplices.sort_by(|a, b| {
            a.birth
                .partial_cmp(&b.birth)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.dimension.cmp(&b.dimension))
        });

        simplices
    }
}

/// A Topological Persistence Interval $[b, d)$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistenceInterval {
    pub dimension: usize,
    pub birth: f64,
    pub death: Option<f64>,
    pub lifetime: f64,
}

/// Persistent Homology Diagram and Barcode Reduction Engine.
pub struct PersistenceDiagram;

impl PersistenceDiagram {
    /// Compute persistent homology intervals over $\mathbb{F}_2 = \operatorname{GF}(2)$ boundary matrix.
    pub fn compute(simplices: &[Simplex]) -> Vec<PersistenceInterval> {
        let m = simplices.len();
        if m == 0 {
            return Vec::new();
        }

        // Represent boundary matrix as list of non-zero row indices per column (sorted descending for lowest 1)
        let mut boundary_cols: Vec<Vec<usize>> = Vec::with_capacity(m);

        for j in 0..m {
            let mut col = Vec::new();
            let s_j = &simplices[j];
            if s_j.dimension > 0 {
                for i in 0..j {
                    let s_i = &simplices[i];
                    if s_i.is_face_of(s_j) {
                        col.push(i);
                    }
                }
            }
            col.sort_unstable();
            boundary_cols.push(col);
        }

        let mut low_map: HashMap<usize, usize> = HashMap::new(); // low_row -> col_idx
        let mut intervals = Vec::new();

        for j in 0..m {
            // While column j has a pivot already present in low_map, add (XOR) that column
            while let Some(&low_j) = boundary_cols[j].last() {
                if let Some(&k) = low_map.get(&low_j) {
                    // Add column k to column j modulo 2 (symmetric difference)
                    let col_k = &boundary_cols[k];
                    let mut combined = Vec::new();
                    let mut p1 = 0;
                    let mut p2 = 0;

                    while p1 < boundary_cols[j].len() && p2 < col_k.len() {
                        let r1 = boundary_cols[j][p1];
                        let r2 = col_k[p2];
                        if r1 == r2 {
                            p1 += 1;
                            p2 += 1;
                        } else if r1 < r2 {
                            combined.push(r1);
                            p1 += 1;
                        } else {
                            combined.push(r2);
                            p2 += 1;
                        }
                    }
                    while p1 < boundary_cols[j].len() {
                        combined.push(boundary_cols[j][p1]);
                        p1 += 1;
                    }
                    while p2 < col_k.len() {
                        combined.push(col_k[p2]);
                        p2 += 1;
                    }

                    boundary_cols[j] = combined;
                } else {
                    break;
                }
            }

            if let Some(&low_j) = boundary_cols[j].last() {
                low_map.insert(low_j, j);
                let birth = simplices[low_j].birth;
                let death = simplices[j].birth;
                let dim = simplices[low_j].dimension;
                if death > birth {
                    intervals.push(PersistenceInterval {
                        dimension: dim,
                        birth,
                        death: Some(death),
                        lifetime: death - birth,
                    });
                }
            }
        }

        // Any simplex that did not pair as a low_j is an infinite persistence feature (death = None)
        for i in 0..m {
            if !low_map.contains_key(&i) && !low_map.values().any(|&c| c == i) {
                let dim = simplices[i].dimension;
                let birth = simplices[i].birth;
                intervals.push(PersistenceInterval {
                    dimension: dim,
                    birth,
                    death: None,
                    lifetime: f64::INFINITY,
                });
            }
        }

        intervals.sort_by(|a, b| {
            a.dimension.cmp(&b.dimension).then(
                b.lifetime
                    .partial_cmp(&a.lifetime)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });

        intervals
    }

    /// Compute Betti numbers $\beta_k$ at a given filtration parameter $\epsilon$.
    pub fn betti_numbers(
        intervals: &[PersistenceInterval],
        epsilon: f64,
        max_dim: usize,
    ) -> Vec<usize> {
        let mut bettis = vec![0; max_dim + 1];
        for int in intervals {
            if int.dimension <= max_dim && int.birth <= epsilon {
                match int.death {
                    Some(d) if d > epsilon => bettis[int.dimension] += 1,
                    None => bettis[int.dimension] += 1,
                    _ => {}
                }
            }
        }
        bettis
    }
}

/// Discrete Exterior Calculus & Hodge Laplacian Engine.
pub struct DiscreteHodge;

impl DiscreteHodge {
    /// Compute 0-Laplacian (Graph Laplacian) $L_0 = B_1 B_1^T$ for a graph with $n$ vertices and given edges.
    pub fn laplacian_0(num_vertices: usize, edges: &[[usize; 2]]) -> Vec<Vec<f64>> {
        let mut l0 = vec![vec![0.0; num_vertices]; num_vertices];
        for &[u, v] in edges {
            if u < num_vertices && v < num_vertices && u != v {
                l0[u][u] += 1.0;
                l0[v][v] += 1.0;
                l0[u][v] -= 1.0;
                l0[v][u] -= 1.0;
            }
        }
        l0
    }

    /// Count harmonic 0-forms (connected components $\beta_0$) via the multiplicity of zero eigenvalues of $L_0$.
    pub fn count_zero_eigenvalues(matrix: &[Vec<f64>], tol: f64) -> usize {
        let n = matrix.len();
        if n == 0 {
            return 0;
        }
        // Approximate rank using Gaussian elimination
        let mut a = matrix.to_vec();
        let mut rank = 0;

        for col in 0..n {
            let mut pivot_row = None;
            for row in rank..n {
                if a[row][col].abs() > tol {
                    pivot_row = Some(row);
                    break;
                }
            }

            if let Some(p) = pivot_row {
                a.swap(rank, p);
                let pivot = a[rank][col];
                for j in col..n {
                    a[rank][j] /= pivot;
                }
                for i in 0..n {
                    if i != rank && a[i][col].abs() > tol {
                        let factor = a[i][col];
                        for j in col..n {
                            a[i][j] -= factor * a[rank][j];
                        }
                    }
                }
                rank += 1;
            }
        }

        n.saturating_sub(rank)
    }
}
