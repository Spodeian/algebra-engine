use algebra_core::Number;

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
