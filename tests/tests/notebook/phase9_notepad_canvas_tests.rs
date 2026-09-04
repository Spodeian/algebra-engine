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

    assert_eq!(state.session.raw_document_text, "x: Variable\na = 5.00 [m]\nf(x) = x^2");
    assert_eq!(state.parsed_lines[1].raw_text, "a = 5.00 [m]");

    // Insert without focus (appends at end)
    state.focused_line = None;
    state.cursor_char_idx = None;
    state.insert_text_at_active_line("g(x) = 2 * x");
    assert_eq!(state.parsed_lines.len(), 4);
    assert_eq!(state.parsed_lines[3].raw_text, "g(x) = 2 * x");
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

#[test]
fn test_universal_parameter_builder_all_number_systems() {
    use urae_notebook::notebook::{generate_universal_parameter_builder_syntax, ParameterBuilderParams};

    // 1. Standard Reals with bound
    let coords_scalar = vec![("Bound".to_string(), 0.0, 10.0, true)];
    let p_reals = ParameterBuilderParams {
        var: "x",
        is_param: true,
        category: 0,
        standard_kind: 0,
        coords: &coords_scalar,
        ..Default::default()
    };
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_reals),
        "x: Parameter in Reals [0.00, 10.00]"
    );

    // 2. Complex with Re and Im bounds
    let coords_complex = vec![
        ("Re".to_string(), -5.0, 5.0, true),
        ("Im".to_string(), 0.0, 10.0, true),
    ];
    let mut p_complex = p_reals.clone();
    p_complex.var = "z";
    p_complex.is_param = false;
    p_complex.standard_kind = 6;
    p_complex.coords = &coords_complex;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_complex),
        "z: Variable in Complex where Re(z) in [-5.00, 5.00], Im(z) in [0.00, 10.00]"
    );

    // 3. Cayley-Dickson depth 2 (Quaternion)
    let coords_quat = vec![
        ("Scalar".to_string(), 0.0, 1.0, true),
        ("Vector".to_string(), -1.0, 1.0, true),
    ];
    let mut p_quat = p_reals.clone();
    p_quat.var = "q";
    p_quat.category = 1;
    p_quat.cayley_depth = 2;
    p_quat.coords = &coords_quat;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_quat),
        "q: Parameter in CayleyDickson(depth=2) /* Quaternion */ where Scalar(q) in [0.00, 1.00], Vector(q) in [-1.00, 1.00]"
    );

    // 4. Adjoined Dual numbers (Forward-mode AutoDiff)
    let coords_dual = vec![
        ("Re".to_string(), -10.0, 10.0, true),
        ("Dual".to_string(), -1.0, 1.0, true),
    ];
    let mut p_dual = p_reals.clone();
    p_dual.var = "d";
    p_dual.is_param = false;
    p_dual.category = 2;
    p_dual.adjoin_eps = true;
    p_dual.coords = &coords_dual;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_dual),
        "d: Variable in Dual(1, epsilon) where Re(d) in [-10.00, 10.00], Dual(d) in [-1.00, 1.00]"
    );

    // 5. Non-Archimedean p-Adics
    let mut p_padic = p_reals.clone();
    p_padic.var = "p";
    p_padic.category = 3;
    p_padic.standard_kind = 0;
    p_padic.padic_prime = 7;
    p_padic.padic_valuation = 0;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_padic),
        "p: Parameter in PAdics(p=7) where valuation(p) >= 0"
    );

    // 6. Surreal numbers
    let mut p_surreal = p_reals.clone();
    p_surreal.var = "s";
    p_surreal.category = 3;
    p_surreal.standard_kind = 1;
    p_surreal.surreal_generation = 4;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_surreal),
        "s: Parameter in Surreals where generation(s) <= 4"
    );

    // 7. Modulo Ring Z/12Z
    let mut p_mod = p_reals.clone();
    p_mod.var = "m";
    p_mod.category = 4;
    p_mod.standard_kind = 0;
    p_mod.modulo_n = 12;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_mod),
        "m: Parameter in Modulo(n=12) where m in [0, 11]"
    );

    // 8. Finite Galois Field GF(2^8)
    let mut p_gf = p_reals.clone();
    p_gf.var = "F";
    p_gf.category = 4;
    p_gf.standard_kind = 1;
    p_gf.discrete_kind = 2;
    p_gf.galois_prime = 2;
    p_gf.galois_power = 8;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_gf),
        "F: Parameter in GaloisField(prime=2, power=8)"
    );

    // 9. Matrix space
    let mut p_mat = p_reals.clone();
    p_mat.var = "A";
    p_mat.category = 5;
    p_mat.standard_kind = 0;
    p_mat.matrix_rows = 3;
    p_mat.matrix_cols = 3;
    p_mat.matrix_domain = "Reals";
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_mat),
        "A: Parameter in Matrix(rows=3, cols=3, domain=Reals)"
    );
}

#[test]
fn test_universal_domain_parsing_and_evaluation() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = r#"
z: Variable in Complex
q: Parameter in Quaternion
d in Dual
p in PAdics
s in Surreals
m in Modulo
F in GaloisField
A in Matrix
"#.trim().to_string();

    state.evaluate_all();
    assert_eq!(state.parsed_lines.len(), 8);

    assert_eq!(state.parsed_lines[0].output_unicode, "∀ z ∈ ℂ");
    assert_eq!(state.parsed_lines[1].output_unicode, "∀ q ∈ ℍ");
    assert_eq!(state.parsed_lines[2].output_unicode, "∀ d ∈ 𝔻 (Dual)");
    assert_eq!(state.parsed_lines[3].output_unicode, "∀ p ∈ ℚₚ (p-adic)");
    assert_eq!(state.parsed_lines[4].output_unicode, "∀ s ∈ 𝐍𝐨 (Surreal)");
    assert_eq!(state.parsed_lines[5].output_unicode, "∀ m ∈ ℤ/nℤ");
    assert_eq!(state.parsed_lines[6].output_unicode, "∀ F ∈ GF(pᵏ)");
    assert_eq!(state.parsed_lines[7].output_unicode, "∀ A ∈ Mₘₓₙ");
}

#[test]
fn test_discrete_parameter_builder_syntax() {
    use urae_notebook::notebook::{generate_universal_parameter_builder_syntax, ParameterBuilderParams};

    let coords_base = vec![("Val".to_string(), 0.0, 11.0, true)];
    let p_base = ParameterBuilderParams {
        var: "m",
        is_param: true,
        category: 4,
        discrete_kind: 0,
        modulo_n: 12,
        coords: &coords_base,
        ..Default::default()
    };

    // 1. Modulo Canonical [0, n-1]
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_base),
        "m: Parameter in Modulo(n=12) where m in [0, 11]"
    );

    // 2. Modulo Balanced / Symmetric [-floor(n/2), floor(n/2)]
    let mut p_sym = p_base.clone();
    p_sym.modulo_n = 7;
    p_sym.modulo_rep = 1;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_sym),
        "m: Parameter in Modulo(n=7, symmetric=true) where m in [-3, 3]"
    );

    // 3. Modulo Multiplicative Units (Z/nZ)*
    let mut p_units = p_base.clone();
    p_units.modulo_rep = 2;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_units),
        "m: Parameter in ModuloUnits(n=12) /* (Z/12Z)* gcd(m, 12)=1 */"
    );

    // 4. Modulo with congruence constraint m == 3 (mod 5)
    let mut p_cong = p_base.clone();
    p_cong.has_congruence = true;
    p_cong.congruence_rem = 3;
    p_cong.congruence_mod = 5;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_cong),
        "m: Parameter in Modulo(n=12) where m in [0, 11], m == 3 (mod 5)"
    );

    // 5. Discrete Even Integers
    let coords_even = vec![("Val".to_string(), 0.0, 100.0, true)];
    let mut p_even = p_base.clone();
    p_even.var = "k";
    p_even.discrete_kind = 1;
    p_even.integer_parity = 1;
    p_even.coords = &coords_even;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_even),
        "k: Parameter in EvenIntegers [0, 100]"
    );

    // 6. Discrete Odd Integers
    let mut p_odd = p_even.clone();
    p_odd.integer_parity = 2;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_odd),
        "k: Parameter in OddIntegers [0, 100]"
    );

    // 7. Discrete Progression with step size
    let mut p_step = p_even.clone();
    p_step.integer_parity = 0;
    p_step.integer_step = 5;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_step),
        "k: Parameter in Integers [0, 100] step 5"
    );

    // 8. Discrete Multiples k*Integers
    let mut p_mult = p_even.clone();
    p_mult.integer_parity = 3;
    p_mult.integer_multiple = 7;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_mult),
        "k: Parameter in Integers where k in 7*Integers [0, 100]"
    );

    // 9. Gaussian Integers Z[i] lattice
    let coords_gauss = vec![
        ("Re".to_string(), -5.0, 5.0, true),
        ("Im".to_string(), -5.0, 5.0, true),
    ];
    let mut p_gauss = p_base.clone();
    p_gauss.var = "g";
    p_gauss.discrete_kind = 3;
    p_gauss.lattice_kind = 0;
    p_gauss.coords = &coords_gauss;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_gauss),
        "g: Parameter in GaussianIntegers /* Z[i] */ where Re(g) in [-5, 5], Im(g) in [-5, 5]"
    );

    // 10. Eisenstein Integers Z[omega] lattice
    let mut p_eisen = p_gauss.clone();
    p_eisen.lattice_kind = 1;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_eisen),
        "g: Parameter in EisensteinIntegers /* Z[omega] */ where Re(g) in [-5, 5], Im(g) in [-5, 5]"
    );

    // 11. Boolean domain
    let mut p_bool = p_base.clone();
    p_bool.var = "b";
    p_bool.discrete_kind = 4;
    p_bool.bit_width = 1;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_bool),
        "b: Parameter in Boolean"
    );

    // 12. Bit-Vector word domain
    let mut p_word = p_bool.clone();
    p_word.var = "w";
    p_word.bit_width = 16;
    p_word.bit_signed = true;
    assert_eq!(
        generate_universal_parameter_builder_syntax(&p_word),
        "w: Parameter in BitVector(width=16, signed=true)"
    );
}

#[test]
fn test_discrete_domain_parsing_and_evaluation() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = r#"
m: Parameter in Modulo
u in ModuloUnits
g in GaussianIntegers
e in EisensteinIntegers
b in Boolean
w in BitVector
k in EvenIntegers
j in OddIntegers
"#.trim().to_string();

    state.evaluate_all();
    assert_eq!(state.parsed_lines.len(), 8);

    assert_eq!(state.parsed_lines[0].output_unicode, "∀ m ∈ ℤ/nℤ");
    assert_eq!(state.parsed_lines[1].output_unicode, "∀ u ∈ (ℤ/nℤ)ˣ");
    assert_eq!(state.parsed_lines[2].output_unicode, "∀ g ∈ ℤ[i] (Gaussian)");
    assert_eq!(state.parsed_lines[3].output_unicode, "∀ e ∈ ℤ[ω] (Eisenstein)");
    assert_eq!(state.parsed_lines[4].output_unicode, "∀ b ∈ 𝔹");
    assert_eq!(state.parsed_lines[5].output_unicode, "∀ w ∈ 𝔹ʷ");
    assert_eq!(state.parsed_lines[6].output_unicode, "∀ k ∈ 2ℤ");
    assert_eq!(state.parsed_lines[7].output_unicode, "∀ j ∈ 2ℤ+1");
}

#[test]
fn test_universal_prime_decomposition_in_notebook() {
    let mut state = NotebookState::default();
    state.session.raw_document_text = r#"
prime_factors 60
prime_factors 3 + 4i
prime_factors 7 in Eisenstein
prime_factors 12 in Modulo(15)
prime_factors 45 in PAdics(p=3)
prime_factors 21/40
"#.trim().to_string();

    state.evaluate_all();
    assert_eq!(state.parsed_lines.len(), 6);

    // 1. Integers 60 = 2^2 * 3 * 5
    assert!(state.parsed_lines[0].output_unicode.contains("Integers (ℤ)"));
    assert!(state.parsed_lines[0].output_unicode.contains("60 = 2^2 * 3 * 5"));
    assert!(state.parsed_lines[0].output_unicode.contains("Even prime"));

    // 2. Gaussian Integers 3 + 4i
    assert!(state.parsed_lines[1].output_unicode.contains("Gaussian Integers (ℤ[i])"));
    assert!(state.parsed_lines[1].output_unicode.contains("3 + 4i ="));

    // 3. Eisenstein Integers 7
    assert!(state.parsed_lines[2].output_unicode.contains("Eisenstein Integers (ℤ[ω])"));
    assert!(state.parsed_lines[2].output_unicode.contains("7 ="));

    // 4. Modulo(15) 12
    assert!(state.parsed_lines[3].output_unicode.contains("Modular Ring (ℤ/15ℤ)"));
    assert!(state.parsed_lines[3].output_unicode.contains("Modular prime/maximal ideal"));

    // 5. p-Adics 45 in Q_3
    assert!(state.parsed_lines[4].output_unicode.contains("p-Adic Field (ℚ_3)"));
    assert!(state.parsed_lines[4].output_unicode.contains("Valuation v_3(45) = 2"));

    // 6. Rationals 21/40
    assert!(state.parsed_lines[5].output_unicode.contains("Rationals (ℚ)"));
    assert!(state.parsed_lines[5].output_unicode.contains("21/40 ="));
}
