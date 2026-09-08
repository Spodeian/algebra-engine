//! Integration Tests for URAE Transactional Undo and Redo Engine.

use std::collections::HashMap;
use urae_notebook::notebook::{HistorySnapshot, NotebookState, UndoRedoHistory};

#[test]
fn test_undo_redo_history_basic_push_and_capacity() {
    let mut history = UndoRedoHistory::new(3);
    assert!(!history.can_undo());
    assert!(!history.can_redo());

    for i in 1..=5 {
        history.push(HistorySnapshot {
            raw_document_text: format!("line {}", i),
            slider_values: HashMap::new(),
            symbol_metadata: HashMap::new(),
            card_modes: HashMap::new(),
            description: format!("Edit {}", i),
        });
    }

    assert!(history.can_undo());
    assert_eq!(history.undo_stack.len(), 3);
    assert_eq!(history.undo_description(), Some("Edit 5"));
}

#[test]
fn test_notebook_state_line_edit_undo_redo() {
    let mut state = NotebookState::new_clean();
    let initial_text = state.session.raw_document_text.clone();

    // 1. Edit line 0
    state.start_editing_line(0);
    state.line_edit_buffer = "# Modified Title by User".to_string();
    state.commit_focused_line();

    assert_ne!(state.session.raw_document_text, initial_text);
    assert!(
        state
            .session
            .raw_document_text
            .starts_with("# Modified Title by User")
    );
    assert!(state.can_undo());

    // 2. Undo
    let undone = state.undo();
    assert!(undone);
    assert_eq!(state.session.raw_document_text, initial_text);
    assert!(state.can_redo());

    // 3. Redo
    let redone = state.redo();
    assert!(redone);
    assert!(
        state
            .session
            .raw_document_text
            .starts_with("# Modified Title by User")
    );
}

#[test]
fn test_notebook_state_insert_text_undo_redo() {
    let mut state = NotebookState::new_clean();
    let initial_lines_count = state.session.raw_document_text.lines().count();

    // 1. Insert new syntax
    state.insert_text_at_active_line("velocity_test = 42.0 [m/s]");
    assert!(
        state
            .session
            .raw_document_text
            .contains("velocity_test = 42.0 [m/s]")
    );
    assert_eq!(
        state.session.raw_document_text.lines().count(),
        initial_lines_count + 1
    );

    // 2. Undo insertion
    assert!(state.can_undo());
    assert!(state.undo());
    assert!(
        !state
            .session
            .raw_document_text
            .contains("velocity_test = 42.0 [m/s]")
    );
    assert_eq!(
        state.session.raw_document_text.lines().count(),
        initial_lines_count
    );

    // 3. Redo insertion
    assert!(state.can_redo());
    assert!(state.redo());
    assert!(
        state
            .session
            .raw_document_text
            .contains("velocity_test = 42.0 [m/s]")
    );
}

#[test]
fn test_notebook_state_preset_undo_redo() {
    let mut state = NotebookState::new_clean();
    let original_text = state.session.raw_document_text.clone();

    // 1. Load preset
    state.load_preset("oscillator");
    assert!(
        state
            .session
            .raw_document_text
            .contains("Harmonic Wave Motion")
    );

    // 2. Undo
    assert!(state.undo());
    assert_eq!(state.session.raw_document_text, original_text);

    // 3. Redo
    assert!(state.redo());
    assert!(
        state
            .session
            .raw_document_text
            .contains("Harmonic Wave Motion")
    );
}

#[test]
fn test_multiple_consecutive_undo_redo_chain() {
    let mut state = NotebookState::new_clean();
    let mut history_checkpoints = Vec::new();
    history_checkpoints.push(state.session.raw_document_text.clone());

    for i in 1..=4 {
        state.insert_text_at_active_line(&format!("x_{} = {}", i, i * 10));
        history_checkpoints.push(state.session.raw_document_text.clone());
    }

    // Step backward 4 times
    for i in (0..4).rev() {
        assert!(state.undo());
        assert_eq!(state.session.raw_document_text, history_checkpoints[i]);
    }

    assert!(!state.can_undo());

    // Step forward 4 times
    for expected_checkpoint in history_checkpoints.iter().skip(1) {
        assert!(state.redo());
        assert_eq!(&state.session.raw_document_text, expected_checkpoint);
    }

    assert!(!state.can_redo());
}
