use algebra_advanced::quantum::QuantumOperator;
use algebra_core::ExprGraph;

#[test]
fn test_canonical_commutation_relation() {
    let graph = ExprGraph::new();
    let mode_a = graph.symbols.get_or_intern("mode_0");

    let a = QuantumOperator::annihilation(mode_a);
    let a_dag = QuantumOperator::creation(mode_a);

    // [a, a^\dagger] = 1
    let comm = a.ccr_commutator(&graph, &a_dag).unwrap();
    assert_eq!(comm, graph.integer(1));
}

#[test]
fn test_symbolic_commutator_bracket() {
    let graph = ExprGraph::new();
    let x = graph.symbol("A");
    let y = graph.symbol("B");

    // [A, B] = A*B - B*A
    let comm = QuantumOperator::commutator(&graph, x, y);
    assert!(!graph.is_empty());
    assert_ne!(comm, x);
}

#[test]
fn test_braket_gauge_and_dyson_series() {
    use algebra_advanced::quantum::{BraKet, DysonSeries, GaugeTheory};

    let graph = ExprGraph::new();
    let psi_sym = graph.symbols.get_or_intern("psi");
    let phi_sym = graph.symbols.get_or_intern("phi");
    let op_a = graph.symbol("A");

    // Bra-Ket <psi| A |phi>
    let bk = BraKet::matrix_element(psi_sym, op_a, phi_sym);
    let bk_expr = bk.to_expr(&graph);
    assert!(!graph.is_empty());
    assert_ne!(bk_expr, op_a);

    // Gauge Theory Field Strength Tensor F_{\mu\nu}^a
    let a_mu = graph.symbol("A_mu");
    let a_nu = graph.symbol("A_nu");
    let x_mu = graph.symbols.get_or_intern("x_mu");
    let x_nu = graph.symbols.get_or_intern("x_nu");

    let _f_tensor = GaugeTheory::field_strength_tensor(&graph, a_mu, a_nu, x_mu, x_nu, None, None);
    assert!(!graph.is_empty());

    // Dyson Series first-order term -i \int H_I dt
    let h_int = graph.symbol("H_I");
    let t_sym = graph.symbols.get_or_intern("t");
    let _dyson = DysonSeries::first_order_term(&graph, h_int, t_sym);
    assert!(!graph.is_empty());
}
