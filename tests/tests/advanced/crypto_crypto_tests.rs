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
