//! Integration Tests for Phase 13: High-Efficiency CAD Infrastructure, BVH Spatial Acceleration,
//! and Complex Parametric Engineering Machinery Generators.

use algebra_engine::cad::{
    BoltHeadType, BvhTree, HelicalSpring, InvoluteGear, Mesh3D, NacaAirfoil, Ray3D, ThreadStandard,
    ThreadedScrew,
};
use algebra_engine::{airfoil, gear, screw, spring};

#[test]
fn test_bvh_spatial_tree_construction_and_ray_intersection() {
    // 1. Create a 3D Cube: 4.0 x 4.0 x 4.0 centered at origin [-2, 2]^3
    let cube = Mesh3D::cube(4.0, 4.0, 4.0);
    let bvh = BvhTree::build(&cube, 2);

    assert_eq!(bvh.triangles.len(), 12);
    assert_eq!(bvh.vertices.len(), 8);

    // 2. Cast ray from (0, 0, -10) along +Z axis towards front face (z = -2)
    let ray_front = Ray3D::new([0.0, 0.0, -10.0], [0.0, 0.0, 1.0]);
    let hit_front = bvh.intersect_ray(&ray_front);

    assert!(hit_front.is_some());
    let hit = hit_front.unwrap();
    assert!((hit.distance - 8.0).abs() < 1e-6); // Hit at z = -2 (distance = 10 - 2 = 8)
    assert!((hit.point[0] - 0.0).abs() < 1e-6);
    assert!((hit.point[1] - 0.0).abs() < 1e-6);
    assert!((hit.point[2] - (-2.0)).abs() < 1e-6);

    // 3. Cast ray that misses the box completely
    let ray_miss = Ray3D::new([10.0, 10.0, -10.0], [0.0, 0.0, 1.0]);
    let hit_miss = bvh.intersect_ray(&ray_miss);
    assert!(hit_miss.is_none());
}

#[test]
fn test_bvh_point_in_mesh_parity_test() {
    let cube = Mesh3D::cube(10.0, 10.0, 10.0);
    let bvh = BvhTree::build(&cube, 2);

    // Center point (0, 0, 0) should be strictly inside
    assert!(bvh.contains_point(&[0.0, 0.0, 0.0]));
    assert!(bvh.contains_point(&[2.0, -3.0, 4.0]));

    // Exterior points outside [-5, 5]^3
    assert!(!bvh.contains_point(&[15.0, 0.0, 0.0]));
    assert!(!bvh.contains_point(&[0.0, -20.0, 0.0]));
    assert!(!bvh.contains_point(&[6.0, 6.0, 6.0]));
}

#[test]
fn test_involute_gear_spur_and_helical() {
    // Standard spur gear: module 2.0, 20 teeth, 10mm face width
    let spur = InvoluteGear::spur(2.0, 20, 10.0);

    assert_eq!(spur.pitch_diameter(), 40.0);
    assert!((spur.base_diameter() - 40.0 * (20.0f64.to_radians()).cos()).abs() < 1e-6);
    assert_eq!(spur.tip_diameter(), 44.0);
    assert_eq!(spur.root_diameter(), 35.0);

    let profile = spur.generate_2d_profile(6);
    assert!(profile.len() >= 20 * 4);

    let mesh_3d = spur.generate_3d_mesh();
    assert!(mesh_3d.vertices.len() > 100);
    assert!(mesh_3d.triangles.len() > 100);
    let vol = mesh_3d.compute_volume();
    assert!(vol > 1000.0); // Positive solid volume

    // Helical gear with 25 degree helix angle
    let helical = InvoluteGear::helical(2.5, 24, 15.0, 25.0);
    assert_eq!(helical.helix_angle_deg, 25.0);
    let mesh_helical = helical.generate_3d_mesh();
    assert!(mesh_helical.compute_volume() > 2000.0);
}

#[test]
fn test_threaded_fastener_generation() {
    // 1. ISO Metric Bolt M10 x 1.5, length 30mm
    let bolt = ThreadedScrew::metric_bolt(10.0, 1.5, 30.0);
    assert_eq!(bolt.nominal_diameter, 10.0);
    assert_eq!(bolt.pitch, 1.5);
    assert_eq!(bolt.head_type, BoltHeadType::Hexagonal);

    let bolt_mesh = bolt.generate_3d_mesh();
    assert!(bolt_mesh.vertices.len() > 100);
    assert!(bolt_mesh.triangles.len() > 100);

    // 2. Socket Cap Screw M6 x 1.0, length 20mm
    let socket = ThreadedScrew::socket_screw(6.0, 1.0, 20.0);
    assert_eq!(socket.head_type, BoltHeadType::SocketCap);
    let socket_mesh = socket.generate_3d_mesh();
    assert!(socket_mesh.triangles.len() > 50);

    // 3. Acme Lead Screw
    let lead = ThreadedScrew::lead_screw(12.0, 3.0, 100.0);
    assert_eq!(lead.thread_standard, ThreadStandard::Acme29);
}

#[test]
fn test_naca_airfoil_2412_and_3d_wing() {
    // NACA 2412: 2% camber at 40% chord, 12% thickness
    let airfoil = NacaAirfoil::new("2412", 100.0, 250.0);

    // Thickness at 30% chord
    let yt = airfoil.thickness_at(0.3, 0.12);
    assert!(yt > 0.05 && yt < 0.07);

    // 2D boundary
    let coords = airfoil.generate_2d_coordinates(20);
    assert!(coords.len() >= 40);

    // 3D wing mesh with 5 degree twist
    let mut twisted_wing = airfoil;
    twisted_wing.twist_deg = 5.0;
    let wing_mesh = twisted_wing.generate_3d_mesh();
    assert!(wing_mesh.vertices.len() >= 40 * 8);
    assert!(wing_mesh.triangles.len() > 200);
}

#[test]
fn test_helical_spring_geometry() {
    // Wire dia 2.0mm, mean dia 20.0mm, pitch 5.0mm, 6 active coils
    let spring = HelicalSpring::new(2.0, 20.0, 5.0, 6.0);
    assert_eq!(spring.free_length(), 30.0);

    let spring_mesh = spring.generate_3d_mesh();
    assert!(spring_mesh.vertices.len() > 500);
    assert!(spring_mesh.triangles.len() > 1000);
}

#[test]
fn test_engineering_dsl_macros() {
    // gear! macro
    let g1 = gear!(module = 2.0, teeth = 18, width = 12.0);
    assert_eq!(g1.pitch_diameter(), 36.0);

    let g2 = gear!(module = 3.0, teeth = 30, width = 20.0, helix = 15.0);
    assert_eq!(g2.helix_angle_deg, 15.0);

    // screw! macro
    let s1 = screw!(dia = 8.0, pitch = 1.25, length = 35.0);
    assert_eq!(s1.nominal_diameter, 8.0);

    let s2 = screw!(socket, dia = 10.0, pitch = 1.5, length = 40.0);
    assert_eq!(s2.head_type, BoltHeadType::SocketCap);

    // airfoil! macro
    let af = airfoil!(code = "0012", chord = 150.0, span = 400.0, twist = 3.0);
    assert_eq!(af.code, "0012");
    assert_eq!(af.twist_deg, 3.0);

    // spring! macro
    let sp = spring!(wire = 1.5, mean_dia = 16.0, pitch = 4.0, coils = 8.0);
    assert_eq!(sp.free_length(), 32.0);
}
