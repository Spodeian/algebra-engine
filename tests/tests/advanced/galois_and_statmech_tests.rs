use algebra_advanced::galois::{FieldExtension, GaloisGroupType};
use algebra_advanced::statmech::{CanonicalPartitionFunction, bose_einstein, fermi_dirac};

#[test]
fn test_galois_theory() {
    let q_sqrt2 = FieldExtension::quadratic(2);
    assert_eq!(q_sqrt2.degree, 2);
    assert!(q_sqrt2.is_galois);
    assert!(q_sqrt2.is_solvable_by_radicals());

    let g_s5 = GaloisGroupType::Symmetric(5);
    assert!(!g_s5.is_solvable()); // S_5 is not solvable by radicals (Abel-Ruffini)

    let g_s4 = GaloisGroupType::Symmetric(4);
    assert!(g_s4.is_solvable()); // S_4 is solvable
}

#[test]
fn test_statistical_mechanics() {
    // Two-level system: E_0 = 0 (g_0 = 1), E_1 = 1.0 (g_1 = 1)
    let sys = CanonicalPartitionFunction::new(vec![(0.0, 1), (1.0, 1)]);
    let beta = 1.0;

    // Z = e^0 + e^-1 = 1 + 1/e ≈ 1.367879
    let z = sys.evaluate_z(beta);
    assert!((z - (1.0 + (-1.0f64).exp())).abs() < 1e-6);

    let free_e = sys.free_energy(beta);
    assert!(free_e < 0.0);

    // Fermi-Dirac at T -> 0
    let fd = fermi_dirac(0.5, 1.0, 100.0); // E < \mu -> occupation ~ 1
    assert!(fd > 0.99);

    // Bose-Einstein
    let be = bose_einstein(2.0, 1.0, 1.0).unwrap();
    assert!(be > 0.0);
}
