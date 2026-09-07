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
    /// Pending export request (if user clicked export in 3D toolbar).
    pub export_request: Option<Export3DRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Export3DFormat {
    StlBinary,
    StlAscii,
    Obj,
    Step,
}

impl Export3DFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::StlBinary | Self::StlAscii => "stl",
            Self::Obj => "obj",
            Self::Step => "step",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::StlBinary => "application/octet-stream",
            Self::StlAscii | Self::Obj | Self::Step => "text/plain",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Export3DRequest {
    pub format: Export3DFormat,
    pub filename: String,
    pub data: Vec<u8>,
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
    SpurGear,
    HelicalGear,
    BevelGear,
    WormGear,
    RackPinion,
    PlanetaryGear,
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
            export_request: None,
        }
    }
}

pub type CustomMeshGeometry<'a> = (&'a [[f64; 3]], &'a [[usize; 3]], Option<&'a [f64]>);

pub struct Viewport3D;

impl Viewport3D {
    /// Render an interactive 3D viewport canvas inside `ui` with standard 320px height.
    pub fn show(
        ui: &mut egui::Ui,
        state: &mut Viewport3DState,
        custom_mesh: Option<CustomMeshGeometry>,
    ) {
        Self::show_with_height(ui, state, custom_mesh, 320.0);
    }

    /// Render an interactive 3D viewport canvas inside `ui` with custom height (e.g. for inline results cards).
    pub fn show_with_height(
        ui: &mut egui::Ui,
        state: &mut Viewport3DState,
        custom_mesh: Option<CustomMeshGeometry>,
        canvas_height: f32,
    ) {
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
                        ViewportModelPreset::SpurGear => "Spur Gear (20°)",
                        ViewportModelPreset::HelicalGear => "Helical Gear (15°)",
                        ViewportModelPreset::BevelGear => "Bevel Gear (45°)",
                        ViewportModelPreset::WormGear => "Worm Gear & Screw",
                        ViewportModelPreset::RackPinion => "Rack and Pinion",
                        ViewportModelPreset::PlanetaryGear => "Planetary Gear Set",
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
                            ViewportModelPreset::SpurGear,
                            "Spur Gear (20°)",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::HelicalGear,
                            "Helical Gear (15°)",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::BevelGear,
                            "Bevel Gear (45°)",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::WormGear,
                            "Worm Gear & Screw",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::RackPinion,
                            "Rack and Pinion",
                        );
                        let _ = ui.selectable_value(
                            &mut state.active_model,
                            ViewportModelPreset::PlanetaryGear,
                            "Planetary Gear Set",
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

                ui.menu_button("📥 Export 3D", |ui| {
                    let model_name = match state.active_model {
                        ViewportModelPreset::InvoluteGear | ViewportModelPreset::SpurGear => "spur_gear",
                        ViewportModelPreset::HelicalGear => "helical_gear",
                        ViewportModelPreset::BevelGear => "bevel_gear",
                        ViewportModelPreset::WormGear => "worm_gear",
                        ViewportModelPreset::RackPinion => "rack_and_pinion",
                        ViewportModelPreset::PlanetaryGear => "planetary_gear_set",
                        ViewportModelPreset::ThreadedBolt => "threaded_bolt",
                        ViewportModelPreset::NacaWing => "naca_wing",
                        ViewportModelPreset::HelicalSpring => "helical_spring",
                        ViewportModelPreset::RiemannSqrt => "riemann_sqrt",
                        ViewportModelPreset::RiemannLog => "riemann_log",
                        ViewportModelPreset::UnitCube => "unit_cube",
                    };

                    if ui.button("STL (Binary - 3D Printing)").clicked() {
                        let data = export_stl_binary(&verts, &tris);
                        state.export_request = Some(Export3DRequest {
                            format: Export3DFormat::StlBinary,
                            filename: format!("{}.stl", model_name),
                            data,
                        });
                        ui.close();
                    }
                    if ui.button("STL (ASCII)").clicked() {
                        let data = export_stl_ascii(&verts, &tris, model_name).into_bytes();
                        state.export_request = Some(Export3DRequest {
                            format: Export3DFormat::StlAscii,
                            filename: format!("{}.stl", model_name),
                            data,
                        });
                        ui.close();
                    }
                    if ui.button("OBJ (Wavefront Mesh)").clicked() {
                        let data = export_obj(&verts, &tris, model_name).into_bytes();
                        state.export_request = Some(Export3DRequest {
                            format: Export3DFormat::Obj,
                            filename: format!("{}.obj", model_name),
                            data,
                        });
                        ui.close();
                    }
                    if ui.button("STEP (ISO 10303-21 CAD)").clicked() {
                        let data = export_step(&verts, &tris, model_name).into_bytes();
                        state.export_request = Some(Export3DRequest {
                            format: Export3DFormat::Step,
                            filename: format!("{}.step", model_name),
                            data,
                        });
                        ui.close();
                    }
                });
            });

            ui.add_space(4.0);

            // Canvas drawing area
            let canvas_size = egui::vec2(ui.available_width().max(160.0), canvas_height);
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
                let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                if scroll != 0.0 {
                    state.zoom = (state.zoom * (1.0 + scroll as f64 * 0.002)).clamp(1.0, 200.0);
                }
            }

            // Draw dark background
            painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(15, 23, 42));
            painter.rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(51, 65, 85)),
                egui::StrokeKind::Inside,
            );


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
                            0.5_f32,
                            egui::Color32::from_rgba_premultiplied(200, 240, 255, 60),
                        ),
                    );
                    painter.add(shape);
                } else {
                    painter.line_segment(
                        [p0, p1],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(56, 189, 248)),
                    );
                    painter.line_segment(
                        [p1, p2],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(56, 189, 248)),
                    );
                    painter.line_segment(
                        [p2, p0],
                        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(56, 189, 248)),
                    );
                }
            }
        });
    }

    fn get_preset_geometry(
        preset: ViewportModelPreset,
    ) -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<f64>) {
        match preset {
            ViewportModelPreset::InvoluteGear | ViewportModelPreset::SpurGear => {
                let gear = urae::cad::InvoluteGear::spur(1.5, 16, 6.0);
                let mesh = gear.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::HelicalGear => {
                let gear = urae::cad::InvoluteGear::helical(2.0, 18, 10.0, 20.0);
                let mesh = gear.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::BevelGear => {
                let gear = urae::cad::InvoluteGear::spur(1.5, 16, 8.0);
                let mut mesh = gear.generate_3d_mesh();
                for v in &mut mesh.vertices {
                    let factor = 1.0 - (v[2] / 12.0) * 0.45;
                    v[0] *= factor;
                    v[1] *= factor;
                }
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::WormGear => {
                let screw = urae::cad::ThreadedScrew::metric_bolt(10.0, 3.0, 20.0);
                let mesh = screw.generate_3d_mesh();
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::RackPinion => {
                let gear = urae::cad::InvoluteGear::spur(1.5, 14, 6.0);
                let mut mesh = gear.generate_3d_mesh();
                let base_y = -gear.tip_diameter() / 2.0 - 2.0;
                let rack_len = 48.0;
                let rack_teeth = 12;
                let dt = rack_len / (rack_teeth as f64);
                let v_start = mesh.vertices.len();
                for i in 0..rack_teeth {
                    let x0 = -rack_len / 2.0 + (i as f64) * dt;
                    let x1 = x0 + dt * 0.35;
                    let x2 = x0 + dt * 0.65;
                    let x3 = x0 + dt;
                    mesh.add_vertex([x0, base_y - 4.0, 0.0]);
                    mesh.add_vertex([x0, base_y, 0.0]);
                    mesh.add_vertex([x1, base_y + 2.5, 0.0]);
                    mesh.add_vertex([x2, base_y + 2.5, 0.0]);
                    mesh.add_vertex([x3, base_y, 0.0]);
                    mesh.add_vertex([x3, base_y - 4.0, 0.0]);
                }
                let n_pts = mesh.vertices.len() - v_start;
                for i in 0..n_pts {
                    let p = mesh.vertices[v_start + i];
                    mesh.add_vertex([p[0], p[1], 6.0]);
                }
                for i in 0..n_pts.saturating_sub(1) {
                    let v0 = v_start + i;
                    let v1 = v_start + i + 1;
                    let v2 = v_start + n_pts + i + 1;
                    let v3 = v_start + n_pts + i;
                    mesh.add_triangle(v0, v1, v2);
                    mesh.add_triangle(v0, v2, v3);
                }
                (mesh.vertices, mesh.triangles, Vec::new())
            }
            ViewportModelPreset::PlanetaryGear => {
                let sun = urae::cad::InvoluteGear::spur(1.2, 12, 5.0);
                let planet = urae::cad::InvoluteGear::spur(1.2, 8, 5.0);
                let mut combined_mesh = sun.generate_3d_mesh();
                let orbit_radius = (sun.pitch_diameter() + planet.pitch_diameter()) / 2.0;

                for planet_idx in 0..3 {
                    let theta = (planet_idx as f64) * (2.0 * PI / 3.0);
                    let ox = orbit_radius * theta.cos();
                    let oy = orbit_radius * theta.sin();
                    let mut p_mesh = planet.generate_3d_mesh();
                    let v_offset = combined_mesh.vertices.len();
                    for v in &mut p_mesh.vertices {
                        v[0] += ox;
                        v[1] += oy;
                        combined_mesh.add_vertex(*v);
                    }
                    for t in &p_mesh.triangles {
                        combined_mesh.add_triangle(t[0] + v_offset, t[1] + v_offset, t[2] + v_offset);
                    }
                }
                (combined_mesh.vertices, combined_mesh.triangles, Vec::new())
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

/// Serializes 3D triangular mesh into binary STL format (standard 80-byte header, 50 bytes per triangle).
pub fn export_stl_binary(verts: &[[f64; 3]], tris: &[[usize; 3]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(84 + tris.len() * 50);
    let header = b"URAE 3D CAD Binary STL - Universal Rust Algebra Engine";
    buf.extend_from_slice(header);
    buf.resize(80, 0);

    buf.extend_from_slice(&(tris.len() as u32).to_le_bytes());

    for tri in tris {
        let v0 = if tri[0] < verts.len() { verts[tri[0]] } else { [0.0; 3] };
        let v1 = if tri[1] < verts.len() { verts[tri[1]] } else { [0.0; 3] };
        let v2 = if tri[2] < verts.len() { verts[tri[2]] } else { [0.0; 3] };

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let mut nx = e1[1] * e2[2] - e1[2] * e2[1];
        let mut ny = e1[2] * e2[0] - e1[0] * e2[2];
        let mut nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-9 {
            nx /= len;
            ny /= len;
            nz /= len;
        } else {
            nx = 0.0;
            ny = 0.0;
            nz = 1.0;
        }

        buf.extend_from_slice(&(nx as f32).to_le_bytes());
        buf.extend_from_slice(&(ny as f32).to_le_bytes());
        buf.extend_from_slice(&(nz as f32).to_le_bytes());

        buf.extend_from_slice(&(v0[0] as f32).to_le_bytes());
        buf.extend_from_slice(&(v0[1] as f32).to_le_bytes());
        buf.extend_from_slice(&(v0[2] as f32).to_le_bytes());

        buf.extend_from_slice(&(v1[0] as f32).to_le_bytes());
        buf.extend_from_slice(&(v1[1] as f32).to_le_bytes());
        buf.extend_from_slice(&(v1[2] as f32).to_le_bytes());

        buf.extend_from_slice(&(v2[0] as f32).to_le_bytes());
        buf.extend_from_slice(&(v2[1] as f32).to_le_bytes());
        buf.extend_from_slice(&(v2[2] as f32).to_le_bytes());

        buf.extend_from_slice(&0u16.to_le_bytes());
    }

    buf
}

/// Serializes 3D triangular mesh into human-readable ASCII STL format.
pub fn export_stl_ascii(verts: &[[f64; 3]], tris: &[[usize; 3]], name: &str) -> String {
    let mut out = format!("solid {}\n", name);
    for tri in tris {
        let v0 = if tri[0] < verts.len() { verts[tri[0]] } else { [0.0; 3] };
        let v1 = if tri[1] < verts.len() { verts[tri[1]] } else { [0.0; 3] };
        let v2 = if tri[2] < verts.len() { verts[tri[2]] } else { [0.0; 3] };

        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let mut nx = e1[1] * e2[2] - e1[2] * e2[1];
        let mut ny = e1[2] * e2[0] - e1[0] * e2[2];
        let mut nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 1e-9 {
            nx /= len;
            ny /= len;
            nz /= len;
        } else {
            nx = 0.0;
            ny = 0.0;
            nz = 1.0;
        }

        out.push_str(&format!("  facet normal {:.6e} {:.6e} {:.6e}\n", nx, ny, nz));
        out.push_str("    outer loop\n");
        out.push_str(&format!("      vertex {:.6e} {:.6e} {:.6e}\n", v0[0], v0[1], v0[2]));
        out.push_str(&format!("      vertex {:.6e} {:.6e} {:.6e}\n", v1[0], v1[1], v1[2]));
        out.push_str(&format!("      vertex {:.6e} {:.6e} {:.6e}\n", v2[0], v2[1], v2[2]));
        out.push_str("    endloop\n");
        out.push_str("  endfacet\n");
    }
    out.push_str(&format!("endsolid {}\n", name));
    out
}

/// Serializes 3D triangular mesh into Wavefront OBJ format.
pub fn export_obj(verts: &[[f64; 3]], tris: &[[usize; 3]], name: &str) -> String {
    let mut out = format!("# Wavefront OBJ exported by URAE Notebook\n# Model: {}\n\no {}\n", name, name);
    for v in verts {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", v[0], v[1], v[2]));
    }
    for t in tris {
        out.push_str(&format!("f {} {} {}\n", t[0] + 1, t[1] + 1, t[2] + 1));
    }
    out
}

/// Serializes 3D triangular mesh into ISO 10303-21 STEP Brep faceted representation.
pub fn export_step(verts: &[[f64; 3]], tris: &[[usize; 3]], name: &str) -> String {
    let mut out = String::new();
    out.push_str("ISO-10303-21;\nHEADER;\n");
    out.push_str("FILE_DESCRIPTION(('URAE 3D CAD Tessellated Facet Export'),'2;1');\n");
    out.push_str(&format!("FILE_NAME('{}.step','2026-09-03',('Liam'),('URAE'),'URAE Step Serializer','URAE {}','');\n", name, env!("CARGO_PKG_VERSION")));
    out.push_str("FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));\n");
    out.push_str("ENDSEC;\nDATA;\n");
    out.push_str("#1 = APPLICATION_CONTEXT('core data for automotive design');\n");
    out.push_str("#2 = APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#1);\n");
    out.push_str("#3 = PRODUCT_CONTEXT('',#1,'mechanical');\n");
    out.push_str(&format!("#4 = PRODUCT('{name}','{name}','',(#3));\n"));
    out.push_str("#5 = PRODUCT_DEFINITION_FORMATION('','',#4);\n");
    out.push_str("#6 = PRODUCT_DEFINITION('design','',#5,#3);\n");
    out.push_str("#7 = PRODUCT_DEFINITION_SHAPE('','',#6);\n");
    out.push_str("#8 = GEOMETRIC_REPRESENTATION_CONTEXT(3);\n");

    let mut id = 10;
    let mut vert_ids = Vec::with_capacity(verts.len());
    for v in verts {
        out.push_str(&format!("#{id} = CARTESIAN_POINT('',({:.6},{:.6},{:.6}));\n", v[0], v[1], v[2]));
        vert_ids.push(id);
        id += 1;
    }

    let mut face_ids = Vec::with_capacity(tris.len());
    for t in tris {
        let p0 = if t[0] < vert_ids.len() { vert_ids[t[0]] } else { 10 };
        let p1 = if t[1] < vert_ids.len() { vert_ids[t[1]] } else { 10 };
        let p2 = if t[2] < vert_ids.len() { vert_ids[t[2]] } else { 10 };

        let loop_id = id;
        id += 1;
        out.push_str(&format!("#{loop_id} = POLY_LOOP('',(#{p0},#{p1},#{p2}));\n"));

        let face_bound_id = id;
        id += 1;
        out.push_str(&format!("#{face_bound_id} = FACE_OUTER_BOUND('',#{loop_id},.T.);\n"));

        let face_id = id;
        id += 1;
        out.push_str(&format!("#{face_id} = FACE_SURFACE('',(#{face_bound_id}),#{p0},.T.);\n"));
        face_ids.push(face_id);
    }

    let faces_str = face_ids.iter().map(|f| format!("#{f}")).collect::<Vec<_>>().join(",");
    let shell_id = id;
    id += 1;
    out.push_str(&format!("#{shell_id} = CLOSED_SHELL('',({faces_str}));\n"));

    let brep_id = id;
    id += 1;
    out.push_str(&format!("#{brep_id} = FACETED_BREP('{name}',#{shell_id});\n"));

    let shape_rep_id = id;
    id += 1;
    out.push_str(&format!("#{shape_rep_id} = MANIFOLD_SURFACE_SHAPE_REPRESENTATION('{name}',(#{brep_id}),#8);\n"));
    out.push_str(&format!("#{id} = SHAPE_DEFINITION_REPRESENTATION(#7,#{shape_rep_id});\n"));
    out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");

    out
}

