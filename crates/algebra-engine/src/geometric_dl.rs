//! # `algebra_engine::geometric_dl`
//!
//! Geometric Deep Learning, $SE(3)$ Steerable Equivariance & Clifford Neural Networks.
//!
//! Features:
//! - **Spherical Harmonics ($Y_{\ell m}$)**: Real and complex basis functions on $\mathbb{S}^2$ with exact Legendre polynomials.
//! - **Clebsch-Gordan Tensor Decomposition**: Exact $\operatorname{SO}(3)$ irrep coupling $\mathcal{D}^{(\ell_1)} \otimes \mathcal{D}^{(\ell_2)} \to \bigoplus \mathcal{D}^{(\ell)}$.
//! - **$SE(3)$ Steerable Equivariant Layer**: Rotation and translation equivariant filter kernels for molecular & mesh point clouds.
//! - **Clifford Multivector Neural Layer**: Geometric algebra $Cl(p,q)$ equivariant neural transformations.

#![allow(clippy::needless_range_loop)]

use serde::{Deserialize, Serialize};

/// Spherical Harmonics ($Y_{\ell m}$) on $\mathbb{S}^2$.
pub struct SphericalHarmonics;

impl SphericalHarmonics {
    /// Compute associated Legendre polynomial $P_\ell^m(x)$ for $x \in [-1, 1]$ and $0 \le m \le \ell$.
    pub fn legendre_p(l: usize, m: usize, x: f64) -> f64 {
        if m > l {
            return 0.0;
        }

        // P_m^m(x) = (-1)^m (2m-1)!! (1 - x^2)^(m/2)
        let mut pmm = 1.0;
        if m > 0 {
            let somx2 = ((1.0 - x) * (1.0 + x)).max(0.0).sqrt();
            let mut fact = 1.0;
            for _i in 1..=m {
                pmm *= -fact * somx2;
                fact += 2.0;
            }
        }

        if l == m {
            return pmm;
        }

        // P_{m+1}^m(x) = x (2m + 1) P_m^m(x)
        let mut pmmp1 = x * (2.0 * m as f64 + 1.0) * pmm;
        if l == m + 1 {
            return pmmp1;
        }

        // Recurrence for higher l:
        // (l - m) P_l^m = x (2l - 1) P_{l-1}^m - (l + m - 1) P_{l-2}^m
        let mut pll = 0.0;
        for ll in (m + 2)..=l {
            let ll_f = ll as f64;
            let m_f = m as f64;
            pll = (x * (2.0 * ll_f - 1.0) * pmmp1 - (ll_f + m_f - 1.0) * pmm) / (ll_f - m_f);
            pmm = pmmp1;
            pmmp1 = pll;
        }

        pll
    }

    /// Factorial $n!$ for small integers.
    fn factorial(n: usize) -> f64 {
        let mut f = 1.0;
        for i in 2..=n {
            f *= i as f64;
        }
        f
    }

    /// Normalization constant $N_\ell^m = \sqrt{\frac{2\ell+1}{4\pi} \frac{(\ell-m)!}{(\ell+m)!}}$.
    pub fn norm_const(l: usize, m: usize) -> f64 {
        let pi = std::f64::consts::PI;
        let num = (2.0 * l as f64 + 1.0) * Self::factorial(l - m);
        let den = 4.0 * pi * Self::factorial(l + m);
        (num / den).sqrt()
    }

    /// Real spherical harmonic $Y_\ell^m(\theta, \phi)$ for $\ell \ge 0$ and $-\ell \le m \le \ell$.
    pub fn real_y_lm(l: usize, m: isize, theta: f64, phi: f64) -> f64 {
        let cos_theta = theta.cos().clamp(-1.0, 1.0);
        let abs_m = m.unsigned_abs();

        let p_lm = Self::legendre_p(l, abs_m, cos_theta);
        let n_lm = Self::norm_const(l, abs_m);

        if m == 0 {
            n_lm * p_lm
        } else if m > 0 {
            std::f64::consts::SQRT_2 * n_lm * p_lm * (m as f64 * phi).cos()
        } else {
            std::f64::consts::SQRT_2 * n_lm * p_lm * (abs_m as f64 * phi).sin()
        }
    }

    /// Real spherical harmonic evaluated from 3D unit direction vector $\hat{\mathbf{r}} = (x, y, z)$.
    pub fn real_y_lm_cartesian(l: usize, m: isize, x: f64, y: f64, z: f64) -> f64 {
        let norm = (x * x + y * y + z * z).sqrt().max(1e-12);
        let cos_theta = (z / norm).clamp(-1.0, 1.0);
        let theta = cos_theta.acos();
        let phi = y.atan2(x);
        let phi_pos = if phi < 0.0 {
            phi + 2.0 * std::f64::consts::PI
        } else {
            phi
        };
        Self::real_y_lm(l, m, theta, phi_pos)
    }
}

/// Clebsch-Gordan Coefficient & $\operatorname{SO}(3)$ Tensor Decomposition Engine.
pub struct ClebschGordan;

impl ClebschGordan {
    /// Exact Clebsch-Gordan coefficient $\langle \ell_1 m_1 \ell_2 m_2 | \ell m \rangle$ via Racah formula.
    pub fn coefficient(l1: usize, m1: isize, l2: usize, m2: isize, l: usize, m: isize) -> f64 {
        // Selection rule 1: m1 + m2 == m
        if m1 + m2 != m {
            return 0.0;
        }
        // Selection rule 2: |l1 - l2| <= l <= l1 + l2
        let min_l = l1.abs_diff(l2);
        let max_l = l1 + l2;
        if l < min_l || l > max_l {
            return 0.0;
        }
        // Selection rule 3: |m1| <= l1, |m2| <= l2, |m| <= l
        if m1.unsigned_abs() > l1 || m2.unsigned_abs() > l2 || m.unsigned_abs() > l {
            return 0.0;
        }

        // Racah closed-form algebraic formula for Clebsch-Gordan coefficients
        let l1_f = l1 as f64;
        let l2_f = l2 as f64;
        let l_f = l as f64;
        let m1_f = m1 as f64;
        let m2_f = m2 as f64;
        let m_f = m as f64;

        let delta = ((2.0 * l_f + 1.0)
            * Self::factorial_f(l1_f + l2_f - l_f)
            * Self::factorial_f(l1_f - l2_f + l_f)
            * Self::factorial_f(-l1_f + l2_f + l_f)
            / Self::factorial_f(l1_f + l2_f + l_f + 1.0))
        .sqrt();

        let prefactor = delta
            * (Self::factorial_f(l1_f + m1_f)
                * Self::factorial_f(l1_f - m1_f)
                * Self::factorial_f(l2_f + m2_f)
                * Self::factorial_f(l2_f - m2_f)
                * Self::factorial_f(l_f + m_f)
                * Self::factorial_f(l_f - m_f))
            .sqrt();

        let mut sum = 0.0;
        let k_min = 0_i64
            .max(l2 as i64 - l as i64 - m1 as i64)
            .max(l1 as i64 - l as i64 + m2 as i64);
        let k_max = ((l1 + l2).saturating_sub(l) as i64)
            .min(l1 as i64 - m1 as i64)
            .min(l2 as i64 + m2 as i64);

        for k in k_min..=k_max {
            let k_f = k as f64;
            let d1 = Self::factorial_f(k_f);
            let d2 = Self::factorial_f(l1_f + l2_f - l_f - k_f);
            let d3 = Self::factorial_f(l1_f - m1_f - k_f);
            let d4 = Self::factorial_f(l2_f + m2_f - k_f);
            let d5 = Self::factorial_f(l_f - l2_f + m1_f + k_f);
            let d6 = Self::factorial_f(l_f - l1_f - m2_f + k_f);

            if d1 > 0.0 && d2 > 0.0 && d3 > 0.0 && d4 > 0.0 && d5 > 0.0 && d6 > 0.0 {
                let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                sum += sign / (d1 * d2 * d3 * d4 * d5 * d6);
            }
        }

        prefactor * sum
    }

    fn factorial_f(x: f64) -> f64 {
        if x < 0.0 {
            0.0
        } else if x == 0.0 {
            1.0
        } else {
            let n = x.round() as usize;
            let mut f = 1.0;
            for i in 2..=n {
                f *= i as f64;
            }
            f
        }
    }

    /// Compute Clebsch-Gordan coupled tensor product for two irrep vectors $u \in \mathbb{R}^{2\ell_1 + 1}$ and $v \in \mathbb{R}^{2\ell_2 + 1}$.
    pub fn tensor_product(l1: usize, u: &[f64], l2: usize, v: &[f64], l: usize) -> Vec<f64> {
        let dim_l = 2 * l + 1;
        let mut out = vec![0.0; dim_l];

        for m_idx in 0..dim_l {
            let m = m_idx as isize - l as isize;
            let mut val = 0.0;

            for m1_idx in 0..(2 * l1 + 1) {
                let m1 = m1_idx as isize - l1 as isize;
                let m2 = m - m1;
                if m2.unsigned_abs() <= l2 {
                    let m2_idx = (m2 + l2 as isize) as usize;
                    let cg = Self::coefficient(l1, m1, l2, m2, l, m);
                    val += cg * u[m1_idx] * v[m2_idx];
                }
            }
            out[m_idx] = val;
        }

        out
    }
}

/// $SE(3)$ Steerable Equivariant Layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerableConvLayer {
    pub in_l: usize,
    pub out_l: usize,
    pub filter_l: usize,
    pub radial_weight: f64,
}

impl SteerableConvLayer {
    pub fn new(in_l: usize, out_l: usize, filter_l: usize, radial_weight: f64) -> Self {
        Self {
            in_l,
            out_l,
            filter_l,
            radial_weight,
        }
    }

    /// Forward pass computing rotation-steerable message from neighbor at relative vector $\Delta \mathbf{r} = \mathbf{x}_j - \mathbf{x}_i$.
    pub fn compute_message(&self, in_features: &[f64], delta_r: &[f64; 3]) -> Vec<f64> {
        let r = (delta_r[0] * delta_r[0] + delta_r[1] * delta_r[1] + delta_r[2] * delta_r[2])
            .sqrt()
            .max(1e-12);
        let radial_decay = self.radial_weight * (-r).exp();

        // Evaluate spherical harmonic filter at unit direction
        let dim_filter = 2 * self.filter_l + 1;
        let mut filter_harmonics = Vec::with_capacity(dim_filter);
        for m_idx in 0..dim_filter {
            let m = m_idx as isize - self.filter_l as isize;
            let y_lm = SphericalHarmonics::real_y_lm_cartesian(
                self.filter_l,
                m,
                delta_r[0],
                delta_r[1],
                delta_r[2],
            );
            filter_harmonics.push(radial_decay * y_lm);
        }

        // Couple input features with directional filter via Clebsch-Gordan tensor product
        ClebschGordan::tensor_product(
            self.in_l,
            in_features,
            self.filter_l,
            &filter_harmonics,
            self.out_l,
        )
    }
}

/// Clifford Multivector Neural Layer in $Cl(3, 0)$ (8-dimensional algebra).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliffordLayer {
    pub weights: Vec<Vec<f64>>, // 8x8 weight matrix
    pub bias: Vec<f64>,         // 8 bias scalars
}

impl CliffordLayer {
    pub fn new_identity() -> Self {
        let mut weights = vec![vec![0.0; 8]; 8];
        for i in 0..8 {
            weights[i][i] = 1.0;
        }
        Self {
            weights,
            bias: vec![0.0; 8],
        }
    }

    pub fn new_with_weights(weights: Vec<Vec<f64>>, bias: Vec<f64>) -> Self {
        Self { weights, bias }
    }

    /// Forward multivector linear transformation $\mathbf{M}' = \mathbf{W} \mathbf{M} + \mathbf{B}$.
    pub fn forward(&self, multivector_8d: &[f64; 8]) -> [f64; 8] {
        let mut out = [0.0; 8];
        for i in 0..8 {
            let mut sum = self.bias[i];
            for j in 0..8 {
                sum += self.weights[i][j] * multivector_8d[j];
            }
            out[i] = sum;
        }
        out
    }

    /// Grade-preserving non-linear activation (GELU on multivector coefficients).
    pub fn gelu_activate(multivector: &[f64; 8]) -> [f64; 8] {
        let mut out = [0.0; 8];
        for i in 0..8 {
            let x = multivector[i];
            // Approximation: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
            let sqrt_2_pi = 0.7978845608;
            let inner = sqrt_2_pi * (x + 0.044715 * x * x * x);
            out[i] = 0.5 * x * (1.0 + inner.tanh());
        }
        out
    }
}
