use algebra_advanced::proof::{ProofTrace, UnificationEngine};
use algebra_core::ExprGraph;

#[test]
fn test_unification_identical() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let engine = UnificationEngine::new();

    let subst = engine.unify(&graph, x, x).unwrap();
    assert!(subst.mappings.is_empty());
}

#[test]
fn test_unification_symbol_and_num() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let num = graph.integer(42);
    let engine = UnificationEngine::new();

    let subst = engine.unify(&graph, x, num).unwrap();
    assert_eq!(subst.mappings.len(), 1);
}

#[test]
fn test_proof_trace_export() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let zero = graph.integer(0);
    let x_plus_0 = graph.add([x, zero]);

    let mut trace = ProofTrace::new();
    trace.add_step("add_zero_identity", x_plus_0, x);

    let lean = trace.export_lean4();
    assert!(lean.contains("add_zero_identity"));

    let coq = trace.export_coq();
    assert!(coq.contains("add_zero_identity"));
}
