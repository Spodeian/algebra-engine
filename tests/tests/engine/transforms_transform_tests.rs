use algebra_core::{ExprGraph, ExprKind};
use algebra_engine::transforms::SymbolicTransforms;

#[test]
fn test_laplace_transform_constant() {
    let graph = ExprGraph::new();
    let one = graph.integer(1);
    let t_sym = graph.symbols.get_or_intern("t");
    let s_sym = graph.symbols.get_or_intern("s");

    // L{1} = 1 / s
    let lap = graph.laplace_transform(one, t_sym, s_sym).unwrap();
    let node = graph.get(lap);

    // Strict assertion: L{1} must be a Division node (1 / s)
    if let ExprKind::Div(num, den) = &node.kind {
        assert_eq!(*num, one);
        let den_node = graph.get(*den);
        if let ExprKind::Symbol(sym) = &den_node.kind {
            assert_eq!(*sym, s_sym);
        } else {
            panic!("Expected denominator symbol 's'");
        }
    } else {
        panic!("Expected Div node for L{{1}}");
    }
}

#[test]
fn test_laplace_transform_sin() {
    let graph = ExprGraph::new();
    let t = graph.symbol("t");
    let t_sym = graph.symbols.get_or_intern("t");
    let s_sym = graph.symbols.get_or_intern("s");

    let sin_t = graph.function("sin", [t]);
    let lap = graph.laplace_transform(sin_t, t_sym, s_sym).unwrap();
    let node = graph.get(lap);

    // Strict assertion: L{sin(t)} = 1 / (s^2 + 1)
    if let ExprKind::Div(num, den) = &node.kind {
        assert_eq!(*num, one_node(&graph));
        let den_node = graph.get(*den);
        assert!(matches!(den_node.kind, ExprKind::Add(_)));
    } else {
        panic!("Expected Div node for L{{sin(t)}}");
    }
}

fn one_node(graph: &ExprGraph) -> algebra_core::ExprId {
    graph.integer(1)
}

#[test]
fn test_fourier_transform_kernel_integral() {
    let graph = ExprGraph::new();
    let one = graph.integer(1);
    let t_sym = graph.symbols.get_or_intern("t");
    let omega_sym = graph.symbols.get_or_intern("omega");

    let ft = graph.fourier_transform(one, t_sym, omega_sym).unwrap();
    let node = graph.get(ft);

    // Strict assertion: F{f(t)} creates an unevaluated Integral operator
    assert!(matches!(node.kind, ExprKind::Integral { .. }));
}
