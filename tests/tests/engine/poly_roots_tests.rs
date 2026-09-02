use algebra_engine::poly::{
    BringRadical, CardanoSolver, DurandKernerSolver, FerrariSolver, SturmSequence,
};

#[test]
fn test_cardano_cubic_single_real_root() {
    // x^3 - x - 1 = 0 (Real root approx 1.324718)
    let roots = CardanoSolver::solve(1.0, 0.0, -1.0, -1.0).unwrap();
    assert_eq!(roots.len(), 3);

    let real_root = roots.iter().find(|r| r.im.abs() < 1e-10).unwrap();
    assert!((real_root.re - 1.324717957).abs() < 1e-5);
}

#[test]
fn test_cardano_cubic_three_real_roots() {
    // x^3 - 3x + 1 = 0 (3 distinct real roots: approx 1.532, 0.347, -1.879)
    let roots = CardanoSolver::solve(1.0, 0.0, -3.0, 1.0).unwrap();
    assert_eq!(roots.len(), 3);

    let real_roots_count = roots.iter().filter(|r| r.im.abs() < 1e-10).count();
    assert_eq!(real_roots_count, 3);
}

#[test]
fn test_ferrari_quartic_exact_roots() {
    // x^4 - 5x^2 + 4 = (x-1)(x+1)(x-2)(x+2) = 0 -> roots +-1, +-2
    let roots = FerrariSolver::solve(1.0, 0.0, -5.0, 0.0, 4.0).unwrap();
    assert_eq!(roots.len(), 4);

    let mut real_parts: Vec<f64> = roots.iter().map(|r| r.re).collect();
    real_parts.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert!((real_parts[0] - (-2.0)).abs() < 1e-4);
    assert!((real_parts[1] - (-1.0)).abs() < 1e-4);
    assert!((real_parts[2] - 1.0).abs() < 1e-4);
    assert!((real_parts[3] - 2.0).abs() < 1e-4);
}

#[test]
fn test_bring_radical_evaluation() {
    // BR(1): root of x^5 + x + 1 = 0 (approx -0.754877666)
    let root = BringRadical::eval(1.0);
    let residual = root.powi(5) + root + 1.0;
    assert!(residual.abs() < 1e-10);
    assert!((root - (-0.754877666)).abs() < 1e-5);
}

#[test]
fn test_sturm_sequence_real_root_count() {
    // P(x) = (x - 1)(x - 2)(x + 3) = x^3 - 7x + 6
    // Coeffs: [1, 0, -7, 6]
    // Roots are -3, 1, 2 (3 real roots in (-10, 10))
    let seq = SturmSequence::from_poly(&[1.0, 0.0, -7.0, 6.0]);
    let total_real = seq.count_real_roots(-10.0, 10.0);
    assert_eq!(total_real, 3);

    // Number of roots in (0, 3] should be 2 (roots 1, 2)
    let pos_real = seq.count_real_roots(0.0, 3.0);
    assert_eq!(pos_real, 2);
}

#[test]
fn test_durand_kerner_complex_roots() {
    // z^5 - 1 = 0 (5th roots of unity) -> [1, 0, 0, 0, 0, -1]
    let coeffs = vec![1.0, 0.0, 0.0, 0.0, 0.0, -1.0];
    let roots = DurandKernerSolver::solve(&coeffs, 100, 1e-10);
    assert_eq!(roots.len(), 5);

    // Check that every computed root z satisfies |z^5 - 1| < 1e-6
    for r in &roots {
        let mut cur_re = 1.0;
        let mut cur_im = 0.0;
        for _ in 0..5 {
            let n_re = cur_re * r.re - cur_im * r.im;
            let n_im = cur_re * r.im + cur_im * r.re;
            cur_re = n_re;
            cur_im = n_im;
        }
        let res_re = cur_re - 1.0;
        let res_im = cur_im;
        assert!(res_re.hypot(res_im) < 1e-6);
    }
}
