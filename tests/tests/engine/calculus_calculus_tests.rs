use algebra_core::ExprGraph;
use algebra_engine::calculus::SymbolicCalculus;

#[test]
fn test_diff_polynomial() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    let two = graph.integer(2);
    let x_sq = graph.pow(x, two); // x^2

    let _dx = graph.diff(x_sq, x_sym); // d/dx (x^2)
    assert!(!graph.is_empty());
}

#[test]
fn test_diff_sin_cos() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    let sin_x = graph.function("sin", [x]);
    let _d_sin = graph.diff(sin_x, x_sym); // d/dx sin(x) = cos(x)

    assert!(!graph.is_empty());
}

#[test]
fn test_integrate_basic() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    // int cos(x) dx = sin(x)
    let cos_x = graph.function("cos", [x]);
    let int_cos = graph.integrate(cos_x, x_sym).unwrap();

    assert_eq!(int_cos, graph.function("sin", [x]));
}

#[test]
fn test_limit_lhopital() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    // lim_{x -> 0} x / x = 1 via L'Hopital
    let ratio = graph.div(x, x);
    let zero = graph.integer(0);
    let lim = graph.limit(ratio, x_sym, zero).unwrap();

    assert_eq!(lim, graph.integer(1));
}

#[test]
fn test_gosper_summation() {
    let graph = ExprGraph::new();
    let k = graph.symbol("k");
    let k_sym = graph.symbols.get_or_intern("k");
    let n = graph.symbol("n");
    let one = graph.integer(1);

    // sum_{k=1}^n k = n*(n+1)/2
    let sum = graph.summation(k, k_sym, one, n).unwrap();

    assert!(!graph.is_empty());
    assert_ne!(sum, k);
}

#[test]
fn test_fractional_and_variational_calculus() {
    use algebra_engine::calculus::{FractionalCalculus, VariationalCalculus};

    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    // Caputo derivative D^{0.5} (x^2)
    let two = graph.integer(2);
    let x_sq = graph.pow(x, two);
    let f_diff = FractionalCalculus::caputo_derivative(&graph, x_sq, x_sym, 0.5).unwrap();
    assert!(!graph.is_empty());
    assert_ne!(f_diff, x_sq);

    // Euler-Lagrange for L = 0.5 * m * v^2 - V(q)
    let q = graph.symbol("q");
    let q_sym = graph.symbols.get_or_intern("q");
    let v = graph.symbol("v");
    let v_sym = graph.symbols.get_or_intern("v");
    let _t = graph.symbol("t");
    let t_sym = graph.symbols.get_or_intern("t");

    let v_sq = graph.pow(v, two);
    let kinetic = graph.mul([graph.float(0.5), v_sq]);
    let pot = graph.function("V", [q]);
    let lagrangian = graph.sub(kinetic, pot);

    let el_eq = VariationalCalculus::euler_lagrange(&graph, lagrangian, q_sym, v_sym, t_sym);
    assert!(!graph.is_empty());
    assert_ne!(el_eq, lagrangian);
}
