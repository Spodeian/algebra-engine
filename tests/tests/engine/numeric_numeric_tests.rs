use algebra_core::{Constant, ExprGraph};
use algebra_engine::numeric::{BigValue, EvalContext, NumericalEval};
use std::f64::consts;

#[test]
fn test_evalf_basic() {
    let graph = ExprGraph::new();
    let two = graph.integer(2);
    let three = graph.integer(3);
    let add = graph.add([two, three]);

    let ctx = EvalContext::default();
    let res = graph.evalf(add, &ctx).unwrap();
    assert_eq!(res, BigValue::Real(5.0));
}

#[test]
fn test_evalf_pi_sqrt() {
    let graph = ExprGraph::new();
    let pi = graph.constant(Constant::Pi);
    let sqrt_pi = graph.function("sqrt", [pi]);

    let ctx = EvalContext::default();
    let res = graph.evalf(sqrt_pi, &ctx).unwrap();
    if let BigValue::Real(v) = res {
        assert!((v - consts::PI.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected real value");
    }
}
