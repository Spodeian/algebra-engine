//! # `urae_notebook::ui`
//!
//! Modular UI Subsystem for the URAE Continuous Mathematics & CAD Notepad.

pub mod command_palette;
pub mod palettes;
pub mod theme;
pub mod viewport3d;
pub mod workspace;

pub use command_palette::{CommandItem, CommandPalette, CommandPaletteState};
pub use palettes::PalettesUi;
pub use theme::{ThemeKind, ThemePalette};
pub use viewport3d::{Viewport3D, Viewport3DState, ViewportColorMode, ViewportModelPreset};
pub use workspace::{WorkspaceLayoutPreset, WorkspaceState, WorkspaceTab};
