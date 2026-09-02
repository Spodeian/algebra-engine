use urae::prelude::*;

#[test]
fn test_facade_prelude_imports() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = graph.symbols.get_or_intern("x");

    let sin_x = graph.function("sin", [x]);
    let df = graph.diff(sin_x, x_sym);

    assert_ne!(sin_x, df);

    let parser = ExprParser::new(&graph);
    let parsed = parser.parse("x + 1").unwrap();

    let formatter = LatexFormatter;
    let formatted = formatter.format(&graph, parsed).unwrap();

    assert!(formatted.contains("x"));
    assert!(formatted.contains("1"));
}

#[test]
fn test_facade_submodule_access() {
    let graph = ExprGraph::new();
    let pauli = urae::matrix::SymbolicMatrix::pauli_matrices(&graph).unwrap();
    assert_eq!(pauli.0.rows, 2);

    let dual = urae::numbers::DualNumber::variable(5.0);
    assert_eq!(dual.real, 5.0);
    assert_eq!(dual.dual, 1.0);
}

#[test]
fn test_facade_clifford_rotor_and_multivectors() {
    use urae::advanced::clifford::{CliffordMultivector, CliffordSignature};

    let sig = CliffordSignature::pga3d();
    let e1 = CliffordMultivector::basis_vector(0, sig).unwrap();
    let e2 = CliffordMultivector::basis_vector(1, sig).unwrap();

    // Geometric product e1 * e2 = e1 ∧ e2 (bivector)
    let bivector = e1.geometric_product(&e2).unwrap();
    assert_eq!(bivector.grade_project(2).blades.len(), 1);

    // Rotor R = exp(-theta/2 B) for 90-degree rotation in e1-e2 plane
    let rotor = CliffordMultivector::make_rotor_3d(&bivector, std::f64::consts::PI / 2.0).unwrap();
    let rotated = e1.rotor_sandwich(&rotor).unwrap();
    assert_eq!(rotated.grade_project(1).blades.len(), 1);
}

#[test]
fn test_facade_cartan_calculus_and_distributions() {
    let coords = vec!["x".into(), "y".into(), "z".into()];
    let dx = urae::engine::cartan::DifferentialForm::dx(0, 3, coords.clone()).unwrap();
    let dy = urae::engine::cartan::DifferentialForm::dx(1, 3, coords).unwrap();

    let dx_dy = dx.wedge(&dy).unwrap();
    assert_eq!(dx_dy.degree, 2);

    let theta = urae::engine::distributions::SchwartzDistribution::step();
    let delta = theta.derivative();
    assert_eq!(
        delta,
        urae::engine::distributions::SchwartzDistribution::DiracDelta {
            shift: 0.0,
            order: 0
        }
    );
}
