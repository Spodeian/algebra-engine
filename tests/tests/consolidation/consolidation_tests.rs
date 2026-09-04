//! # Workspace Architecture Consolidation Integration Tests
//!
//! Comprehensive end-to-end integration tests verifying cross-crate unification:
//! - `UraeSession` state manager across multi-turn expressions
//! - Domain restrictions and Set-Builder notation
//! - `urae-kernel` Jupyter multi-MIME evaluation with state persistence
//! - `urae-lsp` diagnostic and hover intelligence
//! - `urae-notebook` line referencing ($N, sum($1..$N)) and line shifts
//! - `urae-py` and `urae-ffi` session bindings

use algebra_engine::session::UraeSession;
use std::collections::HashMap;
use urae_kernel::UraeKernel;
use urae_lsp::{Position, UraeLanguageServer};
use urae_notebook::notebook::{reconcile_line_references_on_shift, resolve_line_references};
use urae_py::PySession;

#[test]
fn test_session_multi_turn_persistence_and_calculus() {
    let mut session = UraeSession::new();

    // Turn 1: Define parameter and domain
    let r1 = session.execute_line("a = 3");
    assert!(!r1.is_error);
    assert_eq!(session.get_binding("a"), Some(3.0));

    // Turn 2: Define domain restriction
    let r2 = session.execute_line("x in Reals [-10, 10]");
    assert!(!r2.is_error);
    assert!(session.symbol_domains.contains_key("x"));
    let bound = session.symbol_domains.get("x").unwrap();
    assert!(bound.contains_f64(0.0));
    assert!(!bound.contains_f64(20.0));

    // Turn 3: Evaluate expression referencing variable 'a'
    let r3 = session.execute_line("f(x) = a * x^2 + 5");
    assert!(!r3.is_error);

    // Turn 4: Differentiate
    let r4 = session.execute_line("diff(f(x), x)");
    assert!(!r4.is_error, "r4 error: {:?}", r4.error_msg);
    assert!(
        r4.output_unicode.contains('x')
            && (r4.output_unicode.contains('3') || r4.output_unicode.contains('6')),
        "r4 is: {:?}",
        r4
    );
}

#[test]
fn test_session_set_builder_execution() {
    let mut session = UraeSession::new();
    let res = session.execute_line("{ x in Reals | -5 <= x <= 5 }");
    assert!(!res.is_error);
    assert!(res.output_unicode.contains('x'));
    assert!(session.symbol_domains.contains_key("x"));
}

#[test]
fn test_jupyter_kernel_persistent_session() {
    let mut kernel = UraeKernel::new();

    // Cell 1: Set variable
    let reply1 = kernel.execute("k = 7");
    assert_eq!(reply1.status, "ok");

    // Cell 2: Use variable
    let reply2 = kernel.execute("k * 2");
    assert_eq!(reply2.status, "ok");
    let plain = reply2.mime_bundle.data.get("text/plain").unwrap();
    assert_eq!(plain, "14");

    // Cell 3: CAD Macro
    let reply3 = kernel.execute("gear!(teeth = 16, module = 1.5)");
    assert_eq!(reply3.status, "ok");
    assert!(reply3.mime_bundle.data.contains_key("text/html"));
}

#[test]
fn test_lsp_diagnostics_and_hover() {
    let doc = "x = 5\ny = x^2 + 1\n{ x in Reals | -10 <= x <= 10 }\n";
    let diags = UraeLanguageServer::diagnostics(doc);
    assert!(
        diags.is_empty(),
        "Document should have zero errors: {:?}",
        diags
    );

    let hover_item = UraeLanguageServer::hover(
        doc,
        Position {
            line: 1,
            character: 2,
        },
    );
    assert!(hover_item.is_some());
    let hover_text = hover_item.unwrap().contents;
    assert!(hover_text.contains("URAE Evaluated Expression"));

    let comps = UraeLanguageServer::completions("diff");
    assert!(!comps.is_empty());
    assert!(comps.iter().any(|c| c.label == "diff"));
}

#[test]
fn test_notebook_line_referencing_and_range_aggregations() {
    let mut results = HashMap::new();
    results.insert(0, 10.0); // $1
    results.insert(1, 20.0); // $2
    results.insert(2, 30.0); // $3
    results.insert(3, 40.0); // $4

    // Single line reference
    let res_single = resolve_line_references("$1 + $2", None, &results);
    assert_eq!(res_single, "10 + 20");

    // Range mathematical functions
    assert_eq!(
        resolve_line_references("sum($1..$4)", None, &results),
        "100"
    );
    assert_eq!(
        resolve_line_references("mean($1..$4)", None, &results),
        "25"
    );
    assert_eq!(resolve_line_references("min($1..$4)", None, &results), "10");
    assert_eq!(resolve_line_references("max($1..$4)", None, &results), "40");

    // Line shift rewriting
    let old_lines = vec!["a = 10", "b = 20", "c = $1 + $2"];
    let new_lines = vec!["# Title", "a = 10", "b = 20", "c = $1 + $2"];
    let rewritten = reconcile_line_references_on_shift(&old_lines, &new_lines);
    assert!(rewritten.is_some());
    let text = rewritten.unwrap();
    assert!(text.contains("c = $2 + $3"));
}

#[test]
fn test_python_and_ffi_session_runtimes() {
    let mut py_sess = PySession::new();
    py_sess.set_variable("m", 4.0);
    assert_eq!(py_sess.get_variable("m"), Some(4.0));

    let res = py_sess.execute("m * 5");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), "20");

    let diff_res = py_sess.execute("diff(x^3, x)");
    assert!(diff_res.is_ok());
    let diff_str = diff_res.unwrap();
    assert!(diff_str.contains('3') && diff_str.contains('x'));
}
