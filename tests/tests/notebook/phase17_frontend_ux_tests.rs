//! Integration Tests for Phase 17 Frontend UX Enhancements: Command Palette, 3D Viewport & CAD Builder Palettes.

use urae_notebook::UraeNotebookApp;
use urae_notebook::ui::{CommandPalette, Viewport3DState, ViewportModelPreset};

#[test]
fn test_command_palette_command_filtering() {
    let cmds = CommandPalette::all_commands();
    assert!(cmds.len() >= 15);

    // Test filtering by calculus keyword
    let diff_cmd = cmds
        .iter()
        .find(|c| c.title.contains("Differentiate"))
        .expect("diff command missing");
    assert_eq!(diff_cmd.category, "Calculus");
    assert!(diff_cmd.syntax_template.contains("diff("));

    // Test filtering by CAD keyword
    let gear_cmd = cmds
        .iter()
        .find(|c| c.title.contains("Involute"))
        .expect("gear command missing");
    assert_eq!(gear_cmd.category, "CAD Machinery");
    assert!(gear_cmd.syntax_template.contains("gear!"));

    // Test filtering by Greek symbols
    let alpha_cmd = cmds
        .iter()
        .find(|c| c.title.contains("Alpha"))
        .expect("alpha command missing");
    assert_eq!(alpha_cmd.category, "Symbols");
    assert_eq!(alpha_cmd.syntax_template, "α");
}

#[test]
fn test_viewport3d_state_and_projection() {
    let mut state = Viewport3DState::default();
    assert_eq!(state.active_model, ViewportModelPreset::InvoluteGear);
    assert!(!state.wireframe);
    assert!(state.zoom > 0.0);

    // Switch presets
    state.active_model = ViewportModelPreset::ThreadedBolt;
    state.wireframe = true;
    assert_eq!(state.active_model, ViewportModelPreset::ThreadedBolt);
    assert!(state.wireframe);
}

#[test]
fn test_app_3d_and_palette_state_integration() {
    let app = UraeNotebookApp::default();
    assert!(!app.show_viewport_3d);
    assert!(!app.command_palette_state.is_open);
    assert!(!app.show_cad_machinery_palette);
}
