//! Integration and unit tests for Smart Stream Phase A:
//! Reference handling (ans, $N), ViewMode defaults, and layout state.

use std::collections::HashMap;
use urae_notebook::app::{UraeNotebookApp, ViewMode};
use urae_notebook::notebook::NotebookState;

#[test]
fn test_resolve_line_references_ans() {
    let mut line_results = HashMap::new();
    line_results.insert(0, 115.0);

    let resolved = NotebookState::resolve_line_references("ans + 25", Some(115.0), &line_results);
    assert_eq!(resolved, "115 + 25");

    let resolved_no_ans =
        NotebookState::resolve_line_references("pans + 1", Some(115.0), &line_results);
    assert_eq!(resolved_no_ans, "pans + 1");
}

#[test]
fn test_resolve_line_references_dollar_n() {
    let mut line_results = HashMap::new();
    line_results.insert(0, 50.0);
    line_results.insert(1, 100.0);

    let resolved = NotebookState::resolve_line_references("$1 * 2 + $2", None, &line_results);
    assert_eq!(resolved, "50 * 2 + 100");
}

#[test]
fn test_notebook_evaluate_with_references() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "x = 10\nans * 2\n$2 + 5".to_string();
    state.evaluate_all();

    assert_eq!(state.parsed_lines.len(), 3);
    assert!(state.parsed_lines[0].output_unicode.contains("x = 10"));
}

#[test]
fn test_smart_stream_view_mode() {
    let mut app = UraeNotebookApp::default();
    assert_eq!(app.view_mode, ViewMode::SmartStream);

    app.view_mode = ViewMode::FocusEditor;
    assert_eq!(app.view_mode, ViewMode::FocusEditor);
}

#[test]
fn test_parse_domain_declaration_intervals() {
    let bound = NotebookState::parse_domain_declaration("a in Reals [0, 100]")
        .expect("Must parse real interval");
    assert_eq!(bound.name, "a");
    assert_eq!(bound.domain_type, "Reals");
    assert_eq!(bound.min_val, Some(0.0));
    assert_eq!(bound.max_val, Some(100.0));
    assert!(bound.inclusive_min);
    assert!(bound.inclusive_max);

    let int_bound = NotebookState::parse_domain_declaration("n in Integers [1, 10]")
        .expect("Must parse integer interval");
    assert_eq!(int_bound.name, "n");
    assert_eq!(int_bound.domain_type, "Integers");
    assert_eq!(int_bound.min_val, Some(1.0));
    assert_eq!(int_bound.max_val, Some(10.0));

    let pos_bound = NotebookState::parse_domain_declaration("x in Positive")
        .expect("Must parse positive domain");
    assert_eq!(pos_bound.name, "x");
    assert_eq!(pos_bound.domain_type, "Positive");
}

#[test]
fn test_domain_declaration_and_validation() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "a in Reals [0, 100]\na = -5".to_string();
    state.evaluate_all();

    assert_eq!(state.parsed_lines.len(), 2);
    assert!(state.parsed_lines[0]
        .output_unicode
        .contains("a ∈ ℝ ∩ [0.00, 100.00]"));

    // Second line should trigger Out of domain warning badge
    assert!(state.parsed_lines[1].error_msg.is_some());
    assert!(state.parsed_lines[1]
        .error_msg
        .as_ref()
        .unwrap()
        .contains("Out of domain"));
}

#[test]
fn test_domain_propagation_to_plot_points() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "x in Reals [0, 5]\nf(x) = x^2".to_string();
    state.evaluate_all();

    let pts = NotebookState::evaluate_plot_points_with_domains(
        &state.graph,
        &state.session.slider_values,
        Some(&state.session.symbol_metadata),
        "f(x) = x^2",
        "x",
        -5.0,
        10.0,
        100,
    );

    // All sample points must be restricted to [0.0, 5.0]
    assert!(!pts.is_empty());
    for p in pts {
        assert!(p[0] >= 0.0 - 1e-6, "Point x={} below domain min 0.0", p[0]);
        assert!(p[0] <= 5.0 + 1e-6, "Point x={} above domain max 5.0", p[0]);
    }
}

#[test]
fn test_command_palette_slash_matches() {
    let all_cmds = urae_notebook::ui::CommandPalette::all_commands();
    assert!(!all_cmds.is_empty());

    let plot_matches: Vec<_> = all_cmds
        .iter()
        .filter(|c| c.title.to_lowercase().contains("diff") || c.shortcut == Some("diff"))
        .collect();
    assert!(!plot_matches.is_empty());
    assert!(plot_matches[0].syntax_template.contains("diff("));
}

#[test]
fn test_gutter_slider_domain_clamping() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "a in Reals [10, 50]\na = 25".to_string();
    state.evaluate_all();

    let meta = state
        .session
        .symbol_metadata
        .get("a")
        .expect("Symbol 'a' must exist");
    assert_eq!(meta.min_val, 10.0);
    assert_eq!(meta.max_val, 50.0);
    assert_eq!(meta.cur_val, 25.0);
}

#[test]
fn test_markdown_export() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "# Test Notepad\nx = 10\ny = 20\nx + y".to_string();
    state.evaluate_all();

    let md = state.session.export_markdown();
    assert!(md.contains("# Test Notepad") || md.contains("x = 10"));
}

#[test]
fn test_app_debounced_autosave_state() {
    let app = UraeNotebookApp::default();
    assert!(!app.is_edit_dirty);
    assert!(!app.is_save_dirty);
    assert!(app.last_keystroke_time.elapsed().as_secs() < 5);
}

#[test]
fn test_theme_math_syntax_layouter_tokens() {
    let theme = urae_notebook::ui::ThemeKind::Dark;
    let palette = theme.palette();
    // Verify palette colors exist and are distinctive
    assert_ne!(palette.math_var, palette.math_number);
    assert_ne!(palette.math_keyword, palette.math_comment);
    assert_ne!(palette.accent_primary, palette.text_primary);
}

#[test]
fn test_resolve_range_mathematical_aggregations() {
    let mut line_results = HashMap::new();
    line_results.insert(0, 10.0);
    line_results.insert(1, 20.0);
    line_results.insert(2, 30.0);

    let sum_res = NotebookState::resolve_line_references("sum($1..$3)", None, &line_results);
    assert_eq!(sum_res, "60");

    let mean_res = NotebookState::resolve_line_references("mean($1..$3)", None, &line_results);
    assert_eq!(mean_res, "20");

    let min_res = NotebookState::resolve_line_references("min($1..$3)", None, &line_results);
    assert_eq!(min_res, "10");

    let max_res = NotebookState::resolve_line_references("max($1..$3)", None, &line_results);
    assert_eq!(max_res, "30");

    let prod_res = NotebookState::resolve_line_references("prod($1..$3)", None, &line_results);
    assert_eq!(prod_res, "6000");
}

#[test]
fn test_reconcile_line_references_on_shift() {
    let old_lines = vec!["x = 10", "y = 20", "total = $1 + $2"];

    // User inserts a comment at Line 1
    let new_lines = vec!["// Inserted header", "x = 10", "y = 20", "total = $1 + $2"];

    let reconciled = NotebookState::reconcile_line_references_on_shift(&old_lines, &new_lines);
    assert!(reconciled.is_some());
    let reconciled_text = reconciled.unwrap();
    assert!(reconciled_text.contains("total = $2 + $3"));
}

#[test]
fn test_refactor_line_to_named_variable() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "10 + 20\n$1 * 2\n$1 + 5".to_string();
    state.evaluate_all();

    let success = state.refactor_line_to_named_variable(0, "sum_val");
    assert!(success);

    assert!(state
        .session
        .raw_document_text
        .contains("sum_val = 10 + 20"));
    assert!(state.session.raw_document_text.contains("sum_val * 2"));
    assert!(state.session.raw_document_text.contains("sum_val + 5"));
    assert_eq!(state.parsed_lines.len(), 3);
}

#[test]
fn test_background_evaluator_epoch_preemption() {
    use urae_notebook::notebook::BackgroundEvaluator;
    let evaluator = BackgroundEvaluator::new();

    let raw_lines = vec!["x = 42".to_string(), "y = x * 2".to_string()];
    let epoch = evaluator.submit(raw_lines, HashMap::new(), HashMap::new());
    assert!(epoch >= 1);

    // Wait briefly for background evaluation
    std::thread::sleep(std::time::Duration::from_millis(100));

    let res = evaluator.try_recv_latest();
    assert!(res.is_some());
    let response = res.unwrap();
    assert_eq!(response.epoch, epoch);
    assert_eq!(response.parsed_lines.len(), 2);
}

#[test]
fn test_dependency_graph_and_selective_cancellation() {
    use std::collections::HashSet;
    use urae_notebook::notebook::{BackgroundEvaluator, DependencyGraph};

    let lines = vec![
        "a = 5".to_string(),      // Line 0: depends on {a}
        "b = a + 2".to_string(),  // Line 1: depends on {b, a}
        "c = $2 * 3".to_string(), // Line 2: depends on {c}, references line 1 ($2) -> transitively depends on {a}
        "z = 100".to_string(),    // Line 3: depends on {z} (completely independent of a)
    ];

    let dep_graph = DependencyGraph::build(&lines);

    // Transitive dependencies of Line 2 (c = $2 * 3) should include line 1 and symbol 'a'
    let (trans_syms, trans_lines) = dep_graph.get_transitive_dependencies(2);
    assert!(trans_syms.contains("a") || trans_syms.contains("c"));
    assert!(trans_lines.contains(&1));

    // When parameter 'a' is modified:
    let mut changed_syms = HashSet::new();
    changed_syms.insert("a".to_string());
    let changed_lines = HashSet::new();

    assert!(dep_graph.should_cancel_line(0, &changed_syms, &changed_lines));
    assert!(dep_graph.should_cancel_line(1, &changed_syms, &changed_lines));
    assert!(dep_graph.should_cancel_line(2, &changed_syms, &changed_lines));
    // Line 3 (z = 100) must NOT be cancelled!
    assert!(!dep_graph.should_cancel_line(3, &changed_syms, &changed_lines));

    // Test in BackgroundEvaluator
    let evaluator = BackgroundEvaluator::new();
    let mut sliders = HashMap::new();
    sliders.insert("a".to_string(), 5.0);
    evaluator.submit(lines.clone(), sliders.clone(), HashMap::new());

    // Acquire tokens
    let tok0 = evaluator.get_line_token(0);
    let tok3 = evaluator.get_line_token(3);
    assert!(!tok0.is_cancelled());
    assert!(!tok3.is_cancelled());

    // Submit modification to slider 'a'
    sliders.insert("a".to_string(), 10.0);
    evaluator.submit(lines.clone(), sliders, HashMap::new());

    // Token for Line 0 (dependent on 'a') must be cancelled
    assert!(tok0.is_cancelled());
    // Token for Line 3 (independent of 'a') must NOT be cancelled!
    assert!(!tok3.is_cancelled());
}

#[test]
fn test_compute_line_upstream_dependencies() {
    use urae_notebook::app::compute_line_upstream_dependencies;

    let mut state = NotebookState::default();
    state.session.raw_document_text = "a = 10\nb = a * 2\nc = b + $1\nz = 99".to_string();
    state.evaluate_all();

    // Line 2 (c = b + $1) depends directly on Line 1 (b), Line 0 ($1), and transitively on Line 0 (a)
    let deps_line2 = compute_line_upstream_dependencies(2, &state.parsed_lines);
    assert!(deps_line2.contains(&2)); // self
    assert!(deps_line2.contains(&1)); // b
    assert!(deps_line2.contains(&0)); // $1 and a
    assert!(!deps_line2.contains(&3)); // z is independent

    // Line 1 (b = a * 2) depends on Line 0 (a) and self
    let deps_line1 = compute_line_upstream_dependencies(1, &state.parsed_lines);
    assert!(deps_line1.contains(&1));
    assert!(deps_line1.contains(&0));
    assert!(!deps_line1.contains(&2));
    assert!(!deps_line1.contains(&3));

    // Line 3 (z = 99) depends only on self
    let deps_line3 = compute_line_upstream_dependencies(3, &state.parsed_lines);
    assert_eq!(deps_line3.len(), 1);
    assert!(deps_line3.contains(&3));
}

#[test]
fn test_open_or_focus_unique_inspector_windows() {
    use urae_notebook::UraeNotebookApp;

    let mut app = UraeNotebookApp::default();
    assert!(app.active_open_inspectors.is_empty());

    // 1. Open inspector for 'a'
    app.open_or_focus_inspector("a");
    assert_eq!(app.active_open_inspectors, vec!["a"]);

    // 2. Open inspector for 'b'
    app.open_or_focus_inspector("b");
    assert_eq!(app.active_open_inspectors, vec!["a", "b"]);

    // 3. Open inspector for 'c'
    app.open_or_focus_inspector("c");
    assert_eq!(app.active_open_inspectors, vec!["a", "b", "c"]);

    // 4. Press 'a' again from another tooltip: must bring 'a' to the front rather than creating a duplicate
    app.open_or_focus_inspector("a");
    assert_eq!(app.active_open_inspectors.len(), 3);
    assert_eq!(app.active_open_inspectors, vec!["b", "c", "a"]);

    // 5. Press 'b' again: must bring 'b' to the front
    app.open_or_focus_inspector("b");
    assert_eq!(app.active_open_inspectors.len(), 3);
    assert_eq!(app.active_open_inspectors, vec!["c", "a", "b"]);
}

#[test]
fn test_wrapped_lines_do_not_get_duplicate_line_numbers() {
    use urae_notebook::egui;

    let ctx = egui::Context::default();
    let long_formula = "f(x) = sin(x) + cos(x) * exp(-x^2) + sqrt(x^2 + 1) + ln(abs(x) + 1) + tan(x) * 2.50";
    let doc_text = format!("{}\ny = 42", long_formula);

    let mut galley_opt = None;
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let palette = urae_notebook::ui::ThemeKind::Dark.palette();
            let galley = palette.math_syntax_layouter(ui, &doc_text, 120.0);
            galley_opt = Some(galley);
        });
    });

    let galley = galley_opt.expect("Galley must be produced by layouter");
    assert!(galley.rows.len() > 2, "Expected wrapped lines across multiple visual rows (got {})", galley.rows.len());

    // Count how many visual rows are starts of new logical lines
    let mut logical_line_count = 0;
    let mut is_new_logical_line = true;
    let mut unnumbered_wrapped_rows = 0;

    for row in &galley.rows {
        if is_new_logical_line {
            logical_line_count += 1;
        } else {
            unnumbered_wrapped_rows += 1;
        }
        is_new_logical_line = row.ends_with_newline;
    }

    assert_eq!(logical_line_count, 2, "Must identify exactly 2 logical lines");
    assert!(unnumbered_wrapped_rows >= 1, "Wrapped continuation rows must remain unnumbered");
}

#[test]
fn test_smart_plot_bounds_baseline_and_critical_points() {
    let state = NotebookState::default();

    // 1. Simple linear function f(x) = 2*x: must cover at least [-1.0, 1.0]
    let (min_lin, max_lin) = state.compute_smart_plot_bounds("f(x) = 2*x", "x");
    assert!(min_lin <= -1.0, "Must cover at least x = -1.0, got {}", min_lin);
    assert!(max_lin >= 1.0, "Must cover at least x = 1.0, got {}", max_lin);

    // 2. Parabola f(x) = x^2 - 9: roots at x = -3, +3 and vertex at x = 0
    let (min_quad, max_quad) = state.compute_smart_plot_bounds("f(x) = x^2 - 9", "x");
    assert!(min_quad < -3.0, "Must include left root -3 with margin, got {}", min_quad);
    assert!(max_quad > 3.0, "Must include right root +3 with margin, got {}", max_quad);
    assert!(min_quad <= -1.0 && max_quad >= 1.0, "Must contain [-1, 1]");

    // 3. Cubic f(x) = x^3 - 12*x: roots at x = -sqrt(12) (~ -3.46), 0, +3.46; critical points at x = -2, 2
    let (min_cub, max_cub) = state.compute_smart_plot_bounds("f(x) = x^3 - 12*x", "x");
    assert!(min_cub < -3.46, "Must encompass all roots and extrema, got {}", min_cub);
    assert!(max_cub > 3.46, "Must encompass all roots and extrema, got {}", max_cub);
}

#[test]
fn test_smart_plot_bounds_periodic_trigonometric() {
    let state = NotebookState::default();

    // 1. Standard sine wave f(x) = sin(x): period T = 2*pi (~ 6.28)
    // 1-3 periods means bounds should span ~ 12.56
    let (min_sin, max_sin) = state.compute_smart_plot_bounds("f(x) = sin(x)", "x");
    assert!(min_sin <= -1.0 && max_sin >= 1.0, "Must contain [-1, 1]");
    let span_sin = max_sin - min_sin;
    assert!((std::f64::consts::TAU..=20.0).contains(&span_sin), "Must cover 1-3 full periods of sin(x), got span {}", span_sin);

    // 2. High frequency sine wave f(x) = sin(4*x): period T = 2*pi/4 (~ 1.57)
    // Must cover 1-3 full periods AND at least [-1.0, 1.0]
    let (min_fast, max_fast) = state.compute_smart_plot_bounds("f(x) = sin(4*x)", "x");
    assert!(min_fast <= -1.0, "Must cover at least -1.0, got {}", min_fast);
    assert!(max_fast >= 1.0, "Must cover at least 1.0, got {}", max_fast);
}

#[test]
fn test_smart_plot_bounds_multi_frequency_sub_periods() {
    let state = NotebookState::default();

    // Multi-frequency signal: carrier + modulation f(x) = sin(10*x) + cos(x)
    // T_min = 2*pi/10 (~ 0.628), T_max = 2*pi (~ 6.283). Ratio = 10 > 3.0
    // Must cover 1-2 periods of the modulation envelope T_max (~ 6.28)
    let (min_multi, max_multi) = state.compute_smart_plot_bounds("f(x) = sin(10*x) + cos(x)", "x");
    let span_multi = max_multi - min_multi;
    assert!(span_multi >= std::f64::consts::TAU, "Must cover envelope period, got span {}", span_multi);
    assert!(min_multi <= -1.0 && max_multi >= 1.0, "Must contain [-1, 1]");
}

#[test]
fn test_smart_plot_bounds_natural_domain_clamping() {
    let state = NotebookState::default();

    // Non-negative function f(x) = sqrt(x): natural domain is x >= 0
    let (min_sqrt, max_sqrt) = state.compute_smart_plot_bounds("f(x) = sqrt(x)", "x");
    assert_eq!(min_sqrt, 0.0, "Square root domain must be clamped to non-negative 0.0");
    assert!(max_sqrt >= 1.0, "Square root upper bound must cover at least 1.0, got {}", max_sqrt);
}

#[test]
fn test_function_tooltip_vm_and_symbolic_features() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = "f(x) = x^2 - 9\ng(x) = sin(x)".to_string();
    state.evaluate_all();

    // 1. Inspect parabola f(x) = x^2 - 9
    let card_f = state
        .get_symbol_info_card("f")
        .expect("f function card should be resolved");
    assert_eq!(card_f.name, "f");

    if let Some(urae_notebook::notebook::ObjectKind::Function {
        roots,
        critical_points,
        range,
        parity,
        is_linear,
        ..
    }) = &card_f.object_kind
    {
        assert!(!is_linear, "Parabola must be non-linear");
        assert!(!roots.is_empty(), "Must discover roots for x^2 - 9");
        let roots_joined = roots.join(", ");
        assert!(roots_joined.contains("3") || roots_joined.contains("-3"), "Roots must contain 3 or -3, got: {}", roots_joined);

        assert!(!critical_points.is_empty(), "Must discover critical point (vertex) for x^2 - 9");
        let crit_joined = critical_points.join(", ");
        assert!(crit_joined.contains("0"), "Critical point must be at x = 0, got: {}", crit_joined);

        assert!(range.is_some(), "Range must be inferred");
        let range_str = range.as_ref().unwrap();
        assert!(range_str.contains("-9"), "Range for x^2 - 9 must start at -9, got: {}", range_str);

        assert_eq!(parity.as_deref(), Some("Even: f(-x) = f(x)"));
    } else {
        panic!("Expected ObjectKind::Function, got {:?}", card_f.object_kind);
    }

    // 2. Inspect periodic g(x) = sin(x)
    let card_g = state
        .get_symbol_info_card("g")
        .expect("g function card should be resolved");
    if let Some(urae_notebook::notebook::ObjectKind::Function {
        period,
        parity,
        ..
    }) = &card_g.object_kind
    {
        assert!(period.is_some(), "sin(x) must have detected period");
        let t = period.unwrap();
        assert!((t - std::f64::consts::TAU).abs() < 0.1, "Period must be ~ 2*pi, got: {}", t);
        assert_eq!(parity.as_deref(), Some("Odd: f(-x) = -f(x)"));
    } else {
        panic!("Expected ObjectKind::Function, got {:?}", card_g.object_kind);
    }
}

#[test]
fn test_numeric_literal_scrubbing_extraction() {
    use urae_notebook::app::find_numeric_literal_at;

    // 1. Plain integer
    let text = "a = 42";
    let res = find_numeric_literal_at(text, 4).expect("Should find 42");
    assert_eq!(&text[res.0..res.1], "42");
    assert_eq!(res.2, 42.0);
    assert!(!res.3); // no decimal
    assert_eq!(res.4, 0);

    // 2. Negative float with decimals
    let text2 = "f(x) = -3.1415 * x";
    let res2 = find_numeric_literal_at(text2, 9).expect("Should find -3.1415");
    assert_eq!(&text2[res2.0..res2.1], "-3.1415");
    assert!((res2.2 - (-3.1415)).abs() < 1e-6);
    assert!(res2.3); // has decimal
    assert_eq!(res2.4, 4); // 4 decimal places

    // 3. Subtraction distinction: "10 - 5" -> clicking on 5 should give 5, not -5
    let text3 = "y = 10 - 5";
    let res3 = find_numeric_literal_at(text3, 9).expect("Should find 5");
    assert_eq!(&text3[res3.0..res3.1], "5");
    assert_eq!(res3.2, 5.0);

    // 4. Non-number token
    let text4 = "sin(x)";
    assert!(find_numeric_literal_at(text4, 1).is_none());
}

#[test]
fn test_3d_mesh_exporters() {
    use urae_notebook::ui::viewport3d::{
        export_obj, export_step, export_stl_ascii, export_stl_binary,
    };

    // Define a simple unit triangle in 3D
    let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let tris = vec![[0, 1, 2]];

    // 1. Binary STL
    let bin_stl = export_stl_binary(&verts, &tris);
    assert_eq!(bin_stl.len(), 84 + 50); // 80 header + 4 count + 50 bytes/tri
    assert_eq!(&bin_stl[0..4], b"URAE");
    let num_tris = u32::from_le_bytes(bin_stl[80..84].try_into().unwrap());
    assert_eq!(num_tris, 1);

    // 2. ASCII STL
    let asc_stl = export_stl_ascii(&verts, &tris, "unit_tri");
    assert!(asc_stl.starts_with("solid unit_tri\n"));
    assert!(asc_stl.contains("facet normal"));
    assert!(asc_stl.contains("outer loop"));
    assert!(asc_stl.ends_with("endsolid unit_tri\n"));

    // 3. Wavefront OBJ
    let obj = export_obj(&verts, &tris, "unit_tri");
    assert!(obj.contains("v 0.000000 0.000000 0.000000"));
    assert!(obj.contains("v 1.000000 0.000000 0.000000"));
    assert!(obj.contains("f 1 2 3"));

    // 4. ISO 10303-21 STEP
    let step = export_step(&verts, &tris, "unit_tri");
    assert!(step.starts_with("ISO-10303-21;\n"));
    assert!(step.contains("AUTOMOTIVE_DESIGN"));
    assert!(step.contains("CARTESIAN_POINT"));
    assert!(step.contains("FACETED_BREP"));
    assert!(step.ends_with("END-ISO-10303-21;\n"));
}

#[test]
fn test_accessibility_themes_and_workspace_modes() {
    use urae_notebook::ui::theme::ThemeKind;
    use urae_notebook::ui::workspace::{WorkspaceLayoutPreset, WorkspaceState};

    // 1. High contrast themes
    let hc_dark = ThemeKind::HighContrastDark;
    assert!(hc_dark.is_dark());
    let p_dark = hc_dark.palette();
    assert_eq!(p_dark.bg_app, urae_notebook::egui::Color32::BLACK);
    assert_eq!(p_dark.text_primary, urae_notebook::egui::Color32::WHITE);

    let hc_light = ThemeKind::HighContrastLight;
    assert!(!hc_light.is_dark());
    let p_light = hc_light.palette();
    assert_eq!(p_light.bg_app, urae_notebook::egui::Color32::WHITE);
    assert_eq!(p_light.text_primary, urae_notebook::egui::Color32::BLACK);

    // 2. Workspace simplified single tab
    let mut ws = WorkspaceState::default();
    assert!(!ws.simplified_single_tab);
    ws.set_layout(WorkspaceLayoutPreset::SimplifiedSingleTab);
    assert!(ws.simplified_single_tab);
    assert_eq!(ws.split_ratio, 1.0);
}
