use algebra_advanced::morphism::QuaternionToMatrix2x2;
use algebra_core::numbers::Quaternion;
use algebra_core::ExprGraph;
use num_rational::BigRational;
use num_traits::One;

#[test]
fn test_quaternion_matrix_isomorphism() {
    let graph = ExprGraph::new();
    let iso = QuaternionToMatrix2x2;

    let q = Quaternion::new(
        BigRational::one(),
        BigRational::one(),
        BigRational::one(),
        BigRational::one(),
    );

    let mat = iso.to_matrix(&graph, &q).unwrap();
    assert_eq!(mat.rows, 2);
    assert_eq!(mat.cols, 2);
}

struct IdentityFunctor;
impl algebra_advanced::morphism::Functor<String, String> for IdentityFunctor {
    fn map_object(&self, obj: &String) -> String {
        obj.clone()
    }
}

#[test]
fn test_identity_functor() {
    use algebra_advanced::morphism::Functor;
    let f = IdentityFunctor;
    let obj = "VectorSpace_V".to_string();
    assert_eq!(f.map_object(&obj), "VectorSpace_V");
}

#[test]
fn test_equivalence_classifier() {
    use algebra_advanced::morphism::{EquivalenceClassifier, EquivalenceKind};

    let graph = ExprGraph::new();
    let classifier = EquivalenceClassifier::new();

    let x1 = graph.symbol("x");
    let x2 = x1;

    assert_eq!(
        classifier.classify(&graph, x1, x2),
        EquivalenceKind::SyntacticIdentity
    );
    assert!(classifier.check_isomorphism("Quaternion", "M_2(C)"));
    assert!(classifier.check_isomorphism("Complex", "Matrix2x2Real"));
}

#[test]
fn test_simplification_functor() {
    use algebra_advanced::morphism::{Functor, SimplificationFunctor};

    let graph = ExprGraph::new();
    let functor = SimplificationFunctor::new(&graph);

    // x + 0 -> maps under functor to x
    let x = graph.symbol("x");
    let zero = graph.integer(0);
    let x_plus_0 = graph.add([x, zero]);

    let simplified = functor.map_object(&x_plus_0);
    assert_eq!(simplified, x);
}

