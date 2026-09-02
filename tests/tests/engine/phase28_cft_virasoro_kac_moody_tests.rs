//! Integration Tests for Phase 28: Affine Kac-Moody & Virasoro Conformal Field Theory.

use algebra_engine::cft::{KacMoodyAlgebra, OperatorProductExpansion, VirasoroAlgebra};
use algebra_engine::{
    kac_conformal_weight, kac_moody_bracket, sugawara_central_charge, virasoro_bracket,
};

#[test]
fn test_virasoro_bracket_and_central_charge() {
    let c = 2.0;

    // [L_2, L_{-2}] = (2 - (-2)) L_0 + (c / 12) * (2^3 - 2) = 4 L_0 + (2/12) * 6 = 4 L_0 + 1.0
    let (mode, coeff_l, central) = VirasoroAlgebra::bracket(2, -2, c);
    assert_eq!(mode, 0);
    assert_eq!(coeff_l, 4.0);
    assert_eq!(central, 1.0);

    // [L_1, L_{-1}] = 2 L_0 + (c / 12) * (1 - 1) = 2 L_0 + 0
    let (mode1, coeff1, central1) = VirasoroAlgebra::bracket(1, -1, c);
    assert_eq!(mode1, 0);
    assert_eq!(coeff1, 2.0);
    assert_eq!(central1, 0.0);

    // [L_2, L_3] = -1 L_5 + 0
    let (mode2, coeff2, central2) = VirasoroAlgebra::bracket(2, 3, c);
    assert_eq!(mode2, 5);
    assert_eq!(coeff2, -1.0);
    assert_eq!(central2, 0.0);
}

#[test]
fn test_virasoro_jacobi_identity() {
    let c = 0.5;
    assert!(VirasoroAlgebra::verify_jacobi(1, 2, 3, c));
    assert!(VirasoroAlgebra::verify_jacobi(2, -2, 1, c));
    assert!(VirasoroAlgebra::verify_jacobi(3, -1, -2, c));
    assert!(VirasoroAlgebra::verify_jacobi(4, -3, -1, 24.0));
}

#[test]
fn test_kac_determinant_minimal_model_ising() {
    // Ising model: m = 3 -> c = 1 - 6 / (3 * 4) = 1/2
    let c_ising = VirasoroAlgebra::minimal_model_c(3);
    assert_eq!(c_ising, 0.5);

    // Identity field (1, 1): h = 0
    let h11 = VirasoroAlgebra::kac_conformal_weight(3, 1, 1);
    assert_eq!(h11, 0.0);

    // Energy operator epsilon (2, 1): h = 1/2
    let h21 = VirasoroAlgebra::kac_conformal_weight(3, 2, 1);
    assert_eq!(h21, 0.5);

    // Spin operator sigma (1, 2): h = 1/16 = 0.0625
    let h12 = VirasoroAlgebra::kac_conformal_weight(3, 1, 2);
    assert_eq!(h12, 1.0 / 16.0);
}

#[test]
fn test_kac_moody_su2_current_algebra() {
    let k = 2.0;

    // [J^1_0, J^2_0] = i f^{123} J^3_0 = i J^3_0
    let (mode, f_coeffs, central) = KacMoodyAlgebra::su2_bracket(1, 0, 2, 0, k);
    assert_eq!(mode, 0);
    assert_eq!(f_coeffs[2], 1.0); // f^{123} = 1.0
    assert_eq!(central, 0.0);

    // [J^1_2, J^1_{-2}] = 0 + k * 2 = 4.0
    let (mode_diag, f_diag, central_diag) = KacMoodyAlgebra::su2_bracket(1, 2, 1, -2, k);
    assert_eq!(mode_diag, 0);
    assert_eq!(f_diag, [0.0, 0.0, 0.0]);
    assert_eq!(central_diag, 4.0);
}

#[test]
fn test_sugawara_central_charge_construction() {
    // For su(2)_k: dim(g) = 3, h^v = 2
    // For k = 1 -> c = 1 * 3 / (1 + 2) = 1.0
    let c_k1 = KacMoodyAlgebra::sugawara_central_charge(3.0, 2.0, 1.0);
    assert_eq!(c_k1, 1.0);

    // For k = 2 -> c = 2 * 3 / (2 + 2) = 1.5
    let c_k2 = KacMoodyAlgebra::sugawara_central_charge(3.0, 2.0, 2.0);
    assert_eq!(c_k2, 1.5);
}

#[test]
fn test_operator_product_expansion_poles() {
    let (c4, c2, c1) = OperatorProductExpansion::energy_momentum_ope(0.5);
    assert_eq!(c4, 0.25); // c / 2 = 0.25
    assert_eq!(c2, 2.0);
    assert_eq!(c1, 1.0);

    let (h2, h1) = OperatorProductExpansion::primary_field_ope(1.0 / 16.0);
    assert_eq!(h2, 0.0625);
    assert_eq!(h1, 1.0);
}

#[test]
fn test_phase28_dsl_macros() {
    let (mode, coeff, cent) = virasoro_bracket!(2, -2, 2.0);
    assert_eq!(mode, 0);
    assert_eq!(coeff, 4.0);
    assert_eq!(cent, 1.0);

    let (_, f, k_cent) = kac_moody_bracket!(1, 0, 2, 0, 1.0);
    assert_eq!(f[2], 1.0);
    assert_eq!(k_cent, 0.0);

    let c = sugawara_central_charge!(3.0, 2.0, 1.0);
    assert_eq!(c, 1.0);

    let h_sigma = kac_conformal_weight!(3, 1, 2);
    assert_eq!(h_sigma, 0.0625);
}
