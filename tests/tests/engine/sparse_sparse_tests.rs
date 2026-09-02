use ahash::AHashMap;
use algebra_core::ExprGraph;
use algebra_engine::sparse::{CsrMatrix, MemoCache, SparsePoly};

#[test]
fn test_csr_matrix_vector_multiply() {
    let graph = ExprGraph::new();
    let a = graph.symbol("a");
    let b = graph.symbol("b");
    let x1 = graph.symbol("x1");
    let x2 = graph.symbol("x2");

    // 2x2 CSR matrix [[a, 0], [0, b]]
    let csr = CsrMatrix::new(2, 2, vec![a, b], vec![0, 1], vec![0, 1, 2]).unwrap();
    let y = csr.multiply_vec(&graph, &[x1, x2]).unwrap();

    assert_eq!(y.len(), 2);
}

#[test]
fn test_sparse_poly_to_expr() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");

    // 3 * x^2
    let three = graph.integer(3);
    let mut terms = AHashMap::new();
    terms.insert(vec![2], three);

    let poly = SparsePoly::new(vec![x_sym], terms);
    let expr = poly.to_expr(&graph);

    assert!(!graph.is_empty());
    assert_ne!(expr, three);
}

#[test]
fn test_memo_cache() {
    let cache = MemoCache::new();
    let expr = algebra_core::ExprId::new(0);
    let wrt = algebra_core::SymbolId::new(0);
    let rep = algebra_core::ExprId::new(1);

    let val1 = cache.get_or_insert_with(expr, wrt, rep, || algebra_core::ExprId::new(42));
    let val2 = cache.get_or_insert_with(expr, wrt, rep, || algebra_core::ExprId::new(99));

    assert_eq!(val1, val2);
    assert_eq!(val1.index(), 42);
}
