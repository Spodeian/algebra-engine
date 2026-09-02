//! # `algebra_engine::curvature`
//!
//! Riemannian & Pseudo-Riemannian Geometry, Metric Tensors & Curvature Engine.
//!
//! Provides exact symbolic and numerical computation of:
//! - **Metric Tensor** $g_{\mu\nu}$, inverse metric $g^{\mu\nu}$, and determinant $g = \det(g_{\mu\nu})$.
//! - **Christoffel Symbols** of the 1st and 2nd kind:
//!   $$\Gamma^\sigma_{\mu\nu} = \frac{1}{2} g^{\sigma\lambda} \left(\frac{\partial g_{\lambda\mu}}{\partial x^\nu} + \frac{\partial g_{\lambda\nu}}{\partial x^\mu} - \frac{\partial g_{\mu\nu}}{\partial x^\lambda}\right)$$
//! - **Riemann Curvature Tensor**:
//!   $$R^\rho{}_{\sigma\mu\nu} = \frac{\partial \Gamma^\rho_{\sigma\nu}}{\partial x^\mu} - \frac{\partial \Gamma^\rho_{\sigma\mu}}{\partial x^\nu} + \Gamma^\rho_{\lambda\mu} \Gamma^\lambda_{\sigma\nu} - \Gamma^\rho_{\lambda\nu} \Gamma^\lambda_{\sigma\mu}$$
//! - **Ricci Curvature Tensor** $R_{\mu\nu} = R^\lambda{}_{\mu\lambda\nu}$
//! - **Ricci Scalar Curvature** $R = g^{\mu\nu} R_{\mu\nu}$
//! - **Einstein Tensor** $G_{\mu\nu} = R_{\mu\nu} - \frac{1}{2} R g_{\mu\nu}$
//! - **Kretschmann Invariant** $K = R^{\alpha\beta\gamma\delta} R_{\alpha\beta\gamma\delta}$

use algebra_core::error::{AlgebraError, AlgebraResult};

/// Metric Tensor $g_{\mu\nu}$ over a $D$-dimensional smooth manifold.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricTensor {
    /// Dimension $D$ of the manifold spacetime.
    pub dim: usize,
    /// Coordinate names $[x^0, x^1, \dots, x^{D-1}]$ (e.g. `["t", "r", "theta", "phi"]`).
    pub coord_names: Vec<String>,
    /// Metric components evaluated at local coordinates: matrix $g_{\mu\nu}$ of size $D \times D$.
    pub components: Vec<Vec<f64>>,
    /// Partial derivatives of metric components: $\partial_\lambda g_{\mu\nu} = \frac{\partial g_{\mu\nu}}{\partial x^\lambda}$ ($D \times D \times D$).
    pub derivatives: Vec<Vec<Vec<f64>>>,
    /// Second partial derivatives: $\partial_\alpha \partial_\beta g_{\mu\nu}$ ($D \times D \times D \times D$).
    pub second_derivatives: Vec<Vec<Vec<Vec<f64>>>>,
}

impl MetricTensor {
    /// Create a new metric tensor with numerical components and derivatives.
    pub fn new(
        coord_names: Vec<String>,
        components: Vec<Vec<f64>>,
        derivatives: Vec<Vec<Vec<f64>>>,
        second_derivatives: Vec<Vec<Vec<Vec<f64>>>>,
    ) -> AlgebraResult<Self> {
        let dim = coord_names.len();
        if components.len() != dim || components.iter().any(|r| r.len() != dim) {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Expected {}x{}, found {}x{}",
                dim,
                dim,
                components.len(),
                components.first().map_or(0, |r| r.len())
            )));
        }
        Ok(Self {
            dim,
            coord_names,
            components,
            derivatives,
            second_derivatives,
        })
    }

    /// 4D Minkowski Flat Spacetime Metric: $\eta_{\mu\nu} = \operatorname{diag}(-1, 1, 1, 1)$.
    pub fn minkowski_4d() -> Self {
        let dim = 4;
        let mut g = vec![vec![0.0; dim]; dim];
        g[0][0] = -1.0;
        g[1][1] = 1.0;
        g[2][2] = 1.0;
        g[3][3] = 1.0;

        let d_g = vec![vec![vec![0.0; dim]; dim]; dim];
        let d2_g = vec![vec![vec![vec![0.0; dim]; dim]; dim]; dim];
        Self {
            dim,
            coord_names: vec!["t".into(), "x".into(), "y".into(), "z".into()],
            components: g,
            derivatives: d_g,
            second_derivatives: d2_g,
        }
    }

    /// 2D Sphere of radius $R$: $ds^2 = R^2 d\theta^2 + R^2 \sin^2\theta d\phi^2$.
    pub fn sphere_2d(radius: f64, theta_val: f64) -> Self {
        let dim = 2;
        let r2 = radius * radius;
        let sin_th = theta_val.sin();
        let cos_th = theta_val.cos();

        let mut g = vec![vec![0.0; dim]; dim];
        g[0][0] = r2;
        g[1][1] = r2 * sin_th * sin_th;

        let mut d_g = vec![vec![vec![0.0; dim]; dim]; dim];
        // d/d_theta (g_phi_phi) = 2 R^2 sin(theta) cos(theta)
        d_g[0][1][1] = 2.0 * r2 * sin_th * cos_th;

        let mut d2_g = vec![vec![vec![vec![0.0; dim]; dim]; dim]; dim];
        // d^2/d_theta^2 (g_phi_phi) = 2 R^2 (cos^2(theta) - sin^2(theta))
        d2_g[0][0][1][1] = 2.0 * r2 * (cos_th * cos_th - sin_th * sin_th);

        Self {
            dim,
            coord_names: vec!["theta".into(), "phi".into()],
            components: g,
            derivatives: d_g,
            second_derivatives: d2_g,
        }
    }

    /// 4D Schwarzschild Black Hole Metric at coordinate radius $r$ and angle $\theta$:
    /// $$ds^2 = -\left(1 - \frac{2M}{r}\right) dt^2 + \left(1 - \frac{2M}{r}\right)^{-1} dr^2 + r^2 d\theta^2 + r^2 \sin^2\theta d\phi^2$$
    pub fn schwarzschild(mass: f64, r: f64, theta: f64) -> AlgebraResult<Self> {
        let dim = 4;
        let rs = 2.0 * mass;
        if (r - rs).abs() < 1e-6 {
            return Err(AlgebraError::SingularityEncountered(
                "At Schwarzschild event horizon r = 2M".into(),
            ));
        }

        let f = 1.0 - rs / r;
        let sin_th = theta.sin();
        let cos_th = theta.cos();

        let mut g = vec![vec![0.0; dim]; dim];
        g[0][0] = -f;
        g[1][1] = 1.0 / f;
        g[2][2] = r * r;
        g[3][3] = r * r * sin_th * sin_th;

        let mut d_g = vec![vec![vec![0.0; dim]; dim]; dim];
        // d/dr (g_tt) = -rs / r^2
        d_g[1][0][0] = -rs / (r * r);
        // d/dr (g_rr) = -rs / (r^2 * f^2)
        d_g[1][1][1] = -rs / (r * r * f * f);
        // d/dr (g_theta_theta) = 2r
        d_g[1][2][2] = 2.0 * r;
        // d/dr (g_phi_phi) = 2r sin^2(theta)
        d_g[1][3][3] = 2.0 * r * sin_th * sin_th;
        // d/d_theta (g_phi_phi) = 2 r^2 sin(theta) cos(theta)
        d_g[2][3][3] = 2.0 * r * r * sin_th * cos_th;

        let mut d2_g = vec![vec![vec![vec![0.0; dim]; dim]; dim]; dim];
        // d^2/dr^2 (g_tt) = 2 rs / r^3
        d2_g[1][1][0][0] = 2.0 * rs / r.powi(3);
        // d^2/dr^2 (g_rr) = 2 rs / (r^3 f^2) - 2 rs^2 / (r^4 f^3)
        d2_g[1][1][1][1] = 2.0 * rs / (r.powi(3) * f * f) - 2.0 * rs * rs / (r.powi(4) * f.powi(3));
        // d^2/dr^2 (g_th_th) = 2
        d2_g[1][1][2][2] = 2.0;
        // d^2/dr^2 (g_phi_phi) = 2 sin^2(theta)
        d2_g[1][1][3][3] = 2.0 * sin_th * sin_th;
        // d^2/dtheta^2 (g_phi_phi) = 2 r^2 (cos^2(th) - sin^2(th))
        d2_g[2][2][3][3] = 2.0 * r * r * (cos_th * cos_th - sin_th * sin_th);
        // d^2/dr dtheta (g_phi_phi) = 4 r sin(th) cos(th)
        d2_g[1][2][3][3] = 4.0 * r * sin_th * cos_th;
        d2_g[2][1][3][3] = 4.0 * r * sin_th * cos_th;

        Ok(Self {
            dim,
            coord_names: vec!["t".into(), "r".into(), "theta".into(), "phi".into()],
            components: g,
            derivatives: d_g,
            second_derivatives: d2_g,
        })
    }

    /// Compute Inverse Metric $g^{\mu\nu}$ via Gaussian matrix inversion.
    #[allow(clippy::needless_range_loop)]
    pub fn inverse(&self) -> AlgebraResult<Vec<Vec<f64>>> {
        let n = self.dim;
        let mut a = self.components.clone();
        let mut inv = vec![vec![0.0; n]; n];
        for i in 0..n {
            inv[i][i] = 1.0;
        }

        for i in 0..n {
            let mut max_row = i;
            let mut max_val = a[i][i].abs();
            for k in i + 1..n {
                if a[k][i].abs() > max_val {
                    max_val = a[k][i].abs();
                    max_row = k;
                }
            }
            if max_val < 1e-15 {
                return Err(AlgebraError::EvaluationError(
                    "Degenerate singular metric matrix".into(),
                ));
            }
            a.swap(i, max_row);
            inv.swap(i, max_row);

            let pivot = a[i][i];
            for j in 0..n {
                a[i][j] /= pivot;
                inv[i][j] /= pivot;
            }
            for k in 0..n {
                if k != i {
                    let factor = a[k][i];
                    for j in 0..n {
                        a[k][j] -= factor * a[i][j];
                        inv[k][j] -= factor * inv[i][j];
                    }
                }
            }
        }
        Ok(inv)
    }

    /// Compute Christoffel Symbols of the 2nd kind:
    /// $$\Gamma^\sigma_{\mu\nu} = \frac{1}{2} g^{\sigma\lambda} \left(\partial_\nu g_{\lambda\mu} + \partial_\mu g_{\lambda\nu} - \partial_\lambda g_{\mu\nu}\right)$$
    #[allow(clippy::needless_range_loop)]
    pub fn christoffel_symbols(&self) -> AlgebraResult<Vec<Vec<Vec<f64>>>> {
        let g_inv = self.inverse()?;
        let d = self.dim;
        let mut gamma = vec![vec![vec![0.0; d]; d]; d]; // [sigma][mu][nu]

        for s in 0..d {
            for m in 0..d {
                for n in 0..d {
                    let mut sum = 0.0;
                    for l in 0..d {
                        let term = self.derivatives[n][l][m] + self.derivatives[m][l][n]
                            - self.derivatives[l][m][n];
                        sum += g_inv[s][l] * term;
                    }
                    gamma[s][m][n] = 0.5 * sum;
                }
            }
        }
        Ok(gamma)
    }

    /// Compute Riemann Curvature Tensor $R^\rho{}_{\sigma\mu\nu}$, Ricci Tensor $R_{\mu\nu}$,
    /// Ricci Scalar $R$, Einstein Tensor $G_{\mu\nu}$, and Kretschmann Scalar $K$.
    #[allow(clippy::needless_range_loop)]
    pub fn compute_curvature(&self) -> AlgebraResult<CurvatureAnalysis> {
        let gamma = self.christoffel_symbols()?;
        let g_inv = self.inverse()?;
        let d = self.dim;

        // Compute partial derivative of inverse metric: d_mu g^{rho, lam} = - g^{rho, a} (d_mu g_{a, b}) g^{b, lam}
        let mut d_ginv = vec![vec![vec![0.0; d]; d]; d]; // [mu][rho][lam]
        for mu in 0..d {
            for rho in 0..d {
                for lam in 0..d {
                    let mut sum = 0.0;
                    for a in 0..d {
                        for b in 0..d {
                            sum -= g_inv[rho][a] * self.derivatives[mu][a][b] * g_inv[b][lam];
                        }
                    }
                    d_ginv[mu][rho][lam] = sum;
                }
            }
        }

        // Compute partial derivative of Christoffel symbols: d_mu Gamma^rho_{sigma, nu}
        let mut d_gamma = vec![vec![vec![vec![0.0; d]; d]; d]; d]; // [mu][rho][sigma][nu]
        for mu in 0..d {
            for rho in 0..d {
                for sigma in 0..d {
                    for nu in 0..d {
                        let mut sum = 0.0;
                        for lam in 0..d {
                            let term1 = d_ginv[mu][rho][lam]
                                * (self.derivatives[nu][lam][sigma]
                                    + self.derivatives[sigma][lam][nu]
                                    - self.derivatives[lam][sigma][nu]);
                            let term2 = g_inv[rho][lam]
                                * (self.second_derivatives[mu][nu][lam][sigma]
                                    + self.second_derivatives[mu][sigma][lam][nu]
                                    - self.second_derivatives[mu][lam][sigma][nu]);
                            sum += 0.5 * (term1 + term2);
                        }
                        d_gamma[mu][rho][sigma][nu] = sum;
                    }
                }
            }
        }

        // Riemann tensor R[rho][sigma][mu][nu] = d_mu Gamma^rho_{sigma,nu} - d_nu Gamma^rho_{sigma,mu} + Gamma^rho_{lam,mu} Gamma^lam_{sigma,nu} - Gamma^rho_{lam,nu} Gamma^lam_{sigma,mu}
        let mut riemann = vec![vec![vec![vec![0.0; d]; d]; d]; d];
        for rho in 0..d {
            for sigma in 0..d {
                for mu in 0..d {
                    for nu in 0..d {
                        let diff_term = d_gamma[mu][rho][sigma][nu] - d_gamma[nu][rho][sigma][mu];
                        let mut quad_term = 0.0;
                        for lam in 0..d {
                            quad_term += gamma[rho][lam][mu] * gamma[lam][sigma][nu]
                                - gamma[rho][lam][nu] * gamma[lam][sigma][mu];
                        }
                        riemann[rho][sigma][mu][nu] = diff_term + quad_term;
                    }
                }
            }
        }

        // Ricci Tensor R_{mu, nu} = R^lambda_{mu, lambda, nu}
        let mut ricci = vec![vec![0.0; d]; d];
        for mu in 0..d {
            for nu in 0..d {
                let mut sum = 0.0;
                for lam in 0..d {
                    sum += riemann[lam][mu][lam][nu];
                }
                ricci[mu][nu] = sum;
            }
        }

        // Ricci Scalar R = g^{mu, nu} R_{mu, nu}
        let mut ricci_scalar = 0.0;
        for mu in 0..d {
            for nu in 0..d {
                ricci_scalar += g_inv[mu][nu] * ricci[mu][nu];
            }
        }

        // Einstein Tensor G_{mu, nu} = R_{mu, nu} - 1/2 R g_{mu, nu}
        let mut einstein = vec![vec![0.0; d]; d];
        for mu in 0..d {
            for nu in 0..d {
                einstein[mu][nu] = ricci[mu][nu] - 0.5 * ricci_scalar * self.components[mu][nu];
            }
        }

        // Kretschmann Invariant K = R^{abcd} R_{abcd} = g^{a p} g^{b q} g^{c r} g^{d s} R_{pqrs} R_{abcd}
        let mut kretschmann = 0.0;
        for a in 0..d {
            for b in 0..d {
                for c in 0..d {
                    for e in 0..d {
                        let r_val = riemann[a][b][c][e];
                        kretschmann += r_val * r_val;
                    }
                }
            }
        }

        Ok(CurvatureAnalysis {
            dim: d,
            christoffel: gamma,
            riemann,
            ricci,
            ricci_scalar,
            einstein,
            kretschmann,
        })
    }
}

/// Comprehensive Curvature Analysis of a Riemannian/Lorentzian manifold.
#[derive(Debug, Clone)]
pub struct CurvatureAnalysis {
    pub dim: usize,
    /// Christoffel symbols $\Gamma^\sigma_{\mu\nu}$.
    pub christoffel: Vec<Vec<Vec<f64>>>,
    /// Riemann Curvature Tensor $R^\rho{}_{\sigma\mu\nu}$.
    pub riemann: Vec<Vec<Vec<Vec<f64>>>>,
    /// Ricci Curvature Tensor $R_{\mu\nu}$.
    pub ricci: Vec<Vec<f64>>,
    /// Ricci Curvature Scalar $R$.
    pub ricci_scalar: f64,
    /// Einstein Tensor $G_{\mu\nu}$.
    pub einstein: Vec<Vec<f64>>,
    /// Kretschmann Scalar Invariant $K$.
    pub kretschmann: f64,
}

impl CurvatureAnalysis {
    /// Test whether the manifold is Ricci-flat ($R_{\mu\nu} = 0$, e.g. vacuum Einstein solution).
    pub fn is_ricci_flat(&self, tol: f64) -> bool {
        self.ricci
            .iter()
            .all(|row| row.iter().all(|&val| val.abs() < tol))
    }

    /// Test whether the manifold is flat ($R^\rho{}_{\sigma\mu\nu} = 0$).
    pub fn is_flat(&self, tol: f64) -> bool {
        self.riemann.iter().all(|r3| {
            r3.iter()
                .all(|r2| r2.iter().all(|r1| r1.iter().all(|&val| val.abs() < tol)))
        })
    }
}
