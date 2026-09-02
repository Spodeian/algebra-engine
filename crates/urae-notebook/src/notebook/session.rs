//! # `urae_notebook::notebook::session`
//!
//! Session Persistence, Document Configuration, and BSON Export/Import.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[allow(dead_code)]
pub const SESSION_FILE_NAME: &str = "urae_notebook_session.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardDisplayMode {
    Inline,
    Hidden,
    PoppedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolRole {
    Variable,
    Parameter,
    Constant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolMetadata {
    pub name: String,
    pub role: SymbolRole,
    pub cur_val: f64,
    pub min_val: f64,
    pub max_val: f64,
    pub domain_type: String, // "Real", "Complex", "Integer", "Quaternion", "Grassmann", "Positive", "NonNegative"
    pub unit_str: Option<String>,
    #[serde(default)]
    pub parsed_text_val: Option<f64>,
}

impl SymbolMetadata {
    pub fn new(name: impl Into<String>, val: f64, role: SymbolRole) -> Self {
        Self {
            name: name.into(),
            role,
            cur_val: val,
            min_val: -10.0,
            max_val: 10.0,
            domain_type: "Real".to_string(),
            unit_str: None,
            parsed_text_val: Some(val),
        }
    }

    /// Clamp current value within min_val..max_val and domain restrictions.
    pub fn clamp(&self, val: f64) -> f64 {
        let mut clamped = val.clamp(self.min_val, self.max_val);
        match self.domain_type.as_str() {
            "Positive" if clamped <= 0.0 => {
                clamped = 0.001;
            }
            "NonNegative" if clamped < 0.0 => {
                clamped = 0.0;
            }
            "Integer" => {
                clamped = clamped.round();
            }
            _ => {}
        }
        clamped
    }

    /// Switch symbol role while retaining all existing domain, bounds, and unit metadata.
    pub fn set_role(&mut self, new_role: SymbolRole) {
        self.role = new_role;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReactiveComputeMode {
    /// Adaptive Dual-Rate (Recommended Default):
    /// Instant 60 FPS plot curves and algebraic updates during drag, with soft-cap
    /// lightweight previews for heavy solvers, finalizing to full precision on release.
    #[default]
    AdaptiveDualRate,

    /// Soft-Cap Fast Drag:
    /// Clamps ODE steps and Newton iterations strictly during active mouse scrub,
    /// re-evaluating at full precision when the drag ends.
    SoftCapFastDrag,

    /// Frame-Budgeted Guard (8ms budget):
    /// Executes synchronously if DAG delta takes < 8ms; automatically delegates
    /// heavier sub-graphs to background workers if the frame budget is exceeded.
    FrameBudgeted,

    /// Full Synchronous:
    /// Re-evaluates exact full precision on every micro-tick (ideal for high-end multi-core desktop workstations).
    FullSynchronous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MatrixPresetKind {
    #[default]
    Custom,
    Identity,
    Zero,
    Diagonal,
    Symmetric,
    PauliX,
    PauliY,
    PauliZ,
}

/// Palette code generator for multi-dimensional matrices and tensors.
pub fn generate_matrix_syntax(
    rows: usize,
    cols: usize,
    preset: MatrixPresetKind,
    elements: Option<&Vec<Vec<String>>>,
) -> String {
    match preset {
        MatrixPresetKind::Identity => {
            format!("M = eye({})", rows.max(1))
        }
        MatrixPresetKind::Zero => {
            format!("M = zeros({}, {})", rows.max(1), cols.max(1))
        }
        MatrixPresetKind::Diagonal => {
            let diag_elems: Vec<String> = (1..=rows).map(|i| format!("d_{}", i)).collect();
            format!("M = diag([{}])", diag_elems.join(", "))
        }
        MatrixPresetKind::Symmetric => {
            let mut row_strs = Vec::new();
            for r in 0..rows {
                let mut col_strs = Vec::new();
                for c in 0..cols {
                    let (min_i, max_i) = if r <= c {
                        (r + 1, c + 1)
                    } else {
                        (c + 1, r + 1)
                    };
                    col_strs.push(format!("s_{}{}", min_i, max_i));
                }
                row_strs.push(format!("[{}]", col_strs.join(", ")));
            }
            format!("M = matrix([{}])", row_strs.join(", "))
        }
        MatrixPresetKind::PauliX => "sigma_x = matrix([[0, 1], [1, 0]])".to_string(),
        MatrixPresetKind::PauliY => "sigma_y = matrix([[0, -i], [i, 0]])".to_string(),
        MatrixPresetKind::PauliZ => "sigma_z = matrix([[1, 0], [0, -1]])".to_string(),
        MatrixPresetKind::Custom => {
            if let Some(grid) = elements {
                let mut row_strs = Vec::new();
                for r in grid {
                    let col_strs: Vec<String> = r
                        .iter()
                        .map(|s| {
                            if s.trim().is_empty() {
                                "0".to_string()
                            } else {
                                s.trim().to_string()
                            }
                        })
                        .collect();
                    row_strs.push(format!("[{}]", col_strs.join(", ")));
                }
                format!("M = matrix([{}])", row_strs.join(", "))
            } else {
                format!("M = zeros({}, {})", rows.max(1), cols.max(1))
            }
        }
    }
}

/// Palette code generator for set-builder intervals.
pub fn generate_interval_syntax(
    var: &str,
    domain: &str,
    min: f64,
    max: f64,
    inc_min: bool,
    inc_max: bool,
) -> String {
    let left_op = if inc_min { "<=" } else { "<" };
    let right_op = if inc_max { "<=" } else { "<" };
    format!(
        "{{ {} in {} | {:.2} {} {} {} {:.2} }}",
        var.trim(),
        domain.trim(),
        min,
        left_op,
        var.trim(),
        right_op,
        max
    )
}

/// Palette code generator for Riemann branch cuts.
pub fn generate_branch_cut_syntax(fn_name: &str, sheet_k: i32, cut_pos: &str) -> String {
    format!(
        "branch_cut(\"{}\", sheet = {}, cut = \"{}\")",
        fn_name.trim(),
        sheet_k,
        cut_pos.trim()
    )
}

/// Palette code generator for ODE/PDE initial and boundary conditions.
pub fn generate_ode_bc_syntax(var_dep: &str, var_indep: &str, y0: f64, dy0: Option<f64>) -> String {
    if let Some(dy) = dy0 {
        format!(
            "{}(0) = {:.2}, {}'({}) = {:.2}",
            var_dep.trim(),
            y0,
            var_dep.trim(),
            var_indep.trim(),
            dy
        )
    } else {
        format!("{}(0) = {:.2}", var_dep.trim(), y0)
    }
}

/// Palette code generator for physical SI units.
pub fn generate_physical_unit_syntax(var: &str, val: f64, unit: &str) -> String {
    format!("{} = {:.2} [{}]", var.trim(), val, unit.trim())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotebookSettings {
    pub auto_sliders: bool,
    pub auto_plots: bool,
    pub show_hover_tooltips: bool,
    pub show_derivatives: bool,
    pub default_range_min: f64,
    pub default_range_max: f64,
    pub font_size: f32,
    pub logging_level: String,
    pub ai_config: urae::agent::AiConfig,
    pub reactive_mode: ReactiveComputeMode,
    pub target_frame_budget_ms: f64,
    pub auto_throttle_enabled: bool,
    pub theme: crate::ui::ThemeKind,
    pub workspace_preset: crate::ui::WorkspaceLayoutPreset,
    pub show_cell_line_numbers: bool,
}

pub fn default_logging_level() -> String {
    if cfg!(test) {
        "warn".to_string()
    } else if cfg!(debug_assertions) {
        "debug".to_string()
    } else {
        "error".to_string()
    }
}

impl Default for NotebookSettings {
    fn default() -> Self {
        Self {
            auto_sliders: true,
            auto_plots: true,
            show_hover_tooltips: true,
            show_derivatives: true,
            default_range_min: -5.0,
            default_range_max: 5.0,
            font_size: 15.0,
            logging_level: default_logging_level(),
            ai_config: urae::agent::AiConfig::default(),
            reactive_mode: ReactiveComputeMode::default(),
            target_frame_budget_ms: 8.0,
            auto_throttle_enabled: true,
            theme: crate::ui::ThemeKind::default(),
            workspace_preset: crate::ui::WorkspaceLayoutPreset::default(),
            show_cell_line_numbers: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionData {
    pub raw_document_text: String,
    pub slider_values: HashMap<String, f64>,
    pub symbol_metadata: HashMap<String, SymbolMetadata>,
    pub card_modes: HashMap<String, CardDisplayMode>,
    pub explicit_plots: HashSet<String>,
    pub settings: NotebookSettings,
}

/// Export `SessionData` to zlib-compressed BSON binary blob
pub fn export_session_to_compressed_bson(session: &SessionData) -> Result<Vec<u8>, String> {
    let raw_bson =
        bson::serialize_to_vec(session).map_err(|e| format!("BSON serialization error: {}", e))?;
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&raw_bson, 6);
    Ok(compressed)
}

/// Import `SessionData` from compressed or uncompressed BSON binary blob
pub fn import_session_from_compressed_bson(bytes: &[u8]) -> Result<SessionData, String> {
    let decompressed = match miniz_oxide::inflate::decompress_to_vec_zlib(bytes) {
        Ok(decomp) => decomp,
        Err(_) => bytes.to_vec(),
    };
    bson::deserialize_from_slice::<SessionData>(&decompressed)
        .map_err(|e| format!("BSON deserialization error: {}", e))
}

impl Default for SessionData {
    fn default() -> Self {
        let mut metadata = HashMap::new();
        metadata.insert(
            "a".to_string(),
            SymbolMetadata::new("a", 2.0, SymbolRole::Parameter),
        );
        metadata.insert(
            "c".to_string(),
            SymbolMetadata::new("c", -3.0, SymbolRole::Parameter),
        );
        metadata.insert(
            "x".to_string(),
            SymbolMetadata::new("x", 0.0, SymbolRole::Variable),
        );

        let mut slider_values = HashMap::new();
        slider_values.insert("a".to_string(), 2.0);
        slider_values.insert("c".to_string(), -3.0);

        Self {
            raw_document_text: [
                "# URAE Mathematics Notepad",
                "// Declare variables, parameters, set builder domains, and physical units below.",
                "",
                "a: Parameter = 2.00 [m]",
                "c: Parameter = -3.00 [m]",
                "x: Variable",
                "{ x in Reals | -5 <= x <= 5 }",
                "{ q in Quaternion | norm(q) == 1 }",
                "{ v in Grassmann | v^2 == 0 }",
                "f(x) = a * x^2 + c",
                "",
                "g = 9.81 [m/s^2]",
                "y = 1.50 * cos(2.00 * x)",
                "g(x) = x^3 - 3 * x + 2",
            ]
            .join("\n"),
            slider_values,
            symbol_metadata: metadata,
            card_modes: HashMap::new(),
            explicit_plots: HashSet::new(),
            settings: NotebookSettings::default(),
        }
    }
}

impl SessionData {
    pub fn load_from_disk() -> Option<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if std::env::var("URAE_DISABLE_SESSION_FILE").is_ok()
                || std::env::var("CARGO_MANIFEST_DIR").is_ok()
            {
                return None;
            }
            if let Ok(content) = std::fs::read_to_string(SESSION_FILE_NAME) {
                if let Ok(data) = serde_json::from_str::<Self>(&content) {
                    return Some(data);
                }
            }
        }
        None
    }

    pub fn save_to_disk(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if std::env::var("URAE_DISABLE_SESSION_FILE").is_ok()
                || std::env::var("CARGO_MANIFEST_DIR").is_ok()
            {
                return;
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(SESSION_FILE_NAME, json);
            }
        }
    }

    /// Export the notebook session as clean GitHub-Flavored Markdown.
    pub fn export_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("# URAE Mathematics Notepad\n\n");
        md.push_str("```urae\n");
        md.push_str(&self.raw_document_text);
        md.push_str("\n```\n");
        md
    }
}
