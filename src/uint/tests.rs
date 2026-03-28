use super::*;
use num::{BigUint, FromPrimitive, integer::Roots, traits::{CheckedAdd, CheckedSub, Pow}};

#[test]
fn test_promotion_and_demotion() {
    // Start as Machine
    let mut val = Uint::from(usize::MAX);
    assert!(matches!(val, Uint::Machine(_)));

    // Promote via addition
    val += 1usize;
    assert!(matches!(val, Uint::Promoted(_)));

    // Demote via subtraction
    val -= 1usize;
    assert!(matches!(val, Uint::Machine(_)));
    assert_eq!(val, Uint::from(usize::MAX));
}

#[test]
fn test_checked_arithmetic() {
    let a = Uint::from(usize::MAX);
    let b = Uint::from(1usize);

    // CheckedAdd should promote rather than return None
    let sum = a.checked_add(&b).unwrap();
    assert_eq!(sum, Uint::from(BigUint::from(usize::MAX) + 1u8));

    // CheckedSub should handle underflow correctly
    let low = Uint::from(10usize);
    let high = Uint::from(20usize);
    assert!(low.checked_sub(&high).is_none());
    assert_eq!(high.checked_sub(&low).unwrap(), Uint::from(10usize));
}

#[test]
fn test_division_by_zero() {
    let a = Uint::from(100usize);
    let zero = Uint::ZERO;

    let result = std::panic::catch_unwind(|| {
        let _ = a / zero;
    });
    assert!(result.is_err());
}

#[test]
fn test_bit_shifts() {
    let mut val = Uint::from(1usize);

    // Shift into Promoted territory
    val <<= 128usize;
    assert!(matches!(val, Uint::Promoted(_)));

    // Shift back to Machine
    val >>= 128usize;
    assert_eq!(val, Uint::from(1usize));
    assert!(matches!(val, Uint::Machine(1)));
}

#[test]
fn test_roots_lossless() {
    // (2^64)^2 = 2^128
    let big_val = Uint::from(BigUint::from(1u8) << 128);
    let root = big_val.sqrt();

    // Result is 2^64, which is exactly usize::MAX + 1
    assert_eq!(root, Uint::from(BigUint::from(1u8) << 64));

    // Test nth_root
    let cube = Uint::from(27usize);
    assert_eq!(cube.nth_root(3), Uint::from(3usize));
}

#[test]
fn test_iteration() {
    let val = Uint::from_u128(u128::MAX).expect("u128::MAX should convert to Uint::Promoted");
    let limbs: Vec<u64> = val.iter_digits().collect();

    // u128::MAX should yield two 64-bit limbs of all 1s
    assert_eq!(limbs.len(), 2);
    assert_eq!(limbs[0], u64::MAX);
    assert_eq!(limbs[1], u64::MAX);
}

#[test]
fn test_pow() {
    let base = Uint::from(2usize);
    // 2^64 should promote
    let result = base.pow(64u32);
    assert!(matches!(result, Uint::Promoted(_)));
    assert_eq!(result, Uint::from(BigUint::from(1u8) << 64));
}
