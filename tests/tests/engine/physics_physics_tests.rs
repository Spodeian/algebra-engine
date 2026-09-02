use algebra_core::ExprGraph;
use algebra_engine::physics::{Constants, Dimensions, Quantity};

#[test]
fn test_dimensional_analysis_addition() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    let q1 = Quantity::new(x, Dimensions::length());
    let q2 = Quantity::new(y, Dimensions::length());

    let sum = q1.add(&graph, &q2).unwrap();
    assert_eq!(sum.dimensions, Dimensions::length());
}

#[test]
fn test_dimensional_mismatch_error() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let t = graph.symbol("t");

    let len = Quantity::new(x, Dimensions::length());
    let time = Quantity::new(t, Dimensions::time());

    assert!(len.add(&graph, &time).is_err());
}

#[test]
fn test_physical_constant_multiplication() {
    let graph = ExprGraph::new();
    let c = Constants::speed_of_light(&graph);
    let m = Quantity::new(graph.symbol("m"), Dimensions::mass());

    // E = m * c^2 (m * c * c)
    let mc = m.multiply(&graph, &c);
    let mc2 = mc.multiply(&graph, &c);

    // Energy dimension: M L^2 T^-2
    let energy_dim = Dimensions {
        mass: 1.0,
        length: 2.0,
        time: -2.0,
        current: 0.0,
        temperature: 0.0,
        amount: 0.0,
        intensity: 0.0,
    };
    assert_eq!(mc2.dimensions, energy_dim);
}

#[test]
fn test_real_number_dimensions() {
    let graph = ExprGraph::new();

    // Fractional / Anomalous scaling dimension: L^1.5 T^-0.5
    let frac_dim = Dimensions {
        mass: 0.0,
        length: 1.5,
        time: -0.5,
        current: 0.0,
        temperature: 0.0,
        amount: 0.0,
        intensity: 0.0,
    };

    let q = Quantity::new(graph.symbol("f"), frac_dim);
    let q_sq = q.multiply(&graph, &q);

    assert_eq!(q_sq.dimensions.length, 3.0);
    assert_eq!(q_sq.dimensions.time, -1.0);

    // Power scaling by alpha = 0.5
    let half_dim = q_sq.dimensions.powf(0.5);
    assert_eq!(half_dim, frac_dim);
}

#[test]
fn test_analytical_mechanics_and_poisson_brackets() {
    use algebra_engine::physics::{FourVector, HamiltonianSystem, LagrangianSystem};

    let graph = ExprGraph::new();
    let q = graph.symbol("q");
    let v = graph.symbol("v");
    let p = graph.symbol("p");
    let _t = graph.symbol("t");

    let q_sym = graph.symbols.get_or_intern("q");
    let v_sym = graph.symbols.get_or_intern("v");
    let p_sym = graph.symbols.get_or_intern("p");
    let t_sym = graph.symbols.get_or_intern("t");

    // Harmonic Oscillator Lagrangian L = 0.5 * m * v^2 - 0.5 * k * q^2
    let half = graph.float(0.5);
    let m = graph.symbol("m");
    let k = graph.symbol("k");
    let two = graph.integer(2);

    let kinetic = graph.mul([half, m, graph.pow(v, two)]);
    let potential = graph.mul([half, k, graph.pow(q, two)]);
    let lagrangian = graph.sub(kinetic, potential);

    let sys = LagrangianSystem::new(lagrangian);
    let canonical_p = sys.canonical_momentum(&graph, v_sym);
    assert_ne!(canonical_p, graph.integer(0));

    let el_eq = sys.euler_lagrange(&graph, q_sym, v_sym, t_sym);
    assert_ne!(el_eq, graph.integer(0));

    // Hamiltonian System H(q, p) = p^2 / (2m) + 0.5 * k * q^2
    let ham = graph.add([
        graph.div(graph.pow(p, two), graph.mul([graph.integer(2), m])),
        potential,
    ]);
    let h_sys = HamiltonianSystem::new(ham);
    let poisson = h_sys.poisson_bracket(&graph, q, p, q_sym, p_sym);
    assert_ne!(poisson, graph.integer(0));

    // Minkowski 4-vector p^mu p_mu
    let p0 = graph.symbol("E");
    let p1 = graph.symbol("px");
    let p2 = graph.symbol("py");
    let p3 = graph.symbol("pz");

    let four_p = FourVector::new(p0, p1, p2, p3);
    let norm_sq = four_p.norm_squared(&graph);
    assert_ne!(norm_sq, graph.integer(0));
}
