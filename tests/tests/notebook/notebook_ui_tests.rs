use urae_notebook::app::UraeNotebookApp;
use urae_notebook::notebook::NotebookState;

#[test]
fn test_notebook_initialization() {
    let state = NotebookState::default();
    assert!(!state.parsed_lines.is_empty());
    assert!(state.session.slider_values.contains_key("a"));
    assert!(state.session.slider_values.contains_key("c"));
}

#[test]
fn test_notebook_cell_evaluation() {
    let mut state = NotebookState::default();
    if let Some(m) = state.session.symbol_metadata.get_mut("a") {
        m.cur_val = 4.0;
    }
    state.session.raw_document_text = state.session.raw_document_text.replace("2.00", "4.00");
    state.evaluate_all();
    assert_eq!(*state.session.slider_values.get("a").unwrap(), 4.0);
}

#[test]
fn test_app_creation() {
    let app = UraeNotebookApp::default();
    assert!(!app.state.parsed_lines.is_empty());
}
