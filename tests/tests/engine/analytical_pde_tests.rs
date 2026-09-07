use algebra_core::format::{Formatter, LatexFormatter, UnicodeFormatter};
use algebra_core::operation::PdeOpKind;
use algebra_core::parser::ExprParser;
use algebra_core::ExprGraph;
use algebra_engine::executor::{ExecutionContext, OperationExecutor};
use algebra_engine::numeric::{EvalContext, NumericalEval};
use algebra_engine::solver::SymbolicSolver;

#[test]
fn test_wave_dalembert_exact_symbolic() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");

    // u(x, 0) = x^2, u_t(x, 0) = 0, speed = 2
    let f_init = parser.parse("x^2").unwrap();
    let c_node = graph.integer(2);

    let sol = graph.solve_pde_wave_dalembert(f_init, None, c_node, x_sym, t_sym).unwrap();
    let unicode_fmt = UnicodeFormatter;
    let sol_str = unicode_fmt.format(&graph, sol).unwrap();

    // Verify at x=3, t=1: u(3, 1) = 0.5 * ((3 - 2)^2 + (3 + 2)^2) = 0.5 * (1 + 25) = 13
    let mut ctx = EvalContext::default();
    ctx.bindings.insert("x".to_string(), 3.0);
    ctx.bindings.insert("t".to_string(), 1.0);
    let val = graph.evalf(sol, &ctx).unwrap().to_f64();
    assert!((val - 13.0).abs() < 1e-9, "d'Alembert wave at (3, 1) should be 13, got: {} (expression: {})", val, sol_str);
}

#[test]
fn test_wave_separated_modes() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");
    let c_node = graph.symbol("c");

    let (x_mode, t_mode) = graph.solve_pde_wave_1d(c_node, x_sym, t_sym).unwrap();
    let unicode_fmt = UnicodeFormatter;
    let x_str = unicode_fmt.format(&graph, x_mode).unwrap();
    let t_str = unicode_fmt.format(&graph, t_mode).unwrap();

    assert!(x_str.contains("sin"), "Spatial mode must be sinusoidal, got: {}", x_str);
    assert!(t_str.contains("cos"), "Temporal mode must be oscillatory cos, got: {}", t_str);
}

#[test]
fn test_laplace_separated_harmonic_modes() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let y_sym = graph.symbols.get_or_intern("y");
    let a_node = graph.symbol("a");

    let (x_mode, y_mode) = graph.solve_pde_laplace_2d(a_node, x_sym, y_sym).unwrap();
    let unicode_fmt = UnicodeFormatter;
    let x_str = unicode_fmt.format(&graph, x_mode).unwrap();
    let y_str = unicode_fmt.format(&graph, y_mode).unwrap();

    assert!(x_str.contains("sin"), "Laplace X mode must be sin, got: {}", x_str);
    assert!(y_str.contains("sinh"), "Laplace Y mode must be hyperbolic sinh, got: {}", y_str);
}

#[test]
fn test_transport_characteristic_solution() {
    let graph = ExprGraph::new();
    let parser = ExprParser::new(&graph);
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");

    // u_t + 3 u_x = 0, u(x, 0) = x^3 -> u(x, t) = (x - 3t)^3
    let f_init = parser.parse("x^3").unwrap();
    let c_node = graph.integer(3);

    let sol = graph.solve_pde_transport_1d(f_init, c_node, x_sym, t_sym).unwrap();

    // Verify at x=7, t=2: (7 - 6)^3 = 1^3 = 1
    let mut ctx = EvalContext::default();
    ctx.bindings.insert("x".to_string(), 7.0);
    ctx.bindings.insert("t".to_string(), 2.0);
    let val = graph.evalf(sol, &ctx).unwrap().to_f64();
    assert!((val - 1.0).abs() < 1e-9, "Transport characteristic at (7, 2) should be 1, got: {}", val);
}

#[test]
fn test_radial_bessel_eigenmode_and_evaluation() {
    let graph = ExprGraph::new();
    let r_sym = graph.symbols.get_or_intern("r");
    let k_node = graph.integer(2);

    let bessel_sol = graph.solve_pde_radial_bessel(k_node, r_sym, 0).unwrap();
    let unicode_fmt = UnicodeFormatter;
    let latex_fmt = LatexFormatter;

    let uni_str = unicode_fmt.format(&graph, bessel_sol).unwrap();
    let lat_str = latex_fmt.format(&graph, bessel_sol).unwrap();

    assert!(uni_str.contains("J_0"), "Unicode should display J_0, got: {}", uni_str);
    assert!(lat_str.contains("J_{0}"), "LaTeX should display J_{{0}}, got: {}", lat_str);

    // J_0(0) = 1.0
    let mut ctx = EvalContext::default();
    ctx.bindings.insert("r".to_string(), 0.0);
    let val0 = graph.evalf(bessel_sol, &ctx).unwrap().to_f64();
    assert!((val0 - 1.0).abs() < 1e-9, "Bessel J_0(0) must equal 1, got: {}", val0);

    // Test J_1(0) = 0.0
    let bessel_j1 = graph.solve_pde_radial_bessel(k_node, r_sym, 1).unwrap();
    let val1 = graph.evalf(bessel_j1, &ctx).unwrap().to_f64();
    assert!(val1.abs() < 1e-9, "Bessel J_1(0) must equal 0, got: {}", val1);
}

#[test]
fn test_executor_pde_dispatch() {
    let graph = ExprGraph::new();
    let ctx = ExecutionContext::new();

    let wave_op = algebra_core::operation::MathOperation::Pde(PdeOpKind::Wave1D {
        speed: "3".into(),
        x_var: "x".into(),
        t_var: "t".into(),
        initial_pos: Some("sin(x)".into()),
        initial_vel: None,
    });

    let res = OperationExecutor::execute(&graph, &wave_op, &ctx);
    assert!(!res.is_error, "Execution of PdeOpKind::Wave1D must succeed: {:?}", res.error_msg);
    assert!(res.output_text.contains("Wave Equation d'Alembert Solution"));

    let bessel_op = algebra_core::operation::MathOperation::Pde(PdeOpKind::RadialBessel {
        wave_num: "5".into(),
        r_var: "r".into(),
        order: 2,
    });
    let res_bessel = OperationExecutor::execute(&graph, &bessel_op, &ctx);
    assert!(!res_bessel.is_error, "Execution of PdeOpKind::RadialBessel must succeed: {:?}", res_bessel.error_msg);
    assert!(res_bessel.output_text.contains("Bessel"));
}
