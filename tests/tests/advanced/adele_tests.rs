use algebra_advanced::adele::{Adele, ArtinProduct, Idele};

#[test]
fn test_adele_construction_and_addition() {
    let primes = vec![2, 3, 5, 7];
    let a1 = Adele::from_rational(6, 1, &primes, 4).unwrap();
    let a2 = Adele::from_rational(4, 1, &primes, 4).unwrap();

    let sum = a1.add(&a2);
    assert!((sum.archimedean - 10.0).abs() < 1e-10);
}

#[test]
fn test_global_artin_product_formula() {
    // 1. x = 6 = 2 * 3
    // |6|_inf = 6, |6|_2 = 1/2, |6|_3 = 1/3, |6|_p = 1 for p >= 5
    // Product = 6 * (1/2) * (1/3) = 1
    assert!(ArtinProduct::verify_product_formula(6, 1).unwrap());

    // 2. x = -15/4 = -3 * 5 / 2^2
    // |x|_inf = 15/4 = 3.75, |x|_2 = 4, |x|_3 = 1/3, |x|_5 = 1/5
    // Product = 3.75 * 4 * (1/3) * (1/5) = 15 * (1/15) = 1
    assert!(ArtinProduct::verify_product_formula(-15, 4).unwrap());

    // 3. x = 231/10 = 3 * 7 * 11 / (2 * 5)
    assert!(ArtinProduct::verify_product_formula(231, 10).unwrap());
}

#[test]
fn test_idele_norm() {
    let primes = vec![2, 3, 5];
    let adele = Adele::from_rational(30, 1, &primes, 5).unwrap();
    let idele = Idele::from_adele(adele).unwrap();

    // vol(30) = |30|_inf * |30|_2 * |30|_3 * |30|_5 = 30 * (1/2) * (1/3) * (1/5) = 1.0
    let norm = idele.idele_norm();
    assert!((norm - 1.0).abs() < 1e-9);
}
