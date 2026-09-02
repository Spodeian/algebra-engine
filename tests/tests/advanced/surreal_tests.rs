use algebra_advanced::surreal::SurrealNo;

#[test]
fn test_surreal_basic_constants_and_order() {
    let zero = SurrealNo::zero();
    let one = SurrealNo::one();
    let minus_one = SurrealNo::minus_one();
    let half = SurrealNo::half();

    assert_eq!(zero.birthday, 0);
    assert_eq!(one.birthday, 1);
    assert_eq!(minus_one.birthday, 1);
    assert_eq!(half.birthday, 2);

    assert!(minus_one.leq(&zero));
    assert!(zero.leq(&half));
    assert!(half.leq(&one));
    assert!(!one.leq(&zero));
}

#[test]
fn test_surreal_arithmetic() {
    let zero = SurrealNo::zero();
    let one = SurrealNo::one();
    let minus_one = SurrealNo::minus_one();

    // 1 + (-1) == 0
    let sum = one.add(&minus_one);
    assert!(sum.equiv(&zero));

    // 1 + 1 == 2
    let two = SurrealNo::from_int(2);
    let one_plus_one = one.add(&one);
    assert!(one_plus_one.equiv(&two));

    // 1 * 1 == 1
    let prod = one.mul(&one);
    assert!(prod.equiv(&one));
}

#[test]
fn test_surreal_transfinite_and_infinitesimal() {
    let zero = SurrealNo::zero();
    let one = SurrealNo::one();
    let omega = SurrealNo::omega(4);
    let epsilon = SurrealNo::epsilon(4);

    // 0 < epsilon < 1 < omega
    assert!(zero.leq(&epsilon));
    assert!(epsilon.leq(&one));
    assert!(one.leq(&omega));
}
