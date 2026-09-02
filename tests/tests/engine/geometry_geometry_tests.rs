use algebra_core::ExprGraph;
use algebra_engine::geometry::ResultantBuilder;

#[test]
fn test_quadratic_discriminant() {
    let graph = ExprGraph::new();
    let a = graph.symbol("a");
    let b = graph.symbol("b");
    let c = graph.symbol("c");

    // Delta = b^2 - 4ac
    let disc = ResultantBuilder::quadratic_discriminant(&graph, a, b, c);

    assert!(!graph.is_empty());
    assert_ne!(disc, a);
}

#[test]
fn test_sylvester_matrix_construction() {
    let graph = ExprGraph::new();
    let a = graph.symbol("a");
    let b = graph.symbol("b");
    let c = graph.symbol("c");
    let d = graph.symbol("d");
    let e = graph.symbol("e");

    let mat = ResultantBuilder::sylvester_matrix_2_1(&graph, [a, b, c], [d, e]).unwrap();

    assert_eq!(mat.rows, 3);
    assert_eq!(mat.cols, 3);
}

#[test]
fn test_arbitrary_curvilinear_spherical_laplacian() {
    use algebra_engine::geometry::{ArbitraryCurvilinearSystem, CoordinateTransform};

    let graph = ExprGraph::new();
    let sys = ArbitraryCurvilinearSystem::spherical_3d(&graph);

    let r_sym = graph.symbols.get_or_intern("r");
    let theta_sym = graph.symbols.get_or_intern("theta");
    let phi_sym = graph.symbols.get_or_intern("phi");

    let r = graph.symbol("r");
    let scalar_field = graph.pow(r, graph.integer(2));

    let lap =
        CoordinateTransform::laplacian(&graph, &sys, scalar_field, &[r_sym, theta_sym, phi_sym]);

    assert!(!graph.is_empty());
    assert_ne!(lap, scalar_field);
}

#[test]
fn test_csg_boolean_operations() {
    use algebra_engine::geometry::{Circle2D, CsgNode, Point2D};

    let c1 = Circle2D::new(0.0, 0.0, 5.0);
    let c2 = Circle2D::new(3.0, 0.0, 5.0);

    let p = Point2D { x: 1.0, y: 0.0 };

    let csg_intersection =
        CsgNode::Intersection(Box::new(CsgNode::Circle(c1)), Box::new(CsgNode::Circle(c2)));

    assert!(csg_intersection.contains(&p));
}
