use algebra_core::ExprGraph;
use algebra_engine::poly::{GrobnerBasis, MonomialOrder, MultiPoly, Term};

#[test]
fn test_monomial_orderings() {
    let m1 = algebra_engine::poly::multivariate::Monomial::new(vec![2, 1]); // x^2 y
    let m2 = algebra_engine::poly::multivariate::Monomial::new(vec![1, 3]); // x y^3

    // Under Lex: x^2 y > x y^3 (first exponent 2 > 1)
    assert_eq!(
        m1.cmp_with_order(&m2, MonomialOrder::Lex),
        std::cmp::Ordering::Greater
    );

    // Under GradedLex: x y^3 > x^2 y (total degree 4 > 3)
    assert_eq!(
        m1.cmp_with_order(&m2, MonomialOrder::GradedLex),
        std::cmp::Ordering::Less
    );

    // Under GrevLex: total degree 4 > 3
    assert_eq!(
        m1.cmp_with_order(&m2, MonomialOrder::GrevLex),
        std::cmp::Ordering::Less
    );
}

#[test]
fn test_multivariate_poly_division() {
    // Variables: x (0), y (1)
    let vars = vec!["x".to_string(), "y".to_string()];
    let order = MonomialOrder::Lex;

    // f = x^2 y + x y^2 + y^2
    let t1 = Term::new(
        1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![2, 1]),
    );
    let t2 = Term::new(
        1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![1, 2]),
    );
    let t3 = Term::new(
        1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![0, 2]),
    );
    let f = MultiPoly::from_terms(vec![t1, t2, t3], 2, vars.clone(), order);

    // f1 = x y - 1
    let f1_t1 = Term::new(
        1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![1, 1]),
    );
    let f1_t2 = Term::new(
        -1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![0, 0]),
    );
    let f1 = MultiPoly::from_terms(vec![f1_t1, f1_t2], 2, vars.clone(), order);

    // f2 = y^2 - 1
    let f2_t1 = Term::new(
        1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![0, 2]),
    );
    let f2_t2 = Term::new(
        -1.0,
        algebra_engine::poly::multivariate::Monomial::new(vec![0, 0]),
    );
    let f2 = MultiPoly::from_terms(vec![f2_t1, f2_t2], 2, vars, order);

    let (quotients, rem) = f.div_rem(&[f1, f2]);
    assert_eq!(quotients.len(), 2);
    // Remainder should have lower degree terms: x + y + 1
    assert!(!rem.is_zero());
}

#[test]
fn test_grobner_basis_buchberger() {
    let graph = ExprGraph::new();
    let vars = vec!["x".to_string(), "y".to_string()];
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    // Circle: x^2 + y^2 - 1 = 0
    let x_sq = graph.pow(x, graph.integer(2));
    let y_sq = graph.pow(y, graph.integer(2));
    let sum_sq = graph.add([x_sq, y_sq]);
    let eq1 = graph.sub(sum_sq, graph.integer(1));

    // Hyperbola: x * y - 1/2 = 0 -> 2*x*y - 1 = 0
    let xy = graph.mul([x, y]);
    let eq2 = graph.sub(xy, graph.float(0.5));

    let p1 = MultiPoly::from_expr(&graph, eq1, &vars, MonomialOrder::Lex).unwrap();
    let p2 = MultiPoly::from_expr(&graph, eq2, &vars, MonomialOrder::Lex).unwrap();

    let gb = GrobnerBasis::buchberger(&[p1, p2]);
    assert!(!gb.polynomials.is_empty());

    // In Lex order with x > y, one of the polynomials in the reduced basis must be univariate in y
    let has_univariate_y = gb
        .polynomials
        .iter()
        .any(|p| p.terms.iter().all(|t| t.monomial.exponents[0] == 0));
    assert!(has_univariate_y);
}
