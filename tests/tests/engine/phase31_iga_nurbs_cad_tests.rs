//! Integration Tests for Phase 31: Advanced N-Dimensional CAD Topology & Isogeometric Analysis (IGA).

use algebra_engine::iga::{
    CoxDeBoor, IgaStiffnessMatrix, MultiPatchBRep, NurbsPatchND, WeightedControlPoint,
};
use algebra_engine::{cox_de_boor_basis, iga_stiffness_entry_2d};

#[test]
fn test_cox_de_boor_basis_partition_of_unity() {
    let knots = vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0];
    let p = 2;
    let n = knots.len() - p - 1; // 7 - 3 = 4 basis functions

    for &u in &[0.1, 0.35, 0.5, 0.75, 0.95] {
        let sum: f64 = (0..n)
            .map(|i| CoxDeBoor::basis_function(i, p, u, &knots))
            .sum();
        assert!((sum - 1.0).abs() < 1e-9);
    }
}

#[test]
fn test_cox_de_boor_basis_derivatives() {
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let p = 2;
    let u = 0.4;
    let eps = 1e-5;

    for i in 0..3 {
        let exact_deriv = CoxDeBoor::basis_derivative(i, p, u, &knots);
        let num_deriv = (CoxDeBoor::basis_function(i, p, u + eps, &knots)
            - CoxDeBoor::basis_function(i, p, u - eps, &knots))
            / (2.0 * eps);
        assert!((exact_deriv - num_deriv).abs() < 1e-4);
    }
}

#[test]
fn test_nurbs_surface_2d_evaluation_and_metric() {
    let knots_u = vec![0.0, 0.0, 1.0, 1.0];
    let knots_v = vec![0.0, 0.0, 1.0, 1.0];
    let p_u = 1;
    let p_v = 1;

    let pts = vec![
        WeightedControlPoint::new(vec![0.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![0.0, 1.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![1.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![1.0, 1.0, 0.0], 1.0),
    ];

    let patch = NurbsPatchND::surface_2d(p_u, p_v, knots_u, knots_v, 2, 2, pts);

    let center = patch.evaluate_surface(0.5, 0.5);
    assert!((center[0] - 0.5).abs() < 1e-6);
    assert!((center[1] - 0.5).abs() < 1e-6);
    assert!((center[2] - 0.0).abs() < 1e-6);

    let det_j = patch.metric_determinant_2d(0.5, 0.5);
    assert!((det_j - 1.0).abs() < 1e-3);
}

#[test]
fn test_iga_stiffness_matrix_quadrature() {
    let knots_u = vec![0.0, 0.0, 1.0, 1.0];
    let knots_v = vec![0.0, 0.0, 1.0, 1.0];
    let pts = vec![
        WeightedControlPoint::new(vec![0.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![0.0, 1.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![1.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![1.0, 1.0, 0.0], 1.0),
    ];

    let patch = NurbsPatchND::surface_2d(1, 1, knots_u, knots_v, 2, 2, pts);
    let k_00 = IgaStiffnessMatrix::integrate_stiffness_entry_2d(&patch, 0, 0);
    assert!(k_00 > 0.0);

    let brep = MultiPatchBRep::new(vec![patch]);
    assert_eq!(brep.num_patches(), 1);
}

#[test]
fn test_phase31_dsl_macros() {
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let n0 = cox_de_boor_basis!(0, 2, 0.5, knots);
    assert!((n0 - 0.25).abs() < 1e-6);

    let pts = vec![
        WeightedControlPoint::new(vec![0.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![0.0, 2.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![2.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![2.0, 2.0, 0.0], 1.0),
    ];
    let patch = NurbsPatchND::surface_2d(
        1,
        1,
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        2,
        2,
        pts,
    );
    let k_val = iga_stiffness_entry_2d!(patch, 0, 0);
    assert!(k_val > 0.0);
}
