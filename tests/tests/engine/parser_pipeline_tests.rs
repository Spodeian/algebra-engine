//! End-to-end algebraic pipeline workspace integration tests.
//!
//! Verifies interaction across:
//! `algebra-parser` -> `algebra-core` -> `algebra-simplify` -> `algebra-numeric` -> `algebra-format`

use algebra_core::format::{Formatter, LatexFormatter};
use algebra_core::parser::ExprParser;
use algebra_core::{Constant, ExprGraph};
use algebra_engine::numeric::{BigValue, EvalContext, NumericalEval};
use algebra_engine::simplify::Simplifier;

#[test]
fn test_full_pipeline_parse_simplify_eval_format() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);

    // 1. Parse string expression: x + 0
    let expr = parser.parse("x + 0").unwrap();

    // 2. Simplify using egg e-graph rules (x + 0 => x)
    let simplified = Simplifier::simplify(&graph, expr).unwrap();
    assert_eq!(simplified, graph.symbol("x"));

    // 3. Format as LaTeX
    let latex = LatexFormatter.format(&graph, simplified).unwrap();
    assert_eq!(latex, "x");
}

#[test]
fn test_numerical_reduction_pipeline() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);

    // Parse numerical expression: 2 * 3 + 4 * 5
    let expr = parser.parse("2 * 3 + 4 * 5").unwrap();

    // Evaluate numerically
    let ctx = EvalContext::default();
    let val = graph.evalf(expr, &ctx).unwrap();
    assert_eq!(val, BigValue::Real(26.0));
}

#[test]
fn test_transcendental_constant_pipeline() {
    let graph = ExprGraph::new();

    // Build sqrt(pi^2)
    let pi = graph.constant(Constant::Pi);
    let two = graph.integer(2);
    let pi_sq = graph.pow(pi, two);
    let sqrt_pi_sq = graph.function("sqrt", [pi_sq]);

    // Format LaTeX
    let latex = LatexFormatter.format(&graph, sqrt_pi_sq).unwrap();
    assert_eq!(latex, "\\operatorname{sqrt}\\left({\\pi}^{2}\\right)");

    // Evaluate
    let ctx = EvalContext::default();
    let val = graph.evalf(sqrt_pi_sq, &ctx).unwrap();
    if let BigValue::Real(r) = val {
        assert!((r - std::f64::consts::PI).abs() < 1e-10);
    } else {
        panic!("Expected real value");
    }
}
