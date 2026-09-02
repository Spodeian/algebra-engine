use algebra_advanced::stochastic::{ItoProcess, WienerProcess};
use algebra_core::{ExprGraph, ExprKind};

#[test]
fn test_ito_lemma_geometric_brownian_motion() {
    let graph = ExprGraph::new();
    let x = graph.symbol("X");
    let x_sym = graph.symbols.get_or_intern("X");
    let t_sym = graph.symbols.get_or_intern("t");

    let mu_sym = graph.symbol("mu");
    let sigma_sym = graph.symbol("sigma");

    // Geometric Brownian Motion: dX = mu*X dt + sigma*X dW
    let drift = graph.mul([mu_sym, x]);
    let diff = graph.mul([sigma_sym, x]);

    let gbm = ItoProcess::new(x_sym, t_sym, drift, diff);

    // f(X) = ln(X)
    let ln_x = graph.function("ln", [x]);
    let (dt_coeff, dw_coeff) = gbm.apply_ito_lemma(&graph, ln_x).unwrap();

    let dt_node = graph.get(dt_coeff);
    assert!(matches!(dt_node.kind, ExprKind::Add(_)));

    let dw_node = graph.get(dw_coeff);
    assert!(matches!(dw_node.kind, ExprKind::Mul(_)));
}

#[test]
fn test_ito_stratonovich_conversion() {
    let graph = ExprGraph::new();
    let x = graph.symbol("X");
    let x_sym = graph.symbols.get_or_intern("X");
    let t_sym = graph.symbols.get_or_intern("t");

    let mu_sym = graph.symbol("mu");
    let sigma_sym = graph.symbol("sigma");

    let drift = graph.mul([mu_sym, x]);
    let diff = graph.mul([sigma_sym, x]);

    let gbm = ItoProcess::new(x_sym, t_sym, drift, diff);

    let strat_drift = gbm.ito_to_stratonovich(&graph).unwrap();
    let strat_node = graph.get(strat_drift);
    assert!(matches!(strat_node.kind, ExprKind::Add(_)));

    let back_ito = gbm.stratonovich_to_ito(&graph).unwrap();
    let ito_node = graph.get(back_ito);
    assert!(matches!(ito_node.kind, ExprKind::Add(_)));
}

#[test]
fn test_wiener_process_construction() {
    let graph = ExprGraph::new();
    let w_sym = graph.symbols.get_or_intern("W");
    let t_sym = graph.symbols.get_or_intern("t");

    let wiener = WienerProcess::new(w_sym, t_sym);
    assert_eq!(wiener.symbol, w_sym);
    assert_eq!(wiener.time_symbol, t_sym);
}
