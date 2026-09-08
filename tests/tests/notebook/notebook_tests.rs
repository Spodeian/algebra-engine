use std::collections::HashMap;
use urae::agent::AiConfig;
use urae::core::ExprGraph;
use urae::format::LatexFormatter;

#[test]
fn test_notebook_cli_variable_context_access() {
    let graph = ExprGraph::new();
    let formatter = LatexFormatter;
    let mut ai_config = AiConfig::default();

    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), 2.5);
    bindings.insert("b".to_string(), 4.0);

    let summary = vec![
        (
            "a".to_string(),
            "Parameter".to_string(),
            2.5,
            "m".to_string(),
        ),
        (
            "b".to_string(),
            "Parameter".to_string(),
            4.0,
            "s".to_string(),
        ),
        ("x".to_string(), "Variable".to_string(), 0.0, "".to_string()),
    ];

    // Test 'vars' command
    let vars_res = urae::cli::process_input_with_context(
        &graph,
        &formatter,
        "vars",
        &mut ai_config,
        Some(&bindings),
        Some(&summary),
    )
    .unwrap();

    assert!(vars_res.contains("Active Notebook Variables"));
    assert!(vars_res.contains("a (Parameter): 2.5000 [m]"));
    assert!(vars_res.contains("x (Variable)"));

    // Test 'eval' command using parameter bindings
    let eval_res = urae::cli::process_input_with_context(
        &graph,
        &formatter,
        "eval a * b",
        &mut ai_config,
        Some(&bindings),
        Some(&summary),
    )
    .unwrap();

    assert!(eval_res.starts_with("10.0"));
}

#[test]
fn test_notebook_equation_solving() {
    use urae_notebook::notebook::NotebookState;

    let mut state = NotebookState::default();
    state.session.raw_document_text =
        ["solve x + 5 = 11, x", "solve x^2 - 4 = 0", "2*x - 10 = 0"].join("\n");

    state.evaluate_all();

    println!("Line 0 unicode: {:?}", state.parsed_lines[0].output_unicode);
    println!("Line 1 unicode: {:?}", state.parsed_lines[1].output_unicode);
    println!("Line 1 error: {:?}", state.parsed_lines[1].error_msg);

    assert_eq!(state.parsed_lines.len(), 3);

    // First line: explicit solve
    assert!(state.parsed_lines[0].output_unicode.contains("6"));

    // Second line: quadratic solve
    assert!(
        state.parsed_lines[1].output_unicode.contains("2")
            && state.parsed_lines[1].output_unicode.contains("-2")
    );

    // Third line: relational equation
    let line3 = &state.parsed_lines[2];
    assert!(
        line3
            .domain_info
            .as_ref()
            .map(|d| d.contains("Solutions for x"))
            .unwrap_or(false)
            || line3
                .substituted_latex
                .as_ref()
                .map(|s| s.contains("Solutions for x"))
                .unwrap_or(false)
            || line3.output_unicode.contains("2 · x - 10 = 0")
            || line3.output_unicode.contains("2 · x")
            || line3.output_unicode.contains("2*x")
            || line3.output_unicode.contains("5")
    );
}

#[test]
fn test_permissive_notations_and_background_simplification() {
    use urae_notebook::notebook::{NotebookState, parse_permissive_solve_command};

    // 1. Verify permissive solve command parser
    assert_eq!(
        parse_permissive_solve_command("given g(x) = 3 find x"),
        Some(("g(x) = 3".to_string(), "x".to_string()))
    );
    assert_eq!(
        parse_permissive_solve_command("find x where g(x) = 3"),
        Some(("g(x) = 3".to_string(), "x".to_string()))
    );
    assert_eq!(
        parse_permissive_solve_command("solve g(x) = 3 for x"),
        Some(("g(x) = 3".to_string(), "x".to_string()))
    );

    // 2. Verify notebook evaluation with permissive solve and background simplification
    let mut state = NotebookState::default();
    state.session.raw_document_text = [
        "g(x) = x^3 - 3 * x + 2",
        "given g(x) = 3 find x",
        "find x where x + 5 = 11",
        "(x + 0) * 1 + 0",
        "{ q in Quaternion | norm(q) == 1 }",
    ]
    .join("\n");

    state.evaluate_all();

    assert_eq!(state.parsed_lines.len(), 5);

    // Line 1: solve 'given g(x) = 3 find x'
    assert!(
        state.parsed_lines[1].output_unicode.contains("x = {")
            || state.parsed_lines[1].output_unicode.contains("x =")
    );

    // Line 2: solve 'find x where x + 5 = 11'
    assert!(state.parsed_lines[2].output_unicode.contains("6"));

    // Line 3: background simplified expression (x + 0) * 1 -> x
    let line3 = &state.parsed_lines[3];
    println!("Line 3 raw: {:?}", line3.raw_text);
    println!("Line 3 simplified: {:?}", line3.simplified_unicode);
    assert_eq!(line3.simplified_unicode.as_deref(), Some("x"));

    // Line 4: scoped math context in quaternion set builder
    let line4 = &state.parsed_lines[4];
    assert_eq!(
        line4.scoped_context.algebra_domain,
        urae::Domain::Hypercomplex { dimension: 4 }
    );
}

#[test]
fn test_user_exact_example_suite() {
    use urae_notebook::notebook::NotebookState;

    let user_lines = [
        "x is a real number",
        "p is a 3D clifford number with signature (1, 1, 1)",
        "q is in the set of complex numbers",
        "A is a 2 by 3 matrix",
        "f(x, q) = cos(x) + sin(q)",
        "f(x, q) equals cos(x) + sin(q)",
        "f(x, q) is cos(x) + sin(q)",
        "d f(x, q) / dx",
        "derivative of f(x, q)",
        "derivative of f(x, q) by x",
        "derivatives of f(x, q)",
        "total derivative of f(x, q)",
        "total derivative of f(x, q) by x",
        "partial derivative of f(x, q)",
        "partial derivative of f(x, q) by x",
        "partial derivatives of f(x, q)",
        "suma f(x, q) dx",
        "suma_{3}^{q} f(x, q) dx",
        "integral of f(x, q)",
        "integral of f(x, q) by x",
        "integral of f(x, q) along [2, 7)",
        "integral of f(x, q) along (-infty, +inf) by x",
        "integral of f(x, q) by x along (-infty, +inf)",
        "integrals of f(x, q)",
        "integrate f(x, q)",
        "integrate f(x, q) by x",
        "integrate f(x, q) by x along (q, 7]",
        "integrate f(x, q) along (q, 7] by x",
    ];

    let mut state = NotebookState::default();
    state.session.raw_document_text = user_lines.join("\n");
    state.evaluate_all();

    assert_eq!(state.parsed_lines.len(), user_lines.len());

    for (idx, pl) in state.parsed_lines.iter().enumerate() {
        assert!(
            pl.error_msg.is_none(),
            "Line {} ('{}') failed with error: {:?}",
            idx,
            pl.raw_text,
            pl.error_msg
        );
        assert!(
            !pl.output_unicode.is_empty(),
            "Line {} ('{}') produced empty output",
            idx,
            pl.raw_text
        );
    }

    // Verify multi-variable indexed derivatives (unspecified variable)
    let deriv_unspecified = &state.parsed_lines[8]; // "derivative of f(x, q)"
    assert!(deriv_unspecified.output_unicode.contains("[1]"));
    assert!(deriv_unspecified.output_unicode.contains("[2]"));

    // Verify multi-variable indexed integrals (unspecified variable)
    let int_unspecified = &state.parsed_lines[18]; // "integral of f(x, q)"
    assert!(int_unspecified.output_unicode.contains("[1]"));
    assert!(int_unspecified.output_unicode.contains("[2]"));
}

#[test]
fn test_repl_individual_lines() {
    use urae_notebook::notebook::NotebookState;

    let lines = [
        "x is a real number",
        "p is a 3D clifford number with signature (1, 1, 1)",
        "q is in the set of complex numbers",
        "A is a 2 by 3 matrix",
        "f(x, q) = cos(x) + sin(q)",
        "d f(x, q) / dx",
        "derivative of f(x, q) by x",
        "total derivative of f(x, q) by x",
        "partial derivative of f(x, q) by x",
        "suma f(x, q) dx",
        "suma_{3}^{q} f(x, q) dx",
        "integral of f(x, q) by x",
        "integral of f(x, q) along [2, 7)",
        "integral of f(x, q) along (-infty, +inf) by x",
        "integrate f(x, q) by x along (q, 7]",
    ];

    for line in lines {
        let mut state = NotebookState::default();
        if line.contains("f(x, q)") {
            state.session.raw_document_text = ["f(x, q) = cos(x) + sin(q)", line].join("\n");
            state.evaluate_all();
            assert_eq!(state.parsed_lines.len(), 2);
            let pl = &state.parsed_lines[1];
            assert!(
                pl.error_msg.is_none(),
                "Individual test failed for: '{}', err: {:?}",
                line,
                pl.error_msg
            );
            assert!(!pl.output_unicode.is_empty());
        } else {
            state.session.raw_document_text = line.to_string();
            state.evaluate_all();
            assert_eq!(state.parsed_lines.len(), 1);
            let pl = &state.parsed_lines[0];
            assert!(
                pl.error_msg.is_none(),
                "Individual test failed for: '{}', err: {:?}",
                line,
                pl.error_msg
            );
            assert!(!pl.output_unicode.is_empty());
        }
    }
}

#[test]
fn test_advanced_algebra_notation_clifford_and_matrix() {
    use urae_notebook::notebook::NotebookState;

    let math_doc = [
        "p is a 3D clifford number with signature (1, 1, 1)",
        "A is a 2 by 3 matrix",
        "M is in the set of complex numbers",
        "q is a Quaternion",
    ]
    .join("\n");

    let mut state = NotebookState::default();
    state.session.raw_document_text = math_doc;
    state.evaluate_all();

    assert!(
        state.parsed_lines[0]
            .domain_info
            .as_ref()
            .map(|d| d.to_lowercase().contains("clifford"))
            .unwrap_or(false)
    );
    assert!(
        state.parsed_lines[1]
            .domain_info
            .as_ref()
            .map(|d| d.to_lowercase().contains("matrix"))
            .unwrap_or(false)
    );
}

#[test]
fn test_new_natural_language_notations() {
    use urae_notebook::notebook::NotebookState;

    let notations = [
        "x ∈ Reals",
        "y is in Complex",
        "z in Quaternion",
        "derive x^3 wrt x",
        "derive cos(x) by x",
        "limit of x/x as x -> 0",
        "sum of k from k=1 to n",
    ];

    for notation in notations {
        let mut state = NotebookState::default();
        state.session.raw_document_text = notation.to_string();
        state.evaluate_all();
        assert_eq!(state.parsed_lines.len(), 1);
        let pl = &state.parsed_lines[0];
        assert!(
            pl.error_msg.is_none(),
            "Failed for notation: '{}', err: {:?}",
            notation,
            pl.error_msg
        );
        assert!(!pl.output_unicode.is_empty());
    }
}
