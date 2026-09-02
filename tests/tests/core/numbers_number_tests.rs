use algebra_core::numbers::{Quaternion, SurrealNumber};
use num_rational::BigRational;
use num_traits::{One, Zero};

#[test]
fn test_quaternion_hamilton_mul() {
    // i * j = k
    let i = Quaternion::new(
        BigRational::zero(),
        BigRational::one(),
        BigRational::zero(),
        BigRational::zero(),
    );
    let j = Quaternion::new(
        BigRational::zero(),
        BigRational::zero(),
        BigRational::one(),
        BigRational::zero(),
    );
    let k = Quaternion::new(
        BigRational::zero(),
        BigRational::zero(),
        BigRational::zero(),
        BigRational::one(),
    );

    assert_eq!(i.mul(&j), k);
}

#[test]
fn test_surreal_construction() {
    let zero = SurrealNumber::zero();
    let one = SurrealNumber::one();
    assert_eq!(zero.left.len(), 0);
    assert_eq!(one.left.len(), 1);
}

#[test]
fn test_dual_number_automatic_differentiation() {
    use algebra_core::numbers::DualNumber;

    // x = 0.0 with tangent 1.0
    let x = DualNumber::variable(0.0);
    // f(x) = sin(x) => f(0) = 0, f'(0) = cos(0) = 1.0
    let sin_x = x.sin();

    assert_eq!(sin_x.real, 0.0);
    assert_eq!(sin_x.dual, 1.0);
}

#[test]
fn test_hyperreal_standard_part() {
    use algebra_core::numbers::HyperrealNumber;

    let h = HyperrealNumber::new(5.0, 0.001, 0.0);
    assert_eq!(h.standard_part(), 5.0);
}
