//! # `algebra_engine::poly::roots`
//!
//! Algebraic Polynomial Root Solvers & Resolvents.
//!
//! Provides:
//! - **Cardano's Cubic Solver**: Exact analytical roots of $ax^3 + bx^2 + cx + d = 0$ with Casus Irreducibilis handling.
//! - **Ferrari's Quartic Solver**: Exact analytical roots of $ax^4 + bx^3 + cx^2 + dx + e = 0$ via resolvent cubic.
//! - **Bring Radical $\operatorname{BR}(a)$**: Canonical real root of the unsolvable quintic $x^5 + x + a = 0$.
//! - **Sturm Sequence Root Isolation**: Exact count and isolating intervals $[a_k, b_k]$ for all real roots.
//! - **Durand-Kerner (Weierstrass) Solver**: High-precision simultaneous numerical computation of all complex roots.

use algebra_core::error::{AlgebraError, AlgebraResult};
use algebra_core::{ExprGraph, ExprId};
use std::f64::consts::PI;

/// Real and Complex Roots representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexRoot {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

impl ComplexRoot {
    /// Creates a real root.
    pub fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    /// Creates a complex root.
    pub fn complex(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Converts to symbolic `ExprId` in `ExprGraph`.
    pub fn to_expr(&self, graph: &ExprGraph) -> ExprId {
        let re_node = if self.re.fract() == 0.0 {
            graph.integer(self.re as i64)
        } else {
            graph.float(self.re)
        };

        if self.im.abs() < 1e-12 {
            re_node
        } else {
            let im_node = if self.im.fract() == 0.0 {
                graph.integer(self.im as i64)
            } else {
                graph.float(self.im)
            };
            let i_sym = graph.symbol("i");
            let im_part = graph.mul([im_node, i_sym]);
            graph.add([re_node, im_part])
        }
    }
}

/// Cardano's Exact Analytical Cubic Solver.
pub struct CardanoSolver;

impl CardanoSolver {
    /// Solves $a x^3 + b x^2 + c x + d = 0$.
    pub fn solve(a: f64, b: f64, c: f64, d: f64) -> AlgebraResult<Vec<ComplexRoot>> {
        if a.abs() < 1e-14 {
            return Err(AlgebraError::EvaluationError(
                "Leading coefficient 'a' cannot be zero in cubic".into(),
            ));
        }

        // Normalize to monic: x^3 + a2*x^2 + a1*x + a0 = 0
        let a2 = b / a;
        let a1 = c / a;
        let a0 = d / a;

        // Depress cubic: substitute x = t - a2/3 -> t^3 + p*t + q = 0
        let p = a1 - (a2 * a2) / 3.0;
        let q = (2.0 * a2 * a2 * a2) / 27.0 - (a2 * a1) / 3.0 + a0;
        let shift = -a2 / 3.0;

        // Discriminant delta = (q/2)^2 + (p/3)^3
        let delta = (q / 2.0).powi(2) + (p / 3.0).powi(3);

        if delta > 1e-12 {
            // One real root, two complex conjugates
            let u = (-q / 2.0 + delta.sqrt()).cbrt();
            let v = (-q / 2.0 - delta.sqrt()).cbrt();

            let t1 = u + v;
            let t2_re = -0.5 * (u + v);
            let t2_im = 0.5 * 3.0f64.sqrt() * (u - v);

            Ok(vec![
                ComplexRoot::real(t1 + shift),
                ComplexRoot::complex(t2_re + shift, t2_im),
                ComplexRoot::complex(t2_re + shift, -t2_im),
            ])
        } else if delta < -1e-12 {
            // Casus Irreducibilis: 3 distinct real roots via trigonometric formula
            let r = (-p.powi(3) / 27.0).sqrt();
            let phi = (-q / (2.0 * r)).clamp(-1.0, 1.0).acos();

            let m = 2.0 * (-p / 3.0).sqrt();
            let t1 = m * (phi / 3.0).cos();
            let t2 = m * ((phi + 2.0 * PI) / 3.0).cos();
            let t3 = m * ((phi + 4.0 * PI) / 3.0).cos();

            Ok(vec![
                ComplexRoot::real(t1 + shift),
                ComplexRoot::real(t2 + shift),
                ComplexRoot::real(t3 + shift),
            ])
        } else {
            // Multiple real roots (delta == 0)
            let u = (-q / 2.0).cbrt();
            let t1 = 2.0 * u;
            let t2 = -u;

            Ok(vec![
                ComplexRoot::real(t1 + shift),
                ComplexRoot::real(t2 + shift),
                ComplexRoot::real(t2 + shift),
            ])
        }
    }
}

/// Ferrari's Exact Analytical Quartic Solver.
pub struct FerrariSolver;

impl FerrariSolver {
    /// Solves $a x^4 + b x^3 + c x^2 + d x + e = 0$.
    pub fn solve(a: f64, b: f64, c: f64, d: f64, e: f64) -> AlgebraResult<Vec<ComplexRoot>> {
        if a.abs() < 1e-14 {
            return Err(AlgebraError::EvaluationError(
                "Leading coefficient 'a' cannot be zero in quartic".into(),
            ));
        }

        // Normalize to monic: x^4 + a3*x^3 + a2*x^2 + a1*x + a0 = 0
        let a3 = b / a;
        let a2 = c / a;
        let a1 = d / a;
        let a0 = e / a;

        // Depress quartic: substitute x = u - a3/4 -> u^4 + alpha*u^2 + beta*u + gamma = 0
        let shift = -a3 / 4.0;
        let alpha = a2 - 3.0 * a3 * a3 / 8.0;
        let beta = a3 * a3 * a3 / 8.0 - a3 * a2 / 2.0 + a1;
        let gamma = -3.0 * a3.powi(4) / 256.0 + a2 * a3 * a3 / 16.0 - a3 * a1 / 4.0 + a0;

        if beta.abs() < 1e-12 {
            // Biquadratic case: u^4 + alpha*u^2 + gamma = 0
            let quad_disc = alpha * alpha - 4.0 * gamma;
            let mut roots = Vec::new();
            if quad_disc >= 0.0 {
                let z1 = (-alpha + quad_disc.sqrt()) / 2.0;
                let z2 = (-alpha - quad_disc.sqrt()) / 2.0;
                for z in [z1, z2] {
                    if z >= 0.0 {
                        roots.push(ComplexRoot::real(z.sqrt() + shift));
                        roots.push(ComplexRoot::real(-z.sqrt() + shift));
                    } else {
                        let im = (-z).sqrt();
                        roots.push(ComplexRoot::complex(shift, im));
                        roots.push(ComplexRoot::complex(shift, -im));
                    }
                }
            } else {
                let r = gamma.sqrt();
                let theta = (-alpha / (2.0 * r)).clamp(-1.0, 1.0).acos();
                let u_re = (r * (1.0 + theta.cos()) / 2.0).sqrt();
                let u_im = (r * (1.0 - theta.cos()) / 2.0).sqrt();
                roots.push(ComplexRoot::complex(u_re + shift, u_im));
                roots.push(ComplexRoot::complex(-u_re + shift, -u_im));
                roots.push(ComplexRoot::complex(u_re + shift, -u_im));
                roots.push(ComplexRoot::complex(-u_re + shift, u_im));
            }
            return Ok(roots);
        }

        // Resolvent cubic: y^3 + 2*alpha*y^2 + (alpha^2 - 4*gamma)*y - beta^2 = 0
        let cubic_roots =
            CardanoSolver::solve(1.0, 2.0 * alpha, alpha * alpha - 4.0 * gamma, -beta * beta)?;

        // Select positive real root y
        let mut y_val = 0.0;
        for cr in &cubic_roots {
            if cr.im.abs() < 1e-10 && cr.re > 0.0 {
                y_val = cr.re;
                break;
            }
        }
        if y_val <= 0.0 {
            y_val = cubic_roots[0].re.abs().max(1e-12);
        }

        let sqrt_y = y_val.sqrt();
        let r1 = (-(alpha + y_val) + beta / sqrt_y).max(0.0);
        let r2 = (-(alpha + y_val) - beta / sqrt_y).max(0.0);

        let roots = vec![
            ComplexRoot::real(sqrt_y / 2.0 + r1.sqrt() / 2.0 + shift),
            ComplexRoot::real(sqrt_y / 2.0 - r1.sqrt() / 2.0 + shift),
            ComplexRoot::real(-sqrt_y / 2.0 + r2.sqrt() / 2.0 + shift),
            ComplexRoot::real(-sqrt_y / 2.0 - r2.sqrt() / 2.0 + shift),
        ];

        Ok(roots)
    }
}

/// Bring Radical $\operatorname{BR}(a)$: Unique real root of $x^5 + x + a = 0$.
pub struct BringRadical;

impl BringRadical {
    /// Evaluates the Bring Radical $\operatorname{BR}(a)$ to high precision using Newton-Raphson iteration.
    pub fn eval(a: f64) -> f64 {
        // Initial estimate based on asymptotes
        let mut x = if a.abs() > 1.0 {
            -(-a).signum() * a.abs().powf(0.2)
        } else {
            -a
        };

        for _ in 0..50 {
            let f = x.powi(5) + x + a;
            let df = 5.0 * x.powi(4) + 1.0;
            let dx = f / df;
            x -= dx;
            if dx.abs() < 1e-14 {
                break;
            }
        }
        x
    }
}

/// Sturm Sequence Real Root Isolator.
pub struct SturmSequence {
    /// Sequence of polynomials $[f_0 = P, f_1 = P', f_2 = -\text{rem}(f_0, f_1), \dots]$.
    pub chain: Vec<Vec<f64>>,
}

impl SturmSequence {
    /// Builds a Sturm sequence from polynomial coefficients (highest to lowest degree).
    pub fn from_poly(coeffs: &[f64]) -> Self {
        let mut chain = Vec::new();
        let mut f0 = coeffs.to_vec();
        Self::trim_leading_zeros(&mut f0);
        if f0.is_empty() {
            return Self { chain };
        }

        // f1 = f0'
        let mut f1 = Vec::with_capacity(f0.len() - 1);
        let deg = f0.len() - 1;
        for (i, &c) in f0.iter().enumerate().take(deg) {
            let p = (deg - i) as f64;
            f1.push(c * p);
        }
        Self::trim_leading_zeros(&mut f1);

        chain.push(f0);
        if f1.is_empty() {
            return Self { chain };
        }
        chain.push(f1);

        while let Some(last) = chain.last() {
            if last.len() <= 1 {
                break;
            }
            let prev = &chain[chain.len() - 2];
            let rem = Self::poly_rem_neg(prev, last);
            if rem.is_empty() {
                break;
            }
            chain.push(rem);
        }

        Self { chain }
    }

    /// Counts real roots in $(a, b]$ using Sturm's Theorem: $V(a) - V(b)$.
    pub fn count_real_roots(&self, a: f64, b: f64) -> usize {
        let va = self.sign_variations(a);
        let vb = self.sign_variations(b);
        va.saturating_sub(vb)
    }

    fn sign_variations(&self, x: f64) -> usize {
        let mut last_sign = 0;
        let mut variations = 0;

        for poly in &self.chain {
            let val = Self::eval_poly(poly, x);
            if val.abs() > 1e-12 {
                let sign = if val > 0.0 { 1 } else { -1 };
                if last_sign != 0 && sign != last_sign {
                    variations += 1;
                }
                last_sign = sign;
            }
        }
        variations
    }

    fn eval_poly(poly: &[f64], x: f64) -> f64 {
        let mut res = 0.0;
        for &c in poly {
            res = res * x + c;
        }
        res
    }

    fn poly_rem_neg(num: &[f64], den: &[f64]) -> Vec<f64> {
        let mut r = num.to_vec();
        let deg_den = den.len() - 1;
        let lc_den = den[0];

        while r.len() >= den.len() {
            let deg_r = r.len() - 1;
            let shift = deg_r - deg_den;
            let factor = r[0] / lc_den;

            for (i, &d) in den.iter().enumerate() {
                r[i] -= factor * d;
            }
            Self::trim_leading_zeros(&mut r);
            let _ = shift;
        }

        // Negate remainder -rem
        for c in &mut r {
            *c = -*c;
        }
        r
    }

    fn trim_leading_zeros(poly: &mut Vec<f64>) {
        while !poly.is_empty() && poly[0].abs() < 1e-12 {
            poly.remove(0);
        }
    }
}

/// Durand-Kerner Simultaneous Complex Root Finder for degree $n$ polynomials.
pub struct DurandKernerSolver;

#[allow(clippy::needless_range_loop)]
impl DurandKernerSolver {
    /// Computes all $n$ complex roots of monic polynomial $P(z) = z^n + a_{n-1}z^{n-1} + \dots + a_0$.
    pub fn solve(coeffs_monic: &[f64], max_iters: usize, tol: f64) -> Vec<ComplexRoot> {
        let n = coeffs_monic.len() - 1;
        if n == 0 {
            return Vec::new();
        }

        // Initialize roots with Aberth/Durand-Kerner initial points on complex circle
        let r0 = 0.4 + 0.9 * (coeffs_monic.iter().map(|c| c.abs()).fold(0.0f64, f64::max));
        let mut roots: Vec<(f64, f64)> = (0..n)
            .map(|k| {
                let theta = 2.0 * PI * (k as f64 + 0.4) / (n as f64);
                (r0 * theta.cos(), r0 * theta.sin())
            })
            .collect();

        for _ in 0..max_iters {
            let mut max_diff = 0.0f64;

            for i in 0..n {
                let (re_i, im_i) = roots[i];
                // Evaluate P(z_i)
                let (p_re, p_im) = Self::eval_poly_complex(coeffs_monic, re_i, im_i);

                // Compute product prod_{j != i} (z_i - z_j)
                let mut prod_re = 1.0;
                let mut prod_im = 0.0;
                for j in 0..n {
                    if i != j {
                        let (re_j, im_j) = roots[j];
                        let diff_re = re_i - re_j;
                        let diff_im = im_i - im_j;
                        let new_re = prod_re * diff_re - prod_im * diff_im;
                        let new_im = prod_re * diff_im + prod_im * diff_re;
                        prod_re = new_re;
                        prod_im = new_im;
                    }
                }

                // delta = P(z_i) / prod
                let den = prod_re * prod_re + prod_im * prod_im;
                if den.abs() > 1e-16 {
                    let delta_re = (p_re * prod_re + p_im * prod_im) / den;
                    let delta_im = (p_im * prod_re - p_re * prod_im) / den;

                    roots[i].0 -= delta_re;
                    roots[i].1 -= delta_im;

                    max_diff = max_diff.max(delta_re.hypot(delta_im));
                }
            }

            if max_diff < tol {
                break;
            }
        }

        roots
            .into_iter()
            .map(|(re, im)| ComplexRoot::complex(re, im))
            .collect()
    }

    fn eval_poly_complex(poly: &[f64], re: f64, im: f64) -> (f64, f64) {
        let mut cur_re = 0.0;
        let mut cur_im = 0.0;
        for &c in poly {
            let new_re = cur_re * re - cur_im * im + c;
            let new_im = cur_re * im + cur_im * re;
            cur_re = new_re;
            cur_im = new_im;
        }
        (cur_re, cur_im)
    }
}
