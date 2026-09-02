//! Integration Tests for Phase 19: Language Server Protocol & IDE Tooling (`urae-lsp`).

use urae_lsp::{DiagnosticSeverity, Position, Range, UraeLanguageServer};

#[test]
fn test_lsp_diagnostics_validation() {
    let valid_doc = "f(x) = x^2 + 3\ng(x) = sin(x)\n";
    let diags = UraeLanguageServer::diagnostics(valid_doc);
    assert_eq!(diags.len(), 0);

    // Use a standalone invalid expression (not a function definition) so the parser rejects it
    let invalid_doc = "diff (((x + 2\n";
    let diags_err = UraeLanguageServer::diagnostics(invalid_doc);
    assert!(!diags_err.is_empty());
    assert_eq!(diags_err[0].severity, DiagnosticSeverity::Error);
    assert_eq!(diags_err[0].range.start.line, 0);
}

#[test]
fn test_lsp_completions_filtering() {
    // Prefix "diff"
    let diff_items = UraeLanguageServer::completions("diff");
    assert!(diff_items.iter().any(|c| c.label == "diff"));

    // Prefix "gear"
    let gear_items = UraeLanguageServer::completions("gear");
    assert!(gear_items.iter().any(|c| c.label == "gear!"));

    // Prefix "alpha"
    let alpha_items = UraeLanguageServer::completions("alpha");
    assert!(alpha_items.iter().any(|c| c.label == "alpha"));
    assert_eq!(alpha_items[0].insert_text, "α");
}

#[test]
fn test_lsp_hover_tooltip() {
    let doc = "x^2 + 5";
    let hover = UraeLanguageServer::hover(
        doc,
        Position {
            line: 0,
            character: 2,
        },
    )
    .expect("hover missing");
    assert!(hover.contents.contains("URAE Evaluated Expression"));
    assert!(hover.contents.contains("$$\n{x}^{2} + 5\n$$") || hover.contents.contains("x^2"));
}

#[test]
fn test_lsp_code_actions_simplification() {
    let doc = "(x + 0) * 1 + 0";
    let actions = UraeLanguageServer::code_actions(
        doc,
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 15,
            },
        },
    );
    assert!(!actions.is_empty());
    assert_eq!(actions[0].new_text, "x");
    assert!(actions[0].title.contains("Simplify with E-Graphs"));
}
