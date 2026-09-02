use algebra_core::format::{Formatter, LatexFormatter};
use algebra_core::ExprGraph;

#[test]
fn test_latex_formatting() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let two = graph.integer(2);
    let pow = graph.pow(x, two);
    let fn_sin = graph.function("sin", [pow]);

    let latex = LatexFormatter.format(&graph, fn_sin).unwrap();
    assert_eq!(latex, "\\operatorname{sin}\\left({x}^{2}\\right)");
}
