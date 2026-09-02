use algebra_core::interval::RealInterval;

#[test]
fn test_interval_arithmetic() {
    let i1 = RealInterval::new(1.0, 3.0);
    let i2 = RealInterval::new(2.0, 4.0);

    // Addition: [1, 3] + [2, 4] = [3, 7]
    let sum = i1.add(&i2);
    assert_eq!(sum.inf, 3.0);
    assert_eq!(sum.sup, 7.0);

    // Multiplication: [1, 3] * [2, 4] = [2, 12]
    let prod = i1.mul(&i2);
    assert_eq!(prod.inf, 2.0);
    assert_eq!(prod.sup, 12.0);

    // Intersection: [1, 3] ∩ [2, 4] = [2, 3]
    let inter = i1.intersect(&i2).unwrap();
    assert_eq!(inter.inf, 2.0);
    assert_eq!(inter.sup, 3.0);
}

#[test]
fn test_interval_newton_root_confinement() {
    // Root of f(x) = x^2 - 4 on [1.5, 2.5]
    let interval = RealInterval::new(1.5, 2.5);
    let f = |x: f64| x * x - 4.0;
    let df = |i: RealInterval| RealInterval::new(2.0 * i.inf, 2.0 * i.sup);

    let next_interval = interval.newton_step(f, df).unwrap();
    assert!(next_interval.contains(2.0));
    assert!(next_interval.diam() < interval.diam());
}
