use algebra_core::domain::GoalDomain;
use algebra_core::ExprGraph;
use algebra_engine::simplify::{DomainFeatureDetector, DynamicRuleSelector, Simplifier};

#[test]
fn test_domain_feature_detector_standard_vs_stochastic() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let zero = graph.integer(0);
    let expr_standard = graph.add([x, zero]);

    let feat_standard = DomainFeatureDetector::scan(&graph, expr_standard);
    assert!(feat_standard.is_commutative_ring);
    assert!(!feat_standard.has_stochastic_terms);
    assert!(!feat_standard.has_trigonometric);

    // Stochastic Wiener term dW_t
    let dw = graph.symbol("dW_t");
    let expr_stochastic = graph.mul([dw, dw]);
    let feat_stochastic = DomainFeatureDetector::scan(&graph, expr_stochastic);
    assert!(feat_stochastic.has_stochastic_terms);
}

#[test]
fn test_domain_feature_detector_trig_and_quaternion() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let sin_x = graph.function("sin", vec![x]);
    let feat_trig = DomainFeatureDetector::scan(&graph, sin_x);
    assert!(feat_trig.has_trigonometric);

    // Non-commutative Quaternion
    let qi = graph.symbol("qi");
    let qj = graph.symbol("qj");
    let q_expr = graph.mul([qi, qj]);
    let feat_quat = DomainFeatureDetector::scan(&graph, q_expr);
    assert!(feat_quat.is_non_commutative_ring);
    assert!(!feat_quat.is_commutative_ring);
}

#[test]
fn test_dynamic_rule_selector_gated_rules() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let zero = graph.integer(0);
    let expr = graph.add([x, zero]);

    let (rules, features) =
        DynamicRuleSelector::select_rules(&graph, expr, GoalDomain::ConserveOriginal);
    assert!(features.is_commutative_ring);
    assert!(!features.has_stochastic_terms);
    // Should contain commutative ring rules without stochastic/quaternion clutter
    assert!(!rules.is_empty());
}

#[test]
fn test_simplify_with_dynamic_rules_commutative() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let zero = graph.integer(0);
    let expr = graph.add([x, zero]);

    let simplified = Simplifier::simplify(&graph, expr).unwrap();
    assert_eq!(simplified, x);
}

#[test]
fn test_simplify_quaternion_hamilton_identities() {
    let graph = ExprGraph::new();
    let qi = graph.symbol("qi");
    let q_sq = graph.mul([qi, qi]);

    let simplified = Simplifier::simplify(&graph, q_sq).unwrap();
    assert_eq!(simplified, graph.integer(-1));
}

#[test]
fn test_simplify_with_ito_nilpotent_bridge() {
    let graph = ExprGraph::new();
    let dw = graph.symbol("dW");
    let dw_sq = graph.mul([dw, dw]);

    // Simplifies (dW)^2 -> dt under Ito bridge
    let simplified =
        Simplifier::simplify_with_goal(&graph, dw_sq, GoalDomain::NilpotentDual).unwrap();
    let dt = graph.symbol("dt");
    assert_eq!(simplified, dt);
}
