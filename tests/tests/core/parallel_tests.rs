use algebra_core::{ExprGraph, ExprKind};

#[test]
fn test_parallel_filter_and_search() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let y = graph.symbol("y");
    let two = graph.integer(2);
    let expr = graph.add([graph.mul([two, x]), y]);

    let symbols = graph.par_filter_nodes(|node| matches!(node.kind, ExprKind::Symbol(_)));
    assert_eq!(symbols.len(), 2); // x and y

    let contains_y = graph.contains_matching(expr, &|node| match &node.kind {
        ExprKind::Symbol(sym) => graph.symbols.resolve(*sym) == Some("y".into()),
        _ => false,
    });
    assert!(contains_y);
}
