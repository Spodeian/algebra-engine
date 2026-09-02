//! URAE Example: Parametric CAD & Isogeometric Analysis (IGA)
//! Run via: `cargo run --example cad_and_iga`

use urae::prelude::*;

fn main() {
    println!("=== URAE Parametric CAD & Isogeometric Analysis (IGA) ===");

    // 1. Parametric Involute Spur Gear
    let gear = gear!(module = 2.0, teeth = 18, width = 10.0);
    let gear_mesh = gear.generate_3d_mesh();
    println!(
        "Generated Involute Gear: {} vertices, {} triangles",
        gear_mesh.vertices.len(),
        gear_mesh.triangles.len()
    );

    // 2. ISO Metric Threaded Fastener
    let screw = screw!(dia = 10.0, pitch = 1.5, length = 25.0);
    let screw_mesh = screw.generate_3d_mesh();
    println!(
        "Generated ISO Bolt: {} vertices, {} triangles",
        screw_mesh.vertices.len(),
        screw_mesh.triangles.len()
    );

    // 3. Cox-de Boor B-Spline Basis & NURBS Patch
    let knots_u = vec![0.0, 0.0, 1.0, 1.0];
    let knots_v = vec![0.0, 0.0, 1.0, 1.0];
    let control_points = vec![
        WeightedControlPoint::new(vec![0.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![0.0, 2.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![2.0, 0.0, 0.0], 1.0),
        WeightedControlPoint::new(vec![2.0, 2.0, 0.0], 1.0),
    ];
    let patch = NurbsPatchND::surface_2d(1, 1, knots_u, knots_v, 2, 2, control_points);
    let pt = patch.evaluate_surface(0.5, 0.5);
    println!("Evaluated NURBS Surface Midpoint: {:?}", pt);

    // 4. IGA Stiffness Matrix Entry Quadrature
    let k_00 = iga_stiffness_entry_2d!(&patch, 0, 0);
    println!("IGA Stiffness Entry K_{{0,0}} = {}", k_00);
    println!("=== Execution Complete ===");
}
