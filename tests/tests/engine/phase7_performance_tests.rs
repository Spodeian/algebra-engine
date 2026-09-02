//! Phase 7 Tests: Resource Budgets, Soft/Hard Caps, Graceful Degradation, and Schwartz-Zippel Equivalence.

use algebra_core::config::{BudgetStatus, EngineConfig, ResourceBudget};
use algebra_core::ExprGraph;
use algebra_engine::simplify::SchwartzZippel;

#[test]
fn test_resource_budget_soft_and_hard_caps() {
    let budget = ResourceBudget::default(); // egraph: (8000, 25000)

    // Within normal limits
    assert_eq!(budget.check_egraph_nodes(100), BudgetStatus::WithinLimits);

    // Exceeds soft cap (triggers graceful degradation)
    match budget.check_egraph_nodes(10_000) {
        BudgetStatus::SoftCapExceeded {
            current, soft_cap, ..
        } => {
            assert_eq!(current, 10_000);
            assert_eq!(soft_cap, 8_000);
        }
        _ => panic!("Expected SoftCapExceeded"),
    }

    // Exceeds hard cap (triggers safe abort)
    match budget.check_egraph_nodes(30_000) {
        BudgetStatus::HardCapExceeded {
            current, hard_cap, ..
        } => {
            assert_eq!(current, 30_000);
            assert_eq!(hard_cap, 25_000);
        }
        _ => panic!("Expected HardCapExceeded"),
    }
}

#[test]
fn test_engine_config_hardware_presets() {
    let hpc = EngineConfig::hpc();
    assert_eq!(hpc.budget.egraph_nodes, (250_000, 1_000_000));
    assert_eq!(hpc.budget.timeout_secs, 60.0);

    let embedded = EngineConfig::embedded();
    assert_eq!(embedded.budget.egraph_nodes, (2_000, 6_000));
    assert_eq!(embedded.budget.timeout_secs, 1.5);
}

#[test]
fn test_schwartz_zippel_probabilistic_identity() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    // Identity: (x + y)^2 - (x^2 + 2*x*y + y^2) == 0
    let x_plus_y = graph.add([x, y]);
    let lhs = graph.pow(x_plus_y, graph.integer(2));

    let x2 = graph.pow(x, graph.integer(2));
    let two_xy = graph.mul([graph.integer(2), x, y]);
    let y2 = graph.pow(y, graph.integer(2));
    let rhs = graph.add([x2, two_xy, y2]);

    // Fast O(1) probabilistic equivalence proof
    assert!(
        SchwartzZippel::are_equal(&graph, lhs, rhs),
        "(x + y)^2 and x^2 + 2xy + y^2 must be probabilistically identical"
    );

    // Non-equal expressions: (x + y)^2 vs (x^2 + y^2) -> instant O(1) refutation!
    let wrong_rhs = graph.add([x2, y2]);
    assert!(
        !SchwartzZippel::are_equal(&graph, lhs, wrong_rhs),
        "(x + y)^2 and x^2 + y^2 must be proven non-equal in O(1)"
    );
}
