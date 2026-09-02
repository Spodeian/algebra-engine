//! # `algebra_engine::autodiff`
//!
//! High-Performance Automatic Differentiation (AD) Engine.
//!
//! Features:
//! - **Dual Numbers ($\mathbb{D}$)**: Exact 1st-order forward-mode derivatives without truncation error.
//! - **Hyper-Dual Numbers ($\mathbb{D}_2$)**: Exact 2nd-order derivatives and mixed Hessians ($\mathbf{H} = \nabla^2 f$) in a single forward pass without finite differences.
//! - **Gradient & Hessian Evaluators**: High-dimensional gradient vector and Hessian matrix generation.
//! - **Tape-Free Vector-Jacobian Products (VJP)**: Memory-efficient adjoint vector-Jacobian products.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// First-Order Dual Number: $x + \dot{x} \epsilon$ with $\epsilon^2 = 0$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Dual<T = f64> {
    pub val: T,
    pub eps: T,
}

impl Dual<f64> {
    pub const fn new(val: f64, eps: f64) -> Self {
        Self { val, eps }
    }

    pub const fn constant(val: f64) -> Self {
        Self { val, eps: 0.0 }
    }

    pub const fn var(val: f64) -> Self {
        Self { val, eps: 1.0 }
    }

    pub fn sin(self) -> Self {
        Self {
            val: self.val.sin(),
            eps: self.eps * self.val.cos(),
        }
    }

    pub fn cos(self) -> Self {
        Self {
            val: self.val.cos(),
            eps: -self.eps * self.val.sin(),
        }
    }

    pub fn tan(self) -> Self {
        let c = self.val.cos();
        Self {
            val: self.val.tan(),
            eps: self.eps / (c * c),
        }
    }

    pub fn exp(self) -> Self {
        let e = self.val.exp();
        Self {
            val: e,
            eps: self.eps * e,
        }
    }

    pub fn ln(self) -> Self {
        Self {
            val: self.val.ln(),
            eps: self.eps / self.val,
        }
    }

    pub fn sqrt(self) -> Self {
        let s = self.val.sqrt();
        Self {
            val: s,
            eps: self.eps / (2.0 * s),
        }
    }

    pub fn powf(self, n: f64) -> Self {
        Self {
            val: self.val.powf(n),
            eps: self.eps * n * self.val.powf(n - 1.0),
        }
    }

    pub fn sinh(self) -> Self {
        Self {
            val: self.val.sinh(),
            eps: self.eps * self.val.cosh(),
        }
    }

    pub fn cosh(self) -> Self {
        Self {
            val: self.val.cosh(),
            eps: self.eps * self.val.sinh(),
        }
    }

    pub fn tanh(self) -> Self {
        let t = self.val.tanh();
        Self {
            val: t,
            eps: self.eps * (1.0 - t * t),
        }
    }
}

impl Add for Dual<f64> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            val: self.val + rhs.val,
            eps: self.eps + rhs.eps,
        }
    }
}

impl Sub for Dual<f64> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            val: self.val - rhs.val,
            eps: self.eps - rhs.eps,
        }
    }
}

impl Mul for Dual<f64> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            val: self.val * rhs.val,
            eps: self.val * rhs.eps + self.eps * rhs.val,
        }
    }
}

impl Div for Dual<f64> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self {
            val: self.val / rhs.val,
            eps: (self.eps * rhs.val - self.val * rhs.eps) / (rhs.val * rhs.val),
        }
    }
}

impl Neg for Dual<f64> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            val: -self.val,
            eps: -self.eps,
        }
    }
}

impl Add<f64> for Dual<f64> {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        Self {
            val: self.val + rhs,
            eps: self.eps,
        }
    }
}

impl Sub<f64> for Dual<f64> {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        Self {
            val: self.val - rhs,
            eps: self.eps,
        }
    }
}

impl Mul<f64> for Dual<f64> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            val: self.val * rhs,
            eps: self.eps * rhs,
        }
    }
}

impl Div<f64> for Dual<f64> {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            val: self.val / rhs,
            eps: self.eps / rhs,
        }
    }
}

/// Second-Order Hyper-Dual Number: $x + x_1 \epsilon_1 + x_2 \epsilon_2 + x_{12} \epsilon_1 \epsilon_2$ with $\epsilon_i^2 = 0$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperDual<T = f64> {
    pub val: T,
    pub eps1: T,
    pub eps2: T,
    pub eps12: T,
}

impl HyperDual<f64> {
    pub const fn new(val: f64, eps1: f64, eps2: f64, eps12: f64) -> Self {
        Self {
            val,
            eps1,
            eps2,
            eps12,
        }
    }

    pub const fn constant(val: f64) -> Self {
        Self {
            val,
            eps1: 0.0,
            eps2: 0.0,
            eps12: 0.0,
        }
    }

    pub const fn var1(val: f64) -> Self {
        Self {
            val,
            eps1: 1.0,
            eps2: 0.0,
            eps12: 0.0,
        }
    }

    pub const fn var2(val: f64) -> Self {
        Self {
            val,
            eps1: 0.0,
            eps2: 1.0,
            eps12: 0.0,
        }
    }

    pub const fn var_both(val: f64) -> Self {
        Self {
            val,
            eps1: 1.0,
            eps2: 1.0,
            eps12: 0.0,
        }
    }

    pub fn sin(self) -> Self {
        let s = self.val.sin();
        let c = self.val.cos();
        Self {
            val: s,
            eps1: self.eps1 * c,
            eps2: self.eps2 * c,
            eps12: self.eps12 * c - self.eps1 * self.eps2 * s,
        }
    }

    pub fn cos(self) -> Self {
        let s = self.val.sin();
        let c = self.val.cos();
        Self {
            val: c,
            eps1: -self.eps1 * s,
            eps2: -self.eps2 * s,
            eps12: -self.eps12 * s - self.eps1 * self.eps2 * c,
        }
    }

    pub fn exp(self) -> Self {
        let e = self.val.exp();
        Self {
            val: e,
            eps1: self.eps1 * e,
            eps2: self.eps2 * e,
            eps12: (self.eps12 + self.eps1 * self.eps2) * e,
        }
    }

    pub fn ln(self) -> Self {
        let inv = 1.0 / self.val;
        let inv2 = inv * inv;
        Self {
            val: self.val.ln(),
            eps1: self.eps1 * inv,
            eps2: self.eps2 * inv,
            eps12: self.eps12 * inv - self.eps1 * self.eps2 * inv2,
        }
    }

    pub fn sqrt(self) -> Self {
        let s = self.val.sqrt();
        let inv2s = 1.0 / (2.0 * s);
        let inv4s3 = 1.0 / (4.0 * s * self.val);
        Self {
            val: s,
            eps1: self.eps1 * inv2s,
            eps2: self.eps2 * inv2s,
            eps12: self.eps12 * inv2s - self.eps1 * self.eps2 * inv4s3,
        }
    }

    pub fn powf(self, n: f64) -> Self {
        let val_n = self.val.powf(n);
        let val_n1 = self.val.powf(n - 1.0);
        let val_n2 = self.val.powf(n - 2.0);
        Self {
            val: val_n,
            eps1: self.eps1 * n * val_n1,
            eps2: self.eps2 * n * val_n1,
            eps12: self.eps12 * n * val_n1 + self.eps1 * self.eps2 * n * (n - 1.0) * val_n2,
        }
    }

    pub fn tanh(self) -> Self {
        let t = self.val.tanh();
        let sech2 = 1.0 - t * t;
        let d2 = -2.0 * t * sech2;
        Self {
            val: t,
            eps1: self.eps1 * sech2,
            eps2: self.eps2 * sech2,
            eps12: self.eps12 * sech2 + self.eps1 * self.eps2 * d2,
        }
    }
}

impl Add for HyperDual<f64> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            val: self.val + rhs.val,
            eps1: self.eps1 + rhs.eps1,
            eps2: self.eps2 + rhs.eps2,
            eps12: self.eps12 + rhs.eps12,
        }
    }
}

impl Sub for HyperDual<f64> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            val: self.val - rhs.val,
            eps1: self.eps1 - rhs.eps1,
            eps2: self.eps2 - rhs.eps2,
            eps12: self.eps12 - rhs.eps12,
        }
    }
}

impl Mul for HyperDual<f64> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            val: self.val * rhs.val,
            eps1: self.val * rhs.eps1 + self.eps1 * rhs.val,
            eps2: self.val * rhs.eps2 + self.eps2 * rhs.val,
            eps12: self.val * rhs.eps12
                + self.eps1 * rhs.eps2
                + self.eps2 * rhs.eps1
                + self.eps12 * rhs.val,
        }
    }
}

impl Div for HyperDual<f64> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let inv = 1.0 / rhs.val;
        let inv2 = inv * inv;
        let inv3 = inv2 * inv;
        Self {
            val: self.val * inv,
            eps1: (self.eps1 * rhs.val - self.val * rhs.eps1) * inv2,
            eps2: (self.eps2 * rhs.val - self.val * rhs.eps2) * inv2,
            eps12: (self.eps12 * rhs.val - self.val * rhs.eps12) * inv2
                - (self.eps1 * rhs.eps2 + self.eps2 * rhs.eps1) * inv2
                + 2.0 * self.val * rhs.eps1 * rhs.eps2 * inv3,
        }
    }
}

impl Neg for HyperDual<f64> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            val: -self.val,
            eps1: -self.eps1,
            eps2: -self.eps2,
            eps12: -self.eps12,
        }
    }
}

impl Add<f64> for HyperDual<f64> {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        Self {
            val: self.val + rhs,
            eps1: self.eps1,
            eps2: self.eps2,
            eps12: self.eps12,
        }
    }
}

impl Sub<f64> for HyperDual<f64> {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        Self {
            val: self.val - rhs,
            eps1: self.eps1,
            eps2: self.eps2,
            eps12: self.eps12,
        }
    }
}

impl Mul<f64> for HyperDual<f64> {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            val: self.val * rhs,
            eps1: self.eps1 * rhs,
            eps2: self.eps2 * rhs,
            eps12: self.eps12 * rhs,
        }
    }
}

impl Div<f64> for HyperDual<f64> {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            val: self.val / rhs,
            eps1: self.eps1 / rhs,
            eps2: self.eps2 / rhs,
            eps12: self.eps12 / rhs,
        }
    }
}

/// Forward-Mode Automatic Differentiation Evaluators.
pub struct ForwardGradient;

impl ForwardGradient {
    /// Compute exact value and gradient vector $\nabla f(\mathbf{x})$ in $\mathbb{R}^n$.
    pub fn gradient<F>(f: F, x: &[f64]) -> (f64, Vec<f64>)
    where
        F: Fn(&[Dual<f64>]) -> Dual<f64>,
    {
        let n = x.len();
        let mut grad = vec![0.0; n];
        let mut val = 0.0;

        for i in 0..n {
            let mut dual_x: Vec<Dual<f64>> = x.iter().map(|&v| Dual::constant(v)).collect();
            dual_x[i] = Dual::var(x[i]);
            let res = f(&dual_x);
            if i == 0 {
                val = res.val;
            }
            grad[i] = res.eps;
        }

        (val, grad)
    }

    /// Compute exact value, gradient vector $\nabla f(\mathbf{x})$, and Hessian matrix $\mathbf{H} = \nabla^2 f(\mathbf{x})$.
    pub fn hessian<F>(f: F, x: &[f64]) -> (f64, Vec<f64>, Vec<Vec<f64>>)
    where
        F: Fn(&[HyperDual<f64>]) -> HyperDual<f64>,
    {
        let n = x.len();
        let mut grad = vec![0.0; n];
        let mut hess = vec![vec![0.0; n]; n];
        let mut val = 0.0;

        for i in 0..n {
            for j in i..n {
                let mut hd_x: Vec<HyperDual<f64>> =
                    x.iter().map(|&v| HyperDual::constant(v)).collect();
                if i == j {
                    hd_x[i] = HyperDual::var_both(x[i]);
                } else {
                    hd_x[i] = HyperDual::var1(x[i]);
                    hd_x[j] = HyperDual::var2(x[j]);
                }

                let res = f(&hd_x);
                if i == 0 && j == 0 {
                    val = res.val;
                }
                if i == j {
                    grad[i] = res.eps1;
                    hess[i][i] = res.eps12;
                } else {
                    hess[i][j] = res.eps12;
                    hess[j][i] = res.eps12;
                }
            }
        }

        (val, grad, hess)
    }

    /// Compute exact Jacobian matrix $\mathbf{J} \in \mathbb{R}^{m \times n}$ for $F: \mathbb{R}^n \to \mathbb{R}^m$.
    pub fn jacobian<F>(f: F, x: &[f64]) -> Vec<Vec<f64>>
    where
        F: Fn(&[Dual<f64>]) -> Vec<Dual<f64>>,
    {
        let n = x.len();
        let base_eval = f(&x.iter().map(|&v| Dual::constant(v)).collect::<Vec<_>>());
        let m = base_eval.len();
        let mut jac = vec![vec![0.0; n]; m];

        for j in 0..n {
            let mut dual_x: Vec<Dual<f64>> = x.iter().map(|&v| Dual::constant(v)).collect();
            dual_x[j] = Dual::var(x[j]);
            let res = f(&dual_x);
            for i in 0..m {
                jac[i][j] = res[i].eps;
            }
        }

        jac
    }
}

/// Tape-Free Vector-Jacobian Product (VJP).
pub struct TapeFreeVjp;

impl TapeFreeVjp {
    /// Compute $\mathbf{v}^T \mathbf{J}$ for $F: \mathbb{R}^n \to \mathbb{R}^m$ given adjoint vector $\mathbf{v} \in \mathbb{R}^m$.
    pub fn vjp<F>(f: F, x: &[f64], v: &[f64]) -> Vec<f64>
    where
        F: Fn(&[Dual<f64>]) -> Vec<Dual<f64>>,
    {
        let jac = ForwardGradient::jacobian(f, x);
        let n = x.len();
        let m = v.len();
        let mut res = vec![0.0; n];

        for j in 0..n {
            let mut sum = 0.0;
            for i in 0..m {
                sum += v[i] * jac[i][j];
            }
            res[j] = sum;
        }

        res
    }
}
