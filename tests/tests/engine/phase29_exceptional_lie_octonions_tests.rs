//! Integration Tests for Phase 29: Exceptional Lie Groups, Octonions & E8 Root System.

use algebra_engine::exceptional_lie::{AlbertAlgebra, E8Lattice, Octonion};
use algebra_engine::{albert_det, e8_weyl_reflect, octonion, octonion_associator, octonion_mul};

#[test]
fn test_octonion_fano_plane_multiplication() {
    let e1 = Octonion::basis(1);
    let e2 = Octonion::basis(2);
    let e3 = Octonion::basis(3);

    // e1 * e2 = e3
    let e12 = e1.mul(&e2);
    assert_eq!(e12, e3);

    // e2 * e1 = -e3
    let e21 = e2.mul(&e1);
    assert_eq!(e21, e3.scale(-1.0));

    // e1 * e1 = -1
    let e11 = e1.mul(&e1);
    assert_eq!(e11, Octonion::real(-1.0));
}

#[test]
fn test_octonion_non_associativity_and_alternativity() {
    let e1 = Octonion::basis(1);
    let e2 = Octonion::basis(2);
    let e4 = Octonion::basis(4);

    // (e1 * e2) * e4 = e3 * e4 = e7
    // e1 * (e2 * e4) = e1 * e6 = -e7
    // Associator [e1, e2, e4] = 2 e7 != 0
    let assoc = e1.associator(&e2, &e4);
    assert_eq!(assoc, Octonion::basis(7).scale(2.0));

    // Alternativity: [x, x, y] = 0
    let x = octonion!(1, 2, 3, 4, 5, 6, 7, 8);
    let y = octonion!(2, -1, 0, 3, -2, 1, 4, -3);
    let alt_assoc = x.associator(&x, &y);
    assert!(alt_assoc.norm() < 1e-9);

    // Norm multiplicativity: N(x * y) = N(x) * N(y)
    let xy = x.mul(&y);
    let n_xy = xy.norm_squared();
    let nx_ny = x.norm_squared() * y.norm_squared();
    assert!((n_xy - nx_ny).abs() < 1e-6);
}

#[test]
fn test_albert_exceptional_jordan_algebra() {
    let identity = AlbertAlgebra::new(
        [1.0, 1.0, 1.0],
        [Octonion::zero(), Octonion::zero(), Octonion::zero()],
    );

    assert_eq!(identity.trace(), 3.0);
    assert_eq!(identity.determinant(), 1.0);

    let j_prod = identity.jordan_product(&identity);
    assert_eq!(j_prod.diag, [1.0, 1.0, 1.0]);
}

#[test]
fn test_e8_root_lattice_count_and_lengths() {
    let roots = E8Lattice::all_roots();
    assert_eq!(roots.len(), 240);

    // Each root vector in E8 has norm squared = 2.0
    for root in &roots {
        let norm_sq: f64 = root.iter().map(|&x| x * x).sum();
        assert!((norm_sq - 2.0).abs() < 1e-9);
    }

    let cartan = E8Lattice::cartan_matrix();
    for (i, row) in cartan.iter().enumerate() {
        assert_eq!(row[i], 2);
    }
}

#[test]
fn test_e8_weyl_reflections() {
    let simples = E8Lattice::simple_roots();
    let alpha1 = simples[0];

    // s_alpha(alpha) = -alpha
    let reflected = E8Lattice::weyl_reflect(&alpha1, &alpha1);
    for i in 0..8 {
        assert!((reflected[i] - (-alpha1[i])).abs() < 1e-9);
    }

    // s_alpha(s_alpha(v)) = v
    let v = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let v_ref1 = E8Lattice::weyl_reflect(&v, &alpha1);
    let v_ref2 = E8Lattice::weyl_reflect(&v_ref1, &alpha1);
    for i in 0..8 {
        assert!((v_ref2[i] - v[i]).abs() < 1e-9);
    }
}

#[test]
fn test_phase29_dsl_macros() {
    let x = octonion!(1, 0, 0, 0, 0, 0, 0, 0);
    let y = octonion!(0, 1, 0, 0, 0, 0, 0, 0);
    let xy = octonion_mul!(x, y);
    assert_eq!(xy, Octonion::basis(1));

    let assoc = octonion_associator!(x, y, x);
    assert_eq!(assoc, Octonion::zero());

    let albert = AlbertAlgebra::new(
        [1.0, 2.0, 3.0],
        [Octonion::zero(), Octonion::zero(), Octonion::zero()],
    );
    let det = albert_det!(albert);
    assert_eq!(det, 6.0);

    let v = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let alpha = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let ref_v = e8_weyl_reflect!(v, alpha);
    assert_eq!(ref_v, [0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
}
