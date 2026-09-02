use algebra_advanced::codegen::{CodeGenerator, TargetLanguage};
use algebra_core::ExprGraph;

#[test]
fn test_emit_rust_code() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");

    // x^2 + 5
    let two = graph.integer(2);
    let five = graph.integer(5);
    let x_sq = graph.pow(x, two);
    let expr = graph.add([x_sq, five]);

    let rust_code = CodeGenerator::emit_code(&graph, expr, TargetLanguage::Rust).unwrap();
    assert_eq!(rust_code, "(x.powf(2.0) + 5.0)");
}

#[test]
fn test_compile_closure_evaluation() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    // f(x) = x + 3
    let three = graph.integer(3);
    let expr = graph.add([x, three]);

    let f = CodeGenerator::compile_fn1(&graph, expr, x_sym);
    let result = f(10.0);

    assert_eq!(result, 13.0);
}
