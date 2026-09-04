//! # `urae_notebook::notebook`
//!
//! Continuous Mathematics Notepad Model, Incremental Evaluation, and Persistence Engine.

pub mod cache;
pub mod history;
pub mod line_shift;
pub mod parser;
pub mod plot_eval;
pub mod session;
pub mod worker;

pub use cache::{compute_slider_hash, LineCache, LineCacheEntry, PlotCache, PlotCacheKey};
pub use history::{HistorySnapshot, UndoRedoHistory};
pub use line_shift::reconcile_line_references_on_shift;
pub use parser::{
    is_markdown_line, parse_permissive_solve_command, resolve_line_references, CellBlock,
    CellBlockKind, LineKind, ObjectKind, ParsedLine, SymbolInfoCard,
};
pub use plot_eval::{
    compute_smart_plot_bounds, detect_periodic_sub_periods, evaluate_plot_points,
    evaluate_plot_points_with_domains, extract_symbols, find_roots_and_critical_points,
    inspect_function_features, PeriodicAnalysis,
};
pub use session::{
    default_logging_level, export_session_to_compressed_bson, generate_branch_cut_syntax,
    generate_interval_syntax, generate_matrix_syntax, generate_ode_bc_syntax,
    generate_parameter_builder_syntax, generate_physical_unit_syntax,
    generate_universal_parameter_builder_syntax, import_session_from_compressed_bson,
    CardDisplayMode, MatrixPresetKind, NotebookSettings, ParameterBuilderParams,
    ReactiveComputeMode, SessionData, SymbolMetadata, SymbolRole, SESSION_FILE_NAME,
};
pub use worker::{
    BackgroundEvaluator, CancellationToken, DependencyGraph, EvaluationRequest, EvaluationResponse,
};

use std::collections::{HashMap, HashSet};
use urae::prelude::*;

/// Reactive State and Execution Engine for URAE Continuous Notepad.
#[derive(Debug)]
pub struct NotebookState {
    pub session: SessionData,
    pub parsed_lines: Vec<ParsedLine>,
    pub graph: ExprGraph,
    pub formatter: LatexFormatter,
    pub unicode_formatter: urae::format::UnicodeFormatter,
    pub show_settings_modal: bool,
    pub focused_line: Option<usize>,
    pub cursor_char_idx: Option<usize>,
    pub line_edit_buffer: String,
    pub is_scrubbing: bool,
    pub measured_eval_latency_ms: f64,
    pub symbol_dependencies: HashMap<String, HashSet<usize>>,
    pub history: UndoRedoHistory,
    pub line_cache: LineCache,
    pub plot_cache: PlotCache,
}

impl Default for NotebookState {
    fn default() -> Self {
        let session = SessionData::load_from_disk().unwrap_or_default();
        let mut state = Self {
            session,
            parsed_lines: Vec::new(),
            graph: ExprGraph::new(),
            formatter: LatexFormatter,
            unicode_formatter: urae::format::UnicodeFormatter,
            show_settings_modal: false,
            focused_line: None,
            cursor_char_idx: None,
            line_edit_buffer: String::new(),
            is_scrubbing: false,
            measured_eval_latency_ms: 0.0,
            symbol_dependencies: HashMap::new(),
            history: UndoRedoHistory::default(),
            line_cache: LineCache::default(),
            plot_cache: PlotCache::default(),
        };

        state.evaluate_all();
        state
    }
}

impl NotebookState {
    /// Create a pristine in-memory NotebookState with default session data.
    pub fn new_clean() -> Self {
        let session = SessionData::default();
        let mut state = Self {
            session,
            parsed_lines: Vec::new(),
            graph: ExprGraph::new(),
            formatter: LatexFormatter,
            unicode_formatter: urae::format::UnicodeFormatter,
            show_settings_modal: false,
            focused_line: None,
            cursor_char_idx: None,
            line_edit_buffer: String::new(),
            is_scrubbing: false,
            measured_eval_latency_ms: 0.0,
            symbol_dependencies: HashMap::new(),
            history: UndoRedoHistory::default(),
            line_cache: LineCache::default(),
            plot_cache: PlotCache::default(),
        };
        state.evaluate_all();
        state
    }

    /// Capture current session snapshot into the undo stack with a descriptive action name.
    pub fn record_snapshot(&mut self, description: impl Into<String>) {
        let snapshot = HistorySnapshot {
            raw_document_text: self.session.raw_document_text.clone(),
            slider_values: self.session.slider_values.clone(),
            symbol_metadata: self.session.symbol_metadata.clone(),
            card_modes: self.session.card_modes.clone(),
            description: description.into(),
        };
        self.history.push(snapshot);
    }

    /// Undo previous action and restore prior document and parameter state.
    pub fn undo(&mut self) -> bool {
        if let Some(prev_snapshot) = self.history.undo_stack.pop_back() {
            let current_snapshot = HistorySnapshot {
                raw_document_text: self.session.raw_document_text.clone(),
                slider_values: self.session.slider_values.clone(),
                symbol_metadata: self.session.symbol_metadata.clone(),
                card_modes: self.session.card_modes.clone(),
                description: "Current state".to_string(),
            };
            self.history.redo_stack.push_back(current_snapshot);

            self.session.raw_document_text = prev_snapshot.raw_document_text;
            self.session.slider_values = prev_snapshot.slider_values;
            self.session.symbol_metadata = prev_snapshot.symbol_metadata;
            self.session.card_modes = prev_snapshot.card_modes;
            self.focused_line = None;
            self.evaluate_all();
            true
        } else {
            false
        }
    }

    /// Redo previously undone action.
    pub fn redo(&mut self) -> bool {
        if let Some(next_snapshot) = self.history.redo_stack.pop_back() {
            let current_snapshot = HistorySnapshot {
                raw_document_text: self.session.raw_document_text.clone(),
                slider_values: self.session.slider_values.clone(),
                symbol_metadata: self.session.symbol_metadata.clone(),
                card_modes: self.session.card_modes.clone(),
                description: "Current state".to_string(),
            };
            self.history.undo_stack.push_back(current_snapshot);

            self.session.raw_document_text = next_snapshot.raw_document_text;
            self.session.slider_values = next_snapshot.slider_values;
            self.session.symbol_metadata = next_snapshot.symbol_metadata;
            self.session.card_modes = next_snapshot.card_modes;
            self.focused_line = None;
            self.evaluate_all();
            true
        } else {
            false
        }
    }

    /// Check if an undo operation is available.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if a redo operation is available.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Save session state to disk.
    pub fn save_session(&self) {
        self.session.save_to_disk();
    }

    /// Get card mode for a plot/card associated with expression `expr_key`.
    pub fn get_card_mode(&self, expr_key: &str) -> CardDisplayMode {
        if let Some(&mode) = self.session.card_modes.get(expr_key) {
            mode
        } else if self.session.settings.auto_plots {
            CardDisplayMode::Inline
        } else {
            CardDisplayMode::Hidden
        }
    }

    /// Set card mode for a plot/card associated with expression `expr_key`.
    pub fn set_card_mode(&mut self, expr_key: &str, mode: CardDisplayMode) {
        self.session.card_modes.insert(expr_key.to_string(), mode);
        self.save_session();
    }

    /// Helper to parse physical unit string e.g. `[m/s^2]` -> Dimensions struct.
    pub fn parse_dimensions(unit_str: &str) -> Dimensions {
        let clean = unit_str.trim().trim_matches(|c| c == '[' || c == ']');
        match clean {
            "m" | "meter" | "meters" => Dimensions::length(),
            "kg" | "kilogram" => Dimensions::mass(),
            "s" | "sec" | "second" => Dimensions::time(),
            "m/s" => Dimensions::velocity(),
            "m/s^2" => Dimensions {
                mass: 0.0,
                length: 1.0,
                time: -2.0,
                current: 0.0,
                temperature: 0.0,
                amount: 0.0,
                intensity: 0.0,
            },
            "N" | "kg*m/s^2" => Dimensions {
                mass: 1.0,
                length: 1.0,
                time: -2.0,
                current: 0.0,
                temperature: 0.0,
                amount: 0.0,
                intensity: 0.0,
            },
            "J" | "kg*m^2/s^2" => Dimensions {
                mass: 1.0,
                length: 2.0,
                time: -2.0,
                current: 0.0,
                temperature: 0.0,
                amount: 0.0,
                intensity: 0.0,
            },
            _ => Dimensions::dimensionless(),
        }
    }

    /// Load a preset template into the notepad.
    #[allow(dead_code)]
    pub fn load_preset(&mut self, preset_name: &str) {
        self.record_snapshot(format!("Load preset '{}'", preset_name));
        match preset_name {
            "quadratic" => {
                self.session.raw_document_text = [
                    "# Quadratic Function Model with Units & Role Declarations",
                    "// Reactive parameters a and c modulate the parabola curvature.",
                    "",
                    "a: Parameter = 2.00 [m]",
                    "c: Parameter = -3.00 [m]",
                    "x: Variable",
                    "{ x in Reals | -5 <= x <= 5 }",
                    "f(x) = a * x^2 + c",
                ]
                .join("\n");
                self.session.slider_values.insert("a".to_string(), 2.0);
                self.session.slider_values.insert("c".to_string(), -3.0);
            }
            "oscillator" => {
                self.session.raw_document_text = [
                    "# Harmonic Wave Motion",
                    "// Amplitude A and frequency w define the wave function.",
                    "",
                    "A: Parameter = 1.50 [m]",
                    "w: Parameter = 2.00 [rad/s]",
                    "t: Variable",
                    "{ t in Reals | t >= 0 }",
                    "y = A * cos(w * t)",
                ]
                .join("\n");
                self.session.slider_values.insert("A".to_string(), 1.5);
                self.session.slider_values.insert("w".to_string(), 2.0);
            }
            "calculus" => {
                self.session.raw_document_text = [
                    "# Symbolic Polynomial & Hypercomplex Analysis",
                    "// Continuous polynomial differentiation and quaternion domain bounds.",
                    "",
                    "{ q in Quaternion | norm(q) == 1 }",
                    "{ v in Grassmann | v^2 == 0 }",
                    "f(x) = x^3 - 3 * x + 2",
                    "d(x^3 - 3 * x + 2) / dx",
                ]
                .join("\n");
            }
            _ => {
                self.session.raw_document_text = "# Blank Notepad\nx^2 + 1".to_string();
            }
        }

        self.evaluate_all();
        self.save_session();
    }

    /// Parse notebook document into structured CellBlocks for IDE-like presentation.
    pub fn get_cell_blocks(&self) -> Vec<CellBlock> {
        let lines: Vec<&str> = self.session.raw_document_text.lines().collect();
        let mut blocks = Vec::with_capacity(lines.len());
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            let kind = if trimmed.starts_with("# ") {
                CellBlockKind::SectionHeader {
                    level: 1,
                    collapsed: false,
                }
            } else if trimmed.starts_with("## ") {
                CellBlockKind::SectionHeader {
                    level: 2,
                    collapsed: false,
                }
            } else if trimmed.starts_with("### ") {
                CellBlockKind::SectionHeader {
                    level: 3,
                    collapsed: false,
                }
            } else if is_markdown_line(trimmed) {
                CellBlockKind::Markdown
            } else {
                CellBlockKind::Math
            };

            blocks.push(CellBlock {
                id: format!("cell_{}", idx),
                kind,
                content: line.to_string(),
                line_idx: idx,
            });
        }
        blocks
    }

    /// Move line at index `line_idx` up one position in document.
    pub fn move_line_up(&mut self, line_idx: usize) -> bool {
        if line_idx == 0 {
            return false;
        }
        let mut lines: Vec<String> = self
            .session
            .raw_document_text
            .lines()
            .map(|s| s.to_string())
            .collect();
        if line_idx >= lines.len() {
            return false;
        }
        self.record_snapshot(format!("Move line {} up", line_idx + 1));
        lines.swap(line_idx, line_idx - 1);
        self.session.raw_document_text = lines.join("\n");
        self.evaluate_all();
        self.save_session();
        true
    }

    /// Move line at index `line_idx` down one position in document.
    pub fn move_line_down(&mut self, line_idx: usize) -> bool {
        let mut lines: Vec<String> = self
            .session
            .raw_document_text
            .lines()
            .map(|s| s.to_string())
            .collect();
        if line_idx + 1 >= lines.len() {
            return false;
        }
        self.record_snapshot(format!("Move line {} down", line_idx + 1));
        lines.swap(line_idx, line_idx + 1);
        self.session.raw_document_text = lines.join("\n");
        self.evaluate_all();
        self.save_session();
        true
    }

    /// Delete line at index `line_idx`.
    pub fn delete_line(&mut self, line_idx: usize) -> bool {
        let mut lines: Vec<String> = self
            .session
            .raw_document_text
            .lines()
            .map(|s| s.to_string())
            .collect();
        if line_idx >= lines.len() {
            return false;
        }
        self.record_snapshot(format!("Delete line {}", line_idx + 1));
        lines.remove(line_idx);
        self.session.raw_document_text = lines.join("\n");
        self.evaluate_all();
        self.save_session();
        true
    }

    /// Duplicate line at index `line_idx`.
    pub fn duplicate_line(&mut self, line_idx: usize) -> bool {
        let mut lines: Vec<String> = self
            .session
            .raw_document_text
            .lines()
            .map(|s| s.to_string())
            .collect();
        if line_idx >= lines.len() {
            return false;
        }
        self.record_snapshot(format!("Duplicate line {}", line_idx + 1));
        let copy = lines[line_idx].clone();
        lines.insert(line_idx + 1, copy);
        self.session.raw_document_text = lines.join("\n");
        self.evaluate_all();
        self.save_session();
        true
    }

    /// Retrieve detailed hover inspector card metadata for symbol or object `sym_name`.
    pub fn get_symbol_info_card(&self, sym_name: &str) -> Option<SymbolInfoCard> {
        let clean_name = sym_name.trim();
        if clean_name.is_empty() {
            return None;
        }

        // 1. Constants (pi, e, i, gamma, phi, etc.)
        match clean_name {
            "pi" | "π" => {
                return Some(SymbolInfoCard {
                    name: "π".to_string(),
                    role: SymbolRole::Constant,
                    cur_val: std::f64::consts::PI,
                    domain_type: "Transcendental Real (ℝ)".to_string(),
                    unit_str: None,
                    dependent_lines: Vec::new(),
                    formula_references: Vec::new(),
                    object_kind: Some(ObjectKind::Constant {
                        symbol_name: "π".to_string(),
                        exact_desc:
                            "Archimedes' constant ratio of a circle's circumference to diameter"
                                .to_string(),
                        approx_val: std::f64::consts::PI,
                    }),
                });
            }
            "e" => {
                return Some(SymbolInfoCard {
                    name: "e".to_string(),
                    role: SymbolRole::Constant,
                    cur_val: std::f64::consts::E,
                    domain_type: "Transcendental Real (ℝ)".to_string(),
                    unit_str: None,
                    dependent_lines: Vec::new(),
                    formula_references: Vec::new(),
                    object_kind: Some(ObjectKind::Constant {
                        symbol_name: "e".to_string(),
                        exact_desc:
                            "Euler's number: base of the natural logarithm (lim (1 + 1/n)ⁿ)"
                                .to_string(),
                        approx_val: std::f64::consts::E,
                    }),
                });
            }
            "i" => {
                return Some(SymbolInfoCard {
                    name: "i".to_string(),
                    role: SymbolRole::Constant,
                    cur_val: 0.0,
                    domain_type: "Imaginary Unit (ℂ)".to_string(),
                    unit_str: None,
                    dependent_lines: Vec::new(),
                    formula_references: Vec::new(),
                    object_kind: Some(ObjectKind::Constant {
                        symbol_name: "i".to_string(),
                        exact_desc: "Fundamental imaginary unit with defining property i² = -1"
                            .to_string(),
                        approx_val: 0.0,
                    }),
                });
            }
            "gamma" | "γ" => {
                return Some(SymbolInfoCard {
                    name: "γ".to_string(),
                    role: SymbolRole::Constant,
                    cur_val: 0.5772156649015329,
                    domain_type: "Real Constant (ℝ)".to_string(),
                    unit_str: None,
                    dependent_lines: Vec::new(),
                    formula_references: Vec::new(),
                    object_kind: Some(ObjectKind::Constant {
                        symbol_name: "γ".to_string(),
                        exact_desc: "Euler-Mascheroni constant (lim (∑ 1/k - ln(n)))".to_string(),
                        approx_val: 0.5772156649015329,
                    }),
                });
            }
            "phi" | "φ" => {
                return Some(SymbolInfoCard {
                    name: "φ".to_string(),
                    role: SymbolRole::Constant,
                    cur_val: 1.618033988749895,
                    domain_type: "Algebraic Real (ℝ)".to_string(),
                    unit_str: None,
                    dependent_lines: Vec::new(),
                    formula_references: Vec::new(),
                    object_kind: Some(ObjectKind::Constant {
                        symbol_name: "φ".to_string(),
                        exact_desc: "Golden Ratio (1 + √5)/2".to_string(),
                        approx_val: 1.618033988749895,
                    }),
                });
            }
            _ => {}
        }

        // 2. Line References ($N, ans)
        if clean_name == "ans" || clean_name.starts_with('$') {
            let target_line_idx = if clean_name == "ans" {
                self.parsed_lines.len().checked_sub(1)
            } else if let Ok(num) = clean_name[1..].parse::<usize>() {
                num.checked_sub(1)
            } else {
                None
            };

            if let Some(l_idx) = target_line_idx {
                if l_idx < self.parsed_lines.len() {
                    let pl = &self.parsed_lines[l_idx];
                    let val = pl.linearized_estimate.unwrap_or(0.0);
                    return Some(SymbolInfoCard {
                        name: clean_name.to_string(),
                        role: SymbolRole::Variable,
                        cur_val: val,
                        domain_type: pl
                            .domain_info
                            .clone()
                            .unwrap_or_else(|| "Computed Result".to_string()),
                        unit_str: pl.physical_unit.clone(),
                        dependent_lines: vec![l_idx],
                        formula_references: vec![pl.raw_text.clone()],
                        object_kind: Some(ObjectKind::LineResult {
                            line_idx: l_idx + 1,
                            summary: if !pl.output_unicode.is_empty() {
                                pl.output_unicode.clone()
                            } else {
                                pl.raw_text.clone()
                            },
                        }),
                    });
                }
            }
        }

        // 3. Document Function Definitions
        for pl in &self.parsed_lines {
            if pl.kind == LineKind::Markdown {
                continue;
            }
            let raw = pl.raw_text.trim();
            let matches_fn = raw.starts_with(clean_name)
                || (pl.is_function && raw.split('=').next().map(|h| {
                    let h_trim = h.trim();
                    h_trim == clean_name || h_trim.split('(').next().map(|b| b.trim() == clean_name).unwrap_or(false)
                }).unwrap_or(false));

            if matches_fn && pl.is_function {
                let formula_refs = vec![pl.raw_text.clone()];
                let fn_kind = inspect_function_features(
                    &self.graph,
                    &self.session.slider_values,
                    Some(&self.session.symbol_metadata),
                    &pl.raw_text,
                    "x",
                    &self.unicode_formatter,
                );

                let domain_desc = if let ObjectKind::Function { ref domain, ref codomain, .. } = fn_kind {
                    format!("Function Mapping ({} → {})", domain, codomain)
                } else {
                    "Function Mapping (ℝ → ℝ)".to_string()
                };

                return Some(SymbolInfoCard {
                    name: clean_name.to_string(),
                    role: SymbolRole::Variable,
                    cur_val: 0.0,
                    domain_type: domain_desc,
                    unit_str: pl.physical_unit.clone(),
                    dependent_lines: vec![pl.line_idx],
                    formula_references: formula_refs,
                    object_kind: Some(fn_kind),
                });
            }
        }

        // 4. Document Symbols, Variables & Sliders
        if let Some(meta) = self.session.symbol_metadata.get(clean_name) {
            let cur_val = self
                .session
                .slider_values
                .get(clean_name)
                .copied()
                .unwrap_or(meta.cur_val);

            let mut dependent_lines = Vec::new();
            if let Some(deps) = self.symbol_dependencies.get(clean_name) {
                dependent_lines = deps.iter().copied().collect();
                dependent_lines.sort_unstable();
            }

            let mut formula_references = Vec::new();
            for parsed in &self.parsed_lines {
                if parsed.kind != LineKind::Markdown && parsed.raw_text.contains(clean_name) {
                    formula_references.push(parsed.raw_text.clone());
                }
            }

            let object_kind = if let Some(unit) = &meta.unit_str {
                Some(ObjectKind::PhysicalQuantity {
                    dimension: meta.domain_type.clone(),
                    unit: unit.clone(),
                    magnitude: cur_val,
                })
            } else if meta.domain_type.contains('[') {
                Some(ObjectKind::SetOrInterval {
                    set_type: meta.domain_type.clone(),
                    bounds_str: format!("[{:.2}, {:.2}]", meta.min_val, meta.max_val),
                })
            } else {
                Some(ObjectKind::Scalar {
                    type_name: meta.domain_type.clone(),
                    value_str: format!("{:.4}", cur_val),
                    approx_f64: Some(cur_val),
                })
            };

            return Some(SymbolInfoCard {
                name: clean_name.to_string(),
                role: meta.role,
                cur_val,
                domain_type: meta.domain_type.clone(),
                unit_str: meta.unit_str.clone(),
                dependent_lines,
                formula_references,
                object_kind,
            });
        }

        None
    }

    /// Resolve `ans`, `$N`, and range `$M..$N` or `sum($M..$N)` references within a line before parsing.
    pub fn resolve_line_references(
        text: &str,
        last_val: Option<f64>,
        line_results: &HashMap<usize, f64>,
    ) -> String {
        resolve_line_references(text, last_val, line_results)
    }

    /// Reconcile and rewrite line reference numbers ($N, $M..$N) when lines are inserted or deleted.
    pub fn reconcile_line_references_on_shift(
        old_lines: &[&str],
        new_lines: &[&str],
    ) -> Option<String> {
        reconcile_line_references_on_shift(old_lines, new_lines)
    }

    /// Refactor an anonymous formula line into a named variable definition (e.g. `k = <expr>`)
    /// and rewrite all downstream `$N` references to `k`.
    pub fn refactor_line_to_named_variable(&mut self, line_idx: usize, var_name: &str) -> bool {
        let lines: Vec<&str> = self.session.raw_document_text.lines().collect();
        if line_idx >= lines.len() {
            return false;
        }

        let target_line = lines[line_idx].trim();
        let target_ref_token = format!("${}", line_idx + 1);

        let new_target_line = if target_line.contains('=') && !target_line.contains("==") {
            let (_, rhs) = target_line.split_once('=').unwrap();
            format!("{} = {}", var_name, rhs.trim())
        } else {
            format!("{} = {}", var_name, target_line)
        };

        let mut updated_lines = Vec::with_capacity(lines.len());
        for (i, &line) in lines.iter().enumerate() {
            if i == line_idx {
                updated_lines.push(new_target_line.clone());
            } else if i > line_idx && line.contains(&target_ref_token) {
                let replaced = line.replace(&target_ref_token, var_name);
                updated_lines.push(replaced);
            } else {
                updated_lines.push(line.to_string());
            }
        }

        self.session.raw_document_text = updated_lines.join("\n");
        self.evaluate_all();
        true
    }

    /// Parse a natural set-theory domain declaration e.g. `a in Reals [0, 100]` or `x in Positive`.
    pub fn parse_domain_declaration(line: &str) -> Option<DomainBound> {
        parse_domain_declaration(line)
    }

    /// Extract all unique variable symbol names contained in an expression tree.
    pub fn extract_symbols(graph: &ExprGraph, root: ExprId) -> Vec<String> {
        extract_symbols(graph, root)
    }

    /// Evaluate plot points `[x, y]` for a plot expression string across range with domain constraint enforcement.
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_plot_points_with_domains(
        graph: &ExprGraph,
        slider_values: &HashMap<String, f64>,
        symbol_metadata: Option<&HashMap<String, SymbolMetadata>>,
        expr_str: &str,
        fallback_var: &str,
        range_min: f64,
        range_max: f64,
        samples: usize,
    ) -> Vec<[f64; 2]> {
        evaluate_plot_points_with_domains(
            graph,
            slider_values,
            symbol_metadata,
            expr_str,
            fallback_var,
            range_min,
            range_max,
            samples,
        )
    }

    /// Evaluate plot points `[x, y]` with automatic memoization in PlotCache.
    pub fn cached_evaluate_plot_points(
        &self,
        expr_key: &str,
        fallback_var: &str,
        range_min: f64,
        range_max: f64,
        samples: usize,
    ) -> Vec<[f64; 2]> {
        let slider_hash = compute_slider_hash(&self.session.slider_values);
        if let Some(cached) =
            self.plot_cache
                .get(expr_key, slider_hash, range_min, range_max, samples)
        {
            return cached;
        }

        let pts = Self::evaluate_plot_points_with_domains(
            &self.graph,
            &self.session.slider_values,
            Some(&self.session.symbol_metadata),
            expr_key,
            fallback_var,
            range_min,
            range_max,
            samples,
        );

        self.plot_cache.insert(
            expr_key,
            slider_hash,
            range_min,
            range_max,
            samples,
            pts.clone(),
        );
        pts
    }

    /// Compute smart plot bounds [x_min, x_max] covering at least [-1.0, 1.0], critical points, roots, and 1-3 periods for trig.
    pub fn compute_smart_plot_bounds(&self, expr_str: &str, fallback_var: &str) -> (f64, f64) {
        compute_smart_plot_bounds(
            &self.graph,
            &self.session.slider_values,
            Some(&self.session.symbol_metadata),
            expr_str,
            fallback_var,
        )
    }

    /// Helper function to expand user function calls in mathematical strings
    fn expand_user_function_call(text: &str, fn_name: &str, fn_rhs: &str) -> String {
        let prefix = format!("{}(", fn_name);
        let mut result = String::new();
        let mut rest = text;
        while let Some(start) = rest.find(&prefix) {
            let is_word_boundary = if start == 0 {
                true
            } else {
                let prev = rest[..start].chars().last().unwrap();
                !prev.is_alphanumeric() && prev != '_'
            };

            if !is_word_boundary {
                result.push_str(&rest[..start + prefix.len()]);
                rest = &rest[start + prefix.len()..];
                continue;
            }

            result.push_str(&rest[..start]);
            let after_prefix = &rest[start + prefix.len()..];
            let mut depth = 1;
            let mut end_offset = None;
            for (i, c) in after_prefix.char_indices() {
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        end_offset = Some(i);
                        break;
                    }
                }
            }
            if let Some(end) = end_offset {
                result.push_str(&format!("({})", fn_rhs));
                rest = &after_prefix[end + 1..];
            } else {
                result.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }
        result.push_str(rest);
        result
    }

    /// Re-evaluate all document lines in the notepad.
    pub fn evaluate_all(&mut self) {
        let start_time = web_time::Instant::now();
        let lines: Vec<&str> = self.session.raw_document_text.lines().collect();
        let parser = ExprParser::new(&self.graph);
        let mut active_document_symbols = HashSet::new();
        let mut symbol_deps: HashMap<String, HashSet<usize>> = HashMap::new();

        let mut fn_defs: HashMap<String, String> = HashMap::new();

        // 1. Harvest explicit role declarations, parameter definitions, domain rules, and permissive definitions
        for line in &lines {
            let trimmed = line.trim();
            if is_markdown_line(trimmed) {
                continue;
            }

            if let Some(intent) = urae::parser::parse_permissive_intent(trimmed) {
                match intent {
                    urae::parser::PermissiveIntent::Define { name, body, .. } => {
                        fn_defs.insert(name, body);
                    }
                    urae::parser::PermissiveIntent::DomainDeclaration { name, domain_desc } => {
                        active_document_symbols.insert(name.clone());
                        let meta = self
                            .session
                            .symbol_metadata
                            .entry(name.clone())
                            .or_insert_with(|| {
                                SymbolMetadata::new(&name, 1.0, SymbolRole::Variable)
                            });
                        meta.domain_type = domain_desc;
                    }
                    _ => {}
                }
            }

            // Check domain declaration e.g. `a in Reals [0, 100]` or `x in Positive`
            if let Some(bound) = Self::parse_domain_declaration(trimmed) {
                active_document_symbols.insert(bound.name.clone());
                let meta = self
                    .session
                    .symbol_metadata
                    .entry(bound.name.clone())
                    .or_insert_with(|| {
                        SymbolMetadata::new(&bound.name, 1.0, SymbolRole::Parameter)
                    });

                meta.domain_type = bound.domain_type.clone();
                if let Some(min_v) = bound.min_val {
                    meta.min_val = min_v;
                }
                if let Some(max_v) = bound.max_val {
                    meta.max_val = max_v;
                }
                meta.cur_val = meta.clamp(meta.cur_val);
                if meta.role == SymbolRole::Parameter {
                    self.session
                        .slider_values
                        .insert(bound.name.clone(), meta.cur_val);
                }
            }

            // Check role declarations e.g. `a: Parameter` or `x: Variable` or `a: Parameter = 2.0 [m]`
            if let Some((sym_part, decl_part)) = trimmed.split_once(':') {
                let sym_name = sym_part.trim();
                let decl_body = decl_part.trim();
                active_document_symbols.insert(sym_name.to_string());

                let role = if decl_body.starts_with("Parameter") || decl_body.starts_with("param") {
                    SymbolRole::Parameter
                } else if decl_body.starts_with("Variable") || decl_body.starts_with("var") {
                    SymbolRole::Variable
                } else {
                    SymbolRole::Parameter
                };

                let meta = self
                    .session
                    .symbol_metadata
                    .entry(sym_name.to_string())
                    .or_insert_with(|| SymbolMetadata::new(sym_name, 1.0, role));
                meta.set_role(role);

                // Parse value if present e.g. `a: Parameter = 2.0 [m]`
                if let Some((_, val_and_unit)) = decl_body.split_once('=') {
                    let mut val_s = val_and_unit.trim();
                    if let Some(bracket_idx) = val_s.find('[') {
                        let unit_s = val_s[bracket_idx..]
                            .trim_matches(|c| c == '[' || c == ']')
                            .trim();
                        meta.unit_str = Some(unit_s.to_string());
                        val_s = val_s[..bracket_idx].trim();
                    }
                    if let Ok(v) = val_s.parse::<f64>() {
                        let active_slider_val = self.session.slider_values.get(sym_name).copied();

                        if meta.parsed_text_val != Some(v) {
                            meta.parsed_text_val = Some(v);
                            meta.cur_val = meta.clamp(v);
                        } else if let Some(s_val) = active_slider_val {
                            meta.cur_val = s_val;
                        } else {
                            meta.cur_val = meta.clamp(v);
                        }
                        if meta.role == SymbolRole::Parameter {
                            self.session
                                .slider_values
                                .insert(sym_name.to_string(), meta.cur_val);
                        }
                    }
                }
            }

            // Check physical unit format e.g. `g = 9.81 [m/s^2]` or `v = 100 [km/h]`
            let (text_without_unit, unit_opt) = if let Some(bracket_start) = trimmed.find('[') {
                if let Some(bracket_end) = trimmed.find(']') {
                    if bracket_end > bracket_start {
                        let unit_name = &trimmed[bracket_start + 1..bracket_end];
                        (&trimmed[..bracket_start], Some(unit_name.to_string()))
                    } else {
                        (trimmed, None)
                    }
                } else {
                    (trimmed, None)
                }
            } else {
                (trimmed, None)
            };

            // Check explicit parameter assignment `param = val`
            if let Some((name, val_str)) = text_without_unit.split_once('=') {
                let name = name.trim();
                let val_str = val_str.trim();
                if !name.contains(':') && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    if let Ok(f) = val_str.parse::<f64>() {
                        active_document_symbols.insert(name.to_string());
                        let active_slider_val = self.session.slider_values.get(name).copied();
                        let meta = self
                            .session
                            .symbol_metadata
                            .entry(name.to_string())
                            .or_insert_with(|| SymbolMetadata::new(name, f, SymbolRole::Parameter));

                        if meta.parsed_text_val != Some(f) {
                            meta.parsed_text_val = Some(f);
                            meta.cur_val = meta.clamp(f);
                        } else if let Some(s_val) = active_slider_val {
                            meta.cur_val = s_val;
                        } else {
                            meta.cur_val = meta.clamp(f);
                        }

                        if let Some(u) = &unit_opt {
                            meta.unit_str = Some(u.clone());
                        }
                        if meta.role == SymbolRole::Parameter {
                            self.session
                                .slider_values
                                .insert(name.to_string(), meta.cur_val);
                        }
                    }
                }
            }
        }

        let mut parsed_lines = Vec::with_capacity(lines.len());
        let mut current_scoped_context = urae::MathContext::default();
        let mut line_results_f64: HashMap<usize, f64> = HashMap::new();
        let mut last_evaluated_val: Option<f64> = None;

        // 2. Parse and evaluate each line
        for (idx, &line_str) in lines.iter().enumerate() {
            let trimmed_raw = line_str.trim();

            if is_markdown_line(trimmed_raw) {
                parsed_lines.push(ParsedLine {
                    line_idx: idx,
                    raw_text: line_str.to_string(),
                    kind: LineKind::Markdown,
                    output_latex: line_str.to_string(),
                    output_unicode: line_str.to_string(),
                    simplified_unicode: None,
                    substituted_latex: None,
                    derivative_latex: None,
                    domain_info: None,
                    error_msg: None,
                    suggested_symbols: Vec::new(),
                    is_function: false,
                    physical_unit: None,
                    numerical_roots: None,
                    linearized_estimate: None,
                    is_pending: false,
                    eval_time_ms: 0,
                    scoped_context: current_scoped_context.clone(),
                });
                continue;
            }

            // Resolve `ans` and `$N` references before parsing
            let resolved_line =
                Self::resolve_line_references(trimmed_raw, last_evaluated_val, &line_results_f64);
            let trimmed = resolved_line.as_str();

            // Check domain declaration e.g. `a in Reals [0, 100]` or `x in Positive`
            if let Some(bound) = Self::parse_domain_declaration(trimmed) {
                let min_s = bound.min_val.unwrap_or(-10.0);
                let max_s = bound.max_val.unwrap_or(10.0);
                let display_unicode = match bound.domain_type.as_str() {
                    "Reals" if bound.min_val.is_some() && bound.max_val.is_some() => {
                        format!("∀ {} ∈ ℝ ∩ [{:.2}, {:.2}]", bound.name, min_s, max_s)
                    }
                    "Integers" if bound.min_val.is_some() && bound.max_val.is_some() => {
                        format!("∀ {} ∈ ℤ ∩ [{:.0}, {:.0}]", bound.name, min_s, max_s)
                    }
                    "Naturals" if bound.min_val.is_some() && bound.max_val.is_some() => {
                        format!("∀ {} ∈ ℕ ∩ [{:.0}, {:.0}]", bound.name, min_s, max_s)
                    }
                    "Positive" => format!("∀ {} ∈ ℝ⁺", bound.name),
                    "NonNegative" => format!("∀ {} ∈ ℝ⁺₀", bound.name),
                    "Naturals" => format!("∀ {} ∈ ℕ", bound.name),
                    "Integers" => format!("∀ {} ∈ ℤ", bound.name),
                    "Rationals" => format!("∀ {} ∈ ℚ", bound.name),
                    "Complex" => format!("∀ {} ∈ ℂ", bound.name),
                    "Quaternion" => format!("∀ {} ∈ ℍ", bound.name),
                    "Octonion" => format!("∀ {} ∈ 𝕆", bound.name),
                    "Sedenion" => format!("∀ {} ∈ 𝕊", bound.name),
                    "Dual" => format!("∀ {} ∈ 𝔻 (Dual)", bound.name),
                    "DualComplex" => format!("∀ {} ∈ 𝔻(ℂ) (Dual-Complex)", bound.name),
                    "DualQuaternion" => format!("∀ {} ∈ 𝔻(ℍ) (Dual-Quaternion)", bound.name),
                    "SplitComplex" => format!("∀ {} ∈ ℝ[j] (Split-Complex)", bound.name),
                    "Bicomplex" => format!("∀ {} ∈ ℂ₂ (Bicomplex)", bound.name),
                    "Clifford" => format!("∀ {} ∈ Cℓ (Clifford)", bound.name),
                    "PAdics" => format!("∀ {} ∈ ℚₚ (p-adic)", bound.name),
                    "Adeles" => format!("∀ {} ∈ 𝔸 (Adeles)", bound.name),
                    "Surreals" => format!("∀ {} ∈ 𝐍𝐨 (Surreal)", bound.name),
                    "ModuloUnits" => format!("∀ {} ∈ (ℤ/nℤ)ˣ", bound.name),
                    "Modulo" => format!("∀ {} ∈ ℤ/nℤ", bound.name),
                    "GaloisField" => format!("∀ {} ∈ GF(pᵏ)", bound.name),
                    "GaussianIntegers" => format!("∀ {} ∈ ℤ[i] (Gaussian)", bound.name),
                    "EisensteinIntegers" => format!("∀ {} ∈ ℤ[ω] (Eisenstein)", bound.name),
                    "Boolean" => format!("∀ {} ∈ 𝔹", bound.name),
                    "BitVector" => format!("∀ {} ∈ 𝔹ʷ", bound.name),
                    "EvenIntegers" => format!("∀ {} ∈ 2ℤ", bound.name),
                    "OddIntegers" => format!("∀ {} ∈ 2ℤ+1", bound.name),
                    "Matrix" => format!("∀ {} ∈ Mₘₓₙ", bound.name),
                    "Tensor" => format!("∀ {} ∈ Tʳ", bound.name),
                    _ => format!("∀ {} ∈ {}", bound.name, bound.domain_type),
                };

                parsed_lines.push(ParsedLine {
                    line_idx: idx,
                    raw_text: line_str.to_string(),
                    kind: LineKind::DomainRestriction {
                        name: bound.name.clone(),
                        rule_summary: display_unicode.clone(),
                    },
                    output_latex: format!("\\text{{{}}}", display_unicode),
                    output_unicode: display_unicode.clone(),
                    simplified_unicode: None,
                    substituted_latex: None,
                    derivative_latex: None,
                    domain_info: Some(format!(
                        "Domain Bound: {} in {}",
                        bound.name, bound.domain_type
                    )),
                    error_msg: None,
                    suggested_symbols: Vec::new(),
                    is_function: false,
                    physical_unit: None,
                    numerical_roots: None,
                    linearized_estimate: None,
                    is_pending: false,
                    eval_time_ms: 0,
                    scoped_context: current_scoped_context.clone(),
                });
                continue;
            }

            // Check domain restriction rule line e.g., `a in [-5, 5]` or `c >= 0`
            if (trimmed.contains(" in [") && trimmed.ends_with(']'))
                || (trimmed.contains(" >= ") && !trimmed.contains('{'))
            {
                parsed_lines.push(ParsedLine {
                    line_idx: idx,
                    raw_text: line_str.to_string(),
                    kind: LineKind::DomainRestriction {
                        name: trimmed
                            .split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .to_string(),
                        rule_summary: trimmed.to_string(),
                    },
                    output_latex: format!("\\text{{Domain Rule: }} {}", trimmed),
                    output_unicode: format!("Domain Rule: {}", trimmed),
                    simplified_unicode: None,
                    substituted_latex: None,
                    derivative_latex: None,
                    domain_info: Some("Parameter Bound Restriction".to_string()),
                    error_msg: None,
                    suggested_symbols: Vec::new(),
                    is_function: false,
                    physical_unit: None,
                    numerical_roots: None,
                    linearized_estimate: None,
                    is_pending: false,
                    eval_time_ms: 0,
                    scoped_context: current_scoped_context.clone(),
                });
                continue;
            }

            // Check role declaration line e.g., `a: Parameter = 2.0 [m]` or `x: Variable`
            if trimmed.contains(':')
                && (trimmed.contains("Parameter")
                    || trimmed.contains("Variable")
                    || trimmed.contains("param")
                    || trimmed.contains("var"))
            {
                if let Some((s_name, decl)) = trimmed.split_once(':') {
                    let sym_name = s_name.trim();
                    let role_type = if decl.contains("Variable") || decl.contains("var") {
                        SymbolRole::Variable
                    } else {
                        SymbolRole::Parameter
                    };

                    let meta_info = self.session.symbol_metadata.get(sym_name).cloned();
                    let unit_suffix = meta_info
                        .as_ref()
                        .and_then(|m| m.unit_str.as_ref())
                        .map(|u| format!(" [{}]", u))
                        .unwrap_or_default();
                    let val_info = meta_info
                        .as_ref()
                        .map(|m| format!(" = {:.2}", m.cur_val))
                        .unwrap_or_default();

                    parsed_lines.push(ParsedLine {
                        line_idx: idx,
                        raw_text: line_str.to_string(),
                        kind: LineKind::RoleDeclaration {
                            name: sym_name.to_string(),
                            role: role_type,
                        },
                        output_latex: format!(
                            "\\text{{{}: {:?}}}{}{}",
                            sym_name, role_type, val_info, unit_suffix
                        ),
                        output_unicode: format!(
                            "{}: {:?}{}{}",
                            sym_name, role_type, val_info, unit_suffix
                        ),
                        simplified_unicode: None,
                        substituted_latex: None,
                        derivative_latex: None,
                        domain_info: Some(format!(
                            "Declared symbol '{}' as {:?}",
                            sym_name, role_type
                        )),
                        error_msg: None,
                        suggested_symbols: Vec::new(),
                        is_function: false,
                        physical_unit: meta_info.and_then(|m| m.unit_str),
                        numerical_roots: None,
                        linearized_estimate: None,
                        is_pending: false,
                        eval_time_ms: 0,
                        scoped_context: current_scoped_context.clone(),
                    });
                    continue;
                }
            }

            // Check Set Builder Notation line e.g., `{ x in Reals | x >= 0 }` or `x in Reals | x >= 0`
            let sb_line_opt = if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.contains(" in ") && trimmed.contains('|'))
            {
                let s = trimmed.trim_matches(|c| c == '{' || c == '}').trim();
                if let Some((var_dom, cond_part)) = s.split_once('|') {
                    if let Some((v, d)) = var_dom.split_once(" in ") {
                        Some((
                            v.trim().to_string(),
                            d.trim().to_string(),
                            cond_part.trim().to_string(),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((var, dom, cond)) = sb_line_opt {
                match dom.as_str() {
                    "Quaternion" | "Quaternions" | "H" => {
                        current_scoped_context.algebra_domain =
                            urae::Domain::Hypercomplex { dimension: 4 };
                    }
                    "Grassmann" | "Clifford" => {
                        current_scoped_context.algebra_domain =
                            urae::Domain::Hypercomplex { dimension: 8 };
                    }
                    "Complex" | "C" => {
                        current_scoped_context.algebra_domain = urae::Domain::Complex;
                    }
                    "Reals" | "Real" | "R" => {
                        current_scoped_context.algebra_domain = urae::Domain::Reals;
                    }
                    _ => {}
                }

                let symbol_map = match dom.as_str() {
                    "Reals" | "Real" | "R" => "\\mathbb{R}",
                    "Complex" | "C" => "\\mathbb{C}",
                    "Integers" | "Z" => "\\mathbb{Z}",
                    "Quaternion" | "Quaternions" | "H" => "\\mathbb{H}",
                    "Grassmann" | "Clifford" => "\\Lambda(V)",
                    _ => "\\mathbb{R}",
                };

                let latex = format!(
                    "\\left\\{{ {} \\in {} \\mid {} \\right\\}}",
                    var, symbol_map, cond
                );
                let unicode_text = format!("{{ {} in {} | {} }}", var, dom, cond);

                parsed_lines.push(ParsedLine {
                    line_idx: idx,
                    raw_text: line_str.to_string(),
                    kind: LineKind::SetBuilder {
                        var_name: var,
                        domain_type: dom.clone(),
                        condition: cond,
                    },
                    output_latex: latex,
                    output_unicode: unicode_text,
                    simplified_unicode: None,
                    substituted_latex: None,
                    derivative_latex: None,
                    domain_info: Some(format!("Set Builder Domain: {}", dom)),
                    error_msg: None,
                    suggested_symbols: Vec::new(),
                    is_function: false,
                    physical_unit: None,
                    numerical_roots: None,
                    linearized_estimate: None,
                    is_pending: false,
                    eval_time_ms: 0,
                    scoped_context: current_scoped_context.clone(),
                });
                continue;
            }

            // Check permissive natural language mathematical intent (Solve, Differentiate, Integrate, Simplify, Distribution, Compare, Substitute)
            if let Some(intent) = urae::parser::parse_permissive_intent(trimmed) {
                match intent {
                    urae::parser::PermissiveIntent::Distribution {
                        name,
                        dist_type,
                        params,
                    } => {
                        let param_str = params.join(", ");
                        let display_text = format!("{} ~ {}({})", name, dist_type, param_str);
                        parsed_lines.push(ParsedLine {
                            line_idx: idx,
                            raw_text: line_str.to_string(),
                            kind: LineKind::Formula,
                            output_latex: display_text.clone(),
                            output_unicode: display_text,
                            simplified_unicode: None,
                            substituted_latex: None,
                            derivative_latex: None,
                            domain_info: Some(format!(
                                "Probability Distribution: {}({})",
                                dist_type, param_str
                            )),
                            error_msg: None,
                            suggested_symbols: Vec::new(),
                            is_function: false,
                            physical_unit: None,
                            numerical_roots: None,
                            linearized_estimate: None,
                            is_pending: false,
                            eval_time_ms: 0,
                            scoped_context: current_scoped_context.clone(),
                        });
                        continue;
                    }
                    urae::parser::PermissiveIntent::Compare {
                        left,
                        right,
                        comparison_type,
                    } => {
                        use urae::morphism::EquivalenceClassifier;
                        let classifier = EquivalenceClassifier::new();
                        let is_iso = classifier.check_isomorphism(&left, &right);
                        let cmp_result = if is_iso || comparison_type == "isomorphic" {
                            format!("{} ≅ {} (Isomorphic Structure)", left, right)
                        } else {
                            format!("Comparing {} and {}", left, right)
                        };

                        parsed_lines.push(ParsedLine {
                            line_idx: idx,
                            raw_text: line_str.to_string(),
                            kind: LineKind::Formula,
                            output_latex: cmp_result.clone(),
                            output_unicode: cmp_result,
                            simplified_unicode: None,
                            substituted_latex: None,
                            derivative_latex: None,
                            domain_info: Some(
                                "Multi-Tier Equivalence & Isomorphism Check".to_string(),
                            ),
                            error_msg: None,
                            suggested_symbols: Vec::new(),
                            is_function: false,
                            physical_unit: None,
                            numerical_roots: None,
                            linearized_estimate: None,
                            is_pending: false,
                            eval_time_ms: 0,
                            scoped_context: current_scoped_context.clone(),
                        });
                        continue;
                    }
                    urae::parser::PermissiveIntent::Substitute {
                        expression,
                        variable,
                        value,
                    } => {
                        let mut expanded = expression.clone();
                        for (fn_name, fn_rhs) in &fn_defs {
                            expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                        }

                        if let Ok(expr_id) = parser.parse(&expanded) {
                            let var_sym = self.graph.symbols.get_or_intern(&variable);
                            if let Ok(val_id) = parser.parse(&value) {
                                let subbed = self.graph.substitute(expr_id, var_sym, val_id);
                                use urae::simplify::SymbolicSimplifier;
                                let simplified = self.graph.simplify(subbed);
                                let out_fmt = self
                                    .unicode_formatter
                                    .format(&self.graph, simplified)
                                    .unwrap_or_default();

                                parsed_lines.push(ParsedLine {
                                    line_idx: idx,
                                    raw_text: line_str.to_string(),
                                    kind: LineKind::Formula,
                                    output_latex: format!(
                                        "{}|_{{{}={}}} = {}",
                                        expression, variable, value, out_fmt
                                    ),
                                    output_unicode: format!(
                                        "{} [{} ➔ {}] = {}",
                                        expression, variable, value, out_fmt
                                    ),
                                    simplified_unicode: Some(out_fmt.clone()),
                                    substituted_latex: None,
                                    derivative_latex: None,
                                    domain_info: Some(format!(
                                        "Substituted {} = {}",
                                        variable, value
                                    )),
                                    error_msg: None,
                                    suggested_symbols: Vec::new(),
                                    is_function: false,
                                    physical_unit: None,
                                    numerical_roots: None,
                                    linearized_estimate: None,
                                    is_pending: false,
                                    eval_time_ms: 0,
                                    scoped_context: current_scoped_context.clone(),
                                });
                                continue;
                            }
                        }
                    }
                    urae::parser::PermissiveIntent::DomainDeclaration { name, domain_desc } => {
                        let meta = self
                            .session
                            .symbol_metadata
                            .entry(name.clone())
                            .or_insert_with(|| {
                                SymbolMetadata::new(&name, 1.0, SymbolRole::Variable)
                            });
                        meta.domain_type = domain_desc.clone();
                        let display_text = format!("{}: Domain {}", name, domain_desc);

                        parsed_lines.push(ParsedLine {
                            line_idx: idx,
                            raw_text: line_str.to_string(),
                            kind: LineKind::Formula,
                            output_latex: format!("\\text{{{}: Domain {}}}", name, domain_desc),
                            output_unicode: display_text,
                            simplified_unicode: None,
                            substituted_latex: None,
                            derivative_latex: None,
                            domain_info: Some(format!("Declared Domain: {}", domain_desc)),
                            error_msg: None,
                            suggested_symbols: Vec::new(),
                            is_function: false,
                            physical_unit: None,
                            numerical_roots: None,
                            linearized_estimate: None,
                            is_pending: false,
                            eval_time_ms: 0,
                            scoped_context: current_scoped_context.clone(),
                        });
                        continue;
                    }
                    urae::parser::PermissiveIntent::Define { name, args, body } => {
                        fn_defs.insert(name.clone(), body.clone());
                        let fn_sig = format!("{}({})", name, args.join(", "));
                        let display_text = format!("{} = {}", fn_sig, body);

                        let mut sub_latex = None;
                        if let Ok(body_expr) = parser.parse(&body) {
                            for sym in Self::extract_symbols(&self.graph, body_expr) {
                                symbol_deps.entry(sym).or_default().insert(idx);
                            }
                            if !self.session.slider_values.is_empty() {
                                let mut subbed = body_expr;
                                for (p_name, &p_val) in &self.session.slider_values {
                                    if let Some(sym_id) = self.graph.symbols.get(p_name) {
                                        let val_node = self.graph.float(p_val);
                                        subbed = self.graph.substitute(subbed, sym_id, val_node);
                                    }
                                }
                                if let Ok(sub_fmt) = self.formatter.format(&self.graph, subbed) {
                                    sub_latex = Some(sub_fmt);
                                }
                            }
                        }

                        parsed_lines.push(ParsedLine {
                            line_idx: idx,
                            raw_text: line_str.to_string(),
                            kind: LineKind::Formula,
                            output_latex: display_text.clone(),
                            output_unicode: display_text,
                            simplified_unicode: None,
                            substituted_latex: sub_latex,
                            derivative_latex: None,
                            domain_info: Some(format!("User Function Definition: {}", fn_sig)),
                            error_msg: None,
                            suggested_symbols: Vec::new(),
                            is_function: true,
                            physical_unit: None,
                            numerical_roots: None,
                            linearized_estimate: None,
                            is_pending: false,
                            eval_time_ms: 0,
                            scoped_context: current_scoped_context.clone(),
                        });
                        continue;
                    }
                    urae::parser::PermissiveIntent::Differentiate {
                        expression,
                        variable,
                        is_total: _,
                    } => {
                        let mut expanded = expression.clone();
                        for (fn_name, fn_rhs) in &fn_defs {
                            expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                        }

                        let vars = if let Some(v) = variable {
                            vec![v]
                        } else if let Ok(expr_id) = parser.parse(&expanded) {
                            let mut syms = Self::extract_symbols(&self.graph, expr_id);
                            syms.retain(|s| s != "pi" && s != "e");
                            if syms.is_empty() {
                                vec!["x".to_string()]
                            } else {
                                syms
                            }
                        } else {
                            vec!["x".to_string()]
                        };

                        if let Ok(expr_id) = parser.parse(&expanded) {
                            if vars.len() > 1 {
                                let mut results_unicode = Vec::new();
                                let mut results_latex = Vec::new();
                                for (i, v_name) in vars.iter().enumerate() {
                                    let v_sym = self.graph.symbols.get_or_intern(v_name);
                                    let diff_id = self.graph.diff(expr_id, v_sym);
                                    use urae::simplify::SymbolicSimplifier;
                                    let simplified = self.graph.simplify(diff_id);
                                    let out_fmt = self
                                        .unicode_formatter
                                        .format(&self.graph, simplified)
                                        .unwrap_or_default();
                                    let out_latex = self
                                        .formatter
                                        .format(&self.graph, simplified)
                                        .unwrap_or_default();
                                    results_unicode.push(format!(
                                        "[{}] d({})/d{} = {}",
                                        i + 1,
                                        expression,
                                        v_name,
                                        out_fmt
                                    ));
                                    results_latex.push(format!(
                                        "[{}] \\frac{{d}}{{d{}}} \\left({}\\right) = {}",
                                        i + 1,
                                        v_name,
                                        expression,
                                        out_latex
                                    ));
                                }
                                let out_u = results_unicode.join(", ");
                                let out_l = results_latex.join(", ");
                                parsed_lines.push(ParsedLine {
                                    line_idx: idx,
                                    raw_text: line_str.to_string(),
                                    kind: LineKind::Formula,
                                    output_latex: out_l,
                                    output_unicode: out_u,
                                    simplified_unicode: None,
                                    substituted_latex: None,
                                    derivative_latex: None,
                                    domain_info: Some(format!(
                                        "Multi-variable partial derivatives w.r.t {:?}",
                                        vars
                                    )),
                                    error_msg: None,
                                    suggested_symbols: Vec::new(),
                                    is_function: false,
                                    physical_unit: None,
                                    numerical_roots: None,
                                    linearized_estimate: None,
                                    is_pending: false,
                                    eval_time_ms: 0,
                                    scoped_context: current_scoped_context.clone(),
                                });
                                continue;
                            }

                            let var_name = vars[0].clone();
                            let var_sym = self.graph.symbols.get_or_intern(&var_name);
                            let diff_id = self.graph.diff(expr_id, var_sym);
                            use urae::simplify::SymbolicSimplifier;
                            let simplified = self.graph.simplify(diff_id);
                            let out_fmt = self
                                .unicode_formatter
                                .format(&self.graph, simplified)
                                .unwrap_or_default();
                            let out_latex = self
                                .formatter
                                .format(&self.graph, simplified)
                                .unwrap_or_default();

                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: format!(
                                    "\\frac{{d}}{{d{}}} \\left({}\\right) = {}",
                                    var_name, expression, out_latex
                                ),
                                output_unicode: format!(
                                    "d({})/d{} = {}",
                                    expression, var_name, out_fmt
                                ),
                                simplified_unicode: Some(out_fmt.clone()),
                                substituted_latex: None,
                                derivative_latex: Some(out_latex),
                                domain_info: Some(format!("Derivative w.r.t {}", var_name)),
                                error_msg: None,
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                    }
                    urae::parser::PermissiveIntent::Integrate {
                        expression,
                        variable,
                        lower,
                        upper,
                    } => {
                        let mut expanded = expression.clone();
                        for (fn_name, fn_rhs) in &fn_defs {
                            expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                        }

                        let vars = if let Some(v) = variable {
                            vec![v]
                        } else if let Ok(expr_id) = parser.parse(&expanded) {
                            let mut syms = Self::extract_symbols(&self.graph, expr_id);
                            syms.retain(|s| s != "pi" && s != "e");
                            if syms.is_empty() {
                                vec!["x".to_string()]
                            } else {
                                syms
                            }
                        } else {
                            vec!["x".to_string()]
                        };

                        if let Ok(expr_id) = parser.parse(&expanded) {
                            if vars.len() > 1 && lower.is_none() && upper.is_none() {
                                let mut results_unicode = Vec::new();
                                let mut results_latex = Vec::new();
                                for (i, v_name) in vars.iter().enumerate() {
                                    let v_sym = self.graph.symbols.get_or_intern(v_name);
                                    match self.graph.integrate(expr_id, v_sym) {
                                        Ok(int_id) => {
                                            use urae::simplify::SymbolicSimplifier;
                                            let simplified = self.graph.simplify(int_id);
                                            let out_fmt = self
                                                .unicode_formatter
                                                .format(&self.graph, simplified)
                                                .unwrap_or_default();
                                            let out_latex = self
                                                .formatter
                                                .format(&self.graph, simplified)
                                                .unwrap_or_default();
                                            results_unicode.push(format!(
                                                "[{}] ∫ ({}) d{} = {} + C",
                                                i + 1,
                                                expression,
                                                v_name,
                                                out_fmt
                                            ));
                                            results_latex.push(format!(
                                                "[{}] \\int {} \\, d{} = {} + C",
                                                i + 1,
                                                expression,
                                                v_name,
                                                out_latex
                                            ));
                                        }
                                        Err(_) => {
                                            results_unicode.push(format!(
                                                "[{}] ∫ ({}) d{}",
                                                i + 1,
                                                expression,
                                                v_name
                                            ));
                                            results_latex.push(format!(
                                                "[{}] \\int {} \\, d{}",
                                                i + 1,
                                                expression,
                                                v_name
                                            ));
                                        }
                                    }
                                }
                                let out_u = results_unicode.join(", ");
                                let out_l = results_latex.join(", ");
                                parsed_lines.push(ParsedLine {
                                    line_idx: idx,
                                    raw_text: line_str.to_string(),
                                    kind: LineKind::Formula,
                                    output_latex: out_l,
                                    output_unicode: out_u,
                                    simplified_unicode: None,
                                    substituted_latex: None,
                                    derivative_latex: None,
                                    domain_info: Some(format!(
                                        "Multi-variable integrals w.r.t {:?}",
                                        vars
                                    )),
                                    error_msg: None,
                                    suggested_symbols: Vec::new(),
                                    is_function: false,
                                    physical_unit: None,
                                    numerical_roots: None,
                                    linearized_estimate: None,
                                    is_pending: false,
                                    eval_time_ms: 0,
                                    scoped_context: current_scoped_context.clone(),
                                });
                                continue;
                            }

                            let var_name = vars[0].clone();
                            let var_sym = self.graph.symbols.get_or_intern(&var_name);
                            match self.graph.integrate(expr_id, var_sym) {
                                Ok(int_id) => {
                                    use urae::simplify::SymbolicSimplifier;
                                    let simplified = self.graph.simplify(int_id);
                                    let out_fmt = self
                                        .unicode_formatter
                                        .format(&self.graph, simplified)
                                        .unwrap_or_default();
                                    let out_latex = self
                                        .formatter
                                        .format(&self.graph, simplified)
                                        .unwrap_or_default();

                                    let bounds_display =
                                        if let (Some(l), Some(u)) = (&lower, &upper) {
                                            format!(" from {} to {}", l, u)
                                        } else {
                                            String::new()
                                        };

                                    parsed_lines.push(ParsedLine {
                                        line_idx: idx,
                                        raw_text: line_str.to_string(),
                                        kind: LineKind::Formula,
                                        output_latex: format!(
                                            "\\int {} \\, d{} = {} + C",
                                            expression, var_name, out_latex
                                        ),
                                        output_unicode: format!(
                                            "∫ ({}) d{}{} = {} + C",
                                            expression, var_name, bounds_display, out_fmt
                                        ),
                                        simplified_unicode: Some(out_fmt.clone()),
                                        substituted_latex: None,
                                        derivative_latex: None,
                                        domain_info: Some(format!(
                                            "Analytical Integral w.r.t {}",
                                            var_name
                                        )),
                                        error_msg: None,
                                        suggested_symbols: Vec::new(),
                                        is_function: false,
                                        physical_unit: None,
                                        numerical_roots: None,
                                        linearized_estimate: None,
                                        is_pending: false,
                                        eval_time_ms: 0,
                                        scoped_context: current_scoped_context.clone(),
                                    });
                                    continue;
                                }
                                Err(e) => {
                                    parsed_lines.push(ParsedLine {
                                        line_idx: idx,
                                        raw_text: line_str.to_string(),
                                        kind: LineKind::Formula,
                                        output_latex: format!(
                                            "\\int {} \\, d{}",
                                            expression, var_name
                                        ),
                                        output_unicode: format!("∫ ({}) d{}", expression, var_name),
                                        simplified_unicode: None,
                                        substituted_latex: None,
                                        derivative_latex: None,
                                        domain_info: Some("Integral unevaluated".to_string()),
                                        error_msg: Some(format!("Integration error: {}", e)),
                                        suggested_symbols: Vec::new(),
                                        is_function: false,
                                        physical_unit: None,
                                        numerical_roots: None,
                                        linearized_estimate: None,
                                        is_pending: false,
                                        eval_time_ms: 0,
                                        scoped_context: current_scoped_context.clone(),
                                    });
                                    continue;
                                }
                            }
                        }
                    }
                    urae::parser::PermissiveIntent::Simplify { expression } => {
                        if let Ok(expr_id) = parser.parse(&expression) {
                            use urae::simplify::SymbolicSimplifier;
                            let simplified = self.graph.simplify(expr_id);
                            let out_fmt = self
                                .unicode_formatter
                                .format(&self.graph, simplified)
                                .unwrap_or_default();
                            let out_latex = self
                                .formatter
                                .format(&self.graph, simplified)
                                .unwrap_or_default();

                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: out_latex,
                                output_unicode: out_fmt.clone(),
                                simplified_unicode: Some(out_fmt),
                                substituted_latex: None,
                                derivative_latex: None,
                                domain_info: Some("E-Graph Simplified".to_string()),
                                error_msg: None,
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                    }
                    urae::parser::PermissiveIntent::Limit {
                        expression,
                        variable,
                        point,
                    } => {
                        let var_sym = self.graph.symbols.get_or_intern(&variable);
                        let mut expanded = expression.clone();
                        for (fn_name, fn_rhs) in &fn_defs {
                            expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                        }
                        if let Ok(expr_id) = parser.parse(&expanded) {
                            let pt_val = point.parse::<f64>().unwrap_or(0.0);
                            let pt_id = self.graph.float(pt_val);
                            use urae::calculus::SymbolicCalculus;
                            let lim_res = self.graph.limit(expr_id, var_sym, pt_id);
                            let (out_unicode, out_latex, err_msg) = match lim_res {
                                Ok(lim_id) => {
                                    let u = self
                                        .unicode_formatter
                                        .format(&self.graph, lim_id)
                                        .unwrap_or_default();
                                    let l = self
                                        .formatter
                                        .format(&self.graph, lim_id)
                                        .unwrap_or_default();
                                    (u, l, None)
                                }
                                Err(e) => (
                                    format!("Limit Error: {}", e),
                                    format!("\\text{{Limit Error: {}}}", e),
                                    Some(format!("Limit error: {}", e)),
                                ),
                            };
                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: format!(
                                    "\\lim_{{{} \\to {}}} {} = {}",
                                    variable, point, expression, out_latex
                                ),
                                output_unicode: format!(
                                    "lim_{{{}➔{}}} ({}) = {}",
                                    variable, point, expression, out_unicode
                                ),
                                simplified_unicode: Some(out_unicode.clone()),
                                substituted_latex: None,
                                derivative_latex: None,
                                domain_info: Some(format!("Limit as {} -> {}", variable, point)),
                                error_msg: err_msg,
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                    }
                    urae::parser::PermissiveIntent::Sum {
                        expression,
                        variable,
                        lower,
                        upper,
                    } => {
                        let mut expanded = expression.clone();
                        for (fn_name, fn_rhs) in &fn_defs {
                            expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                        }
                        if let Ok(expr_id) = parser.parse(&expanded) {
                            let out_unicode = self
                                .unicode_formatter
                                .format(&self.graph, expr_id)
                                .unwrap_or_default();
                            let out_latex = self
                                .formatter
                                .format(&self.graph, expr_id)
                                .unwrap_or_default();
                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: format!(
                                    "\\sum_{{{}={}}}^{{{}}} {} = {}",
                                    variable, lower, upper, expression, out_latex
                                ),
                                output_unicode: format!(
                                    "∑_{{{}={}}}^{{{}}} ({})",
                                    variable, lower, upper, out_unicode
                                ),
                                simplified_unicode: Some(out_unicode.clone()),
                                substituted_latex: None,
                                derivative_latex: None,
                                domain_info: Some(format!(
                                    "Summation from {}={} to {}",
                                    variable, lower, upper
                                )),
                                error_msg: None,
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                    }
                    _ => {}
                }
            }

            // Check solve command intent
            if let Some((eq_str, var_name)) = parse_permissive_solve_command(trimmed) {
                let var_sym = self.graph.symbols.get_or_intern(&var_name);
                let clean_eq = eq_str.trim();

                let mut expanded = clean_eq.to_string();
                for (fn_name, fn_rhs) in &fn_defs {
                    expanded = Self::expand_user_function_call(&expanded, fn_name, fn_rhs);
                }

                if let Ok(eq_id) = parser.parse(&expanded) {
                    match self.graph.solveset(eq_id, var_sym) {
                        Ok(sols) => {
                            let sol_strs: Vec<String> = sols
                                .iter()
                                .map(|&s| {
                                    self.unicode_formatter
                                        .format(&self.graph, s)
                                        .unwrap_or_default()
                                })
                                .collect();
                            let sol_latex: Vec<String> = sols
                                .iter()
                                .map(|&s| self.formatter.format(&self.graph, s).unwrap_or_default())
                                .collect();

                            let display_sol = format!("{} = {{{}}}", var_name, sol_strs.join(", "));
                            let display_latex = format!(
                                "{} \\in \\left\\{{ {} \\right\\}}",
                                var_name,
                                sol_latex.join(", ")
                            );

                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: display_latex,
                                output_unicode: display_sol,
                                simplified_unicode: None,
                                substituted_latex: None,
                                derivative_latex: None,
                                domain_info: Some(format!(
                                    "Exact Algebraic Solutions for {}",
                                    var_name
                                )),
                                error_msg: None,
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                        Err(err) => {
                            parsed_lines.push(ParsedLine {
                                line_idx: idx,
                                raw_text: line_str.to_string(),
                                kind: LineKind::Formula,
                                output_latex: format!("\\text{{Solve Error: {}}}", err),
                                output_unicode: format!("Solve Error: {}", err),
                                simplified_unicode: None,
                                substituted_latex: None,
                                derivative_latex: None,
                                domain_info: Some(format!("Solver for {}", var_name)),
                                error_msg: Some(format!("Solve error: {}", err)),
                                suggested_symbols: Vec::new(),
                                is_function: false,
                                physical_unit: None,
                                numerical_roots: None,
                                linearized_estimate: None,
                                is_pending: false,
                                eval_time_ms: 0,
                                scoped_context: current_scoped_context.clone(),
                            });
                            continue;
                        }
                    }
                }
            }

            // Standard Expression or Equation parsing
            let (expr_body, unit_suffix) = if let Some(bracket_idx) = trimmed.find('[') {
                let u = trimmed[bracket_idx..]
                    .trim_matches(|c| c == '[' || c == ']')
                    .trim();
                (&trimmed[..bracket_idx], Some(u.to_string()))
            } else {
                (trimmed, None)
            };

            let clean_expr = expr_body.trim();

            match parser.parse(clean_expr) {
                Ok(expr_id) => {
                    let mut latex = self
                        .formatter
                        .format(&self.graph, expr_id)
                        .unwrap_or_else(|_| clean_expr.to_string());
                    let mut unicode = self
                        .unicode_formatter
                        .format(&self.graph, expr_id)
                        .unwrap_or_else(|_| clean_expr.to_string());

                    if let Some(u) = &unit_suffix {
                        latex.push_str(&format!(" \\left[\\text{{{}}}\\right]", u));
                        unicode.push_str(&format!(" [{}]", u));
                    }

                    // Extract symbol dependencies
                    let syms = Self::extract_symbols(&self.graph, expr_id);
                    for s in &syms {
                        symbol_deps.entry(s.clone()).or_default().insert(idx);
                    }

                    // Background simplification
                    let mut simp_unicode = None;
                    use urae::simplify::{Simplifier, SymbolicSimplifier};
                    let simp_candidate1 = self.graph.simplify(expr_id);
                    let simp_candidate2 =
                        Simplifier::min_tree(&self.graph, expr_id).unwrap_or(expr_id);

                    let fmt1 = self
                        .unicode_formatter
                        .format(&self.graph, simp_candidate1)
                        .unwrap_or_default();
                    let fmt2 = self
                        .unicode_formatter
                        .format(&self.graph, simp_candidate2)
                        .unwrap_or_default();

                    let best_fmt =
                        if !fmt2.is_empty() && (fmt1.is_empty() || fmt2.len() < fmt1.len()) {
                            fmt2
                        } else {
                            fmt1
                        };

                    if !best_fmt.is_empty() && best_fmt != unicode && best_fmt != clean_expr {
                        simp_unicode = Some(best_fmt);
                    } else if unicode != clean_expr
                        && clean_expr
                            .chars()
                            .any(|c| c == '+' || c == '*' || c == '-' || c == '/' || c == '(')
                    {
                        simp_unicode = Some(unicode.clone());
                    }

                    // Parameter substitution
                    let mut sub_latex = None;
                    let mut eval_val_opt = None;
                    if !self.session.slider_values.is_empty() {
                        let mut subbed = expr_id;
                        for (param, &val) in &self.session.slider_values {
                            if let Some(sym_id) = self.graph.symbols.get(param) {
                                let val_node = self.graph.float(val);
                                subbed = self.graph.substitute(subbed, sym_id, val_node);
                            }
                        }
                        if let Ok(sub_fmt) = self.formatter.format(&self.graph, subbed) {
                            sub_latex = Some(sub_fmt);
                        }

                        let mut ctx = EvalContext::default();
                        for (p, &v) in &self.session.slider_values {
                            ctx.bindings.insert(p.clone(), v);
                        }
                        if let Ok(eval_res) = self.graph.evalf(expr_id, &ctx) {
                            let f = eval_res.to_f64();
                            if f.is_finite() {
                                eval_val_opt = Some(f);
                                line_results_f64.insert(idx, f);
                                last_evaluated_val = Some(f);
                            }
                        }
                    }

                    // Check domain bounds violations for assignments e.g. `a = -5`
                    let mut out_of_domain_err = None;
                    if let Some((sym_name, val_part)) = clean_expr.split_once('=') {
                        let s_name = sym_name.trim();
                        if let Ok(val_num) = val_part.trim().parse::<f64>() {
                            if let Some(meta) = self.session.symbol_metadata.get(s_name) {
                                if val_num < meta.min_val || val_num > meta.max_val {
                                    out_of_domain_err = Some(format!(
                                        "Out of domain: value {:.2} violates bound [{:.2}, {:.2}]",
                                        val_num, meta.min_val, meta.max_val
                                    ));
                                }
                            }
                        }
                    }

                    // Derivative preview if single variable
                    let mut deriv_latex = None;
                    let free_vars: Vec<String> = syms
                        .into_iter()
                        .filter(|s| {
                            !self.session.slider_values.contains_key(s) && s != "pi" && s != "e"
                        })
                        .collect();
                    if free_vars.len() == 1 {
                        let v_sym = self.graph.symbols.get_or_intern(&free_vars[0]);
                        let d_id = self.graph.diff(expr_id, v_sym);
                        use urae::simplify::SymbolicSimplifier;
                        let simp_d = self.graph.simplify(d_id);
                        if let Ok(d_fmt) = self.formatter.format(&self.graph, simp_d) {
                            deriv_latex = Some(d_fmt);
                        }
                    }

                    parsed_lines.push(ParsedLine {
                        line_idx: idx,
                        raw_text: line_str.to_string(),
                        kind: LineKind::Formula,
                        output_latex: latex,
                        output_unicode: unicode,
                        simplified_unicode: simp_unicode,
                        substituted_latex: sub_latex,
                        derivative_latex: deriv_latex,
                        domain_info: None,
                        error_msg: out_of_domain_err,
                        suggested_symbols: Vec::new(),
                        is_function: false,
                        physical_unit: unit_suffix,
                        numerical_roots: None,
                        linearized_estimate: eval_val_opt,
                        is_pending: false,
                        eval_time_ms: 0,
                        scoped_context: current_scoped_context.clone(),
                    });
                }
                Err(err) => {
                    let mut out_of_domain_err = None;
                    if let Some((sym_name, val_part)) = clean_expr.split_once('=') {
                        let s_name = sym_name.trim();
                        if let Ok(val_num) = val_part.trim().parse::<f64>() {
                            if let Some(meta) = self.session.symbol_metadata.get(s_name) {
                                if val_num < meta.min_val || val_num > meta.max_val {
                                    out_of_domain_err = Some(format!(
                                        "Out of domain: value {:.2} violates bound [{:.2}, {:.2}]",
                                        val_num, meta.min_val, meta.max_val
                                    ));
                                }
                            }
                        }
                    }

                    parsed_lines.push(ParsedLine {
                        line_idx: idx,
                        raw_text: line_str.to_string(),
                        kind: LineKind::Formula,
                        output_latex: format!("\\text{{{}}}", clean_expr),
                        output_unicode: clean_expr.to_string(),
                        simplified_unicode: None,
                        substituted_latex: None,
                        derivative_latex: None,
                        domain_info: None,
                        error_msg: out_of_domain_err.or_else(|| Some(format!("{}", err))),
                        suggested_symbols: Vec::new(),
                        is_function: false,
                        physical_unit: unit_suffix,
                        numerical_roots: None,
                        linearized_estimate: None,
                        is_pending: false,
                        eval_time_ms: 0,
                        scoped_context: current_scoped_context.clone(),
                    });
                }
            }
        }

        // Garbage collect unused symbols
        if !active_document_symbols.is_empty() {
            self.session
                .symbol_metadata
                .retain(|sym, _| active_document_symbols.contains(sym));
            self.session.slider_values.retain(|sym, _| {
                active_document_symbols.contains(sym)
                    || active_document_symbols.iter().any(|s| sym.starts_with(s))
            });
        }

        self.symbol_dependencies = symbol_deps;
        self.measured_eval_latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        self.parsed_lines = parsed_lines;
    }

    /// Focus a line for in-situ click-to-edit modification.
    pub fn start_editing_line(&mut self, line_idx: usize) {
        let lines: Vec<&str> = self.session.raw_document_text.lines().collect();
        if line_idx < lines.len() {
            self.focused_line = Some(line_idx);
            self.line_edit_buffer = lines[line_idx].to_string();
        }
    }

    /// Commit active line edit buffer back to document and re-evaluate.
    pub fn commit_focused_line(&mut self) {
        if let Some(target_idx) = self.focused_line {
            let mut lines: Vec<String> = self
                .session
                .raw_document_text
                .lines()
                .map(|s| s.to_string())
                .collect();
            if target_idx < lines.len() && lines[target_idx] != self.line_edit_buffer {
                self.record_snapshot(format!("Edit line {}", target_idx + 1));
                lines[target_idx] = self.line_edit_buffer.clone();
                self.session.raw_document_text = lines.join("\n");
                self.evaluate_all();
            }
            self.focused_line = None;
        }
    }

    /// Insert generated syntax from builder palettes at the active cursor location.
    pub fn insert_text_at_active_line(&mut self, text_to_insert: &str) {
        self.record_snapshot("Insert syntax at cursor");

        // 1. If we have a precise cursor character index in the document
        if let Some(char_idx) = self.cursor_char_idx {
            let doc = &self.session.raw_document_text;
            let clamped = char_idx.min(doc.len());

            let before = &doc[..clamped];
            let after = &doc[clamped..];

            let line_start = before.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
            let line_end = clamped + after.find('\n').unwrap_or(after.len());
            let current_line = &doc[line_start..line_end];

            let mut new_doc = String::new();
            let new_cursor_pos;

            if current_line.trim().is_empty() {
                // Current line is blank: replace this line with text_to_insert
                new_doc.push_str(&doc[..line_start]);
                new_doc.push_str(text_to_insert);
                new_cursor_pos = new_doc.len();
                new_doc.push_str(&doc[line_end..]);
            } else {
                // Current line has text: insert a new line right after it!
                new_doc.push_str(&doc[..line_end]);
                new_doc.push('\n');
                new_doc.push_str(text_to_insert);
                new_cursor_pos = new_doc.len();
                new_doc.push_str(&doc[line_end..]);
            }

            self.session.raw_document_text = new_doc;
            self.cursor_char_idx = Some(new_cursor_pos);
            let new_line_idx = self.session.raw_document_text[..new_cursor_pos]
                .chars()
                .filter(|&c| c == '\n')
                .count();
            self.focused_line = Some(new_line_idx);
            self.evaluate_all();
            return;
        }

        // 2. Fallback to focused_line if cursor_char_idx was not yet captured
        let mut lines: Vec<String> = self
            .session
            .raw_document_text
            .lines()
            .map(|s| s.to_string())
            .collect();

        if let Some(target_idx) = self.focused_line {
            if target_idx < lines.len() {
                if lines[target_idx].trim().is_empty() {
                    lines[target_idx] = text_to_insert.to_string();
                } else {
                    lines.insert(target_idx + 1, text_to_insert.to_string());
                    self.focused_line = Some(target_idx + 1);
                }
            } else {
                lines.push(text_to_insert.to_string());
                self.focused_line = Some(lines.len().saturating_sub(1));
            }
        } else {
            lines.push(text_to_insert.to_string());
            self.focused_line = Some(lines.len().saturating_sub(1));
        }

        self.session.raw_document_text = lines.join("\n");
        self.evaluate_all();
    }

    /// Selective fine-grained delta re-evaluation when parameters change at 60 FPS.
    pub fn evaluate_delta(&mut self, changed_symbol: &str) {
        let start_time = web_time::Instant::now();
        let dep_indices = match self.symbol_dependencies.get(changed_symbol) {
            Some(indices) => indices.clone(),
            None => {
                self.evaluate_all();
                return;
            }
        };

        let parser = ExprParser::new(&self.graph);

        for &idx in &dep_indices {
            if idx < self.parsed_lines.len() {
                let pl = &mut self.parsed_lines[idx];
                if pl.kind == LineKind::Formula {
                    let trimmed = pl.raw_text.trim();
                    let clean_expr = if let Some(idx_br) = trimmed.find('[') {
                        trimmed[..idx_br].trim()
                    } else {
                        trimmed
                    };

                    let expr_to_parse =
                        if let Some(intent) = urae::parser::parse_permissive_intent(clean_expr) {
                            match intent {
                                urae::parser::PermissiveIntent::Define { body, .. } => body,
                                _ => clean_expr.to_string(),
                            }
                        } else if let Some((_lhs, rhs)) = clean_expr.split_once('=') {
                            rhs.trim().to_string()
                        } else {
                            clean_expr.to_string()
                        };

                    if let Ok(expr_id) = parser.parse(&expr_to_parse) {
                        let mut cur_expr = expr_id;
                        for (param, &val) in &self.session.slider_values {
                            let param_sym = self.graph.symbols.get_or_intern(param);
                            let val_node = self.graph.float(val);
                            cur_expr = self.graph.substitute(cur_expr, param_sym, val_node);
                        }
                        if let Ok(sub_fmt) = self.formatter.format(&self.graph, cur_expr) {
                            pl.substituted_latex = Some(sub_fmt);
                        }
                    }
                }
            }
        }
        self.measured_eval_latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notebook_state_initialization_and_evaluation() {
        let state = NotebookState::new_clean();
        assert!(!state.parsed_lines.is_empty());
        assert!(state.session.slider_values.contains_key("a"));
        assert!(state.session.slider_values.contains_key("c"));
    }

    #[test]
    fn test_resolve_line_references_and_ranges() {
        let mut results = HashMap::new();
        results.insert(0, 10.0); // Line 1 ($1)
        results.insert(1, 20.0); // Line 2 ($2)
        results.insert(2, 30.0); // Line 3 ($3)

        // Single reference
        let resolved = resolve_line_references("$1 + 5", None, &results);
        assert_eq!(resolved, "10 + 5");

        // Sum aggregation
        let resolved_sum = resolve_line_references("sum($1..$3)", None, &results);
        assert_eq!(resolved_sum, "60");

        // Ans substitution
        let resolved_ans = resolve_line_references("ans * 2", Some(42.0), &results);
        assert_eq!(resolved_ans, "42 * 2");
    }

    #[test]
    fn test_reconcile_line_references_on_shift() {
        let old_lines = vec!["x = 5", "y = $1 * 2"];
        let new_lines = vec!["// header", "x = 5", "y = $1 * 2"];

        let shifted = reconcile_line_references_on_shift(&old_lines, &new_lines);
        assert!(shifted.is_some());
        let shifted_text = shifted.unwrap();
        assert!(shifted_text.contains("y = $2 * 2"));
    }

    #[test]
    fn test_domain_bound_and_role_handling() {
        let bound = parse_domain_declaration("a in Reals [0, 100]").unwrap();
        assert_eq!(bound.name, "a");
        assert_eq!(bound.min_val, Some(0.0));
        assert_eq!(bound.max_val, Some(100.0));

        let mut meta = SymbolMetadata::new("a", 150.0, SymbolRole::Parameter);
        meta.min_val = 0.0;
        meta.max_val = 100.0;
        assert_eq!(meta.clamp(150.0), 100.0);
    }

    #[test]
    fn test_undo_redo_history() {
        let mut state = NotebookState::new_clean();
        let initial_text = state.session.raw_document_text.clone();

        state.record_snapshot("Initial state");
        state.session.raw_document_text.push_str("\nz = 99");
        state.evaluate_all();

        assert!(state.can_undo());
        assert!(state.undo());
        assert_eq!(state.session.raw_document_text, initial_text);

        assert!(state.can_redo());
        assert!(state.redo());
        assert!(state.session.raw_document_text.contains("z = 99"));
    }

    #[test]
    fn test_insert_text_at_cursor_and_focused_line() {
        let mut state = NotebookState::new_clean();
        state.session.raw_document_text = "line1 = 1\nline2 = 2\nline3 = 3".to_string();
        state.evaluate_all();

        // 1. With focused_line = 0 (first line with content)
        state.focused_line = Some(0);
        state.cursor_char_idx = None;
        state.insert_text_at_active_line("inserted_after_1 = 100");

        let lines: Vec<&str> = state.session.raw_document_text.lines().collect();
        assert_eq!(lines[0], "line1 = 1");
        assert_eq!(lines[1], "inserted_after_1 = 100");
        assert_eq!(lines[2], "line2 = 2");

        // 2. With cursor_char_idx in the middle of line2
        // Find position of "line2"
        let line2_pos = state.session.raw_document_text.find("line2").unwrap();
        state.cursor_char_idx = Some(line2_pos + 2);
        state.insert_text_at_active_line("inserted_after_line2 = 200");

        let lines2: Vec<&str> = state.session.raw_document_text.lines().collect();
        assert_eq!(lines2[2], "line2 = 2");
        assert_eq!(lines2[3], "inserted_after_line2 = 200");
        assert_eq!(lines2[4], "line3 = 3");
    }
}
