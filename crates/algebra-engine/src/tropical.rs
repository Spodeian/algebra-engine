//! # `algebra_engine::tropical`
//!
//! Tropical Semirings and Min-Plus / Max-Plus Algebra:
//! - $\mathbb{T}_{\max} = (\mathbb{R} \cup \{-\infty\}, \oplus_{\max}, \otimes)$ where $a \oplus b = \max(a, b)$, $a \otimes b = a + b$
//! - $\mathbb{T}_{\min} = (\mathbb{R} \cup \{+\infty\}, \oplus_{\min}, \otimes)$ where $a \oplus b = \min(a, b)$, $a \otimes b = a + b$
//! - Tropical matrix multiplication for shortest-path graph algorithms and Viterbi dynamic programming
//! - Log-Sum-Exp smooth tropical approximation: $\text{LSE}_\varepsilon(x, y) = \varepsilon \ln(e^{x/\varepsilon} + e^{y/\varepsilon})$

use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

/// Max-Plus semiring element: $(\mathbb{R} \cup \{-\infty\}, \max, +)$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MaxPlus {
    NegInf,
    Val(f64),
}

impl MaxPlus {
    pub fn zero() -> Self {
        MaxPlus::NegInf
    }

    pub fn one() -> Self {
        MaxPlus::Val(0.0)
    }

    pub fn val(x: f64) -> Self {
        MaxPlus::Val(x)
    }
}

impl Add for MaxPlus {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (MaxPlus::NegInf, x) | (x, MaxPlus::NegInf) => x,
            (MaxPlus::Val(a), MaxPlus::Val(b)) => MaxPlus::Val(a.max(b)),
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul for MaxPlus {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (MaxPlus::NegInf, _) | (_, MaxPlus::NegInf) => MaxPlus::NegInf,
            (MaxPlus::Val(a), MaxPlus::Val(b)) => MaxPlus::Val(a + b),
        }
    }
}

/// Min-Plus semiring element: $(\mathbb{R} \cup \{+\infty\}, \min, +)$.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MinPlus {
    PosInf,
    Val(f64),
}

impl MinPlus {
    pub fn zero() -> Self {
        MinPlus::PosInf
    }

    pub fn one() -> Self {
        MinPlus::Val(0.0)
    }

    pub fn val(x: f64) -> Self {
        MinPlus::Val(x)
    }
}

impl Add for MinPlus {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (MinPlus::PosInf, x) | (x, MinPlus::PosInf) => x,
            (MinPlus::Val(a), MinPlus::Val(b)) => MinPlus::Val(a.min(b)),
        }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Mul for MinPlus {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (MinPlus::PosInf, _) | (_, MinPlus::PosInf) => MinPlus::PosInf,
            (MinPlus::Val(a), MinPlus::Val(b)) => MinPlus::Val(a + b),
        }
    }
}

/// Dense matrix over tropical semiring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TropicalMatrix<T> {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<T>,
}

impl<T: Copy + Clone> TropicalMatrix<T> {
    pub fn new(rows: usize, cols: usize, default_val: T) -> Self {
        Self {
            rows,
            cols,
            data: vec![default_val; rows * cols],
        }
    }

    pub fn get(&self, r: usize, c: usize) -> T {
        self.data[r * self.cols + c]
    }

    pub fn set(&mut self, r: usize, c: usize, val: T) {
        self.data[r * self.cols + c] = val;
    }
}

impl<T> TropicalMatrix<T>
where
    T: Copy + Clone + Add<Output = T> + Mul<Output = T>,
{
    /// Tropical Matrix Multiplication $C_{i, j} = \bigoplus_k (A_{i, k} \otimes B_{k, j})$.
    pub fn mul(&self, other: &TropicalMatrix<T>) -> Option<TropicalMatrix<T>> {
        if self.cols != other.rows {
            return None;
        }
        let mut result = Vec::with_capacity(self.rows * other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = self.get(i, 0) * other.get(0, j);
                for k in 1..self.cols {
                    sum = sum + (self.get(i, k) * other.get(k, j));
                }
                result.push(sum);
            }
        }
        Some(TropicalMatrix {
            rows: self.rows,
            cols: other.cols,
            data: result,
        })
    }
}

/// Smooth Log-Sum-Exp approximation to $\max(x, y)$:
/// $\text{LSE}_\varepsilon(x, y) = \varepsilon \ln(e^{x/\varepsilon} + e^{y/\varepsilon})$
pub fn log_sum_exp(x: f64, y: f64, epsilon: f64) -> f64 {
    let max_val = x.max(y);
    max_val + epsilon * (((x - max_val) / epsilon).exp() + ((y - max_val) / epsilon).exp()).ln()
}
