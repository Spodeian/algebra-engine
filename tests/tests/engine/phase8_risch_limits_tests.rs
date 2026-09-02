//! Phase 8 Tests: Transcendental Risch Differential Fields, Multi-Order L'Hôpital / Gruntz Limits, and Lagrange-Bürmann Inversion.

use algebra_core::ExprGraph;
use algebra_engine::lagrange::LagrangeBurmann;
use algebra_engine::limits::LimitEngine;
use algebra_engine::risch::{DifferentialFieldTower, RischIntegrator};

#[test]
fn test_risch_differential_field_tower_construction() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = match graph.get(x).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };

    let mut tower = DifferentialFieldTower::new_base(x_sym);
    assert_eq!(tower.extensions.len(), 1);

    // Add theta_1 = ln(x) -> theta_1' = 1/x
    let ln_x_sym = graph.symbol("theta1");
    let ln_x_id = match graph.get(ln_x_sym).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };
    tower.push_log(x, ln_x_id);
    assert_eq!(tower.extensions.len(), 2);

    let d_theta1 = tower.derivative_of_generator(&graph, 1, x_sym);
    let expected_d = graph.div(graph.integer(1), x);
    assert_eq!(d_theta1, expected_d);
}

#[test]
fn test_risch_rational_and_transcendental_integrals() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = match graph.get(x).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };

    // 1. int (3*x^2 + 5) dx = x^3 + 5x
    let three = graph.integer(3);
    let five = graph.integer(5);
    let x_sq = graph.pow(x, graph.integer(2));
    let three_x_sq = graph.mul([three, x_sq]);
    let integrand1 = graph.add([three_x_sq, five]);
    let int1 = RischIntegrator::integrate(&graph, integrand1, x_sym).unwrap();
    assert!(int1 != graph.integer(0));

    // 2. int 1/x dx = ln(x)
    let inv_x = graph.div(graph.integer(1), x);
    let int2 = RischIntegrator::integrate(&graph, inv_x, x_sym).unwrap();
    let ln_x = graph.function("ln", [x]);
    assert_eq!(int2, ln_x);

    // 3. int exp(x) dx = exp(x)
    let exp_x = graph.function("exp", [x]);
    let int3 = RischIntegrator::integrate(&graph, exp_x, x_sym).unwrap();
    assert_eq!(int3, exp_x);

    // 4. Gaussian integral: int exp(-x^2) dx = sqrt(pi)/2 * erf(x)
    let neg_x_sq = graph.neg(x_sq);
    let gauss = graph.function("exp", [neg_x_sq]);
    let int_gauss = RischIntegrator::integrate(&graph, gauss, x_sym).unwrap();
    let pi = graph.constant(algebra_core::Constant::Pi);
    let sqrt_pi = graph.function("sqrt", [pi]);
    let sqrt_pi_over_2 = graph.div(sqrt_pi, graph.integer(2));
    let erf_x = graph.function("erf", [x]);
    let expected_gauss = graph.mul([sqrt_pi_over_2, erf_x]);
    assert_eq!(int_gauss, expected_gauss);
}

#[test]
fn test_multi_order_lhopital_limits() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = match graph.get(x).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };

    // lim_{x -> 0} sin(x) / x = 1 (L'Hopital order 1)
    let sin_x = graph.function("sin", [x]);
    let sin_over_x = graph.div(sin_x, x);
    let lim1 = LimitEngine::limit(&graph, sin_over_x, x_sym, graph.integer(0)).unwrap();
    assert_eq!(
        graph.get(lim1).kind,
        algebra_core::ExprKind::Number(algebra_core::Number::Integer(1))
    );

    // lim_{x -> 0} (1 - cos(x)) / x^2 = 1/2 (L'Hopital order 2)
    let one = graph.integer(1);
    let cos_x = graph.function("cos", [x]);
    let num2 = graph.sub(one, cos_x);
    let den2 = graph.pow(x, graph.integer(2));
    let expr2 = graph.div(num2, den2);
    let lim2 = LimitEngine::limit(&graph, expr2, x_sym, graph.integer(0)).unwrap();
    assert_eq!(
        graph.get(lim2).kind,
        algebra_core::ExprKind::Number(algebra_core::Number::Rational(1, 2))
    );
}

#[test]
fn test_lagrange_burmann_inversion_and_lambert_w() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let x_sym = match graph.get(x).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };

    // Lambert W series: W(x) = x - x^2 + 3/2 x^3 - 8/3 x^4 + 125/24 x^5
    let w_res = LagrangeBurmann::lambert_w_series(&graph, x_sym, 5);
    assert_eq!(w_res.coefficients.len(), 5);

    // c_1 = 1/1
    assert_eq!(
        graph.get(w_res.coefficients[0]).kind,
        algebra_core::ExprKind::Number(algebra_core::Number::Rational(1, 1))
    );
    // c_2 = -1/1
    assert_eq!(
        graph.get(w_res.coefficients[1]).kind,
        algebra_core::ExprKind::Number(algebra_core::Number::Rational(-1, 1))
    );
    // c_3 = 3/2 (represented as 3/2)
    assert_eq!(
        graph.get(w_res.coefficients[2]).kind,
        algebra_core::ExprKind::Number(algebra_core::Number::Rational(3, 2))
    );

    // Kepler series: E(M, e) = M + e sin(M) + e^2/2 sin(2M) + ...
    let m = graph.symbol("M");
    let m_sym = match graph.get(m).kind {
        algebra_core::ExprKind::Symbol(s) => s,
        _ => unreachable!(),
    };
    let e = graph.symbol("e");
    let kepler_res = LagrangeBurmann::solve_kepler_series(&graph, m_sym, e, 3);
    assert_eq!(kepler_res.coefficients.len(), 3);
}
