use algebra_core::ExprGraph;
use algebra_engine::matrix::SymbolicMatrix;
use algebra_engine::systems::{
    DifferentialSystemSolver, DiophantineSystemSolver, LinearSystemSolver, NumericalSystemConfig,
    NumericalSystemSolver,
};

#[test]
fn test_linear_system_solver_unique() {
    let graph = ExprGraph::new();
    // 2x + y = 5
    // x - y = 1
    // -> x = 2, y = 1
    let a_data = vec![
        graph.integer(2),
        graph.integer(1),
        graph.integer(1),
        graph.integer(-1),
    ];
    let a = SymbolicMatrix::new(2, 2, a_data).unwrap();
    let b = vec![graph.integer(5), graph.integer(1)];

    let sol = LinearSystemSolver::solve(&graph, &a, &b).unwrap();
    assert_eq!(sol.rank, 2);
    assert!(!sol.has_infinite_solutions);
    assert_eq!(sol.particular.len(), 2);
}

#[test]
fn test_linear_system_solver_nullspace() {
    let graph = ExprGraph::new();
    // x + 2y + 3z = 0
    let a_data = vec![graph.integer(1), graph.integer(2), graph.integer(3)];
    let a = SymbolicMatrix::new(1, 3, a_data).unwrap();
    let b = vec![graph.integer(0)];

    let sol = LinearSystemSolver::solve(&graph, &a, &b).unwrap();
    assert_eq!(sol.rank, 1);
    assert!(sol.has_infinite_solutions);
    assert_eq!(sol.nullspace_basis.len(), 2);
}

#[test]
fn test_linear_system_lu_decomposition() {
    let graph = ExprGraph::new();
    let a_data = vec![
        graph.integer(4),
        graph.integer(3),
        graph.integer(6),
        graph.integer(3),
    ];
    let a = SymbolicMatrix::new(2, 2, a_data).unwrap();

    let (l, u) = LinearSystemSolver::lu_decomposition(&graph, &a).unwrap();
    assert_eq!(l.rows, 2);
    assert_eq!(u.rows, 2);
}

#[test]
fn test_numerical_system_newton_raphson() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let y_sym = graph.symbols.get_or_intern("y");
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    // Non-linear system:
    // F1(x, y) = x^2 + y^2 - 2 = 0
    // F2(x, y) = x - y = 0
    // Solution: x = 1, y = 1 (or -1, -1)
    let x_sq = graph.pow(x, graph.integer(2));
    let y_sq = graph.pow(y, graph.integer(2));
    let sum_sq = graph.add([x_sq, y_sq]);
    let f1 = graph.sub(sum_sq, graph.integer(2));
    let f2 = graph.sub(x, y);

    let config = NumericalSystemConfig {
        tol: 1e-6,
        ..Default::default()
    };

    let sol = NumericalSystemSolver::solve_newton_raphson(
        &graph,
        &[f1, f2],
        &[x_sym, y_sym],
        &[0.8, 1.2],
        &config,
    )
    .unwrap();

    assert_eq!(sol.len(), 2);
    assert!((sol[0] - 1.0).abs() < 1e-4);
    assert!((sol[1] - 1.0).abs() < 1e-4);
}

#[test]
fn test_coupled_differential_system_2x2() {
    let graph = ExprGraph::new();
    let t_sym = graph.symbols.get_or_intern("t");

    // y1' = y2, y2' = -y1 (Harmonic Oscillator)
    let a_data = vec![
        graph.integer(0),
        graph.integer(1),
        graph.integer(-1),
        graph.integer(0),
    ];
    let a = SymbolicMatrix::new(2, 2, a_data).unwrap();

    let y0 = [graph.integer(1), graph.integer(0)];
    let sol = DifferentialSystemSolver::solve_linear_system_2x2(&graph, &a, None, Some(&y0), t_sym)
        .unwrap();

    assert_eq!(sol.len(), 2);
    assert_ne!(sol[0], graph.integer(0));
}

#[test]
fn test_diophantine_system_smith_normal_form() {
    // 2x + 4y = 6
    // 3x + 9y = 12
    let matrix = vec![vec![2, 4], vec![3, 9]];
    let b = vec![6, 12];

    let sol = DiophantineSystemSolver::solve(&matrix, &b).unwrap();
    assert_eq!(sol.particular.len(), 2);
    // Verify particular solution satisfies original system:
    // 2 * x + 4 * y = 6
    let x = sol.particular[0];
    let y = sol.particular[1];
    assert_eq!(2 * x + 4 * y, 6);
    assert_eq!(3 * x + 9 * y, 12);
}
