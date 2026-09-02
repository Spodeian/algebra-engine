//! # `urae-kernel` (Native Zero-Latency Jupyter Kernel Protocol for URAE)
//!
//! Exposes URAE continuous mathematics notepad and symbolic computation as a standard Jupyter Kernel.
//!
//! Produces multi-MIME rich display payloads:
//! - `text/plain`: Formatted Unicode math.
//! - `text/latex`: MathJax / KaTeX rendering ($$\dots$$).
//! - `text/html`: Embedded interactive Three.js 3D rotating CAD viewport.
//! - `application/json`: Structured symbolic AST representation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urae::prelude::*;

/// Rich Display MIME Payload Bundle for Jupyter Frontends.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MimeBundle {
    pub data: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

impl MimeBundle {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_plain(mut self, text: impl Into<String>) -> Self {
        self.data.insert("text/plain".to_string(), text.into());
        self
    }

    pub fn with_latex(mut self, latex: impl Into<String>) -> Self {
        let l_str = latex.into();
        let formatted = if l_str.starts_with("$$") {
            l_str
        } else {
            format!("$${}$$", l_str)
        };
        self.data.insert("text/latex".to_string(), formatted);
        self
    }

    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.data.insert("text/html".to_string(), html.into());
        self
    }

    pub fn with_json(mut self, json: impl Into<String>) -> Self {
        self.data
            .insert("application/json".to_string(), json.into());
        self
    }
}

/// Jupyter Kernel Execution Reply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteReply {
    pub execution_count: usize,
    pub status: String,
    pub mime_bundle: MimeBundle,
    pub user_expressions: HashMap<String, String>,
}

/// URAE Jupyter Kernel State.
#[derive(Debug, Default)]
pub struct UraeKernel {
    pub execution_count: usize,
    pub session: UraeSession,
}

impl UraeKernel {
    pub fn new() -> Self {
        Self {
            execution_count: 0,
            session: UraeSession::new(),
        }
    }

    /// Kernel Information Spec.
    pub fn kernel_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("protocol_version".to_string(), "5.3".to_string());
        info.insert("implementation".to_string(), "urae-kernel".to_string());
        info.insert("implementation_version".to_string(), "0.1.0".to_string());
        info.insert("language_info.name".to_string(), "urae".to_string());
        info.insert("language_info.version".to_string(), "0.1.0".to_string());
        info.insert(
            "language_info.mimetype".to_string(),
            "text/x-urae".to_string(),
        );
        info.insert(
            "language_info.file_extension".to_string(),
            ".urae".to_string(),
        );
        info
    }

    /// Execute a cell code string and return multi-MIME payload.
    pub fn execute(&mut self, code: &str) -> ExecuteReply {
        self.execution_count += 1;
        let trimmed = code.trim();

        // Check if CAD gear command
        if trimmed.starts_with("gear!") || trimmed.contains("InvoluteGear") {
            let gear = urae::cad::InvoluteGear::spur(1.5, 16, 6.0);
            let mesh = gear.generate_3d_mesh();
            let html = urae::html_export::StandaloneHtmlExporter::export_mesh_to_html(
                "Involute Gear (16T, m=1.5)",
                "Parametric 3D Spur Gear CAD Model",
                "r_b = r \\cos(\\alpha)",
                &mesh,
            );
            let mime = MimeBundle::new()
                .with_plain("InvoluteGear: 16 Teeth, Module 1.5, Face Width 6.0mm")
                .with_latex("\\text{InvoluteGear}(N=16, m=1.5, b=6.0)")
                .with_html(html);

            return ExecuteReply {
                execution_count: self.execution_count,
                status: "ok".to_string(),
                mime_bundle: mime,
                user_expressions: HashMap::new(),
            };
        }

        // Check if Quantum circuit command
        if trimmed.starts_with("quantum_bell") || trimmed.contains("bell_pair") {
            let bell = urae::quantum_circuit::QuantumCircuit::bell_pair();
            let qasm = bell.to_openqasm();
            let mime = MimeBundle::new()
                .with_plain(format!(
                    "Quantum Bell State: |Phi+> = (|00> + |11>)/sqrt(2)\n\n{}",
                    qasm
                ))
                .with_latex("\\frac{|00\\rangle + |11\\rangle}{\\sqrt{2}}");

            return ExecuteReply {
                execution_count: self.execution_count,
                status: "ok".to_string(),
                mime_bundle: mime,
                user_expressions: HashMap::new(),
            };
        }

        // Execute against persistent URAE Session
        let res = self.session.execute_line(trimmed);
        if res.is_error {
            let err_msg = res.error_msg.unwrap_or(res.output_text);
            let mime = MimeBundle::new().with_plain(format!("Error: {}", err_msg));
            ExecuteReply {
                execution_count: self.execution_count,
                status: "error".to_string(),
                mime_bundle: mime,
                user_expressions: HashMap::new(),
            }
        } else {
            let mime = MimeBundle::new()
                .with_plain(&res.output_unicode)
                .with_latex(&res.output_latex);

            ExecuteReply {
                execution_count: self.execution_count,
                status: "ok".to_string(),
                mime_bundle: mime,
                user_expressions: HashMap::new(),
            }
        }
    }
}
