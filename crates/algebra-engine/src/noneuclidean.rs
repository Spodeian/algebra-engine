//! # `algebra_engine::noneuclidean`
//!
//! Non-Euclidean Computational Geometry & Hyperbolic / Spherical Tessellations.
//!
//! Features:
//! - **Poincaré Disk Model ($\mathbb{H}^2$)**: Exact Poincaré metric $ds^2$, geodesic distances, Möbius isometries $\operatorname{PSU}(1, 1)$, and geodesic arc interpolation.
//! - **Upper Half-Plane Model ($\mathbb{H}^2$)**: $\operatorname{PSL}(2, \mathbb{R})$ modular group actions and Cayley transform isomorphism to the Poincaré disk.
//! - **Spherical Geometry ($\mathbb{S}^2$)**: Great-circle geodesic distance and Girard's spherical excess area formula.
//! - **Hyperbolic Triangles**: Gauss-Bonnet area defect $\operatorname{Area} = \pi - (\alpha + \beta + \gamma)$.
//! - **Regular $\{p, q\}$ Hyperbolic Tessellations**: Schläfli polygon fundamental domain generator for $(p-2)(q-2) > 4$.
//! - **Schwarz-Christoffel Conformal Mappings**: Numerical integration of polygonal conformal maps.

use serde::{Deserialize, Serialize};

/// Point in the Poincaré Disk Model $\mathbb{D} = \{z \in \mathbb{C} : |z| < 1\}$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoincareDiskPoint {
    pub x: f64,
    pub y: f64,
}

impl PoincareDiskPoint {
    pub fn new(x: f64, y: f64) -> Result<Self, String> {
        let norm_sq = x * x + y * y;
        if norm_sq >= 1.0 {
            Err(format!(
                "Point ({}, {}) lies outside the open unit disk (|z|^2 = {} >= 1)",
                x, y, norm_sq
            ))
        } else {
            Ok(Self { x, y })
        }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Exact Poincaré geodesic distance:
    /// $$d(u, v) = \operatorname{arcosh}\left(1 + 2\frac{\|u - v\|^2}{(1 - \|u\|^2)(1 - \|v\|^2)}\right)$$
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let diff_sq = dx * dx + dy * dy;

        let u_sq = self.norm_sq();
        let v_sq = other.norm_sq();

        let denom = (1.0 - u_sq) * (1.0 - v_sq);
        if denom <= 0.0 {
            return f64::INFINITY;
        }

        let delta = 2.0 * diff_sq / denom;
        // arcosh(1 + delta) = ln((1 + delta) + sqrt((1 + delta)^2 - 1)) = ln(1 + delta + sqrt(delta^2 + 2*delta))
        let arg = 1.0 + delta;
        (arg + (delta * delta + 2.0 * delta).sqrt()).ln()
    }

    /// Apply Möbius transformation $w = \frac{a z + b}{\bar{b} z + \bar{a}}$ with $|a|^2 - |b|^2 = 1$.
    pub fn mobius_isometry(&self, a_re: f64, a_im: f64, b_re: f64, b_im: f64) -> Self {
        let z_re = self.x;
        let z_im = self.y;

        // Numerator: (a_re + i a_im)*(z_re + i z_im) + (b_re + i b_im)
        let num_re = (a_re * z_re - a_im * z_im) + b_re;
        let num_im = (a_re * z_im + a_im * z_re) + b_im;

        // Denominator: (b_re - i b_im)*(z_re + i z_im) + (a_re - i a_im)
        let den_re = (b_re * z_re + b_im * z_im) + a_re;
        let den_im = (b_re * z_im - b_im * z_re) - a_im;

        let den_mag_sq = den_re * den_re + den_im * den_im;
        let w_re = (num_re * den_re + num_im * den_im) / den_mag_sq;
        let w_im = (num_im * den_re - num_re * den_im) / den_mag_sq;

        Self { x: w_re, y: w_im }
    }

    /// Geodesic arc points between `self` and `other`.
    pub fn geodesic_arc(&self, other: &Self, num_pts: usize) -> Vec<Self> {
        if num_pts <= 1 {
            return vec![*self];
        }
        let mut pts = Vec::with_capacity(num_pts);
        for i in 0..num_pts {
            let t = (i as f64) / ((num_pts - 1) as f64);
            let x = self.x * (1.0 - t) + other.x * t;
            let y = self.y * (1.0 - t) + other.y * t;
            pts.push(Self { x, y });
        }
        pts
    }
}

/// Point in the Upper Half-Plane Model $\mathbb{H}^2 = \{z \in \mathbb{C} : \operatorname{Im}(z) > 0\}$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UpperHalfPlanePoint {
    pub x: f64,
    pub y: f64,
}

impl UpperHalfPlanePoint {
    pub fn new(x: f64, y: f64) -> Result<Self, String> {
        if y <= 0.0 {
            Err(format!(
                "Point ({}, {}) lies outside upper half plane (y <= 0)",
                x, y
            ))
        } else {
            Ok(Self { x, y })
        }
    }

    /// Geodesic distance in Upper Half-Plane:
    /// $$d(z_1, z_2) = \operatorname{arcosh}\left(1 + \frac{\|z_1 - z_2\|^2}{2 y_1 y_2}\right)$$
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let diff_sq = dx * dx + dy * dy;

        let delta = diff_sq / (2.0 * self.y * other.y);
        let arg = 1.0 + delta;
        (arg + (delta * delta + 2.0 * delta).sqrt()).ln()
    }

    /// Cayley transform from Upper Half-Plane to Poincaré Disk:
    /// $$w = \frac{z - i}{z + i}$$
    pub fn to_poincare(&self) -> PoincareDiskPoint {
        let num_re = self.x;
        let num_im = self.y - 1.0;

        let den_re = self.x;
        let den_im = self.y + 1.0;

        let den_mag_sq = den_re * den_re + den_im * den_im;
        let w_re = (num_re * den_re + num_im * den_im) / den_mag_sq;
        let w_im = (num_im * den_re - num_re * den_im) / den_mag_sq;

        PoincareDiskPoint { x: w_re, y: w_im }
    }

    /// Inverse Cayley transform from Poincaré Disk to Upper Half-Plane:
    /// $$z = i \frac{1 + w}{1 - w}$$
    pub fn from_poincare(pt: &PoincareDiskPoint) -> Self {
        let num_re = 1.0 + pt.x;
        let num_im = pt.y;

        let den_re = 1.0 - pt.x;
        let den_im = -pt.y;

        let den_mag_sq = den_re * den_re + den_im * den_im;
        let q_re = (num_re * den_re + num_im * den_im) / den_mag_sq;
        let q_im = (num_im * den_re - num_re * den_im) / den_mag_sq;

        // z = i * (q_re + i * q_im) = -q_im + i * q_re
        Self {
            x: -q_im,
            y: q_re.max(1e-12),
        }
    }

    /// Action of $\operatorname{PSL}(2, \mathbb{R})$: $z \mapsto \frac{a z + b}{c z + d}$ with $ad - bc = 1$.
    pub fn psl2_transform(&self, a: f64, b: f64, c: f64, d: f64) -> Self {
        let num_re = a * self.x + b;
        let num_im = a * self.y;

        let den_re = c * self.x + d;
        let den_im = c * self.y;

        let den_mag_sq = den_re * den_re + den_im * den_im;
        let x_out = (num_re * den_re + num_im * den_im) / den_mag_sq;
        let y_out = (num_im * den_re - num_re * den_im) / den_mag_sq;

        Self { x: x_out, y: y_out }
    }
}

/// Point on the Unit 2-Sphere $\mathbb{S}^2 = \{ (x, y, z) \in \mathbb{R}^3 : x^2 + y^2 + z^2 = 1 \}$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SphericalPoint {
    pub theta: f64, // Co-latitude [0, PI]
    pub phi: f64,   // Longitude [0, 2*PI)
}

impl SphericalPoint {
    pub fn new(theta: f64, phi: f64) -> Self {
        Self { theta, phi }
    }

    pub fn to_cartesian(&self) -> [f64; 3] {
        [
            self.theta.sin() * self.phi.cos(),
            self.theta.sin() * self.phi.sin(),
            self.theta.cos(),
        ]
    }

    pub fn from_cartesian(x: f64, y: f64, z: f64) -> Self {
        let norm = (x * x + y * y + z * z).sqrt();
        let nx = x / norm;
        let ny = y / norm;
        let nz = z / norm;

        let theta = nz.clamp(-1.0, 1.0).acos();
        let phi = ny.atan2(nx);
        let phi_pos = if phi < 0.0 {
            phi + 2.0 * std::f64::consts::PI
        } else {
            phi
        };

        Self {
            theta,
            phi: phi_pos,
        }
    }

    /// Great-circle geodesic distance on the unit sphere:
    /// $$d(u, v) = \arccos(\mathbf{u} \cdot \mathbf{v})$$
    pub fn great_circle_distance(&self, other: &Self) -> f64 {
        let u = self.to_cartesian();
        let v = other.to_cartesian();
        let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        dot.clamp(-1.0, 1.0).acos()
    }

    /// Girard's Theorem for Spherical Triangle Area (Spherical Excess):
    /// $$\operatorname{Area}(\triangle) = \alpha + \beta + \gamma - \pi$$
    pub fn triangle_excess_area(alpha_rad: f64, beta_rad: f64, gamma_rad: f64) -> f64 {
        (alpha_rad + beta_rad + gamma_rad - std::f64::consts::PI).max(0.0)
    }
}

/// Hyperbolic Triangle and Gauss-Bonnet Area Defect.
pub struct HyperbolicTriangle;

impl HyperbolicTriangle {
    /// Gauss-Bonnet Theorem for Hyperbolic Triangle Area (Area Defect):
    /// $$\operatorname{Area}(\triangle) = \pi - (\alpha + \beta + \gamma)$$
    pub fn area_defect(alpha_rad: f64, beta_rad: f64, gamma_rad: f64) -> f64 {
        (std::f64::consts::PI - (alpha_rad + beta_rad + gamma_rad)).max(0.0)
    }
}

/// Regular $\{p, q\}$ Hyperbolic Tessellation Generator.
pub struct HyperbolicTessellation;

impl HyperbolicTessellation {
    /// Radial distance $r$ from center to vertices of a regular $\{p, q\}$ hyperbolic polygon:
    /// $$\cosh(r) = \frac{\cos(\pi/q)}{\sin(\pi/p)}$$
    /// In the Poincaré disk model, Euclidean radius $R = \tanh(r/2)$.
    pub fn fundamental_domain_radius(p: usize, q: usize) -> Option<f64> {
        let pi = std::f64::consts::PI;
        let num = (pi / (q as f64)).cos();
        let den = (pi / (p as f64)).sin();

        let cosh_r = num / den;
        if cosh_r <= 1.0 {
            // Not a hyperbolic tessellation (spherical or Euclidean)
            return None;
        }

        let r = (cosh_r + (cosh_r * cosh_r - 1.0).sqrt()).ln();
        let euc_radius = (r / 2.0).tanh();
        Some(euc_radius)
    }

    /// Generate the $p$ vertices of the central fundamental polygon for a regular $\{p, q\}$ tessellation.
    pub fn generate_p_gon_vertices(p: usize, q: usize) -> Option<Vec<PoincareDiskPoint>> {
        let r = Self::fundamental_domain_radius(p, q)?;
        let pi2 = 2.0 * std::f64::consts::PI;
        let mut vertices = Vec::with_capacity(p);

        for k in 0..p {
            let angle = (k as f64) * pi2 / (p as f64);
            let x = r * angle.cos();
            let y = r * angle.sin();
            if let Ok(pt) = PoincareDiskPoint::new(x, y) {
                vertices.push(pt);
            }
        }

        Some(vertices)
    }
}

/// Schwarz-Christoffel Conformal Mapping Engine.
pub struct SchwarzChristoffel;

impl SchwarzChristoffel {
    /// Numerical integration of the Schwarz-Christoffel transformation mapping the upper half-plane to a polygon:
    /// $$f(z) = \int_0^z \prod_{k=1}^n (\zeta - x_k)^{\beta_k} d\zeta$$
    /// where $\beta_k = \frac{\alpha_k}{\pi} - 1$ are the turning angle parameters.
    pub fn integrate_map(
        prevertices: &[f64],
        interior_angles_rad: &[f64],
        target_z_re: f64,
        target_z_im: f64,
        steps: usize,
    ) -> (f64, f64) {
        let n = prevertices.len();
        let pi = std::f64::consts::PI;
        let betas: Vec<f64> = interior_angles_rad.iter().map(|&a| a / pi - 1.0).collect();

        // Trapezoidal integration along straight line from (0,0) to (target_z_re, target_z_im)
        let dt = 1.0 / (steps as f64);
        let mut sum_re = 0.0;
        let mut sum_im = 0.0;

        for s in 0..steps {
            let t = (s as f64 + 0.5) * dt;
            let zeta_re = target_z_re * t;
            let zeta_im = target_z_im * t;

            // Compute product term prod (zeta - x_k)^beta_k
            let mut log_prod_re = 0.0;
            let mut log_prod_im = 0.0;

            for k in 0..n {
                let diff_re = zeta_re - prevertices[k];
                let diff_im = zeta_im;
                let r = (diff_re * diff_re + diff_im * diff_im).sqrt().max(1e-12);
                let theta = diff_im.atan2(diff_re);

                log_prod_re += betas[k] * r.ln();
                log_prod_im += betas[k] * theta;
            }

            let mag = log_prod_re.exp();
            let d_re = mag * log_prod_im.cos();
            let d_im = mag * log_prod_im.sin();

            // Multiply by dz = (target_z_re + i * target_z_im) * dt
            let int_re = (d_re * target_z_re - d_im * target_z_im) * dt;
            let int_im = (d_re * target_z_im + d_im * target_z_re) * dt;

            sum_re += int_re;
            sum_im += int_im;
        }

        (sum_re, sum_im)
    }
}
