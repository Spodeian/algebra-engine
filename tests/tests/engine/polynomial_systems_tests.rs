use algebra_core::ExprGraph;
use algebra_engine::systems::PolynomialSystemSolver;

#[test]
fn test_solve_polynomial_system_intersection() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    // System:
    // x^2 + y^2 - 1 = 0
    // x - y = 0
    // -> Solutions: (1/sqrt(2), 1/sqrt(2)) and (-1/sqrt(2), -1/sqrt(2))
    let x_sq = graph.pow(x, graph.integer(2));
    let y_sq = graph.pow(y, graph.integer(2));
    let sum_sq = graph.add([x_sq, y_sq]);
    let eq1 = graph.sub(sum_sq, graph.integer(1));
    let eq2 = graph.sub(x, y);

    let vars = vec!["x".to_string(), "y".to_string()];
    let sols = PolynomialSystemSolver::solve(&graph, &[eq1, eq2], &vars).unwrap();

    assert_eq!(sols.len(), 2);
    let expected = 1.0 / 2.0f64.sqrt();

    let mut found_pos = false;
    let mut found_neg = false;

    for sol in &sols {
        let x_val = sol[0].re;
        let y_val = sol[1].re;
        if (x_val - expected).abs() < 1e-4 && (y_val - expected).abs() < 1e-4 {
            found_pos = true;
        }
        if (x_val - (-expected)).abs() < 1e-4 && (y_val - (-expected)).abs() < 1e-4 {
            found_neg = true;
        }
    }

    assert!(found_pos);
    assert!(found_neg);
}
