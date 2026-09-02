use algebra_core::ExprGraph;
use algebra_engine::ode::{
    NumericalOdeConfig, NumericalOdeMethod, NumericalOdeSolver, SymbolicOdeSolver,
};
use algebra_engine::pde::{
    BoundaryCondition, NumericalPdeSolver, PdeClassification, SymbolicPdeSolver,
};

#[test]
fn test_symbolic_first_order_linear_ode() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");
    let p = graph.integer(1);
    let q = x;

    // y' + 1*y = x -> y(x) = exp(-x) * (int x*exp(x) dx + C1)
    let sol = SymbolicOdeSolver::solve_first_order_linear(&graph, p, q, x_sym).unwrap();
    assert_ne!(sol, graph.integer(0));
}

#[test]
fn test_symbolic_bernoulli_ode() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let p = graph.integer(1);
    let q = graph.integer(1);

    // y' + y = y^2 (n = 2)
    let sol = SymbolicOdeSolver::solve_bernoulli(&graph, p, q, 2, x_sym).unwrap();
    assert_ne!(sol, graph.integer(0));
}

#[test]
fn test_symbolic_second_order_const_coeff_ode() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");

    // 1. Distinct real roots: y'' - 5y' + 6y = 0 (r1 = 3, r2 = 2)
    let sol_distinct = SymbolicOdeSolver::solve_second_order_const_coeff_homogeneous(
        &graph, 1.0, -5.0, 6.0, x_sym,
    )
    .unwrap();
    assert_ne!(sol_distinct, graph.integer(0));

    // 2. Complex roots: y'' + 4y = 0 (alpha = 0, beta = 2)
    let sol_complex =
        SymbolicOdeSolver::solve_second_order_const_coeff_homogeneous(&graph, 1.0, 0.0, 4.0, x_sym)
            .unwrap();
    assert_ne!(sol_complex, graph.integer(0));
}

#[test]
fn test_symbolic_cauchy_euler_ode() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");

    // x^2 y'' - 2x y' + 2y = 0 -> r1 = 2, r2 = 1
    let sol = SymbolicOdeSolver::solve_cauchy_euler(&graph, 1.0, -2.0, 2.0, x_sym).unwrap();
    assert_ne!(sol, graph.integer(0));
}

#[test]
fn test_symbolic_pde_classification() {
    // Laplace: u_xx + u_yy = 0 (A=1, B=0, C=1 -> disc = -4 < 0 -> Elliptic)
    assert_eq!(
        SymbolicPdeSolver::classify_second_order(1.0, 0.0, 1.0),
        PdeClassification::Elliptic
    );

    // Heat: u_t - alpha * u_xx = 0 (A=-alpha, B=0, C=0 -> disc = 0 -> Parabolic)
    assert_eq!(
        SymbolicPdeSolver::classify_second_order(-1.0, 0.0, 0.0),
        PdeClassification::Parabolic
    );

    // Wave: u_tt - c^2 * u_xx = 0 (A=-c^2, B=0, C=1 -> disc = 4c^2 > 0 -> Hyperbolic)
    assert_eq!(
        SymbolicPdeSolver::classify_second_order(-1.0, 0.0, 1.0),
        PdeClassification::Hyperbolic
    );
}

#[test]
fn test_symbolic_wave_dalembert() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");
    let x = graph.symbol("x");

    // f(x) = x^2, g(x) = None, c = 2.0
    let f_init = graph.pow(x, graph.integer(2));
    let sol =
        SymbolicPdeSolver::solve_wave_dalembert(&graph, f_init, None, 2.0, x_sym, t_sym).unwrap();
    assert_ne!(sol, graph.integer(0));
}

#[test]
fn test_symbolic_heat_kernel() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let t_sym = graph.symbols.get_or_intern("t");

    let kernel = SymbolicPdeSolver::heat_kernel_1d(&graph, 1.0, x_sym, t_sym).unwrap();
    assert_ne!(kernel, graph.integer(0));
}

#[test]
fn test_numerical_ode_dopri5_adaptive() {
    // Harmonic oscillator \dot{x} = v, \dot{v} = -x
    let f = |_t: f64, y: &[f64]| -> Vec<f64> { vec![y[1], -y[0]] };

    let config = NumericalOdeConfig {
        method: NumericalOdeMethod::DormandPrince54,
        rtol: 1e-5,
        ..Default::default()
    };

    let y0 = [1.0, 0.0]; // x(0) = 1, v(0) = 0
    let traj =
        NumericalOdeSolver::solve(f, (0.0, std::f64::consts::PI * 2.0), &y0, &config).unwrap();

    assert!(traj.steps_accepted > 0);
    let final_y = traj.y.last().unwrap();
    // After 2*pi, x(2*pi) ~= 1.0, v(2*pi) ~= 0.0
    assert!((final_y[0] - 1.0).abs() < 1e-2);
    assert!((final_y[1] - 0.0).abs() < 1e-2);
}

#[test]
fn test_numerical_ode_stiff_radau() {
    // Stiff decay \dot{y} = -1000 * y, y(0) = 1
    let f = |_t: f64, y: &[f64]| -> Vec<f64> { vec![-1000.0 * y[0]] };

    let config = NumericalOdeConfig {
        method: NumericalOdeMethod::RadauIIA5,
        initial_step: Some(0.01),
        ..Default::default()
    };

    let y0 = [1.0];
    let traj = NumericalOdeSolver::solve(f, (0.0, 0.1), &y0, &config).unwrap();

    assert!(traj.steps_accepted > 0);
    let final_y = traj.y.last().unwrap();
    assert!(final_y[0].abs() < 1e-5);
}

#[test]
fn test_numerical_ode_symplectic_verlet() {
    // Harmonic oscillator d^2q/dt^2 = -q (Hamiltonian H = 0.5*v^2 + 0.5*q^2)
    let f = |_t: f64, y: &[f64]| -> Vec<f64> { vec![y[1], -y[0]] };

    let config = NumericalOdeConfig {
        method: NumericalOdeMethod::VelocityVerlet,
        initial_step: Some(0.01),
        ..Default::default()
    };

    let y0 = [1.0, 0.0];
    let traj = NumericalOdeSolver::solve(f, (0.0, 10.0), &y0, &config).unwrap();

    // Verify energy conservation: H(t) = 0.5*q^2 + 0.5*v^2 ~= 0.5
    for state in &traj.y {
        let q = state[0];
        let v = state[1];
        let energy = 0.5 * (q * q + v * v);
        assert!((energy - 0.5).abs() < 1e-3);
    }
}

#[test]
fn test_numerical_pde_mol_diffusion() {
    // 1D Heat Equation u_t = u_xx on [0, 1] with u(x, 0) = sin(pi * x), u(0, t) = u(1, t) = 0
    let u0 = |x: f64| (std::f64::consts::PI * x).sin();
    let reaction = |_u: f64, _x: f64, _t: f64| 0.0;

    let config = NumericalOdeConfig::default();
    let sol = NumericalPdeSolver::solve_diffusion_reaction_mol(
        1.0,
        reaction,
        (0.0, 1.0),
        (0.0, 0.1),
        11,
        u0,
        BoundaryCondition::Dirichlet(0.0),
        BoundaryCondition::Dirichlet(0.0),
        &config,
    )
    .unwrap();

    assert_eq!(sol.x.len(), 11);
    assert!(!sol.t.is_empty());
    // Center point u(0.5, t) decays exponentially
    let initial_center = sol.u[0][5];
    let final_center = sol.u.last().unwrap()[5];
    assert!(final_center < initial_center);
}

#[test]
fn test_numerical_pde_fdm_wave() {
    // 1D Wave equation u_tt = c^2 u_xx with pulse initial condition
    let u0 = |x: f64| (std::f64::consts::PI * x).sin();
    let v0 = |_x: f64| 0.0;

    let sol =
        NumericalPdeSolver::solve_wave_fdm(1.0, (0.0, 1.0), (0.0, 0.2), 21, u0, v0, None).unwrap();

    assert_eq!(sol.x.len(), 21);
    assert!(sol.t.len() > 2);
}

#[test]
fn test_numerical_pde_fem_poisson() {
    // -u''(x) = 1 on [0, 1] with u(0) = 0, u(1) = 0
    // Exact analytical solution: u(x) = 0.5 * x * (1 - x)
    let f_source = |_x: f64| 1.0;

    let (x_nodes, u_fem) =
        NumericalPdeSolver::solve_poisson_fem_1d((0.0, 1.0), 10, f_source, 0.0, 0.0).unwrap();

    assert_eq!(x_nodes.len(), 11);
    assert_eq!(u_fem.len(), 11);

    // Check center point x = 0.5 -> u(0.5) = 0.5 * 0.5 * 0.5 = 0.125
    let center_val = u_fem[5];
    assert!((center_val - 0.125).abs() < 1e-2);
}
