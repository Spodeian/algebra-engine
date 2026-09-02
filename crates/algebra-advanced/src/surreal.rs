//! # `algebra_advanced::surreal`
//!
//! Conway Surreal Numbers $\mathbf{No}$.
//!
//! Encompasses:
//! - **Dedekind-Conway Games**: Inductive construction $x = \{ L_x \mid R_x \}$ where no $l \in L_x \ge r \in R_x$.
//! - **Canonical Surreal Forms**: Dyadic fractions $\frac{m}{2^n}$, integers $\mathbb{Z}$, and birthday ordering.
//! - **Surreal Arithmetic**:
//!   - Addition: $x + y = \{ L_x + y, x + L_y \mid R_x + y, x + R_y \}$
//!   - Negation: $-x = \{ -R_x \mid -L_x \}$
//!   - Multiplication: $x \cdot y = \{ x'y + xy' - x'y' \mid \dots \}$
//! - **Transfinite & Infinitesimal Constants**:
//!   - Zero $\mathbf{0} = \{ \emptyset \mid \emptyset \}$
//!   - One $\mathbf{1} = \{ 0 \mid \emptyset \}$
//!   - Half $1/2 = \{ 0 \mid 1 \}$
//!   - Infinitesimal $\epsilon = \{ 0 \mid 1, 1/2, 1/4, \dots \}$
//!   - Transfinite $\omega = \{ 0, 1, 2, 3, \dots \mid \emptyset \}$

use std::cmp::Ordering;
use std::fmt;

/// Inductive Conway Surreal Number representation.
#[derive(Debug, Clone, PartialEq)]
pub struct SurrealNo {
    /// Left set of surreal numbers $\{ l \in L \}$.
    pub left: Vec<SurrealNo>,
    /// Right set of surreal numbers $\{ r \in R \}$.
    pub right: Vec<SurrealNo>,
    /// Birthday / generation index.
    pub birthday: usize,
    /// Cached dyadic / float approximation.
    pub approx: f64,
}

impl SurrealNo {
    /// Creates a surreal number from left and right sets: $\{ L \mid R \}$.
    pub fn new(left: Vec<SurrealNo>, right: Vec<SurrealNo>) -> Self {
        let max_b_l = left.iter().map(|s| s.birthday).max().unwrap_or(0);
        let max_b_r = right.iter().map(|s| s.birthday).max().unwrap_or(0);
        let birthday = if left.is_empty() && right.is_empty() {
            0
        } else {
            max_b_l.max(max_b_r) + 1
        };

        let left_approx = left
            .iter()
            .map(|s| s.approx)
            .fold(f64::NEG_INFINITY, f64::max);
        let right_approx = right.iter().map(|s| s.approx).fold(f64::INFINITY, f64::min);

        let approx = if left.is_empty() && right.is_empty() {
            0.0
        } else if right.is_empty() {
            left_approx + 1.0
        } else if left.is_empty() {
            right_approx - 1.0
        } else {
            (left_approx + right_approx) / 2.0
        };

        Self {
            left,
            right,
            birthday,
            approx,
        }
    }

    /// Surreal Zero $\mathbf{0} = \{ \emptyset \mid \emptyset \}$.
    pub fn zero() -> Self {
        Self {
            left: Vec::new(),
            right: Vec::new(),
            birthday: 0,
            approx: 0.0,
        }
    }

    /// Surreal One $\mathbf{1} = \{ 0 \mid \emptyset \}$.
    pub fn one() -> Self {
        Self::new(vec![Self::zero()], Vec::new())
    }

    /// Surreal Minus One $\mathbf{-1} = \{ \emptyset \mid 0 \}$.
    pub fn minus_one() -> Self {
        Self::new(Vec::new(), vec![Self::zero()])
    }

    /// Surreal Half $1/2 = \{ 0 \mid 1 \}$.
    pub fn half() -> Self {
        Self::new(vec![Self::zero()], vec![Self::one()])
    }

    /// Surreal Integer from $n \in \mathbb{Z}$.
    pub fn from_int(n: i64) -> Self {
        if n == 0 {
            Self::zero()
        } else if n > 0 {
            let mut cur = Self::zero();
            for _ in 0..n {
                cur = Self::new(vec![cur], Vec::new());
            }
            cur
        } else {
            let mut cur = Self::zero();
            for _ in 0..(-n) {
                cur = Self::new(Vec::new(), vec![cur]);
            }
            cur
        }
    }

    /// Surreal Transfinite Ordinal $\omega = \{ 0, 1, 2, \dots \mid \emptyset \}$.
    pub fn omega(depth: usize) -> Self {
        let mut left = Vec::with_capacity(depth);
        let mut cur = Self::zero();
        for _ in 0..depth {
            left.push(cur.clone());
            cur = Self::new(vec![cur], Vec::new());
        }
        Self::new(left, Vec::new())
    }

    /// Surreal Infinitesimal $\epsilon = \{ 0 \mid 1, 1/2, 1/4, \dots \}$.
    pub fn epsilon(depth: usize) -> Self {
        let mut right = Vec::with_capacity(depth);
        let mut cur = Self::one();
        for _ in 0..depth {
            right.push(cur.clone());
            cur = Self::new(vec![Self::zero()], vec![cur]);
        }
        Self::new(vec![Self::zero()], right)
    }

    /// Conway Partial Order: $x \le y \iff (\forall l \in L_x, l \not\ge y) \land (\forall r \in R_y, x \not\ge r)$.
    pub fn leq(&self, other: &Self) -> bool {
        // Fast float check if separated
        if (self.approx - other.approx).abs() > 1e-9 {
            return self.approx <= other.approx;
        }

        for l in &self.left {
            if other.leq(l) {
                return false;
            }
        }
        for r in &other.right {
            if r.leq(self) {
                return false;
            }
        }
        true
    }

    /// Surreal Equivalence $x \equiv y \iff x \le y \land y \le x$.
    pub fn equiv(&self, other: &Self) -> bool {
        self.leq(other) && other.leq(self)
    }

    /// Negation $-x = \{ -R_x \mid -L_x \}$.
    pub fn neg(&self) -> Self {
        let new_left: Vec<SurrealNo> = self.right.iter().map(|r| r.neg()).collect();
        let new_right: Vec<SurrealNo> = self.left.iter().map(|l| l.neg()).collect();
        Self::new(new_left, new_right)
    }

    /// Addition $x + y = \{ L_x + y, x + L_y \mid R_x + y, x + R_y \}$.
    pub fn add(&self, other: &Self) -> Self {
        let mut new_left = Vec::new();
        let mut new_right = Vec::new();

        for lx in &self.left {
            new_left.push(lx.add(other));
        }
        for ly in &other.left {
            new_left.push(self.add(ly));
        }

        for rx in &self.right {
            new_right.push(rx.add(other));
        }
        for ry in &other.right {
            new_right.push(self.add(ry));
        }

        Self::new(new_left, new_right)
    }

    /// Subtraction $x - y = x + (-y)$.
    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    /// Multiplication $x \cdot y$.
    pub fn mul(&self, other: &Self) -> Self {
        // Base cases
        if self.equiv(&Self::zero()) || other.equiv(&Self::zero()) {
            return Self::zero();
        }
        if self.equiv(&Self::one()) {
            return other.clone();
        }
        if other.equiv(&Self::one()) {
            return self.clone();
        }

        let mut new_left = Vec::new();
        let mut new_right = Vec::new();

        // Left set: x'y + xy' - x'y' (for x' in Lx, y' in Ly and x'' in Rx, y'' in Ry)
        for lx in &self.left {
            for ly in &other.left {
                // lx * y + x * ly - lx * ly
                let term = lx.mul(other).add(&self.mul(ly)).sub(&lx.mul(ly));
                new_left.push(term);
            }
        }
        for rx in &self.right {
            for ry in &other.right {
                let term = rx.mul(other).add(&self.mul(ry)).sub(&rx.mul(ry));
                new_left.push(term);
            }
        }

        // Right set: x'y + xy'' - x'y''
        for lx in &self.left {
            for ry in &other.right {
                let term = lx.mul(other).add(&self.mul(ry)).sub(&lx.mul(ry));
                new_right.push(term);
            }
        }
        for rx in &self.right {
            for ly in &other.left {
                let term = rx.mul(other).add(&self.mul(ly)).sub(&rx.mul(ly));
                new_right.push(term);
            }
        }

        Self::new(new_left, new_right)
    }
}

impl PartialOrd for SurrealNo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let le = self.leq(other);
        let ge = other.leq(self);
        match (le, ge) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            (false, false) => None,
        }
    }
}

impl fmt::Display for SurrealNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} (b={})", self.approx, self.birthday)
    }
}
