use algebra_core::domain::GoalDomain;
use algebra_core::ExprGraph;
use algebra_engine::lifting::DomainCompatibilityChecker;
use algebra_engine::tropical::MinPlus;

#[test]
fn test_tropical_matrix_powers_shortest_path() {
    // 3-node graph with distance weights:
    // d(0, 1) = 3, d(1, 2) = 2, d(0, 2) = 10
    let a00 = MinPlus::val(0.0);
    let a01 = MinPlus::val(3.0);
    let a02 = MinPlus::val(10.0);

    let a10 = MinPlus::zero();
    let a11 = MinPlus::val(0.0);
    let a12 = MinPlus::val(2.0);

    let a20 = MinPlus::zero();
    let a21 = MinPlus::zero();
    let a22 = MinPlus::val(0.0);

    // Min-plus matrix multiply row 0 by col 2:
    // (A * A)_{0,2} = min(a00 + a02, a01 + a12, a02 + a22) = min(10, 3 + 2, 10) = 5
    let path_0 = a00 * a02;
    let path_1 = a01 * a12;
    let path_2 = a02 * a22;

    let shortest = path_0 + path_1 + path_2;
    assert_eq!(shortest, MinPlus::val(5.0));

    let _ = (a10, a11, a20, a21);
}

#[test]
fn test_quantum_lie_commutator_hybrid() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let p = graph.symbol("p");

    let xp = graph.mul([x, p]);
    let px = graph.mul([p, x]);
    let commutator = graph.sub(xp, px);

    // Commutator [x, p] != 0
    assert_ne!(commutator, graph.integer(0));
}

#[test]
fn test_graceful_incompatibility_detection() {
    // Weierstrass rational fraction cannot directly combine with Clifford blades
    let check = DomainCompatibilityChecker::validate_compatibility(
        GoalDomain::RationalWeierstrass,
        GoalDomain::CliffordBlade,
    );
    assert!(check.is_err());

    // Nilpotent dual cannot combine with Adele valuations
    let check_adele = DomainCompatibilityChecker::validate_compatibility(
        GoalDomain::NilpotentDual,
        GoalDomain::AdeleRepresentation,
    );
    assert!(check_adele.is_err());
}
