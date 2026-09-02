use algebra_advanced::transcendents::{HeunEquation, PainleveEquation, PainleveKind};

#[test]
fn test_painleve_1_evaluation_and_integration() {
    let p1 = PainleveEquation::new(PainleveKind::P1);

    // At t=0, y=1, y'=0: y'' = 6(1)^2 + 0 = 6
    let d2y = p1.eval_d2y(0.0, 1.0, 0.0).unwrap();
    assert!((d2y - 6.0).abs() < 1e-10);

    // Integrate trajectory
    let traj = p1.integrate(0.0, 1.0, 0.0, 0.5, 0.01).unwrap();
    assert!(traj.len() > 10);
    let (last_t, last_y, _) = traj.last().unwrap();
    assert!((*last_t - 0.5).abs() < 1e-4);
    assert!(*last_y > 1.0);
}

#[test]
fn test_painleve_2_evaluation() {
    let p2 = PainleveEquation::new(PainleveKind::P2 { alpha: 0.5 });
    // y'' = 2y^3 + t*y + 0.5
    // At t=1, y=2, y'=0: y'' = 2(8) + 1(2) + 0.5 = 16 + 2 + 0.5 = 18.5
    let d2y = p2.eval_d2y(1.0, 2.0, 0.0).unwrap();
    assert!((d2y - 18.5).abs() < 1e-10);
}

#[test]
fn test_painleve_1_laurent_pole_series() {
    let t0 = 2.0;
    let t = 2.001;
    let val = PainleveEquation::laurent_p1_pole_series(t, t0);
    // Leading term is 1/(0.001)^2 = 1,000,000
    assert!((val - 1_000_000.0).abs() < 100.0);
}

#[test]
fn test_heun_equation_evaluation() {
    let heun = HeunEquation::general(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    let d2y = heun.eval_d2y(0.5, 2.0, 1.0, 0.0).unwrap();
    // At y=1, dy=0: d2y = -q(z)*y
    assert!(d2y.is_finite());
}
