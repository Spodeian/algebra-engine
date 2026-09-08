//! Integration Tests for In-App Example Notebook Gallery & Interactive Tutorials.

use std::collections::HashSet;
use urae_notebook::UraeNotebookApp;
use urae_notebook::examples::{ExampleCategory, ExampleRegistry};

#[test]
fn test_example_registry_all_examples_count_and_uniqueness() {
    let examples = ExampleRegistry::all_examples();
    assert!(examples.len() >= 12);

    let mut ids = HashSet::new();
    for ex in examples {
        assert!(!ex.id.is_empty());
        assert!(!ex.title.is_empty());
        assert!(!ex.description.is_empty());
        assert!(!ex.content.is_empty());
        assert!(!ex.tags.is_empty());
        assert!(ids.insert(ex.id), "Duplicate example ID found: {}", ex.id);
    }
}

#[test]
fn test_example_registry_category_filtering() {
    for cat in ExampleCategory::all() {
        let list = ExampleRegistry::filter_by_category(*cat);
        assert!(!list.is_empty(), "Category {:?} has no examples", cat);
        for ex in list {
            assert_eq!(ex.category, *cat);
        }
    }
}

#[test]
fn test_example_registry_search() {
    // Search by title or tag
    let qft_results = ExampleRegistry::search("quantum");
    assert!(!qft_results.is_empty());

    let cad_results = ExampleRegistry::search("gear");
    assert!(!cad_results.is_empty());

    let weyl_results = ExampleRegistry::search("zeilberger");
    assert!(!weyl_results.is_empty());

    let empty_search = ExampleRegistry::search("");
    assert_eq!(empty_search.len(), ExampleRegistry::all_examples().len());
}

#[test]
fn test_app_example_loading_state_transition() {
    let mut app = UraeNotebookApp::default();
    let qft_ex = ExampleRegistry::find_by_id("02_quantum_field_theory_and_clifford").unwrap();

    // Simulate loading example
    app.state.session.raw_document_text = qft_ex.content.to_string();
    app.state.evaluate_all();
    app.active_notebook = format!("{}.math", qft_ex.id);

    assert!(app.state.session.raw_document_text.contains("dirac_trace"));
    assert_eq!(
        app.active_notebook,
        "02_quantum_field_theory_and_clifford.math"
    );
}
