use algebra_core::ExprGraph;
use algebra_engine::analysis::{CriticalPointKind, FunctionAnalyzer, ZeroType};
use algebra_engine::poly::roots::ComplexRoot;

#[test]
fn test_classify_zeroes_multiplicity_and_types() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");

    // f(x) = (x - 2)^1 * (x - 3)^2 * (x - 4)^3
    let x_minus_2 = graph.sub(x, graph.integer(2));
    let x_minus_3 = graph.sub(x, graph.integer(3));
    let x_minus_4 = graph.sub(x, graph.integer(4));

    let term1 = x_minus_2;
    let term2 = graph.pow(x_minus_3, graph.integer(2));
    let term3 = graph.pow(x_minus_4, graph.integer(3));

    let f = graph.mul([term1, term2, term3]);

    let roots = vec![
        ComplexRoot::real(2.0),
        ComplexRoot::real(3.0),
        ComplexRoot::real(4.0),
    ];

    let classified = FunctionAnalyzer::classify_zeroes_1d(&graph, f, x_sym, &roots).unwrap();
    assert_eq!(classified.len(), 3);

    // Root 2: Simple
    assert_eq!(classified[0].multiplicity, 1);
    assert_eq!(classified[0].zero_type, ZeroType::Simple);

    // Root 3: Tangential Extremum (mult 2)
    assert_eq!(classified[1].multiplicity, 2);
    assert_eq!(classified[1].zero_type, ZeroType::TangentialExtremum);

    // Root 4: Inflection Crossing (mult 3)
    assert_eq!(classified[2].multiplicity, 3);
    assert_eq!(classified[2].zero_type, ZeroType::InflectionCrossing);
}

#[test]
fn test_univariate_critical_points_extrema_and_inflection() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");

    // f(x) = x^3 - 3*x
    // f'(x) = 3x^2 - 3 -> critical points x = -1 (max), x = 1 (min)
    // f''(x) = 6x -> inflection point x = 0
    let x_cube = graph.pow(x, graph.integer(3));
    let three_x = graph.mul([graph.integer(3), x]);
    let f = graph.sub(x_cube, three_x);

    let search_points = vec![-2.0, -0.5, 0.0, 0.5, 2.0];
    let critical_points =
        FunctionAnalyzer::analyze_critical_points_1d(&graph, f, x_sym, &search_points).unwrap();

    let max_pt = critical_points
        .iter()
        .find(|p| p.kind == CriticalPointKind::LocalMaximum);
    let min_pt = critical_points
        .iter()
        .find(|p| p.kind == CriticalPointKind::LocalMinimum);
    let infl_pt = critical_points
        .iter()
        .find(|p| p.kind == CriticalPointKind::Inflection);

    assert!(max_pt.is_some());
    assert!((max_pt.unwrap().x - (-1.0)).abs() < 1e-3);

    assert!(min_pt.is_some());
    assert!((min_pt.unwrap().x - 1.0).abs() < 1e-3);

    assert!(infl_pt.is_some());
    assert!((infl_pt.unwrap().x - 0.0).abs() < 1e-3);
}

#[test]
fn test_univariate_stationary_inflection() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let x = graph.symbol("x");

    // f(x) = x^3
    // At x = 0: f'(0) = 0, f''(0) = 0, f'''(0) = 6 -> Stationary Inflection
    let f = graph.pow(x, graph.integer(3));
    let search_points = vec![0.0];
    let critical_points =
        FunctionAnalyzer::analyze_critical_points_1d(&graph, f, x_sym, &search_points).unwrap();

    let stat_infl = critical_points
        .iter()
        .find(|p| p.kind == CriticalPointKind::StationaryInflection);
    assert!(stat_infl.is_some());
    assert!((stat_infl.unwrap().x - 0.0).abs() < 1e-4);
}

#[test]
fn test_multivariate_critical_points_hessian_saddle_and_minimum() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let y_sym = graph.symbols.get_or_intern("y");
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    // 1. Saddle: f(x, y) = x^2 - y^2
    let x_sq = graph.pow(x, graph.integer(2));
    let y_sq = graph.pow(y, graph.integer(2));
    let f_saddle = graph.sub(x_sq, y_sq);

    let guesses = vec![vec![0.1, 0.1]];
    let saddle_pts =
        FunctionAnalyzer::analyze_critical_points_nd(&graph, f_saddle, &[x_sym, y_sym], &guesses)
            .unwrap();

    assert_eq!(saddle_pts.len(), 1);
    assert!((saddle_pts[0].coords[0]).abs() < 1e-4);
    assert!((saddle_pts[0].coords[1]).abs() < 1e-4);
    assert_eq!(
        saddle_pts[0].kind,
        CriticalPointKind::SaddlePoint {
            positive_inertia: 1,
            negative_inertia: 1,
        }
    );

    // 2. Local Minimum: f(x, y) = x^2 + y^2
    let f_min = graph.add([x_sq, y_sq]);
    let min_pts =
        FunctionAnalyzer::analyze_critical_points_nd(&graph, f_min, &[x_sym, y_sym], &guesses)
            .unwrap();

    assert_eq!(min_pts.len(), 1);
    assert_eq!(min_pts[0].kind, CriticalPointKind::LocalMinimum);
}
