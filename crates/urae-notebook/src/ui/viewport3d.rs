//! # `urae_notebook::ui::viewport3d`
//!
//! Interactive 3D Perspective/Isometric Viewport Canvas for CAD Meshes, FEA Stress Contours & Riemann Surfaces.

use eframe::egui;
use std::f64::consts::PI;

/// 3D Camera & Viewport State.
#[derive(Debug, Clone)]
pub struct Viewport3DState {
    /// Camera rotation yaw in radians.
    pub yaw: f64,
    /// Camera rotation pitch in radians.
    pub pitch: f64,
    /// Camera zoom / scale factor.
    pub zoom: f64,
    /// Camera pan translation [x, y].
    pub pan: [f64; 2],
    /// Wireframe display mode.
    pub wireframe: bool,
    /// Color map mode: None (Single Color), StressContour, PhaseHue.
    pub color_mode: ViewportColorMode,
    /// Active 3D model preset for interactive demo.
    pub active_model: ViewportModelPreset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportColorMode {
    #[default]
    SolidBlue,
    WireframeOnly,
    StressContour,
    PhaseHue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportModelPreset {
    #[default]
    InvoluteGear,
    ThreadedBolt,
    NacaWing,
    HelicalSpring,
    RiemannSqrt,
    RiemannLog,
    UnitCube,
}

impl Default for Viewport3DState {
    fn default() -> Self {
        Self {
            yaw: PI / 4.0,
            pitch: PI / 6.0,
            zoom: 15.0,
            pan: [0.0, 0.0],
            wireframe: false,
            color_mode: ViewportColorMode::SolidBlue,
            active_model: ViewportModelPreset::InvoluteGear,
        }
    }
}

pub type CustomMeshGeometry<'a> = (&'a [[f64; 3]], &'a [[usize; 3]], Option<&'a [f64]>);

pub struct Viewport3D;

impl Viewport3D {
    /// Render an interactive 3D viewport canvas inside `ui`.
    pub fn show(
        ui: &mut egui::Ui,
        state: &mut Viewport3DState,
        custom_mesh: Option<CustomMeshGeometry>,
    ) {
        ui.vertical(|ui| {
            // Controls toolbar
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("3D Viewport:")
                        .strong()
                        .color(egui::Color32::from_rgb(56, 189, 248)),
                );

                egui::ComboBox::from_id_salt("3d_model_combo")
                    .selected_text(match state.active_model {
                        ViewportModelPreset::InvoluteGear => "Involute Gear",
                        ViewportModelPreset::ThreadedBolt => "Threaded Bolt",
                        ViewportModelPreset::NacaWing => "NACA Wing",
                        ViewportModelPreset::HelicalSpring => "Helical Spring",
                        ViewportModelPreset::RiemannSqrt => "Riemann Sqrt(z)",
                        ViewportModelPreset::RiemannLog => "Riemann Ln(z)",
                        ViewportModelPreset::UnitCube => "Unit Cube",
                    })
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::InvoluteGear,
                            "Involute Gear",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::ThreadedBolt,
                            "Threaded Bolt",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::NacaWing,
                            "NACA Wing",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::HelicalSpring,
                            "Helical Spring",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::RiemannSqrt,
                            "Riemann Sqrt(z)",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::RiemannLog,
                            "Riemann Ln(z)",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::UnitCube,
                            "Unit Cube",
                        );
                    });

                ui.checkbox(&mut state.wireframe, "Wireframe");

                if ui.button("Reset Camera").clicked() {
                    state.yaw = PI / 4.0;
                    state.pitch = PI / 6.0;
                    state.zoom = 15.0;
                    state.pan = [0.0, 0.0];
                }
            });

            ui.add_space(4.0);

            // Canvas drawing area
            let canvas_size = egui::vec2(ui.available_width().max(300.0), 320.0);
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::drag());

            let rect = response.rect;
            let center = rect.center();

            // Handle mouse dragging for camera orbit / pan
            if response.dragged() {
                let delta = response.drag_delta();
                if ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary)) {
                    // Pan
                    state.pan[0] += delta.x as f64;
                    state.pan[1] += delta.y as f64;
                } else {
                    // Orbit
                    state.yaw += delta.x as f64 * 0.01;
                    state.pitch = (state.pitch + delta.y as f64 * 0.01).clamp(-PI / 2.2, PI / 2.2);
                }
            }

            // Handle scroll zoom
            if response.hovered() {
                let scroll = ui.input(|i| i.raw_scroll_delta.y);
                if scroll != 0.0 {
                    state.zoom = (state.zoom * (1.0 + scroll as f64 * 0.002)).clamp(1.0, 200.0);
                }
            }

            // Draw dark background
            painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 65, 85)),
            );

            // Fetch or generate model geometry
            let (verts, tris, stresses): (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<f64>) =
                if let Some((v, t, s)) = custom_mesh {
                    (
                        v.to_vec(),
                        t.to_vec(),
                        s.map(|arr| arr.to_vec()).unwrap_or_default(),
                    )
                } else {
                    Self::get_preset_geometry(state.active_model)
                };

            // Project 3D vertices to 2D screen space
            let cos_y = state.yaw.cos();
            let sin_y = state.yaw.sin();
            let cos_p = state.pitch.cos();
            let sin_p = state.pitch.sin();

            let projected: Vec<(egui::Pos2, f64)> = verts
                .iter()
                .map(|&[x, y, z]| {
                    // 1. Yaw rotation (around Z axis)
                    let x1 = x * cos_y - y * sin_y;
                    let y1 = x * sin_y + y * cos_y;
                    let z1 = z;

                    // 2. Pitch rotation (around X axis)
                    let x2 = x1;
                    let y2 = y1 * cos_p - z1 * sin_p;
                    let z2 = y1 * sin_p + z1 * cos_p;

                    // 3. Screen projection
                    let screen_x = center.x + (x2 * state.zoom + state.pan[0]) as f32;
                    let screen_y = center.y - (z2 * state.zoom - state.pan[1]) as f32;

                    (egui::pos2(screen_x, screen_y), y2)
                })
                .collect();

            // Sort triangles by depth (painter's algorithm)
            let mut sorted_tris: Vec<(usize, [usize; 3], f64)> = tris
                .iter()
                .enumerate()
                .map(|(idx, &tri)| {
                    let avg_depth = if tri[0] < projected.len()
                        && tri[1] < projected.len()
                        && tri[2] < projected.len()
                    {
                        (projected[tri[0]].1 + projected[tri[1]].1 + projected[tri[2]].1) / 3.0
                    } else {
                        0.0
                    };
                    (idx, tri, avg_depth)
                })
                .collect();

            sorted_tris.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            // Render triangles
            for (idx, [i0, i1, i2], _) in sorted_tris {
                if i0 >= projected.len() || i1 >= projected.len() || i2 >= projected.len() {
                    continue;
                }
                let p0 = projected[i0].0;
                let p1 = projected[i1].0;
                let p2 = projected[i2].0;

                // Color calculation
                let fill_color = if !stresses.is_empty() && idx < stresses.len() {
                    let s_val = (stresses[idx] / 1000.0).clamp(0.0, 1.0) as f32;
                    egui::Color32::from_rgb(
                        (s_val * 255.0) as u8,
                        ((1.0 - (s_val - 0.5).abs() * 2.0).clamp(0.0, 1.0) * 200.0) as u8,
                        ((1.0 - s_val) * 255.0) as u8,
                    )
                } else {
                    egui::Color32::from_rgb(56, 189, 248)
                };

                if !state.wireframe {
                    let shape = egui::epaint::PathShape::convex_polygon(
                        vec![p0, p1, p2],
                        fill_color,
                        egui::Stroke::new(
                            0.5,
                            egui::Color32::from_rgba_premultiplied(200, 240, 255, 60),
                        ),
                    );
                    painter.add(shape);
                } else {
                    painter.line_segment(
                        [p0, p1],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 189, 248)),
                    );
                    painter.line_segment(
                        [p1, p2],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 189, 248)),
                    );
                    painter.line_segment(
                        [p2, p0],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 189, 248)),
                    );
                }
            }
        });
    }

    fn get_preset_geometry(
        preset: ViewportModelPreset,
    ) -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<f64>) {
        match preset {
            ViewportModelPreset::InvoluteGear => {
                let gear = urae::cad::InvoluteGear::spur(1.5, 12, 4.0);
                let mesh = gear.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::ThreadedBolt => {
                let screw = urae::cad::ThreadedScrew::metric_bolt(6.0, 1.0, 12.0);
                let mesh = screw.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::NacaWing => {
                let wing = urae::cad::NacaAirfoil::new("2412", 10.0, 15.0);
                let mesh = wing.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::HelicalSpring => {
                let spring = urae::cad::HelicalSpring::new(6.0, 1.0, 2.5, 5.0);
                let mesh = spring.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::RiemannSqrt => {
                let mesh = urae::visualizer::RiemannSurface::sqrt_surface(3.0, 8, 16);
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::RiemannLog => {
                let mesh = urae::visualizer::RiemannSurface::log_surface(3.0, 2, 8, 16);
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::UnitCube => {
                let mesh = urae::cad::Mesh3D::cube(4.0, 4.0, 4.0);
                (mesh.vertices, mesh.triangles, Vec::new())
            }
        }
    }
}
