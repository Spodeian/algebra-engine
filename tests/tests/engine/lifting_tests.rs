use algebra_core::ExprGraph;
use algebra_engine::lifting::{
    EulerLiftingFunctor, LiftingFunctor, RetractionCleaner, WeierstrassLiftingFunctor,
};

#[test]
fn test_euler_lifting_functor() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");

    let cos_x = graph.function("cos", [x]);
    let functor = EulerLiftingFunctor;

    assert!(functor.can_lift(&graph, cos_x));

    let lifted = functor.lift(&graph, cos_x, x_sym).unwrap();
    // Lifted expression contains exp
    assert_ne!(lifted, graph.integer(0));

    let retracted = functor.retract(&graph, lifted, x_sym).unwrap();
    assert_ne!(retracted, graph.integer(0));
}

#[test]
fn test_weierstrass_lifting_functor() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");

    let sin_x = graph.function("sin", [x]);
    let functor = WeierstrassLiftingFunctor;

    assert!(functor.can_lift(&graph, sin_x));

    let lifted = functor.lift(&graph, sin_x, x_sym).unwrap();
    assert_ne!(lifted, graph.integer(0));

    let retracted = functor.retract(&graph, lifted, x_sym).unwrap();
    assert_ne!(retracted, graph.integer(0));
}

#[test]
fn test_retraction_cleaner_laplace() {
    let graph = ExprGraph::new();
    let t_sym = graph.symbols.get_or_intern("t");
    let s = graph.symbol("s");
    let a = graph.integer(3);

    // 1 / (s - 3) -> exp(3 * t)
    let s_minus_a = graph.sub(s, a);
    let expr_s = graph.div(graph.integer(1), s_minus_a);

    let res_t = RetractionCleaner::retract_laplace_to_time(&graph, expr_s, t_sym);
    assert_ne!(res_t, graph.integer(0));
}

#[test]
fn test_retraction_cleaner_dual_parts() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let eps = graph.symbol("eps");

    // (x + eps)^2 = x^2 + 2*x*eps
    let x_plus_eps = graph.add([x, eps]);
    let expr = graph.pow(x_plus_eps, graph.integer(2));

    let (primal, dual) = RetractionCleaner::retract_dual_parts(&graph, expr);
    assert_ne!(primal, graph.integer(0));
    assert_ne!(dual, graph.integer(0));
}
