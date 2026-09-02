use algebra_engine::combinatorics::{
    bell_number, catalan_number, combinations, derangements, factorial, integer_partitions,
    permutations, stirling_second_kind,
};
use algebra_engine::control::StateSpaceSystem;
use algebra_engine::numbertheory::{extended_gcd, is_prime, legendre_symbol, ContinuedFraction};
use algebra_engine::topology::SimplicialComplex;
use num_bigint::BigUint;

#[test]
fn test_combinatorics() {
    assert_eq!(factorial(5), BigUint::from(120u32));
    assert_eq!(combinations(5, 2), BigUint::from(10u32));
    assert_eq!(permutations(5, 2), BigUint::from(20u32));
    assert_eq!(derangements(4), BigUint::from(9u32));
    assert_eq!(catalan_number(3), BigUint::from(5u32));
    assert_eq!(catalan_number(4), BigUint::from(14u32));

    // Partitions of 4: (4, 3+1, 2+2, 2+1+1, 1+1+1+1) -> p(4) = 5
    assert_eq!(integer_partitions(4), BigUint::from(5u32));
    // Partitions of 5: p(5) = 7
    assert_eq!(integer_partitions(5), BigUint::from(7u32));

    assert_eq!(stirling_second_kind(4, 2), BigUint::from(7u32));
    assert_eq!(bell_number(3), BigUint::from(5u32));
}

#[test]
fn test_simplicial_topology() {
    let mut complex = SimplicialComplex::new();
    // Triangle (0, 1, 2)
    complex.add_simplex(&[0, 1, 2]);

    assert_eq!(complex.count_simplices(0), 3); // 3 vertices
    assert_eq!(complex.count_simplices(1), 3); // 3 edges
    assert_eq!(complex.count_simplices(2), 1); // 1 2-face

    // Euler characteristic for filled triangle: 3 - 3 + 1 = 1
    assert_eq!(complex.euler_characteristic(), 1);
}

#[test]
fn test_control_state_space() {
    let a = vec![vec![-2.0, 1.0], vec![0.0, -3.0]];
    let b = vec![vec![1.0], vec![1.0]];
    let c = vec![vec![1.0, 0.0]];
    let d = vec![vec![0.0]];

    let sys = StateSpaceSystem::new(a, b, c, d);
    assert_eq!(sys.is_stable_2x2(), Some(true));

    let ctrl_mat = sys.controllability_matrix_2x2().unwrap();
    assert_eq!(ctrl_mat.len(), 2);
}

#[test]
fn test_number_theory() {
    assert!(is_prime(17));
    assert!(is_prime(101));
    assert!(!is_prime(100));

    // Extended GCD: 30*x + 20*y = 10
    let (gcd, x, y) = extended_gcd(30, 20);
    assert_eq!(gcd, 10);
    assert_eq!(30 * x + 20 * y, 10);

    // Continued fraction of sqrt(2) approx 1.41421356 -> [1; 2, 2, 2, ...]
    let cf = ContinuedFraction::from_f64(std::f64::consts::SQRT_2, 5);
    assert_eq!(cf.terms[0], 1);
    assert_eq!(cf.terms[1], 2);

    // Legendre symbol (2/7) = 1 (2 is a quadratic residue mod 7: 3^2 = 9 = 2 mod 7)
    assert_eq!(legendre_symbol(2, 7), 1);
    // (3/7) = -1 (3 is not a quadratic residue mod 7)
    assert_eq!(legendre_symbol(3, 7), -1);
}
