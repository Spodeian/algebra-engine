use auto_promoters::uint::Uint;
use num::{integer::Roots, traits::Pow, BigUint, Integer, One, Zero};

#[test]
fn test_uint_machine_arithmetic() {
    let a = Uint::from(100usize);
    let b = Uint::from(50usize);

    assert_eq!(a.clone() + b.clone(), Uint::from(150usize));
    assert_eq!(a.clone() - b.clone(), Uint::from(50usize));
    assert_eq!(a.clone() * b.clone(), Uint::from(5000usize));
    assert_eq!(a.clone() / b.clone(), Uint::from(2usize));
    assert_eq!(a.clone() % Uint::from(30usize), Uint::from(10usize));
}

#[test]
fn test_uint_auto_promotion_on_addition_overflow() {
    let max = Uint::from(usize::MAX);
    let one = Uint::from(1usize);

    let promoted = max + one;
    match &promoted {
        Uint::Promoted(big) => {
            assert_eq!(big, &(BigUint::from(usize::MAX) + 1u32));
        }
        _ => panic!("Expected auto-promotion to BigUint on usize overflow!"),
    }
}

#[test]
fn test_uint_auto_promotion_on_multiplication_overflow() {
    let a = Uint::from(usize::MAX);
    let b = Uint::from(2usize);

    let promoted = a * b;
    match &promoted {
        Uint::Promoted(big) => {
            assert_eq!(big, &(BigUint::from(usize::MAX) * 2u32));
        }
        _ => panic!("Expected auto-promotion to BigUint on multiplication overflow!"),
    }
}

#[test]
fn test_uint_demotion_when_fitting_machine() {
    let max = BigUint::from(usize::MAX);
    let promoted = Uint::from(max + 10u32);
    assert!(matches!(promoted, Uint::Promoted(_)));

    // Subtract 20 so it fits back into usize
    let demoted = promoted - Uint::from(20usize);
    match demoted {
        Uint::Machine(val) => {
            assert_eq!(val, usize::MAX - 10);
        }
        _ => panic!("Expected demotion back to Machine word when value fits in usize!"),
    }
}

#[test]
fn test_uint_powers_and_roots() {
    let two = Uint::from(2usize);
    let pow = two.pow(10);
    assert_eq!(pow, Uint::from(1024usize));

    let val = Uint::from(1024usize);
    assert_eq!(val.sqrt(), Uint::from(32usize));
}

#[test]
fn test_uint_comparison_and_ordering() {
    let a = Uint::from(42usize);
    let b = Uint::from(100usize);
    let c = Uint::Promoted(BigUint::from(usize::MAX) + 1u32);

    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
    assert_eq!(a, Uint::from(42usize));
}

#[test]
fn test_uint_formatting_and_traits() {
    let u = Uint::from(255usize);
    assert_eq!(format!("{}", u), "255");
    assert_eq!(format!("{:x}", u), "ff");
    assert_eq!(format!("{:X}", u), "FF");
    assert_eq!(format!("{:b}", Uint::from(5usize)), "101");
    assert_eq!(format!("{:o}", Uint::from(8usize)), "10");

    assert!(Uint::from(0usize).is_zero());
    assert!(Uint::from(1usize).is_one());
    assert!(Uint::from(4usize).is_even());
    assert!(Uint::from(5usize).is_odd());
}
