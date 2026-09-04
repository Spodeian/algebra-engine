use algebra_advanced::finite::{GaloisField, ModuloInt};
use num_bigint::BigInt;

#[test]
fn test_modulo_arithmetic() {
    let m1 = ModuloInt::new(BigInt::from(7), BigInt::from(5)); // 7 mod 5 = 2
    let m2 = ModuloInt::new(BigInt::from(4), BigInt::from(5)); // 4 mod 5 = 4

    let sum = m1.add(&m2).unwrap(); // (2 + 4) mod 5 = 1
    assert_eq!(sum.value, BigInt::from(1));

    let pow = m1.pow(&BigInt::from(3)); // 2^3 mod 5 = 8 mod 5 = 3
    assert_eq!(pow.value, BigInt::from(3));
}

#[test]
fn test_galois_field_cardinality() {
    let gf = GaloisField::new(2, 8); // GF(2^8) AES field
    assert_eq!(gf.cardinality(), 256);
}

#[test]
fn test_inert_primes_to_galois_fields() {
    // p = 3 is inert in Z[i] (3 = 3 mod 4) -> GF(3^2) with modulus x^2 + 1
    let gf3 = GaloisField::from_gaussian_inert_prime(3).expect("3 is inert in Z[i]");
    assert_eq!(gf3.p, 3);
    assert_eq!(gf3.degree, 2);
    assert_eq!(gf3.cardinality(), 9);
    assert_eq!(gf3.modulus_poly, Some(vec![1, 0, 1]));

    // p = 7 is inert in Z[i] (7 = 3 mod 4) -> GF(7^2) with modulus x^2 + 1
    let gf7 = GaloisField::from_gaussian_inert_prime(7).expect("7 is inert in Z[i]");
    assert_eq!(gf7.cardinality(), 49);

    // p = 5 splits in Z[i] (5 = 1 mod 4) -> should return None
    assert!(GaloisField::from_gaussian_inert_prime(5).is_none());

    // p = 2 is inert in Z[omega] (2 = 2 mod 3) -> GF(2^2) with modulus x^2 - x + 1
    let gf2_omega = GaloisField::from_eisenstein_inert_prime(2).expect("2 is inert in Z[omega]");
    assert_eq!(gf2_omega.p, 2);
    assert_eq!(gf2_omega.degree, 2);
    assert_eq!(gf2_omega.cardinality(), 4);
    assert_eq!(gf2_omega.modulus_poly, Some(vec![1, -1, 1]));

    // p = 5 is inert in Z[omega] (5 = 2 mod 3) -> GF(5^2)
    let gf5_omega = GaloisField::from_eisenstein_inert_prime(5).expect("5 is inert in Z[omega]");
    assert_eq!(gf5_omega.cardinality(), 25);

    // p = 7 splits in Z[omega] (7 = 1 mod 3) -> should return None
    assert!(GaloisField::from_eisenstein_inert_prime(7).is_none());
}
