//! Integration Tests for Phase 23: Non-Euclidean Computational Geometry & Hyperbolic/Spherical Tessellations.

use algebra_engine::noneuclidean::{
    HyperbolicTessellation, HyperbolicTriangle, PoincareDiskPoint, SchwarzChristoffel,
    SphericalPoint, UpperHalfPlanePoint,
};
use algebra_engine::{
    hyperbolic_dist, hyperbolic_triangle_area, spherical_dist, spherical_triangle_area,
};

#[test]
fn test_poincare_disk_distance_and_mobius_invariance() {
    let u = PoincareDiskPoint::new(0.2, 0.3).unwrap();
    let v = PoincareDiskPoint::new(-0.4, 0.5).unwrap();

    let d_orig = u.distance(&v);
    assert!(d_orig > 0.0);

    // Pure hyperbolic translation: a = cosh(0.5), b = sinh(0.5)
    let theta = 0.5_f64;
    let a_re = theta.cosh();
    let b_re = theta.sinh();

    let u_trans = u.mobius_isometry(a_re, 0.0, b_re, 0.0);
    let v_trans = v.mobius_isometry(a_re, 0.0, b_re, 0.0);

    let d_trans = u_trans.distance(&v_trans);
    assert!((d_orig - d_trans).abs() < 1e-10);
}

#[test]
fn test_upper_half_plane_distance_and_cayley_isomorphism() {
    let z1 = UpperHalfPlanePoint::new(1.0, 2.0).unwrap();
    let z2 = UpperHalfPlanePoint::new(-0.5, 1.5).unwrap();

    let d_uhp = z1.distance(&z2);

    let w1 = z1.to_poincare();
    let w2 = z2.to_poincare();

    let d_poincare = w1.distance(&w2);
    assert!((d_uhp - d_poincare).abs() < 1e-10);

    // Inverse roundtrip
    let z1_rec = UpperHalfPlanePoint::from_poincare(&w1);
    assert!((z1.x - z1_rec.x).abs() < 1e-10);
    assert!((z1.y - z1_rec.y).abs() < 1e-10);
}

#[test]
fn test_spherical_geometry_distance_and_excess() {
    // North pole: (0, 0, 1) -> theta = 0
    let north_pole = SphericalPoint::from_cartesian(0.0, 0.0, 1.0);
    // Equator point: (1, 0, 0) -> theta = PI/2, phi = 0
    let equator_pt = SphericalPoint::from_cartesian(1.0, 0.0, 0.0);

    let dist = north_pole.great_circle_distance(&equator_pt);
    let pi_2 = std::f64::consts::FRAC_PI_2;
    assert!((dist - pi_2).abs() < 1e-10);

    // Tri-rectangular spherical triangle: angles are 90, 90, 90 deg = (pi/2, pi/2, pi/2)
    let area = SphericalPoint::triangle_excess_area(pi_2, pi_2, pi_2);
    // Excess: 3*(pi/2) - pi = pi/2
    assert!((area - pi_2).abs() < 1e-10);
}

#[test]
fn test_hyperbolic_triangle_defect() {
    let pi = std::f64::consts::PI;
    // Equilateral hyperbolic triangle with angles 45 deg = pi/4
    let alpha = pi / 4.0;
    let beta = pi / 4.0;
    let gamma = pi / 4.0;

    let area = HyperbolicTriangle::area_defect(alpha, beta, gamma);
    // Defect: pi - 3*(pi/4) = pi/4
    assert!((area - pi / 4.0).abs() < 1e-10);

    // Ideal triangle with all zero angles
    let ideal_area = HyperbolicTriangle::area_defect(0.0, 0.0, 0.0);
    assert!((ideal_area - pi).abs() < 1e-10);
}

#[test]
fn test_regular_hyperbolic_tessellation_p_q() {
    // {7, 3} heptagonal tiling (hyperbolic)
    let r_7_3 = HyperbolicTessellation::fundamental_domain_radius(7, 3);
    assert!(r_7_3.is_some());
    let r = r_7_3.unwrap();
    assert!(r > 0.0 && r < 1.0);

    let vertices = HyperbolicTessellation::generate_p_gon_vertices(7, 3).unwrap();
    assert_eq!(vertices.len(), 7);
    for v in &vertices {
        assert!((v.norm() - r).abs() < 1e-10);
    }

    // {5, 4} pentagonal tiling (hyperbolic)
    let r_5_4 = HyperbolicTessellation::fundamental_domain_radius(5, 4);
    assert!(r_5_4.is_some());

    // {4, 4} is Euclidean square grid -> None
    assert!(HyperbolicTessellation::fundamental_domain_radius(4, 4).is_none());
    // {3, 3} is spherical tetrahedron -> None
    assert!(HyperbolicTessellation::fundamental_domain_radius(3, 3).is_none());
}

#[test]
fn test_schwarz_christoffel_conformal_map() {
    let prevertices = [-1.0, 1.0];
    let angles = [std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2];

    let (re, im) = SchwarzChristoffel::integrate_map(&prevertices, &angles, 0.5, 0.5, 50);
    assert!(re.is_finite());
    assert!(im.is_finite());
}

#[test]
fn test_phase23_dsl_macros() {
    let p1 = PoincareDiskPoint::new(0.1, 0.2).unwrap();
    let p2 = PoincareDiskPoint::new(-0.2, 0.3).unwrap();
    let d_hyp = hyperbolic_dist!(&p1, &p2);
    assert!(d_hyp > 0.0);

    let s1 = SphericalPoint::from_cartesian(0.0, 0.0, 1.0);
    let s2 = SphericalPoint::from_cartesian(0.0, 1.0, 0.0);
    let d_sph = spherical_dist!(&s1, &s2);
    assert!((d_sph - std::f64::consts::FRAC_PI_2).abs() < 1e-10);

    let pi_2 = std::f64::consts::FRAC_PI_2;
    let hyp_area = hyperbolic_triangle_area!(0.5, 0.5, 0.5);
    assert!(hyp_area > 0.0);

    let sph_area = spherical_triangle_area!(pi_2, pi_2, pi_2);
    assert!((sph_area - pi_2).abs() < 1e-10);
}
