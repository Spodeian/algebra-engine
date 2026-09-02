//! # Workspace Management & Docking Layout Subsystem
//!
//! Provides a flexible multi-panel workspace supporting tabbed navigation, split views,
//! floating vs docked inspectors, and IDE-grade layout presets.

use serde::{Deserialize, Serialize};

/// Available tabs in the URAE workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceTab {
    MathDocument,
    Plot2D,
    Viewport3D,
    TerminalCLI,
    PalettesBuilder,
    ExampleGallery,
    StorageDiagnostics,
    Settings,
}

impl WorkspaceTab {
    pub fn title(&self) -> &'static str {
        match self {
            Self::MathDocument => "📝 Math Notepad",
            Self::Plot2D => "📈 2D Plots",
            Self::Viewport3D => "🧊 3D Viewport",
            Self::TerminalCLI => "💻 Terminal CLI",
            Self::PalettesBuilder => "🧮 Builder Wizards",
            Self::ExampleGallery => "📚 Example Gallery",
            Self::StorageDiagnostics => "💾 Storage & Sync",
            Self::Settings => "⚙️ Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::MathDocument => "📝",
            Self::Plot2D => "📈",
            Self::Viewport3D => "🧊",
            Self::TerminalCLI => "💻",
            Self::PalettesBuilder => "🧮",
            Self::ExampleGallery => "📚",
            Self::StorageDiagnostics => "💾",
            Self::Settings => "⚙️",
        }
    }
}

/// Workspace layout arrangement presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceLayoutPreset {
    #[default]
    FluidNotepad,
    SplitDual,
    TripleIDE,
    ZenMode,
}

impl WorkspaceLayoutPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::FluidNotepad => "Fluid Inline Notepad",
            Self::SplitDual => "Dual Split (Editor + Visualizer)",
            Self::TripleIDE => "Triple IDE (Editor + Plots + Terminal)",
            Self::ZenMode => "Zen Mode (Full Screen Canvas)",
        }
    }
}

/// State tracking the active workspace layout and docked panels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub active_tab: WorkspaceTab,
    pub secondary_tab: Option<WorkspaceTab>,
    pub layout_preset: WorkspaceLayoutPreset,
    pub split_ratio: f32,
    pub show_sidebar: bool,
    pub show_bottom_panel: bool,
    pub pinned_tabs: Vec<WorkspaceTab>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            active_tab: WorkspaceTab::MathDocument,
            secondary_tab: Some(WorkspaceTab::Plot2D),
            layout_preset: WorkspaceLayoutPreset::FluidNotepad,
            split_ratio: 0.55,
            show_sidebar: true,
            show_bottom_panel: false,
            pinned_tabs: vec![
                WorkspaceTab::MathDocument,
                WorkspaceTab::Plot2D,
                WorkspaceTab::Viewport3D,
                WorkspaceTab::TerminalCLI,
            ],
        }
    }
}

impl WorkspaceState {
    pub fn select_tab(&mut self, tab: WorkspaceTab) {
        self.active_tab = tab;
    }

    pub fn toggle_sidebar(&mut self) {
        self.show_sidebar = !self.show_sidebar;
    }

    pub fn toggle_bottom_panel(&mut self) {
        self.show_bottom_panel = !self.show_bottom_panel;
    }

    pub fn set_layout(&mut self, preset: WorkspaceLayoutPreset) {
        self.layout_preset = preset;
        match preset {
            WorkspaceLayoutPreset::FluidNotepad => {
                self.show_sidebar = true;
                self.show_bottom_panel = false;
                self.split_ratio = 0.60;
            }
            WorkspaceLayoutPreset::SplitDual => {
                self.show_sidebar = true;
                self.show_bottom_panel = false;
                self.split_ratio = 0.50;
            }
            WorkspaceLayoutPreset::TripleIDE => {
                self.show_sidebar = true;
                self.show_bottom_panel = true;
                self.split_ratio = 0.50;
            }
            WorkspaceLayoutPreset::ZenMode => {
                self.show_sidebar = false;
                self.show_bottom_panel = false;
                self.split_ratio = 1.0;
            }
        }
    }
}
