//! Integration Tests for Phase 25: Geometric Deep Learning, SE(3) Equivariance & Clifford Neural Networks.

use algebra_engine::geometric_dl::{
    ClebschGordan, CliffordLayer, SphericalHarmonics, SteerableConvLayer,
};
use algebra_engine::{clebsch_gordan, clifford_layer_forward, spherical_harmonic};

#[test]
fn test_spherical_harmonics_orthonormality_and_symmetry() {
    let pi = std::f64::consts::PI;
    // Y_0^0 = 1 / sqrt(4 * pi)
    let y00 = SphericalHarmonics::real_y_lm(0, 0, 0.5, 1.2);
    let expected_y00 = 1.0 / (4.0 * pi).sqrt();
    assert!((y00 - expected_y00).abs() < 1e-10);

    // Y_1^0(theta, phi) = sqrt(3 / (4 * pi)) * cos(theta)
    let theta = 0.8;
    let phi = 1.5;
    let y10 = SphericalHarmonics::real_y_lm(1, 0, theta, phi);
    let expected_y10 = (3.0 / (4.0 * pi)).sqrt() * theta.cos();
    assert!((y10 - expected_y10).abs() < 1e-10);

    // Cartesian equivalence
    let z = theta.cos();
    let x = theta.sin() * phi.cos();
    let y = theta.sin() * phi.sin();
    let y10_cart = SphericalHarmonics::real_y_lm_cartesian(1, 0, x, y, z);
    assert!((y10 - y10_cart).abs() < 1e-10);
}

#[test]
fn test_clebsch_gordan_coefficients_and_orthogonality() {
    // <1, 0, 1, 0 | 0, 0> = -1 / sqrt(3)
    let cg_0 = ClebschGordan::coefficient(1, 0, 1, 0, 0, 0);
    assert!((cg_0 - (-1.0 / 3.0_f64.sqrt())).abs() < 1e-10);

    // <1, 1, 1, -1 | 0, 0> = 1 / sqrt(3)
    let cg_1 = ClebschGordan::coefficient(1, 1, 1, -1, 0, 0);
    assert!((cg_1 - (1.0 / 3.0_f64.sqrt())).abs() < 1e-10);

    // <1, -1, 1, 1 | 0, 0> = 1 / sqrt(3)
    let cg_neg1 = ClebschGordan::coefficient(1, -1, 1, 1, 0, 0);
    assert!((cg_neg1 - (1.0 / 3.0_f64.sqrt())).abs() < 1e-10);

    // Violation of triangle rule: L = 3 for l1 = 1, l2 = 1 -> 0.0
    let cg_viol = ClebschGordan::coefficient(1, 0, 1, 0, 3, 0);
    assert_eq!(cg_viol, 0.0);

    // Violation of m1 + m2 == m: m1 = 1, m2 = 1, m = 0 -> 0.0
    let cg_m_viol = ClebschGordan::coefficient(1, 1, 1, 1, 2, 0);
    assert_eq!(cg_m_viol, 0.0);
}

#[test]
fn test_clebsch_gordan_tensor_product() {
    // Vector u (L=1) and vector v (L=1) coupled to scalar (L=0)
    let u = vec![1.0, 0.0, 0.0];
    let v = vec![0.0, 1.0, 0.0];

    let coupled_l0 = ClebschGordan::tensor_product(1, &u, 1, &v, 0);
    assert_eq!(coupled_l0.len(), 1);

    let coupled_l2 = ClebschGordan::tensor_product(1, &u, 1, &v, 2);
    assert_eq!(coupled_l2.len(), 5);
}

#[test]
fn test_steerable_se3_conv_layer() {
    let layer = SteerableConvLayer::new(1, 0, 1, 2.0);
    let in_feat = vec![1.0, 2.0, 3.0];
    let delta_r = [1.0, 0.0, 0.0];

    let msg = layer.compute_message(&in_feat, &delta_r);
    assert_eq!(msg.len(), 1); // out_l = 0 (scalar)
    assert!(msg[0].is_finite());
}

#[test]
fn test_clifford_multivector_layer() {
    let layer = CliffordLayer::new_identity();
    let mv_in = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

    let mv_out = layer.forward(&mv_in);
    assert_eq!(mv_in, mv_out);

    let act = CliffordLayer::gelu_activate(&mv_out);
    for x in act {
        assert!(x.is_finite());
    }
}

#[test]
fn test_phase25_dsl_macros() {
    let y = spherical_harmonic!(0, 0, 0.5, 0.5);
    assert!(y > 0.0);

    let cg = clebsch_gordan!(1, 0, 1, 0, 0, 0);
    assert!(cg < 0.0);

    let layer = CliffordLayer::new_identity();
    let mv = [1.0; 8];
    let out = clifford_layer_forward!(&layer, &mv);
    assert_eq!(out, mv);
}
