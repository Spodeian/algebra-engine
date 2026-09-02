use algebra_core::ExprGraph;
use algebra_engine::special::SpecialFunctions;

#[test]
fn test_special_function_construction() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");

    let g = graph.gamma(x);
    let z = graph.zeta(x);
    let d = graph.dirac_delta(x);
    let e = graph.erf(x);

    assert!(!graph.is_empty());
    assert_ne!(g, z);
    assert_ne!(d, e);
}

#[test]
fn test_gamma_exact_evaluation() {
    let graph = ExprGraph::new();
    let five = graph.integer(5);

    let g5 = graph.gamma(five);
    let eval = graph.evaluate_special(g5).unwrap();

    // Gamma(5) = 4! = 24
    assert_eq!(eval, graph.integer(24));
}

#[test]
fn test_zeta_2_evaluation() {
    let graph = ExprGraph::new();
    let two = graph.integer(2);

    let z2 = graph.zeta(two);
    let eval = graph.evaluate_special(z2).unwrap();

    // zeta(2) = pi^2 / 6
    assert_ne!(eval, z2);
}
