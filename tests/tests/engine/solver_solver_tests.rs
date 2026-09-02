use algebra_core::ExprGraph;
use algebra_engine::solver::SymbolicSolver;

#[test]
fn test_solveset_linear() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");
    let five = graph.integer(5);

    // x + 5 = 0 => x = -5
    let eq = graph.add([x, five]);
    let roots = graph.solveset(eq, x_sym).unwrap();

    assert_eq!(roots.len(), 1);
    assert_eq!(graph.get(roots[0]).kind, graph.get(graph.integer(-5)).kind);
}

#[test]
fn test_solve_ode_linear() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");

    // y' + 0 * y = 1 => y(x) = x + C
    let zero = graph.integer(0);
    let one = graph.integer(1);

    let sol = graph.solve_ode_linear(zero, one, x_sym).unwrap();
    assert!(!graph.is_empty());
    assert_ne!(sol, zero);
}

#[test]
fn test_solve_pde_heat_1d() {
    let graph = ExprGraph::new();
    let alpha = graph.symbol("alpha");
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");

    let (x_sol, t_sol) = graph.solve_pde_heat_1d(alpha, x_sym, t_sym).unwrap();
    assert!(!graph.is_empty());
    assert_ne!(x_sol, t_sol);
}

#[test]
fn test_diophantine_solvers() {
    use algebra_engine::solver::DiophantineSolver;

    // 3x + 5y = 1 => x = 2, y = -1 (3(2) + 5(-1) = 6 - 5 = 1)
    let (x, y) = DiophantineSolver::solve_linear_2var(3, 5, 1).unwrap();
    assert_eq!(3 * x + 5 * y, 1);

    // Pell's equation x^2 - 2 y^2 = 1 => fundamental solution x = 3, y = 2
    let (px, py) = DiophantineSolver::solve_pell(2).unwrap();
    assert_eq!(px * px - 2 * py * py, 1);
}
