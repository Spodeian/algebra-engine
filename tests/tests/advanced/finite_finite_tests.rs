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
