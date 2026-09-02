use algebra_core::ExprGraph;

#[test]
fn test_graph_deduplication() {
    let graph = ExprGraph::new();
    let x1 = graph.symbol("x");
    let x2 = graph.symbol("x");
    assert_eq!(x1, x2);
    assert_eq!(graph.len(), 1);

    let two = graph.integer(2);
    let mul1 = graph.mul([two, x1]);
    let mul2 = graph.mul([two, x2]);
    assert_eq!(mul1, mul2);
    assert_eq!(graph.len(), 3); // x, 2, 2*x
}
