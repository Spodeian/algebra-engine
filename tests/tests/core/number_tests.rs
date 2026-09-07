use algebra_core::Number;
use num_bigint::BigInt;

#[test]
fn test_number_properties() {
    assert!(Number::Integer(0).is_zero());
    assert!(Number::Rational(0, 5).is_zero());
    assert!(Number::Integer(1).is_one());
    assert!(Number::Rational(7, 7).is_one());
    assert_eq!(
        Number::float(std::f64::consts::PI).as_f64().unwrap(),
        std::f64::consts::PI
    );
}

#[test]
fn test_number_lossless_decimal_and_scientific_parsing() {
    // "0.5" -> Rational(1, 2)
    let half = Number::from_decimal_str("0.5").unwrap();
    assert_eq!(half, Number::Rational(1, 2));
    assert!(half.is_lossless());

    // "0.125" -> Rational(1, 8)
    let eighth = Number::from_decimal_str("0.125").unwrap();
    assert_eq!(eighth, Number::Rational(1, 8));

    // "1.25" -> Rational(5, 4)
    let five_quarters = Number::from_decimal_str("1.25").unwrap();
    assert_eq!(five_quarters, Number::Rational(5, 4));

    // "1.5e-3" -> Rational(3, 2000)
    let sci = Number::from_decimal_str("1.5e-3").unwrap();
    assert_eq!(sci, Number::Rational(3, 2000));

    // "2.0" -> Integer(2)
    let two = Number::from_decimal_str("2.0").unwrap();
    assert_eq!(two, Number::Integer(2));
}

#[test]
fn test_number_lossless_f64_and_f32_decomposition() {
    // 0.5f64 -> Rational(1, 2)
    let r1 = Number::from_f64_lossless(0.5);
    assert_eq!(r1, Number::Rational(1, 2));

    // 0.25f32 -> Rational(1, 4)
    let r2 = Number::from_f32_lossless(0.25f32);
    assert_eq!(r2, Number::Rational(1, 4));

    // 0.3333333333333333f64 -> Rational(1, 3) via continued fraction convergent
    let r3 = Number::from_f64_lossless(1.0 / 3.0);
    assert_eq!(r3, Number::Rational(1, 3));

    // 0.2f64 -> Rational(1, 5)
    let r4 = Number::from_f64_lossless(0.2);
    assert_eq!(r4, Number::Rational(1, 5));
}

#[test]
fn test_number_exact_arithmetic_and_auto_promotion() {
    // Rational addition: 1/2 + 1/3 = 5/6
    let half = Number::Rational(1, 2);
    let third = Number::Rational(1, 3);
    let sum = half.add_lossless(&third).unwrap();
    assert_eq!(sum, Number::Rational(5, 6));

    // Rational multiplication: (3/4) * (2/3) = 1/2
    let three_fourths = Number::Rational(3, 4);
    let two_thirds = Number::Rational(2, 3);
    let prod = three_fourths.mul_lossless(&two_thirds).unwrap();
    assert_eq!(prod, Number::Rational(1, 2));

    // Integer overflow auto-promotes to BigInteger
    let max = Number::Integer(i64::MAX);
    let one = Number::Integer(1);
    let promoted = max.add_lossless(&one).unwrap();
    match promoted {
        Number::BigInteger(b) => {
            assert_eq!(b, BigInt::from(i64::MAX) + 1i32);
        }
        _ => panic!("Expected auto-promotion to BigInteger on integer overflow!"),
    }
}
