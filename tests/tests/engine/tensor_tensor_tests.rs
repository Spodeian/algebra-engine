use algebra_core::ExprGraph;
use algebra_engine::tensor::{SymbolicTensor, TensorIndex, TensorIndexKind};

#[test]
fn test_tensor_contraction() {
    let graph = ExprGraph::new();
    let name_a = graph.symbols.get_or_intern("A");
    let name_b = graph.symbols.get_or_intern("B");
    let mu = graph.symbols.get_or_intern("mu");
    let nu = graph.symbols.get_or_intern("nu");

    // A_\mu^\nu
    let t_a = SymbolicTensor::new(
        name_a,
        vec![
            TensorIndex {
                symbol: mu,
                kind: TensorIndexKind::Covariant,
            },
            TensorIndex {
                symbol: nu,
                kind: TensorIndexKind::Contravariant,
            },
        ],
        vec![],
    );

    // B_\nu
    let t_b = SymbolicTensor::new(
        name_b,
        vec![TensorIndex {
            symbol: nu,
            kind: TensorIndexKind::Covariant,
        }],
        vec![],
    );

    // Contraction over dummy index \nu gives vector C_\mu
    let contracted = t_a.contract(&graph, &t_b).unwrap();
    assert_eq!(contracted.indices.len(), 1);
    assert_eq!(contracted.indices[0].symbol, mu);
}

#[test]
fn test_exterior_derivative() {
    let graph = ExprGraph::new();
    let name_omega = graph.symbols.get_or_intern("omega");
    let dx = graph.symbols.get_or_intern("x");

    let form = SymbolicTensor::new(
        name_omega,
        vec![TensorIndex {
            symbol: dx,
            kind: TensorIndexKind::Covariant,
        }],
        vec![],
    );

    let d_form = form.exterior_derivative(&graph);
    assert_eq!(d_form.indices.len(), 2);
}

#[test]
fn test_summation_and_tensor_bridge() {
    use algebra_core::format::{Formatter, LatexFormatter, UnicodeFormatter};
    use algebra_engine::tensor::TensorSumBridge;

    let graph = ExprGraph::new();
    let a = graph.symbol("A");
    let b = graph.symbol("B");
    let j_sym = graph.symbols.get_or_intern("j");
    let one = graph.integer(1);
    let n_sym = graph.symbol("N");

    // 1. Create repeated summation \sum_{j=1}^N A * B
    let mul_ab = graph.mul([a, b]);
    let sum_expr = graph.sum(mul_ab, j_sym, Some(one), Some(n_sym));

    // Test LaTeX & Unicode formatting
    let latex_out = LatexFormatter.format(&graph, sum_expr).unwrap();
    let unicode_out = UnicodeFormatter.format(&graph, sum_expr).unwrap();

    assert!(latex_out.contains("\\sum_{j=1}^{N}"));
    assert!(unicode_out.contains("∑_(j=1)^(N)"));

    // 2. Convert repeated summation \sum_j A * B -> TensorContraction(A, B)
    let contraction = TensorSumBridge::sum_to_tensor_contraction(&graph, sum_expr);
    assert_ne!(contraction, sum_expr);

    // 3. Convert TensorContraction(A, B) back to repeated summation \sum_{j=1}^N A * B
    let sum_back = TensorSumBridge::tensor_contraction_to_sum(&graph, contraction, one, n_sym);
    assert_ne!(sum_back, contraction);
    assert_eq!(
        LatexFormatter.format(&graph, sum_back).unwrap(),
        LatexFormatter.format(&graph, sum_expr).unwrap()
    );

    // 4. Create repeated product \prod_{j=1}^N A
    let prod_expr = graph.product(a, j_sym, Some(one), Some(n_sym));
    let prod_latex = LatexFormatter.format(&graph, prod_expr).unwrap();
    assert!(prod_latex.contains("\\prod_{j=1}^{N}"));
}
