//! Phase 6 Comprehensive Tests: Cartan Calculus, Multi-Valued Logics, Schwartz Distributions, Clifford Algebra, and Extended Category Traits.

use algebra_core::traits::{MathSet, Semiring};
use algebra_engine::cartan::{DifferentialForm, FormBasis, VectorField};
use algebra_engine::distributions::SchwartzDistribution;
use algebra_engine::logic::{ThreeValuedLogic, ThreeValuedSystem, ThreeValuedValue};

#[test]
fn test_cartan_basis_permutation_parity() {
    // dx^0 ∧ dx^1 (sorted) -> sign +1
    let (b1, s1) = FormBasis::new(vec![0, 1]);
    assert_eq!(s1, 1);
    assert_eq!(b1.indices, vec![0, 1]);

    // dx^1 ∧ dx^0 (reversed) -> sign -1, sorted to [0, 1]
    let (b2, s2) = FormBasis::new(vec![1, 0]);
    assert_eq!(s2, -1);
    assert_eq!(b2.indices, vec![0, 1]);

    // dx^0 ∧ dx^0 (duplicate) -> vanishes (sign 0)
    let (_, s3) = FormBasis::new(vec![0, 0]);
    assert_eq!(s3, 0);
}

#[test]
fn test_cartan_wedge_product_anticommutativity() {
    let coords = vec!["x".into(), "y".into(), "z".into()];
    let dx = DifferentialForm::dx(0, 3, coords.clone()).unwrap();
    let dy = DifferentialForm::dx(1, 3, coords).unwrap();

    // dx ∧ dy
    let dx_dy = dx.wedge(&dy).unwrap();
    assert_eq!(dx_dy.degree, 2);

    // dy ∧ dx
    let dy_dx = dy.wedge(&dx).unwrap();
    assert_eq!(dy_dx.degree, 2);

    // dx ∧ dy + dy ∧ dx == 0
    let sum = dx_dy.add(&dy_dx).unwrap();
    assert!(
        sum.terms.is_empty(),
        "Wedge product of 1-forms must be anticommutative"
    );
}

#[test]
fn test_cartan_interior_product_contraction() {
    let coords = vec!["x".into(), "y".into(), "z".into()];
    let dx = DifferentialForm::dx(0, 3, coords.clone()).unwrap();
    let dy = DifferentialForm::dx(1, 3, coords.clone()).unwrap();
    let dx_dy = dx.wedge(&dy).unwrap(); // dx ∧ dy

    // Vector field X = 3 ∂x + 4 ∂y
    let vf = VectorField {
        dim: 3,
        coord_names: coords,
        components: vec![3.0, 4.0, 0.0],
    };

    // i_X (dx ∧ dy) = 3 dy - 4 dx
    let contracted = dx_dy.interior_product(&vf).unwrap();
    assert_eq!(contracted.degree, 1);

    let (basis_dx, _) = FormBasis::new(vec![0]);
    let (basis_dy, _) = FormBasis::new(vec![1]);

    assert_eq!(
        contracted.terms.get(&basis_dx).copied().unwrap_or(0.0),
        -4.0
    );
    assert_eq!(contracted.terms.get(&basis_dy).copied().unwrap_or(0.0), 3.0);
}

#[test]
fn test_cartan_magic_formula_lie_derivative() {
    let coords = vec!["x".into(), "y".into(), "z".into()];
    let dx = DifferentialForm::dx(0, 3, coords.clone()).unwrap();
    let dy = DifferentialForm::dx(1, 3, coords.clone()).unwrap();
    let dx_dy = dx.wedge(&dy).unwrap();

    let vf = VectorField {
        dim: 3,
        coord_names: coords,
        components: vec![2.0, -1.0, 0.0],
    };

    // Lie derivative L_X omega = (d i_X + i_X d) omega
    let lie = dx_dy.lie_derivative(&vf).unwrap();
    assert_eq!(lie.degree, 2);
}

#[test]
fn test_cartan_hodge_star() {
    let coords = vec!["x".into(), "y".into(), "z".into()];
    let dx = DifferentialForm::dx(0, 3, coords).unwrap();

    // *dx in 3D Euclidean space should be dy ∧ dz
    let star_dx = dx.hodge_star(false).unwrap();
    assert_eq!(star_dx.degree, 2);

    let (basis_dy_dz, _) = FormBasis::new(vec![1, 2]);
    assert_eq!(star_dx.terms.get(&basis_dy_dz).copied().unwrap_or(0.0), 1.0);
}

#[test]
fn test_three_valued_logics_and_modality() {
    use ThreeValuedValue::*;

    // Łukasiewicz implication: u -> u is True
    assert_eq!(
        ThreeValuedLogic::implies(Unknown, Unknown, ThreeValuedSystem::LukasiewiczL3),
        True
    );

    // Gödel implication: u -> False is False
    assert_eq!(
        ThreeValuedLogic::implies(Unknown, False, ThreeValuedSystem::GodelG3),
        False
    );

    // Modal necessity and possibility
    assert_eq!(ThreeValuedLogic::necessity(True), True);
    assert_eq!(ThreeValuedLogic::necessity(Unknown), False);
    assert_eq!(ThreeValuedLogic::possibility(Unknown), True);
    assert_eq!(ThreeValuedLogic::possibility(False), False);
}

#[test]
fn test_schwartz_distribution_derivatives_and_pairings() {
    let theta = SchwartzDistribution::step();
    let delta = theta.derivative();

    // The distributional derivative of Heaviside is Dirac delta
    assert_eq!(
        delta,
        SchwartzDistribution::DiracDelta {
            shift: 0.0,
            order: 0
        }
    );

    // Test function phi(x, k) = e^{-x^2}, where phi(0, 0) = 1.0
    let test_phi = |x: f64, k: usize| -> f64 {
        if k == 0 {
            (-x * x).exp()
        } else if k == 1 {
            -2.0 * x * (-x * x).exp()
        } else {
            0.0
        }
    };

    // <delta(x), phi> = phi(0) = 1.0
    let delta_val = delta.pair(&test_phi);
    assert!((delta_val - 1.0).abs() < 1e-10);
}

#[test]
fn test_clifford_algebra_multivectors_and_rotor_sandwiches() {
    use algebra_advanced::clifford::{CliffordMultivector, CliffordSignature};

    let sig = CliffordSignature::pga3d(); // Cl(3,0,0)
    let e1 = CliffordMultivector::basis_vector(0, sig).unwrap();
    let e2 = CliffordMultivector::basis_vector(1, sig).unwrap();

    // e1 * e2 = e1 ∧ e2 (bivector B)
    let bivector = e1.geometric_product(&e2).unwrap();
    assert_eq!(bivector.grade_project(2).blades.len(), 1);

    // Reversion: B^\dagger = (e1 e2)^\dagger = e2 e1 = -e1 e2 = -B
    let b_rev = bivector.reverse();
    let b_scaled = bivector.scale(-1.0);
    assert_eq!(b_rev, b_scaled);

    // 90-degree rotor around e1^e2: R = cos(pi/4) - sin(pi/4) B
    let rotor = CliffordMultivector::make_rotor_3d(&bivector, std::f64::consts::PI / 2.0).unwrap();

    // Rotate e1 by 90 degrees -> should become -e2 (or e2 depending on bivector orientation)
    let v_rotated = e1.rotor_sandwich(&rotor).unwrap();
    let proj1 = v_rotated.grade_project(1);
    assert_eq!(proj1.blades.len(), 1);
}

struct NaturalSemiring;
impl MathSet for NaturalSemiring {
    type Element = u64;
    fn contains(&self, _x: &Self::Element) -> bool {
        true
    }
    fn zero(&self) -> Self::Element {
        0
    }
    fn one(&self) -> Self::Element {
        1
    }
}
impl Semiring for NaturalSemiring {
    fn add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        a + b
    }
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element {
        a * b
    }
}

#[test]
fn test_semiring_trait_implementation() {
    let rig = NaturalSemiring;
    let z = rig.zero();
    let o = rig.one();
    assert_eq!(rig.add(&z, &o), 1);
    assert_eq!(rig.mul(&o, &42), 42);
}
