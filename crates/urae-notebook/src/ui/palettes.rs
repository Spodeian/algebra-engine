//! # `urae_notebook::ui::palettes`
//!
//! Interactive Builder Palettes & Wizards (Matrix Builder, ODE Wizard, Domain Picker, Units, CAD Machinery).

use crate::notebook::{
    generate_branch_cut_syntax, generate_interval_syntax, generate_matrix_syntax,
    generate_ode_bc_syntax, generate_physical_unit_syntax, MatrixPresetKind, NotebookState,
};
use eframe::egui;

/// Interactive Palettes and Wizard Manager.
pub struct PalettesUi;

impl PalettesUi {
    /// Render the Matrix Builder Palette window.
    pub fn render_matrix_builder(
        ctx: &egui::Context,
        is_open: &mut bool,
        rows: &mut usize,
        cols: &mut usize,
        preset: &mut MatrixPresetKind,
        elements: &mut Vec<Vec<String>>,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("Matrix Builder Palette")
            .collapsible(true)
            .resizable(true)
            .default_size([400.0, 320.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Dimensions:");
                    let mut r = *rows;
                    let mut c = *cols;
                    if ui
                        .add(egui::DragValue::new(&mut r).range(1..=10).prefix("Rows: "))
                        .changed()
                    {
                        *rows = r;
                    }
                    if ui
                        .add(egui::DragValue::new(&mut c).range(1..=10).prefix("Cols: "))
                        .changed()
                    {
                        *cols = c;
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Preset:");
                    egui::ComboBox::from_id_salt("matrix_preset_combo")
                        .selected_text(match preset {
                            MatrixPresetKind::Custom => "Custom",
                            MatrixPresetKind::Identity => "Identity",
                            MatrixPresetKind::Zero => "Zero",
                            MatrixPresetKind::Diagonal => "Diagonal",
                            MatrixPresetKind::Symmetric => "Symmetric",
                            MatrixPresetKind::PauliX => "Pauli-X",
                            MatrixPresetKind::PauliY => "Pauli-Y",
                            MatrixPresetKind::PauliZ => "Pauli-Z",
                        })
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_value(preset, MatrixPresetKind::Custom, "Custom");
                            let _ =
                                ui.selectable_value(preset, MatrixPresetKind::Identity, "Identity");
                            let _ = ui.selectable_value(preset, MatrixPresetKind::Zero, "Zero");
                            let _ =
                                ui.selectable_value(preset, MatrixPresetKind::Diagonal, "Diagonal");
                            let _ = ui.selectable_value(
                                preset,
                                MatrixPresetKind::Symmetric,
                                "Symmetric",
                            );
                            let _ =
                                ui.selectable_value(preset, MatrixPresetKind::PauliX, "Pauli-X");
                            let _ =
                                ui.selectable_value(preset, MatrixPresetKind::PauliY, "Pauli-Y");
                            let _ =
                                ui.selectable_value(preset, MatrixPresetKind::PauliZ, "Pauli-Z");
                        });
                });

                // Ensure element grid size matches rows x cols
                if elements.len() != *rows {
                    elements.resize(*rows, vec!["0".to_string(); *cols]);
                }
                for row_vec in elements.iter_mut() {
                    if row_vec.len() != *cols {
                        row_vec.resize(*cols, "0".to_string());
                    }
                }

                ui.separator();
                ui.label("Matrix Entries:");
                egui::Grid::new("matrix_grid").show(ui, |ui| {
                    for row in elements.iter_mut().take(*rows) {
                        for elem in row.iter_mut().take(*cols) {
                            ui.add(egui::TextEdit::singleline(elem).desired_width(45.0));
                        }
                        ui.end_row();
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert into Notepad").clicked() {
                        let syntax = generate_matrix_syntax(*rows, *cols, *preset, Some(elements));
                        state.insert_text_at_active_line(&syntax);
                        close_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });

        if close_requested {
            *is_open = false;
        }
    }

    /// Render the CAD & Machinery Generator Palette window.
    pub fn render_cad_machinery_palette(
        ctx: &egui::Context,
        is_open: &mut bool,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("Parametric CAD Machinery Builder")
            .collapsible(true)
            .resizable(true)
            .default_size([440.0, 360.0])
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Parametric 3D Engineering Shapes:").strong().color(egui::Color32::from_rgb(56, 189, 248)));
                ui.separator();

                // 1. Involute Gear
                ui.group(|ui| {
                    ui.label(egui::RichText::new("⚙ Involute Spur / Helical Gear").strong());
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Insert 16-Tooth Spur Gear").clicked() {
                            state.insert_text_at_active_line("gear!(teeth = 16, module = 2.0, pressure_angle = 20.0, face_width = 8.0)");
                            close_requested = true;
                        }
                        if ui.button("Insert 24-Tooth Helical Gear (15°)").clicked() {
                            state.insert_text_at_active_line("gear!(teeth = 24, module = 2.5, pressure_angle = 20.0, face_width = 12.0, helix_angle = 15.0)");
                            close_requested = true;
                        }
                    });
                });

                ui.add_space(4.0);

                // 2. Threaded Bolt / Screw
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🔩 Threaded Metric Fastener / Screw").strong());
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Insert M8 Hex Bolt").clicked() {
                            state.insert_text_at_active_line("screw!(dia = 8.0, pitch = 1.25, len = 25.0, head = Hex)");
                            close_requested = true;
                        }
                        if ui.button("Insert M6 Socket Cap Screw").clicked() {
                            state.insert_text_at_active_line("screw!(dia = 6.0, pitch = 1.0, len = 16.0, head = SocketCap)");
                            close_requested = true;
                        }
                    });
                });

                ui.add_space(4.0);

                // 3. NACA Wing Airfoil
                ui.group(|ui| {
                    ui.label(egui::RichText::new("✈ NACA 4-Digit Airfoil & 3D Wing").strong());
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Insert NACA 2412 Wing").clicked() {
                            state.insert_text_at_active_line("airfoil!(code = \"2412\", chord = 10.0, span = 30.0)");
                            close_requested = true;
                        }
                        if ui.button("Insert Symmetrical NACA 0012 Wing").clicked() {
                            state.insert_text_at_active_line("airfoil!(code = \"0012\", chord = 12.0, span = 40.0)");
                            close_requested = true;
                        }
                    });
                });

                ui.add_space(4.0);

                // 4. Helical Spring
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🌀 Helical Coil Spring").strong());
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Insert Compression Spring").clicked() {
                            state.insert_text_at_active_line("spring!(mean_dia = 10.0, wire_dia = 1.5, pitch = 4.0, coils = 8)");
                            close_requested = true;
                        }
                    });
                });

                ui.separator();
                if ui.button("Close").clicked() {
                    close_requested = true;
                }
            });

        if close_requested {
            *is_open = false;
        }
    }

    /// Render the ODE Initial Value Problem Wizard window.
    #[allow(clippy::too_many_arguments)]
    pub fn render_ode_wizard(
        ctx: &egui::Context,
        is_open: &mut bool,
        var: &mut String,
        indep: &mut String,
        y0: &mut f64,
        dy0: &mut Option<f64>,
        has_deriv: &mut bool,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("ODE Initial Value Problem Wizard")
            .collapsible(true)
            .resizable(false)
            .default_size([360.0, 240.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Dependent Variable:");
                    ui.text_edit_singleline(var);
                });
                ui.horizontal(|ui| {
                    ui.label("Independent Variable:");
                    ui.text_edit_singleline(indep);
                });
                ui.horizontal(|ui| {
                    ui.label(format!("{}(0) =", var));
                    ui.add(egui::DragValue::new(y0));
                });
                ui.checkbox(has_deriv, "2nd-Order Initial Derivative Condition (y'(0))");
                if *has_deriv {
                    let mut dy_val = dy0.unwrap_or(0.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("{}'(0) =", var));
                        if ui.add(egui::DragValue::new(&mut dy_val)).changed() {
                            *dy0 = Some(dy_val);
                        }
                    });
                } else {
                    *dy0 = None;
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert Condition into Notepad").clicked() {
                        let syntax = generate_ode_bc_syntax(var, indep, *y0, *dy0);
                        state.insert_text_at_active_line(&syntax);
                        close_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });

        if close_requested {
            *is_open = false;
        }
    }

    /// Render the Domain & Interval Picker Palette.
    #[allow(clippy::too_many_arguments)]
    pub fn render_domain_picker(
        ctx: &egui::Context,
        is_open: &mut bool,
        var: &mut String,
        domain: &mut String,
        min_val: &mut f64,
        max_val: &mut f64,
        inc_min: &mut bool,
        inc_max: &mut bool,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("Domain & Interval Picker")
            .collapsible(true)
            .resizable(false)
            .default_size([340.0, 220.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Variable:");
                    ui.text_edit_singleline(var);
                    ui.label("Domain:");
                    ui.text_edit_singleline(domain);
                });

                ui.horizontal(|ui| {
                    ui.label("Bounds:");
                    ui.add(egui::DragValue::new(min_val).prefix("Min: "));
                    ui.add(egui::DragValue::new(max_val).prefix("Max: "));
                });

                ui.horizontal(|ui| {
                    ui.checkbox(inc_min, "Include Min (<=)");
                    ui.checkbox(inc_max, "Include Max (<=)");
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert Interval Syntax").clicked() {
                        let syntax = generate_interval_syntax(
                            var, domain, *min_val, *max_val, *inc_min, *inc_max,
                        );
                        state.insert_text_at_active_line(&syntax);
                        close_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });

        if close_requested {
            *is_open = false;
        }
    }

    /// Render the Physical Units Palette.
    pub fn render_units_palette(
        ctx: &egui::Context,
        is_open: &mut bool,
        var: &mut String,
        val: &mut f64,
        unit: &mut String,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("Physical Units Palette")
            .collapsible(true)
            .resizable(false)
            .default_size([320.0, 200.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Variable:");
                    ui.text_edit_singleline(var);
                    ui.label("Value:");
                    ui.add(egui::DragValue::new(val));
                });

                ui.horizontal(|ui| {
                    ui.label("Unit:");
                    egui::ComboBox::from_id_salt("unit_combo")
                        .selected_text(unit.as_str())
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_value(unit, "m".to_string(), "m (meters)");
                            let _ = ui.selectable_value(unit, "s".to_string(), "s (seconds)");
                            let _ = ui.selectable_value(unit, "kg".to_string(), "kg (kilograms)");
                            let _ = ui.selectable_value(unit, "m/s".to_string(), "m/s (velocity)");
                            let _ = ui.selectable_value(
                                unit,
                                "m/s^2".to_string(),
                                "m/s² (acceleration)",
                            );
                            let _ = ui.selectable_value(unit, "N".to_string(), "N (force)");
                            let _ = ui.selectable_value(unit, "J".to_string(), "J (energy)");
                            let _ = ui.selectable_value(
                                unit,
                                "Pa".to_string(),
                                "Pa (pressure / stress)",
                            );
                            let _ = ui.selectable_value(
                                unit,
                                "W/(m*K)".to_string(),
                                "W/(m·K) (thermal conductivity)",
                            );
                        });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert Unit Syntax").clicked() {
                        let syntax = generate_physical_unit_syntax(var, *val, unit);
                        state.insert_text_at_active_line(&syntax);
                        close_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });

        if close_requested {
            *is_open = false;
        }
    }

    /// Render the Branch Cut & Riemann Sheet Wizard.
    pub fn render_branch_cut_wizard(
        ctx: &egui::Context,
        is_open: &mut bool,
        fn_name: &mut String,
        sheet: &mut i32,
        pos: &mut String,
        state: &mut NotebookState,
    ) {
        if !*is_open {
            return;
        }

        let mut close_requested = false;
        egui::Window::new("Branch Cut & Riemann Sheet Wizard")
            .collapsible(true)
            .resizable(false)
            .default_size([320.0, 200.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Function:");
                    egui::ComboBox::from_id_salt("branch_fn_combo")
                        .selected_text(fn_name.as_str())
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_value(fn_name, "sqrt".to_string(), "sqrt(z)");
                            let _ = ui.selectable_value(fn_name, "log".to_string(), "log(z)");
                            let _ = ui.selectable_value(fn_name, "asin".to_string(), "asin(z)");
                            let _ = ui.selectable_value(fn_name, "atan".to_string(), "atan(z)");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Riemann Sheet Index:");
                    ui.add(egui::DragValue::new(sheet).range(-5..=5));
                });

                ui.horizontal(|ui| {
                    ui.label("Cut Topology:");
                    egui::ComboBox::from_id_salt("branch_pos_combo")
                        .selected_text(pos.as_str())
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_value(
                                pos,
                                "NegativeReal".to_string(),
                                "Negative Real Axis (-inf, 0]",
                            );
                            let _ = ui.selectable_value(
                                pos,
                                "PositiveReal".to_string(),
                                "Positive Real Axis [0, +inf)",
                            );
                            let _ = ui.selectable_value(
                                pos,
                                "PureImaginary".to_string(),
                                "Pure Imaginary Axis",
                            );
                        });
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Insert Branch Cut Syntax").clicked() {
                        let syntax = generate_branch_cut_syntax(fn_name, *sheet, pos);
                        state.insert_text_at_active_line(&syntax);
                        close_requested = true;
                    }
                    if ui.button("Close").clicked() {
                        close_requested = true;
                    }
                });
            });

        if close_requested {
            *is_open = false;
        }
    }
}
