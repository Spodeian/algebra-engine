//! Phase 6 Comprehensive Tests: Clifford Algebras, Geometric Product, Rotor Sandwiches, and Free Tensors.

use algebra_advanced::clifford::{BladeMask, CliffordMultivector, CliffordSignature, FreeTensor};

#[test]
fn test_clifford_geometric_product_and_involutions() {
    let sig = CliffordSignature::pga3d(); // Cl(3,0,0)
    let e1 = CliffordMultivector::basis_vector(0, sig).unwrap();
    let e2 = CliffordMultivector::basis_vector(1, sig).unwrap();

    // e1 * e1 = +1
    let e1_sq = e1.geometric_product(&e1).unwrap();
    assert_eq!(
        e1_sq
            .blades
            .get(&BladeMask::scalar())
            .copied()
            .unwrap_or(0.0),
        1.0
    );

    // e1 * e2 = e12 (bivector)
    let e12 = e1.geometric_product(&e2).unwrap();
    assert_eq!(e12.blades.get(&BladeMask(3)).copied().unwrap_or(0.0), 1.0);

    // e2 * e1 = -e12 (anticommutative)
    let e21 = e2.geometric_product(&e1).unwrap();
    assert_eq!(e21.blades.get(&BladeMask(3)).copied().unwrap_or(0.0), -1.0);

    // Reversion: (e12)^dagger = -e12
    let e12_rev = e12.reverse();
    assert_eq!(
        e12_rev.blades.get(&BladeMask(3)).copied().unwrap_or(0.0),
        -1.0
    );
}

#[test]
fn test_clifford_rotor_sandwich_rotation() {
    let sig = CliffordSignature::pga3d();
    let e1 = CliffordMultivector::basis_vector(0, sig).unwrap(); // X-axis
    let e2 = CliffordMultivector::basis_vector(1, sig).unwrap(); // Y-axis
    let e12 = e1.wedge(&e2).unwrap(); // XY plane bivector

    // Rotate e1 by 90 degrees (pi/2) in XY plane -> should become e2
    let angle = std::f64::consts::PI / 2.0;
    let rotor = CliffordMultivector::make_rotor_3d(&e12, angle).unwrap();

    let rotated_e1 = e1.rotor_sandwich(&rotor).unwrap();

    let e1_comp = rotated_e1
        .blades
        .get(&BladeMask::vector(0))
        .copied()
        .unwrap_or(0.0);
    let e2_comp = rotated_e1
        .blades
        .get(&BladeMask::vector(1))
        .copied()
        .unwrap_or(0.0);

    assert!(e1_comp.abs() < 1e-10, "X component should rotate to 0");
    assert!(
        (e2_comp - 1.0).abs() < 1e-10,
        "Y component should rotate to 1.0"
    );
}

#[test]
fn test_free_tensor_outer_product_and_trace() {
    // 2D rank-1 vector T1 = [1.0, 2.0]
    let mut t1 = FreeTensor::zero(2, 1);
    t1.data = vec![1.0, 2.0];

    // 2D rank-1 vector T2 = [3.0, 4.0]
    let mut t2 = FreeTensor::zero(2, 1);
    t2.data = vec![3.0, 4.0];

    // Outer product T = T1 ⊗ T2 (rank 2 tensor of size 2x2 = 4)
    // T = [[3.0, 4.0], [6.0, 8.0]]
    let t_prod = t1.tensor_product(&t2).unwrap();
    assert_eq!(t_prod.rank, 2);
    assert_eq!(t_prod.data, vec![3.0, 4.0, 6.0, 8.0]);

    // Contraction (Trace) = 3.0 + 8.0 = 11.0
    let contracted = t_prod.contract(0, 1).unwrap();
    assert_eq!(contracted.rank, 0);
    assert_eq!(contracted.data, vec![11.0]);
}

#[test]
fn test_clifford_pauli_dirac_isomorphisms() {
    let sig3 = CliffordSignature::pga3d(); // Cl(3,0,0)
    let e1 = CliffordMultivector::basis_vector(0, sig3).unwrap();
    let e2 = CliffordMultivector::basis_vector(1, sig3).unwrap();
    let e3 = CliffordMultivector::basis_vector(2, sig3).unwrap();

    // Test Pauli matrix representation
    // sigma_x = e1
    let pauli_e1 = e1.to_pauli_spinor_2x2().unwrap();
    assert_eq!(pauli_e1, [[(0.0, 0.0), (1.0, 0.0)], [(1.0, 0.0), (0.0, 0.0)]]);

    // sigma_y = e2
    let pauli_e2 = e2.to_pauli_spinor_2x2().unwrap();
    assert_eq!(pauli_e2, [[(0.0, 0.0), (0.0, -1.0)], [(0.0, 1.0), (0.0, 0.0)]]);

    // sigma_z = e3
    let pauli_e3 = e3.to_pauli_spinor_2x2().unwrap();
    assert_eq!(pauli_e3, [[(1.0, 0.0), (0.0, 0.0)], [(0.0, 0.0), (-1.0, 0.0)]]);

    // Even subalgebra to Quaternion
    // Rotor R = cos(pi/4) - sin(pi/4) e12 = 1/sqrt(2) (1 - e12)
    let e12 = e1.wedge(&e2).unwrap();
    let rotor = CliffordMultivector::make_rotor_3d(&e12, std::f64::consts::PI / 2.0).unwrap();
    let (w, x, y, z) = rotor.to_quaternion().unwrap();
    assert!((w - (std::f64::consts::PI / 4.0).cos()).abs() < 1e-10);
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
    assert!((z - (std::f64::consts::PI / 4.0).sin()).abs() < 1e-10);

    // Spacetime Algebra Cl(1,3,0) to Dirac matrices
    let sta = CliffordSignature::spacetime();
    let gamma0 = CliffordMultivector::basis_vector(0, sta).unwrap();
    let dirac0 = gamma0.to_dirac_gamma_4x4().unwrap();
    // gamma^0 = diag(1, 1, -1, -1)
    assert_eq!(dirac0[0][0].0, 1.0);
    assert_eq!(dirac0[1][1].0, 1.0);
    assert_eq!(dirac0[2][2].0, -1.0);
    assert_eq!(dirac0[3][3].0, -1.0);
}

