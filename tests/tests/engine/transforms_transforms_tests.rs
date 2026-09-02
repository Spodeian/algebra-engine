use algebra_core::ExprGraph;
use algebra_engine::transforms::{SymbolicTransforms, TransferFunction};

#[test]
fn test_transfer_function_lti() {
    let graph = ExprGraph::new();

    let tf = TransferFunction::continuous(&graph, "1", "s^2 + 2*s + 1").unwrap();
    assert!(tf.is_stable());
    assert!(!tf.is_discrete);

    let impulse = tf.impulse_response(&graph).unwrap();
    assert!(
        graph.get(impulse).kind != algebra_core::ExprKind::Number(algebra_core::Number::Integer(0))
    );
}

#[test]
fn test_laplace_transform_basic() {
    let graph = ExprGraph::new();
    let t = graph.symbols.get_or_intern("t");
    let s = graph.symbols.get_or_intern("s");

    let one = graph.integer(1);
    let lap = graph.laplace_transform(one, t, s).unwrap();

    let s_sym = graph.symbol("s");
    let expected = graph.div(one, s_sym);

    assert_eq!(lap, expected);
}
