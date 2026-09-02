//! Workspace integration tests for boolean logic and symbol assumptions.

use algebra_core::ExprGraph;
use algebra_engine::logic::{AssumptionsContext, BoolExpr, Predicate, SatSolver};

#[test]
fn test_symbol_assumptions_integration() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");

    let mut assumptions = AssumptionsContext::new();
    assumptions.assume(x_sym, Predicate::Positive);

    // Derived predicates automatically inferred
    assert!(assumptions.is(x_sym, Predicate::Positive));
    assert!(assumptions.is(x_sym, Predicate::NonNegative));
    assert!(assumptions.is(x_sym, Predicate::NonZero));
    assert!(assumptions.is(x_sym, Predicate::Real));

    // Non-assumed predicate should return false
    assert!(!assumptions.is(x_sym, Predicate::Integer));
    assert!(assumptions.is_consistent());
}

#[test]
fn test_assumptions_contradiction_detection() {
    let graph = ExprGraph::new();
    let y_sym = graph.symbols.get_or_intern("y");

    let mut assumptions = AssumptionsContext::new();
    assumptions.assume(y_sym, Predicate::Positive);
    assumptions.assume(y_sym, Predicate::Negative);

    assert!(!assumptions.is_consistent());
}

#[test]
fn test_boolean_sat_solver_integration() {
    // (A AND True) OR NOT(False) => A
    let expr = BoolExpr::Or(vec![
        BoolExpr::And(vec![BoolExpr::Var(1), BoolExpr::True]),
        BoolExpr::Not(Box::new(BoolExpr::False)),
    ]);

    let simplified = expr.simplify();
    assert_eq!(simplified, BoolExpr::True);
    assert!(SatSolver::is_satisfiable(&expr));
}
