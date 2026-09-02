//! Integration and verification tests for Phase 9: Fluid In-Situ Reactive Notepad & Canvas Frontend.

use urae_notebook::app::{UraeNotebookApp, ViewMode};
use urae_notebook::notebook::{
    generate_branch_cut_syntax, generate_interval_syntax, generate_matrix_syntax,
    generate_ode_bc_syntax, generate_physical_unit_syntax, MatrixPresetKind, NotebookSettings,
    NotebookState, ReactiveComputeMode, SymbolRole,
};

#[test]
fn test_phase9_view_mode_defaults() {
    let app = UraeNotebookApp::default();
    assert_eq!(app.view_mode, ViewMode::SmartStream);
    assert_eq!(
        app.state.session.settings.reactive_mode,
        ReactiveComputeMode::AdaptiveDualRate
    );
    assert!(app.state.session.settings.auto_throttle_enabled);
    assert_eq!(app.state.session.settings.target_frame_budget_ms, 8.0);
}

#[test]
fn test_phase9_reactive_compute_modes() {
    let mut settings = NotebookSettings::default();
    assert_eq!(
        settings.reactive_mode,
        ReactiveComputeMode::AdaptiveDualRate
    );

    settings.reactive_mode = ReactiveComputeMode::SoftCapFastDrag;
    assert_eq!(settings.reactive_mode, ReactiveComputeMode::SoftCapFastDrag);

    settings.reactive_mode = ReactiveComputeMode::FrameBudgeted;
    assert_eq!(settings.reactive_mode, ReactiveComputeMode::FrameBudgeted);

    settings.reactive_mode = ReactiveComputeMode::FullSynchronous;
    assert_eq!(settings.reactive_mode, ReactiveComputeMode::FullSynchronous);
}

#[test]
fn test_phase9_in_situ_line_editing_and_focus() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "a = 2.0\nf(x) = a * x^2 + 1".to_string();
    state.evaluate_all();

    assert_eq!(state.parsed_lines.len(), 2);
    assert_eq!(state.focused_line, None);

    // Start editing Line 1
    state.start_editing_line(1);
    assert_eq!(state.focused_line, Some(1));
    assert_eq!(state.line_edit_buffer, "f(x) = a * x^2 + 1");

    // Modify buffer and commit
    state.line_edit_buffer = "f(x) = a * x^3 - 4".to_string();
    state.commit_focused_line();

    assert_eq!(state.focused_line, None);
    assert_eq!(
        state.session.raw_document_text,
        "a = 2.0\nf(x) = a * x^3 - 4"
    );
    assert_eq!(state.parsed_lines[1].raw_text, "f(x) = a * x^3 - 4");
}

#[test]
fn test_phase9_insert_text_at_active_line() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "x: Variable\nf(x) = x^2".to_string();
    state.evaluate_all();

    // Insert at focused line 0
    state.start_editing_line(0);
    state.insert_text_at_active_line("a = 5.00 [m]");

    assert_eq!(state.session.raw_document_text, "a = 5.00 [m]\nf(x) = x^2");
    assert_eq!(state.parsed_lines[0].raw_text, "a = 5.00 [m]");

    // Insert without focus (appends at end)
    state.focused_line = None;
    state.insert_text_at_active_line("g(x) = 2 * x");
    assert_eq!(state.parsed_lines.len(), 3);
    assert_eq!(state.parsed_lines[2].raw_text, "g(x) = 2 * x");
}

#[test]
fn test_phase9_reactive_delta_invalidation() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "a = 2.0\nf(x) = a * x^2 + 3".to_string();
    state.evaluate_all();

    assert!(state.symbol_dependencies.contains_key("a"));
    assert_eq!(
        state.parsed_lines[1].substituted_latex,
        Some("2 \\cdot {x}^{2} + 3".to_string())
    );

    // Change parameter slider value for 'a' to 10.0
    state.session.slider_values.insert("a".to_string(), 10.0);
    state.evaluate_delta("a");

    assert_eq!(
        state.parsed_lines[1].substituted_latex,
        Some("10 \\cdot {x}^{2} + 3".to_string())
    );
    assert!(state.measured_eval_latency_ms >= 0.0);
}

#[test]
fn test_phase9_matrix_builder_syntax_generation() {
    assert_eq!(
        generate_matrix_syntax(3, 3, MatrixPresetKind::Identity, None),
        "M = eye(3)"
    );
    assert_eq!(
        generate_matrix_syntax(2, 4, MatrixPresetKind::Zero, None),
        "M = zeros(2, 4)"
    );
    assert_eq!(
        generate_matrix_syntax(3, 3, MatrixPresetKind::Diagonal, None),
        "M = diag([d_1, d_2, d_3])"
    );
    assert_eq!(
        generate_matrix_syntax(2, 2, MatrixPresetKind::Symmetric, None),
        "M = matrix([[s_11, s_12], [s_12, s_22]])"
    );
    assert_eq!(
        generate_matrix_syntax(2, 2, MatrixPresetKind::PauliX, None),
        "sigma_x = matrix([[0, 1], [1, 0]])"
    );
    assert_eq!(
        generate_matrix_syntax(2, 2, MatrixPresetKind::PauliY, None),
        "sigma_y = matrix([[0, -i], [i, 0]])"
    );
    assert_eq!(
        generate_matrix_syntax(2, 2, MatrixPresetKind::PauliZ, None),
        "sigma_z = matrix([[1, 0], [0, -1]])"
    );

    let custom_grid = vec![
        vec!["1".to_string(), "2".to_string()],
        vec!["3".to_string(), "4".to_string()],
    ];
    assert_eq!(
        generate_matrix_syntax(2, 2, MatrixPresetKind::Custom, Some(&custom_grid)),
        "M = matrix([[1, 2], [3, 4]])"
    );
}

#[test]
fn test_phase9_interval_picker_syntax_generation() {
    let syn1 = generate_interval_syntax("x", "Reals", -5.0, 5.0, true, true);
    assert_eq!(syn1, "{ x in Reals | -5.00 <= x <= 5.00 }");

    let syn2 = generate_interval_syntax("t", "Reals", 0.0, 100.0, true, false);
    assert_eq!(syn2, "{ t in Reals | 0.00 <= t < 100.00 }");
}

#[test]
fn test_phase9_branch_cut_syntax_generation() {
    let syn = generate_branch_cut_syntax("log", 0, "(-infinity, 0]");
    assert_eq!(
        syn,
        "branch_cut(\"log\", sheet = 0, cut = \"(-infinity, 0]\")"
    );

    let syn_sqrt = generate_branch_cut_syntax("sqrt", 1, "(-infinity, 0]");
    assert_eq!(
        syn_sqrt,
        "branch_cut(\"sqrt\", sheet = 1, cut = \"(-infinity, 0]\")"
    );
}

#[test]
fn test_phase9_ode_wizard_syntax_generation() {
    let syn_cauchy = generate_ode_bc_syntax("y", "x", 1.0, Some(0.0));
    assert_eq!(syn_cauchy, "y(0) = 1.00, y'(x) = 0.00");

    let syn_dirichlet = generate_ode_bc_syntax("u", "t", 0.0, None);
    assert_eq!(syn_dirichlet, "u(0) = 0.00");
}

#[test]
fn test_phase9_physical_units_palette_syntax_generation() {
    let syn_v = generate_physical_unit_syntax("v", 10.0, "m/s");
    assert_eq!(syn_v, "v = 10.00 [m/s]");

    let syn_a = generate_physical_unit_syntax("a", 9.81, "m/s^2");
    assert_eq!(syn_a, "a = 9.81 [m/s^2]");
}

#[test]
fn test_phase9_full_notepad_app_initialization() {
    let app = UraeNotebookApp::default();
    assert!(!app.state.parsed_lines.is_empty());
    assert!(app.state.session.symbol_metadata.contains_key("a"));
    assert_eq!(
        app.state.session.symbol_metadata["a"].role,
        SymbolRole::Parameter
    );
    assert_eq!(
        app.state.session.symbol_metadata["x"].role,
        SymbolRole::Variable
    );
}
