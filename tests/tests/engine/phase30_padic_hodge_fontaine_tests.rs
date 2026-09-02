//! Integration Tests for Phase 30: p-Adic Hodge Theory, Fontaine Modules & Galois Representations.

use algebra_engine::padic_hodge::{FontaineModule, PadicGaloisRepresentation, TateTwist};
use algebra_engine::{fontaine_module, tate_twist_frob, tate_twist_weights};

#[test]
fn test_fontaine_relation_n_phi_commutation() {
    let p = 3;
    // For a 2D semistable module:
    // Phi = diag(1.0, 3.0), N = [[0, 1], [0, 0]]
    // N * Phi = [[0, 1], [0, 0]] * [[1, 0], [0, 3]] = [[0, 3], [0, 0]]
    // p * Phi * N = 3 * [[1, 0], [0, 3]] * [[0, 1], [0, 0]] = 3 * [[0, 1], [0, 0]] = [[0, 3], [0, 0]]
    let frob = vec![vec![1.0, 0.0], vec![0.0, 3.0]];
    let mono = vec![vec![0.0, 1.0], vec![0.0, 0.0]];
    let weights = vec![0, 1];

    let module = FontaineModule::new(2, p, frob, mono, weights);
    assert!(module.verify_monodromy_frobenius_commutation());
    assert!(!module.is_crystalline());
    assert!(module.is_semistable());
}

#[test]
fn test_galois_representation_crystalline_and_semistable_classification() {
    let p = 5;
    // Crystalline module: N = 0
    let frob_cris = vec![vec![1.0, 0.0], vec![0.0, 5.0]];
    let mono_cris = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
    let mod_cris = FontaineModule::new(2, p, frob_cris, mono_cris, vec![0, 1]);

    let rep_cris = PadicGaloisRepresentation::from_fontaine_module(&mod_cris);
    assert!(rep_cris.is_crystalline());
    assert!(rep_cris.is_semistable());
    assert!(rep_cris.is_de_rham());

    // Semistable non-crystalline module
    let frob_st = vec![vec![1.0, 0.0], vec![0.0, 5.0]];
    let mono_st = vec![vec![0.0, 1.0], vec![0.0, 0.0]];
    let mod_st = FontaineModule::new(2, p, frob_st, mono_st, vec![0, 1]);

    let rep_st = PadicGaloisRepresentation::from_fontaine_module(&mod_st);
    assert!(!rep_st.is_crystalline());
    assert!(rep_st.is_semistable());
    assert!(rep_st.is_de_rham());
}

#[test]
fn test_tate_twists_on_hodge_tate_weights_and_frobenius() {
    let weights = vec![0, 1, 2];
    let evals = vec![1.0, 5.0, 25.0];
    let p = 5;
    let r = 1;

    // Twist by Q_p(1): weights shift by -1 -> [-1, 0, 1]
    let shifted_weights = TateTwist::twist_hodge_tate_weights(&weights, r);
    assert_eq!(shifted_weights, vec![-1, 0, 1]);

    // Frobenius eigenvalues scale by p^(-1) = 1/5 -> [0.2, 1.0, 5.0]
    let scaled_evals = TateTwist::twist_frobenius_eigenvalues(&evals, p, r);
    assert!((scaled_evals[0] - 0.2).abs() < 1e-9);
    assert!((scaled_evals[1] - 1.0).abs() < 1e-9);
    assert!((scaled_evals[2] - 5.0).abs() < 1e-9);
}

#[test]
fn test_phase30_dsl_macros() {
    let frob = vec![vec![1.0, 0.0], vec![0.0, 7.0]];
    let mono = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
    let weights = vec![0, 1];

    let module = fontaine_module!(2, 7, frob, mono, weights);
    assert!(module.is_crystalline());

    let shifted = tate_twist_weights!([0, 2], 2);
    assert_eq!(shifted, vec![-2, 0]);

    let scaled = tate_twist_frob!([7.0, 49.0], 7, 1);
    assert!((scaled[0] - 1.0).abs() < 1e-9);
    assert!((scaled[1] - 7.0).abs() < 1e-9);
}
