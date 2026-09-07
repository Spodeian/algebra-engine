use algebra_core::format::{Formatter, LatexFormatter, UnicodeFormatter};
use algebra_core::parser::ExprParser;
use algebra_core::{ExprGraph, ExprKind, Number};
use algebra_engine::numeric::{EvalContext, NumericalEval};
use algebra_engine::solver::SymbolicSolver;

#[test]
fn test_binomial_radicals_cubic() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^3 - 8 = 0
    let eq = parser.parse("x^3 - 8").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    // Must find real root 2 and complex roots
    assert!(!roots.is_empty());
    let mut found_two = false;
    for &r in &roots {
        if let ExprKind::Number(Number::Integer(2)) = graph.get(r).kind {
            found_two = true;
        }
    }
    assert!(found_two, "Cubic binomial x^3 - 8 must have exact integer root 2");
}

#[test]
fn test_binomial_radicals_quartic() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^4 - 16 = 0
    let eq = parser.parse("x^4 - 16").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 4, "Quartic binomial x^4 - 16 must yield 4 roots: 2, -2, 2i, -2i");
    let unicode_fmt = UnicodeFormatter;
    let root_strs: Vec<String> = roots.iter().map(|&r| unicode_fmt.format(&graph, r).unwrap()).collect();
    assert!(root_strs.contains(&"2".to_string()));
    assert!(root_strs.contains(&"-2".to_string()));
}

#[test]
fn test_biquadratic_nested_radicals() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^4 - 5*x^2 + 4 = 0 -> u = x^2, u in {1, 4} -> x in {1, -1, 2, -2}
    let eq = parser.parse("x^4 - 5*x^2 + 4").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 4, "Biquadratic x^4 - 5x^2 + 4 must yield exactly 4 exact integer roots");
    let mut vals = Vec::new();
    for &r in &roots {
        if let ExprKind::Number(Number::Integer(i)) = graph.get(r).kind {
            vals.push(i);
        }
    }
    vals.sort();
    assert_eq!(vals, vec![-2, -1, 1, 2]);
}

#[test]
fn test_biquadratic_irrational_nested_radicals() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^4 - 2*x^2 - 1 = 0 -> u = 1 +- sqrt(2) -> x = +-sqrt(1 + sqrt(2)), +-i*sqrt(sqrt(2) - 1)
    let eq = parser.parse("x^4 - 2*x^2 - 1").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 4, "Biquadratic x^4 - 2x^2 - 1 must produce 4 exact nested radical roots");
}

#[test]
fn test_cardano_cubic_rational_factorization() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^3 - 6*x^2 + 11*x - 6 = (x-1)(x-2)(x-3) = 0
    let eq = parser.parse("x^3 - 6*x^2 + 11*x - 6").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 3, "Factored cubic x^3 - 6x^2 + 11x - 6 must yield {{1, 2, 3}}");
    let mut vals = Vec::new();
    for &r in &roots {
        if let ExprKind::Number(Number::Integer(i)) = graph.get(r).kind {
            vals.push(i);
        }
    }
    vals.sort();
    assert_eq!(vals, vec![1, 2, 3]);
}

#[test]
fn test_bring_radical_exact_and_numerical() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // x^5 + x + 1 = 0 -> root is BR(1)
    let eq = parser.parse("x^5 + x + 1").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 1, "Bring radical quintic must yield exact BR(1) root");
    let unicode_fmt = UnicodeFormatter;
    let latex_fmt = LatexFormatter;

    let br_str = unicode_fmt.format(&graph, roots[0]).unwrap();
    assert!(br_str.contains("BR"), "Unicode representation should display BR(1), got: {}", br_str);

    let latex_str = latex_fmt.format(&graph, roots[0]).unwrap();
    assert!(latex_str.contains("\\operatorname{BR}"), "LaTeX should display \\operatorname{{BR}}, got: {}", latex_str);

    // Verify numerical evaluation of BR(1)
    let ctx = EvalContext::default();
    let num_val = graph.evalf(roots[0], &ctx).unwrap().to_f64();
    // Known real root of x^5 + x + 1 = 0 is approx -0.7548776662466927
    assert!((num_val - (-0.7548776662)).abs() < 1e-6, "Numerical evaluation of BR(1) was {}", num_val);

    let residual = num_val.powi(5) + num_val + 1.0;
    assert!(residual.abs() < 1e-9, "Residual of Bring radical root must be < 1e-9, got: {}", residual);
}

#[test]
fn test_higher_degree_rootof_fallback() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");

    // Irreducible degree 6 polynomial: x^6 + x + 1 = 0
    let eq = parser.parse("x^6 + x + 1").unwrap();
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 6, "Degree 6 polynomial must return 6 exact RootOf branches");
    let unicode_fmt = UnicodeFormatter;
    let r0_str = unicode_fmt.format(&graph, roots[0]).unwrap();
    assert!(r0_str.contains("RootOf"), "Must represent unsolved algebraic roots symbolically as RootOf, got: {}", r0_str);
}
