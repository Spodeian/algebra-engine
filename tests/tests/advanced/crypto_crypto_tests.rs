use algebra_advanced::crypto::{EllipticCurve, EllipticPoint, InformationTheory, ReedSolomonCode};
use algebra_core::{ExprGraph, ExprKind};
use num_bigint::BigInt;

#[test]
fn test_elliptic_curve_validation() {
    // Curve: y^2 = x^3 + 2x + 3 (mod 97)
    let a = BigInt::from(2);
    let b = BigInt::from(3);
    let p = BigInt::from(97);

    let curve = EllipticCurve::new(a, b, p).unwrap();

    // Check point (3, 6): 6^2 = 36. 3^3 + 2*3 + 3 = 27 + 6 + 3 = 36.
    let point = EllipticPoint::Point {
        x: BigInt::from(3),
        y: BigInt::from(6),
    };

    assert!(curve.contains_point(&point));
    assert!(curve.contains_point(&EllipticPoint::IdentityAtInfinity));
}

#[test]
fn test_reed_solomon() {
    let rs = ReedSolomonCode::new(255, 223).unwrap();
    assert_eq!(rs.parity_symbols(), 32);
}

#[test]
fn test_information_entropy() {
    let graph = ExprGraph::new();
    let p1 = graph.symbol("p1");
    let p2 = graph.symbol("p2");

    let info = InformationTheory;
    let entropy = info.entropy(&graph, &[p1, p2]);

    let node = graph.get(entropy);
    if let ExprKind::Mul(factors) = &node.kind {
        assert_eq!(factors.len(), 2);
    } else {
        panic!("Expected Mul node for entropy");
    }
}

#[test]
fn test_elliptic_curve_reduction_and_frobenius() {
    use algebra_advanced::crypto::{RationalEllipticCurve, ReductionType};

    // Consider the curve E: y^2 = x^3 - x (Gauss curve)
    // a = -1, b = 0.
    // Discriminant Delta = -16 * (4*(-1)^3 + 27*0) = -16 * (-4) = 64 = 2^6.
    let curve = RationalEllipticCurve::new(-1, 0).unwrap();
    assert_eq!(curve.discriminant(), 64);
    assert_eq!(curve.bad_reduction_places(), vec![2]);

    // Bad reduction at p = 2:
    // c_4 = -48 * (-1) = 48 (divisible by 2) -> Additive reduction
    assert_eq!(curve.reduction_at(2), ReductionType::Additive);

    // Good reduction at all odd primes p >= 3:
    assert_eq!(curve.reduction_at(3), ReductionType::Good);
    assert_eq!(curve.reduction_at(5), ReductionType::Good);
    assert_eq!(curve.reduction_at(7), ReductionType::Good);

    // Frobenius traces a_p = p + 1 - #E(F_p):
    // Over F_3: x = 0 -> 0; x = 1 -> 0; x = 2 -> 2^3 - 2 = 6 = 0 mod 3.
    // Points: (0,0), (1,0), (2,0) + O -> 4 points.
    // a_3 = 3 + 1 - 4 = 0.
    assert_eq!(curve.frobenius_trace(3), Some(0));

    // Over F_5:
    // x = 0 -> 0 (1 pt)
    // x = 1 -> 0 (1 pt)
    // x = 2 -> 8 - 2 = 6 = 1 mod 5 (2 pts: y^2 = 1 -> y = 1, 4)
    // x = 3 -> 27 - 3 = 24 = 4 mod 5 (2 pts: y^2 = 4 -> y = 2, 3)
    // x = 4 -> 64 - 4 = 60 = 0 mod 5 (1 pt)
    // Infinity (1 pt)
    // Total #E(F_5) = 1 + 1 + 2 + 2 + 1 + 1 = 8 points.
    // a_5 = 5 + 1 - 8 = -2.
    assert_eq!(curve.frobenius_trace(5), Some(-2));

    // Over F_7:
    // Gauss curve has CM by Z[i]. For p = 3 mod 4, a_p = 0 always!
    assert_eq!(curve.frobenius_trace(7), Some(0));

    // Reduction mod p yields valid EllipticCurve
    let e5 = curve.reduce_mod_p(5).unwrap();
    assert_eq!(e5.p, num_bigint::BigInt::from(5));
}
