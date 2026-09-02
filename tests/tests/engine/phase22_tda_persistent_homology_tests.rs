//! Integration Tests for Phase 22: Topological Data Analysis (TDA), Persistent Homology & Discrete Hodge Laplacian.

use algebra_engine::tda::{DiscreteHodge, PersistenceDiagram, VietorisRipsFiltration};
use algebra_engine::{discrete_laplacian_0, persistence_diagram, vietoris_rips};

#[test]
fn test_vietoris_rips_filtration_points() {
    let square_pts = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![1.0, 1.0],
        vec![0.0, 1.0],
    ];

    let simplices = VietorisRipsFiltration::from_point_cloud(&square_pts, 2, 1.5);
    // 4 vertices (dim 0)
    let v_count = simplices.iter().filter(|s| s.dimension == 0).count();
    assert_eq!(v_count, 4);

    // 4 perimeter edges (len 1.0) + 2 diagonal edges (len sqrt(2) ~ 1.414) = 6 edges
    let e_count = simplices.iter().filter(|s| s.dimension == 1).count();
    assert_eq!(e_count, 6);

    // 4 triangles
    let t_count = simplices.iter().filter(|s| s.dimension == 2).count();
    assert_eq!(t_count, 4);
}

#[test]
fn test_persistent_homology_circle_s1_hole_detection() {
    let n = 12;
    let mut circle_pts = Vec::with_capacity(n);
    for i in 0..n {
        let theta = (i as f64) * 2.0 * std::f64::consts::PI / (n as f64);
        circle_pts.push(vec![theta.cos(), theta.sin()]);
    }

    let simplices = VietorisRipsFiltration::from_point_cloud(&circle_pts, 2, 1.8);
    let intervals = PersistenceDiagram::compute(&simplices);

    // H0 features
    let h0_features: Vec<_> = intervals.iter().filter(|i| i.dimension == 0).collect();
    assert!(!h0_features.is_empty());
    // Exactly one H0 feature persists infinitely (death == None)
    let inf_h0 = h0_features.iter().filter(|i| i.death.is_none()).count();
    assert_eq!(inf_h0, 1);

    // H1 features (1-dimensional topological loop)
    let h1_features: Vec<_> = intervals.iter().filter(|i| i.dimension == 1).collect();
    assert!(!h1_features.is_empty());

    // The dominant H1 hole should have a significant lifetime
    let max_h1_lifetime = h1_features
        .iter()
        .map(|i| i.lifetime)
        .fold(0.0_f64, f64::max);
    assert!(max_h1_lifetime > 0.5);
}

#[test]
fn test_betti_number_tracking() {
    let n = 10;
    let mut circle_pts = Vec::with_capacity(n);
    for i in 0..n {
        let theta = (i as f64) * 2.0 * std::f64::consts::PI / (n as f64);
        circle_pts.push(vec![theta.cos(), theta.sin()]);
    }

    let simplices = VietorisRipsFiltration::from_point_cloud(&circle_pts, 2, 1.8);
    let intervals = PersistenceDiagram::compute(&simplices);

    // At small epsilon = 0.05, all 10 vertices are disconnected: beta_0 = 10, beta_1 = 0
    let betti_init = PersistenceDiagram::betti_numbers(&intervals, 0.05, 1);
    assert_eq!(betti_init[0], 10);
    assert_eq!(betti_init[1], 0);

    // At intermediate epsilon = 0.8, connected circle with 1 hole: beta_0 = 1, beta_1 = 1
    let betti_mid = PersistenceDiagram::betti_numbers(&intervals, 0.8, 1);
    assert_eq!(betti_mid[0], 1);
    assert_eq!(betti_mid[1], 1);
}

#[test]
fn test_discrete_hodge_laplacian_and_harmonic_0_forms() {
    // 2 disconnected components: triangle 1 (0, 1, 2) and triangle 2 (3, 4, 5)
    let edges = [[0, 1], [1, 2], [2, 0], [3, 4], [4, 5], [5, 3]];

    let l0 = DiscreteHodge::laplacian_0(6, &edges);
    assert_eq!(l0.len(), 6);
    assert_eq!(l0[0].len(), 6);

    // Degree of each vertex is 2
    for (i, row) in l0.iter().enumerate().take(6) {
        assert_eq!(row[i], 2.0);
    }

    // Number of zero eigenvalues equals number of connected components = 2
    let beta_0 = DiscreteHodge::count_zero_eigenvalues(&l0, 1e-6);
    assert_eq!(beta_0, 2);
}

#[test]
fn test_phase22_dsl_macros() {
    let pts = vec![vec![0.0, 0.0], vec![1.0, 0.0], vec![0.0, 1.0]];
    let simplices = vietoris_rips!(&pts, max_dim = 2, max_edge = 1.5);
    assert_eq!(simplices.len(), 7); // 3 vertices + 3 edges + 1 triangle

    let intervals = persistence_diagram!(&simplices);
    assert!(!intervals.is_empty());

    let edges = [[0, 1], [1, 2], [2, 0]];
    let l0 = discrete_laplacian_0!(3, &edges);
    assert_eq!(l0.len(), 3);
}
