use algebra_engine::hyperop::{ackermann, hyperoperation, knuth_up_arrow, super_log, tetration};
use algebra_engine::tropical::{log_sum_exp, MaxPlus, MinPlus, TropicalMatrix};
use num_bigint::BigUint;

#[test]
fn test_knuth_up_arrow_and_tetration() {
    // 2 \uparrow 3 = 2^3 = 8
    assert_eq!(knuth_up_arrow(2, 1, 3), BigUint::from(8u32));

    // 2 \uparrow\uparrow 3 = 2^(2^2) = 2^4 = 16
    assert_eq!(tetration(2, 3), BigUint::from(16u32));
    assert_eq!(knuth_up_arrow(2, 2, 3), BigUint::from(16u32));

    // 3 \uparrow\uparrow 2 = 3^3 = 27
    assert_eq!(tetration(3, 2), BigUint::from(27u32));

    // Hyperoperation levels
    assert_eq!(hyperoperation(1, 2, 3), BigUint::from(5u32)); // 2 + 3 = 5
    assert_eq!(hyperoperation(2, 2, 3), BigUint::from(6u32)); // 2 * 3 = 6
    assert_eq!(hyperoperation(3, 2, 3), BigUint::from(8u32)); // 2^3 = 8
    assert_eq!(hyperoperation(4, 2, 3), BigUint::from(16u32)); // 2 ^^ 3 = 16
}

#[test]
fn test_ackermann_function() {
    assert_eq!(ackermann(0, 0), 1);
    assert_eq!(ackermann(1, 2), 4);
    assert_eq!(ackermann(2, 2), 7);
    assert_eq!(ackermann(3, 2), 29);
}

#[test]
fn test_super_log() {
    // slog_2(16) = 3 because 2 ^^ 3 = 16
    let slog = super_log(2.0, 16.0);
    assert!((slog - 3.0).abs() < 1e-6);
}

#[test]
fn test_tropical_semirings() {
    // Max-Plus: a \oplus b = max(a, b), a \otimes b = a + b
    let a = MaxPlus::Val(3.0);
    let b = MaxPlus::Val(5.0);
    assert_eq!(a + b, MaxPlus::Val(5.0));
    assert_eq!(a * b, MaxPlus::Val(8.0));

    // Min-Plus: a \oplus b = min(a, b), a \otimes b = a + b
    let c = MinPlus::Val(3.0);
    let d = MinPlus::Val(5.0);
    assert_eq!(c + d, MinPlus::Val(3.0));
    assert_eq!(c * d, MinPlus::Val(8.0));

    // Tropical Matrix Multiplication
    let mut m1 = TropicalMatrix::new(2, 2, MinPlus::zero());
    m1.set(0, 0, MinPlus::Val(0.0));
    m1.set(0, 1, MinPlus::Val(3.0));
    m1.set(1, 0, MinPlus::Val(1.0));
    m1.set(1, 1, MinPlus::Val(0.0));

    let m2 = m1.clone();
    let res = m1.mul(&m2).unwrap();
    // res(0, 1) = min(0+3, 3+0) = 3
    assert_eq!(res.get(0, 1), MinPlus::Val(3.0));

    // Log-Sum-Exp smooth tropical
    let lse = log_sum_exp(3.0, 5.0, 0.01);
    assert!((lse - 5.0).abs() < 0.05);
}
