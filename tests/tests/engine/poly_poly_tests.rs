use algebra_core::SymbolId;
use algebra_engine::poly::Polynomial;
use num_rational::BigRational;
use num_traits::One;

#[test]
fn test_poly_add_mul() {
    let x_sym = SymbolId::new(1);
    let x = Polynomial::variable(x_sym);
    let one = Polynomial::constant(BigRational::one());

    let x_plus_1 = x.add(&one);
    let squared = x_plus_1.mul(&x_plus_1); // (x+1)^2 = x^2 + 2x + 1

    assert_eq!(squared.terms.len(), 3);
}

#[test]
fn test_poly_derivative() {
    let x_sym = SymbolId::new(1);
    let x = Polynomial::variable(x_sym);
    let x_sq = x.mul(&x); // x^2

    let dx = x_sq.derivative(x_sym); // d/dx (x^2) = 2x
    let two_x = x.mul(&Polynomial::constant(BigRational::from_integer(2.into())));

    assert_eq!(dx, two_x);
}
