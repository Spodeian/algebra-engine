use algebra_core::Domain;

#[test]
fn test_domain_subdomain_hierarchy() {
    assert!(Domain::Integers.is_subdomain_of(&Domain::Reals));
    assert!(Domain::Reals.is_subdomain_of(&Domain::Complex));
    assert!(!Domain::Complex.is_subdomain_of(&Domain::Reals));
}

#[test]
fn test_non_commutativity() {
    let mat_domain = Domain::Matrix {
        rows: 3,
        cols: 3,
        element_domain: Box::new(Domain::Reals),
    };
    assert!(!mat_domain.is_commutative_mul());
    assert!(Domain::Reals.is_commutative_mul());
}
