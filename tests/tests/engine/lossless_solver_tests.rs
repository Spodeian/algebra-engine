//! Integration tests verifying lossless number types, parsing, evaluation, and exact hybrid polynomial solving.

use algebra_core::parser::ExprParser;
use algebra_core::{ExprGraph, ExprKind, Number};
use algebra_engine::numeric::{BigValue, EvalContext, NumericalEval};
use algebra_engine::solver::SymbolicSolver;

#[test]
fn test_lossless_parsing_and_exact_eval() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);

    // 1. Parsing decimal "0.5" creates exact Rational(1, 2)
    let p_half = parser.parse("0.5").unwrap();
    match &graph.get(p_half).kind {
        ExprKind::Number(Number::Rational(num, den)) => {
            assert_eq!((*num, *den), (1, 2));
        }
        other => panic!("Expected Rational(1, 2), got {:?}", other),
    }

    // 2. Parsing decimal "0.125" creates exact Rational(1, 8)
    let p_eighth = parser.parse("0.125").unwrap();
    match &graph.get(p_eighth).kind {
        ExprKind::Number(Number::Rational(num, den)) => {
            assert_eq!((*num, *den), (1, 8));
        }
        other => panic!("Expected Rational(1, 8), got {:?}", other),
    }

    // 3. Parsing scientific notation "1.5e-3" creates exact Rational(3, 2000)
    let p_sci = parser.parse("1.5e-3").unwrap();
    match &graph.get(p_sci).kind {
        ExprKind::Number(Number::Rational(num, den)) => {
            assert_eq!((*num, *den), (3, 2000));
        }
        other => panic!("Expected Rational(3, 2000), got {:?}", other),
    }

    // 4. Parsing "2.0" creates exact Integer(2)
    let p_two = parser.parse("2.0").unwrap();
    match &graph.get(p_two).kind {
        ExprKind::Number(Number::Integer(val)) => {
            assert_eq!(*val, 2);
        }
        other => panic!("Expected Integer(2), got {:?}", other),
    }

    // 5. Rational arithmetic in graph: 1/2 + 1/3 = 5/6
    let half = graph.rational(1, 2);
    let third = graph.rational(1, 3);
    let sum = graph.add([half, third]);
    match &graph.get(sum).kind {
        ExprKind::Number(Number::Rational(num, den)) => {
            assert_eq!((*num, *den), (5, 6));
        }
        other => panic!("Expected Rational(5, 6), got {:?}", other),
    }

    // 6. Explicit numerical evaluation
    let ctx = EvalContext::default();
    let val = graph.evalf(sum, &ctx).unwrap();
    match val {
        BigValue::Real(v) => {
            assert!((v - (5.0 / 6.0)).abs() < 1e-12);
        }
        other => panic!("Expected Real(0.8333...), got {:?}", other),
    }
}

#[test]
fn test_evalf_command_syntax() {
    use algebra_core::parser::parse_operation;
    use algebra_engine::executor::{ExecutionContext, OperationExecutor};

    let graph = ExprGraph::new();
    let ctx = ExecutionContext::new();

    // "evalf 1/2" evaluates to float 0.5 via OperationExecutor
    let op1 = parse_operation("evalf 1/2");
    let res1 = OperationExecutor::execute(&graph, &op1, &ctx);
    assert!(!res1.is_error, "evalf failed: {:?}", res1.error_msg);
    assert!(
        res1.output_text.contains("0.5"),
        "Expected 0.5, got {}",
        res1.output_text
    );

    // "N(1/4)" evaluates to float 0.25
    let op2 = parse_operation("N(1/4)");
    let res2 = OperationExecutor::execute(&graph, &op2, &ctx);
    assert!(!res2.is_error, "N(1/4) failed: {:?}", res2.error_msg);
    assert!(
        res2.output_text.contains("0.25"),
        "Expected 0.25, got {}",
        res2.output_text
    );
}

#[test]
fn test_solveset_exact_radicals_and_complex() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");
    let two = graph.integer(2);

    // 1. Quadratic with perfect square discriminant: x^2 - 4 = 0 -> roots [-2, 2]
    let four = graph.integer(4);
    let x2 = graph.pow(x, two);
    let eq_square = graph.sub(x2, four);
    let roots_square = graph.solveset(eq_square, x_sym).unwrap();
    assert_eq!(roots_square.len(), 2);
    let r1_num = match &graph.get(roots_square[0]).kind {
        ExprKind::Number(Number::Integer(n)) => *n,
        ExprKind::Number(Number::Rational(n, d)) => *n / *d,
        other => panic!("Expected rational/integer root, got {:?}", other),
    };
    let r2_num = match &graph.get(roots_square[1]).kind {
        ExprKind::Number(Number::Integer(n)) => *n,
        ExprKind::Number(Number::Rational(n, d)) => *n / *d,
        other => panic!("Expected rational/integer root, got {:?}", other),
    };
    assert!(
        (r1_num == 2 && r2_num == -2) || (r1_num == -2 && r2_num == 2),
        "Expected roots [2, -2], got [{}, {}]",
        r1_num,
        r2_num
    );

    // 2. Quadratic with non-square discriminant: x^2 - 2 = 0 -> exact radical roots (-b ± sqrt(D))/2a
    let eq_radical = graph.sub(x2, two);
    let roots_radical = graph.solveset(eq_radical, x_sym).unwrap();
    assert_eq!(roots_radical.len(), 2);

    // Verify roots are NOT floating point numbers!
    for &r in &roots_radical {
        assert!(
            !matches!(graph.get(r).kind, ExprKind::Number(Number::Float(_))),
            "Root should be exact symbolic expression, not Float!"
        );
    }

    // Verify numerical evaluation of exact radical roots gives ±sqrt(2)
    let ctx = EvalContext::default();
    let val0 = graph.evalf(roots_radical[0], &ctx).unwrap().to_f64();
    let val1 = graph.evalf(roots_radical[1], &ctx).unwrap().to_f64();
    let sqrt2 = 2.0f64.sqrt();
    assert!(
        (val0.abs() - sqrt2).abs() < 1e-10 && (val1.abs() - sqrt2).abs() < 1e-10,
        "Expected numerical values to be ±sqrt(2), got {}, {}",
        val0,
        val1
    );

    // 3. Quadratic with negative discriminant: x^2 + 1 = 0 -> exact complex radical roots ± i
    let one = graph.integer(1);
    let eq_complex = graph.add([x2, one]);
    let roots_complex = graph.solveset(eq_complex, x_sym).unwrap();
    assert_eq!(roots_complex.len(), 2);

    // Verify roots are NOT floats
    for &r in &roots_complex {
        assert!(
            !matches!(graph.get(r).kind, ExprKind::Number(Number::Float(_))),
            "Complex root should be exact symbolic expression, not Float!"
        );
    }
}

#[test]
fn test_solveset_hybrid_factored_polynomial() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    // Factored polynomial: (x - 1/3) * (x^2 - 5) = 0
    // Root from first factor: exact Rational(1, 3)
    // Roots from second factor: exact radicals ±sqrt(5)
    let one_third = graph.rational(1, 3);
    let f1 = graph.sub(x, one_third);

    let two = graph.integer(2);
    let five = graph.integer(5);
    let x2 = graph.pow(x, two);
    let f2 = graph.sub(x2, five);

    let poly = graph.mul([f1, f2]);
    let roots = graph.solveset(poly, x_sym).unwrap();

    assert_eq!(roots.len(), 3, "Factored polynomial must yield 3 roots");

    // Verify one root is exactly Rational(1, 3)
    let has_exact_third = roots
        .iter()
        .any(|&r| matches!(&graph.get(r).kind, ExprKind::Number(Number::Rational(1, 3))));
    assert!(
        has_exact_third,
        "Expected exact root 1/3 in solveset output"
    );

    // Verify none of the roots are degraded to Float
    for &r in &roots {
        assert!(
            !matches!(graph.get(r).kind, ExprKind::Number(Number::Float(_))),
            "No root should be degraded to Float in hybrid exact solving!"
        );
    }
}
