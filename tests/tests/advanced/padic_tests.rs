use algebra_advanced::padic::{HenselLifter, PadicNumber};

#[test]
fn test_padic_valuation_and_norm() {
    // 75 = 3 * 5^2 -> in Q_5, valuation is 2, norm is 5^(-2) = 1/25 = 0.04
    let num_75 = PadicNumber::from_rational(75, 1, 5, 5).unwrap();
    assert_eq!(num_75.valuation, 2);
    assert!((num_75.norm() - 0.04).abs() < 1e-10);

    // 1/3 in Q_5: 3 * x = 1 (mod 5) -> x = 2 (mod 5)
    let one_third = PadicNumber::from_rational(1, 3, 5, 5).unwrap();
    assert_eq!(one_third.valuation, 0);
    assert_eq!(one_third.digits[0], 2); // 3 * 2 = 6 = 1 mod 5
}

#[test]
fn test_padic_ultrametric_inequality() {
    let a = PadicNumber::from_rational(15, 1, 5, 5).unwrap(); // val 1, norm 0.2
    let b = PadicNumber::from_rational(25, 1, 5, 5).unwrap(); // val 2, norm 0.04
    assert!(a.verify_ultrametric(&b));
}

#[test]
fn test_hensel_lemma_root_lifting() {
    // P(x) = x^2 - 2 in Z_7
    // Coeffs: [1, 0, -2]
    // 3^2 = 9 = 2 mod 7 -> initial root r0 = 3 (or 4)
    let p_root = HenselLifter::lift_root(&[1, 0, -2], 3, 7, 4).unwrap();
    assert_eq!(p_root.p, 7);
    assert_eq!(p_root.digits[0], 3);

    // Verify root modulo 7^2 = 49
    let r_approx = p_root.digits[0] + p_root.digits[1] * 7;
    let residual = (r_approx as i64 * r_approx as i64 - 2) % 49;
    assert_eq!(residual, 0);
}
