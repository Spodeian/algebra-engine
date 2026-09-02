//! # `algebra_advanced::clifford`
//!
//! Free Tensor Algebra $T(V)$ and Clifford Geometric Algebra $\operatorname{Cl}(p,q,r)$.
//!
//! ## Mathematical Foundations
//! - **Clifford Algebra $\operatorname{Cl}(p,q,r)$**: Quadratic space $(V, Q)$ with signature $(p,q,r)$:
//!   $$e_i e_j + e_j e_i = 2 \eta_{ij} \mathbf{1}$$
//!   where $\eta_{ii} = +1$ for $1 \le i \le p$, $-1$ for $p < i \le p+q$, and $0$ for $p+q < i \le p+q+r$.
//! - **Geometric Product**: $u v = u \cdot v + u \wedge v$.
//! - **Involutions**:
//!   - Reversion: $A^\dagger = \sum_{k} (-1)^{k(k-1)/2} \langle A \rangle_k$
//!   - Grade Involution: $\widehat{A} = \sum_{k} (-1)^k \langle A \rangle_k$
//!   - Clifford Conjugate: $\bar{A} = \widehat{A}^\dagger$
//! - **Rotor Sandwiches**: $v' = R v R^\dagger$ where $R R^\dagger = 1$.
//! - **Free Tensor Algebra $T(V)$**: Graded tensor direct sum $\bigoplus_{k=0}^N V^{\otimes k}$ with outer product $\otimes$,
//!   contraction $\operatorname{Tr}_{i,j}$, and (anti-)symmetrization.

use algebra_core::error::{AlgebraError, AlgebraResult};
use std::collections::BTreeMap;

/// Metric signature $(p, q, r)$ defining the quadratic form of the Clifford algebra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CliffordSignature {
    /// Number of basis vectors with $e_i^2 = +1$.
    pub p: usize,
    /// Number of basis vectors with $e_j^2 = -1$.
    pub q: usize,
    /// Number of degenerate null basis vectors with $e_k^2 = 0$.
    pub r: usize,
}

impl CliffordSignature {
    /// 3D Euclidean Geometric Algebra: $\operatorname{Cl}(3,0,0) \cong \mathbb{H} \oplus \mathbb{H}$ / Pauli algebra $\mathbb{C}(2)$.
    pub const PGA3D: Self = Self { p: 3, q: 0, r: 0 };
    /// Spacetime Algebra (STA): $\operatorname{Cl}(1,3,0)$ (Dirac algebra $\mathbb{C}(4)$).
    pub const SPACETIME: Self = Self { p: 1, q: 3, r: 0 };
    /// Conformal Geometric Algebra (CGA): $\operatorname{Cl}(4,1,0)$.
    pub const CONFORMAL: Self = Self { p: 4, q: 1, r: 0 };
    /// 2D Euclidean Plane: $\operatorname{Cl}(2,0,0)$.
    pub const EUCLIDEAN2D: Self = Self { p: 2, q: 0, r: 0 };

    /// 3D Euclidean Geometric Algebra: $\operatorname{Cl}(3,0,0) \cong \mathbb{H} \oplus \mathbb{H}$ / Pauli algebra $\mathbb{C}(2)$.
    #[inline]
    pub const fn pga3d() -> Self {
        Self::PGA3D
    }

    /// Spacetime Algebra (STA): $\operatorname{Cl}(1,3,0)$ (Dirac algebra $\mathbb{C}(4)$).
    #[inline]
    pub const fn spacetime() -> Self {
        Self::SPACETIME
    }

    /// Conformal Geometric Algebra (CGA): $\operatorname{Cl}(4,1,0)$.
    #[inline]
    pub const fn conformal() -> Self {
        Self::CONFORMAL
    }

    /// 2D Euclidean Plane: $\operatorname{Cl}(2,0,0)$.
    #[inline]
    pub const fn euclidean2d() -> Self {
        Self::EUCLIDEAN2D
    }

    /// Total dimension of the underlying vector space $V$.
    #[inline]
    pub const fn dim(&self) -> usize {
        self.p + self.q + self.r
    }

    /// Metric tensor entry $\eta_{ii} \in \{+1, -1, 0\}$.
    #[inline]
    pub const fn metric_diag(&self, i: usize) -> i64 {
        if i < self.p {
            1
        } else if i < self.p + self.q {
            -1
        } else {
            0
        }
    }
}

/// Basis blade represented as a bitmask of basis vector indices.
/// Bit $i$ is set if $e_i$ is present in the blade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BladeMask(pub u64);

impl BladeMask {
    /// Scalar identity blade $e_0 = \mathbf{1}$.
    pub const SCALAR: Self = Self(0);
    /// First basis vector $e_1$.
    pub const E1: Self = Self(1 << 0);
    /// Second basis vector $e_2$.
    pub const E2: Self = Self(1 << 1);
    /// Third basis vector $e_3$.
    pub const E3: Self = Self(1 << 2);
    /// Fourth basis vector $e_4$.
    pub const E4: Self = Self(1 << 3);

    #[inline]
    pub const fn scalar() -> Self {
        Self::SCALAR
    }

    #[inline]
    pub const fn vector(i: usize) -> Self {
        Self(1 << i)
    }

    #[inline]
    pub const fn from_raw(mask: u64) -> Self {
        Self(mask)
    }

    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn grade(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Multiply two basis blades $e_A \cdot e_B$ in signature $(p,q,r)$, returning resulting blade and sign.
    pub const fn multiply(&self, other: &BladeMask, sig: &CliffordSignature) -> (BladeMask, i64) {
        let a = self.0;
        let b = other.0;
        let mut sign = 1i64;

        // Count sign swaps needed to reorder basis vectors
        // For each bit set in a, count how many bits in b are to its left
        let mut swaps = 0;
        let mut i = 0;
        while i < 64 {
            if (a & (1 << i)) != 0 {
                let mask = (1 << i) - 1;
                swaps += (b & mask).count_ones();
            }
            i += 1;
        }
        if swaps % 2 != 0 {
            sign = -sign;
        }

        // Common basis vectors square to metric_diag(i)
        let common = a & b;
        let mut j = 0;
        let d = sig.dim();
        while j < d {
            if (common & (1 << j)) != 0 {
                let m = sig.metric_diag(j);
                if m == 0 {
                    return (BladeMask(0), 0);
                }
                sign *= m;
            }
            j += 1;
        }

        let res_mask = a ^ b;
        (BladeMask(res_mask), sign)
    }
}

/// General Multivector in Clifford Algebra $\operatorname{Cl}(p,q,r)$.
#[derive(Debug, Clone, PartialEq)]
pub struct CliffordMultivector {
    pub sig: CliffordSignature,
    /// Non-zero blade coefficients: `BladeMask -> Coefficient`.
    pub blades: BTreeMap<BladeMask, f64>,
}

impl CliffordMultivector {
    /// Create a zero multivector.
    pub fn zero(sig: CliffordSignature) -> Self {
        Self {
            sig,
            blades: BTreeMap::new(),
        }
    }

    /// Create a scalar multivector $\alpha \mathbf{1}$.
    pub fn scalar(val: f64, sig: CliffordSignature) -> Self {
        let mut blades = BTreeMap::new();
        if val.abs() > 1e-15 {
            blades.insert(BladeMask::scalar(), val);
        }
        Self { sig, blades }
    }

    /// Create a basis vector $e_i$.
    pub fn basis_vector(i: usize, sig: CliffordSignature) -> AlgebraResult<Self> {
        if i >= sig.dim() {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Basis vector index {} exceeds dimension {}",
                i,
                sig.dim()
            )));
        }
        let mut blades = BTreeMap::new();
        blades.insert(BladeMask::vector(i), 1.0);
        Ok(Self { sig, blades })
    }

    /// Create a 1-vector from Cartesian coordinates $(v_0, \dots, v_{n-1})$.
    pub fn from_vector(coords: &[f64], sig: CliffordSignature) -> AlgebraResult<Self> {
        if coords.len() > sig.dim() {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Coordinate vector length {} exceeds dimension {}",
                coords.len(),
                sig.dim()
            )));
        }
        let mut blades = BTreeMap::new();
        for (i, &v) in coords.iter().enumerate() {
            if v.abs() > 1e-15 {
                blades.insert(BladeMask::vector(i), v);
            }
        }
        Ok(Self { sig, blades })
    }

    /// Multivector Addition: $A + B$.
    pub fn add(&self, other: &Self) -> AlgebraResult<Self> {
        if self.sig != other.sig {
            return Err(AlgebraError::EvaluationError(
                "Cannot add multivectors with different Clifford signatures".into(),
            ));
        }
        let mut res = self.blades.clone();
        for (mask, &coeff) in &other.blades {
            *res.entry(*mask).or_insert(0.0) += coeff;
        }
        res.retain(|_, v| v.abs() > 1e-14);
        Ok(Self {
            sig: self.sig,
            blades: res,
        })
    }

    /// Multivector Subtraction: $A - B$.
    pub fn sub(&self, other: &Self) -> AlgebraResult<Self> {
        self.add(&other.scale(-1.0))
    }

    /// Scalar multiplication: $c \cdot A$.
    pub fn scale(&self, scalar: f64) -> Self {
        let mut res = BTreeMap::new();
        if scalar.abs() > 1e-15 {
            for (mask, &c) in &self.blades {
                let val = c * scalar;
                if val.abs() > 1e-14 {
                    res.insert(*mask, val);
                }
            }
        }
        Self {
            sig: self.sig,
            blades: res,
        }
    }

    /// Geometric Product $A B$.
    pub fn geometric_product(&self, other: &Self) -> AlgebraResult<Self> {
        if self.sig != other.sig {
            return Err(AlgebraError::EvaluationError(
                "Cannot multiply multivectors with different Clifford signatures".into(),
            ));
        }
        let mut res = BTreeMap::new();

        for (m1, &c1) in &self.blades {
            for (m2, &c2) in &other.blades {
                let (res_mask, sign) = m1.multiply(m2, &self.sig);
                if sign != 0 {
                    let term = c1 * c2 * (sign as f64);
                    *res.entry(res_mask).or_insert(0.0) += term;
                }
            }
        }

        res.retain(|_, v| v.abs() > 1e-14);
        Ok(Self {
            sig: self.sig,
            blades: res,
        })
    }

    /// Outer Product (Wedge Product) $A \wedge B = \sum_{r, s} \langle \langle A \rangle_r \langle B \rangle_s \rangle_{r+s}$.
    pub fn wedge(&self, other: &Self) -> AlgebraResult<Self> {
        if self.sig != other.sig {
            return Err(AlgebraError::EvaluationError(
                "Signatures must match for wedge product".into(),
            ));
        }
        let mut res = BTreeMap::new();

        for (m1, &c1) in &self.blades {
            for (m2, &c2) in &other.blades {
                if (m1.0 & m2.0) == 0 {
                    let (res_mask, sign) = m1.multiply(m2, &self.sig);
                    if sign != 0 && res_mask.grade() == m1.grade() + m2.grade() {
                        let term = c1 * c2 * (sign as f64);
                        *res.entry(res_mask).or_insert(0.0) += term;
                    }
                }
            }
        }

        res.retain(|_, v| v.abs() > 1e-14);
        Ok(Self {
            sig: self.sig,
            blades: res,
        })
    }

    /// Inner Product (Left Contraction) $A \cdot B = \sum_{r, s} \langle \langle A \rangle_r \langle B \rangle_s \rangle_{|r - s|}$.
    pub fn dot(&self, other: &Self) -> AlgebraResult<Self> {
        if self.sig != other.sig {
            return Err(AlgebraError::EvaluationError(
                "Signatures must match for inner product".into(),
            ));
        }
        let mut res = BTreeMap::new();

        for (m1, &c1) in &self.blades {
            for (m2, &c2) in &other.blades {
                let g1 = m1.grade();
                let g2 = m2.grade();
                if g1 > 0 && g2 > 0 {
                    let (res_mask, sign) = m1.multiply(m2, &self.sig);
                    if sign != 0 && res_mask.grade() == (g1 as isize - g2 as isize).unsigned_abs() {
                        let term = c1 * c2 * (sign as f64);
                        *res.entry(res_mask).or_insert(0.0) += term;
                    }
                }
            }
        }

        res.retain(|_, v| v.abs() > 1e-14);
        Ok(Self {
            sig: self.sig,
            blades: res,
        })
    }

    /// Grade Projection $\langle A \rangle_k$.
    pub fn grade_project(&self, k: usize) -> Self {
        let mut res = BTreeMap::new();
        for (m, &c) in &self.blades {
            if m.grade() == k {
                res.insert(*m, c);
            }
        }
        Self {
            sig: self.sig,
            blades: res,
        }
    }

    /// Reversion involution $A^\dagger = \sum_k (-1)^{k(k-1)/2} \langle A \rangle_k$.
    pub fn reverse(&self) -> Self {
        let mut res = BTreeMap::new();
        for (m, &c) in &self.blades {
            let k = m.grade() as i64;
            let sign = if (k * (k - 1) / 2) % 2 == 0 {
                1.0
            } else {
                -1.0
            };
            res.insert(*m, c * sign);
        }
        Self {
            sig: self.sig,
            blades: res,
        }
    }

    /// Grade Involution $\widehat{A} = \sum_k (-1)^k \langle A \rangle_k$.
    pub fn grade_involution(&self) -> Self {
        let mut res = BTreeMap::new();
        for (m, &c) in &self.blades {
            let sign = if m.grade() % 2 == 0 { 1.0 } else { -1.0 };
            res.insert(*m, c * sign);
        }
        Self {
            sig: self.sig,
            blades: res,
        }
    }

    /// Clifford Conjugate $\bar{A} = \widehat{A}^\dagger$.
    pub fn conjugate(&self) -> Self {
        self.grade_involution().reverse()
    }

    /// Rotor Sandwich Transformation: $v' = R v R^\dagger$.
    pub fn rotor_sandwich(&self, rotor: &Self) -> AlgebraResult<Self> {
        let r_rev = rotor.reverse();
        let r_v = rotor.geometric_product(self)?;
        r_v.geometric_product(&r_rev)
    }

    /// Construct a 3D rotation Rotor $R = \exp(-\frac{\theta}{2} B) = \cos(\theta/2) - \sin(\theta/2) B$
    /// where $B = e_i \wedge e_j$ is a unit bivector.
    pub fn make_rotor_3d(bivector: &Self, angle_rad: f64) -> AlgebraResult<Self> {
        let half_angle = angle_rad / 2.0;
        let cos_t = half_angle.cos();
        let sin_t = half_angle.sin();

        let scalar_part = Self::scalar(cos_t, bivector.sig);
        let bivector_part = bivector.scale(-sin_t);

        scalar_part.add(&bivector_part)
    }
}

/// Free Graded Tensor Algebra $T(V) = \bigoplus_{k=0}^N V^{\otimes k}$.
#[derive(Debug, Clone, PartialEq)]
pub struct FreeTensor {
    /// Manifold vector space dimension $n$.
    pub dim: usize,
    /// Tensor rank / grade $k$.
    pub rank: usize,
    /// Flat row-major components tensor of size $n^k$.
    pub data: Vec<f64>,
}

impl FreeTensor {
    /// Create a zero tensor of given rank and dimension.
    pub fn zero(dim: usize, rank: usize) -> Self {
        let size = dim.pow(rank as u32);
        Self {
            dim,
            rank,
            data: vec![0.0; size],
        }
    }

    /// Tensor Outer Product $T_1 \otimes T_2$ of rank $k_1 + k_2$.
    pub fn tensor_product(&self, other: &Self) -> AlgebraResult<Self> {
        if self.dim != other.dim {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Vector space dimension mismatch: {} vs {}",
                self.dim, other.dim
            )));
        }
        let res_rank = self.rank + other.rank;
        let res_size = self.dim.pow(res_rank as u32);
        let mut data = Vec::with_capacity(res_size);

        for &v1 in &self.data {
            for &v2 in &other.data {
                data.push(v1 * v2);
            }
        }

        Ok(Self {
            dim: self.dim,
            rank: res_rank,
            data,
        })
    }

    /// Tensor Contraction (Trace) over index positions $(p, q)$ reducing rank by 2.
    pub fn contract(&self, p: usize, q: usize) -> AlgebraResult<Self> {
        if self.rank < 2 {
            return Err(AlgebraError::EvaluationError(
                "Cannot contract tensor with rank less than 2".into(),
            ));
        }
        if p >= self.rank || q >= self.rank || p == q {
            return Err(AlgebraError::IndexOutOfBounds(format!(
                "Invalid contraction indices ({}, {}) for tensor of rank {}",
                p, q, self.rank
            )));
        }

        let (idx_p, idx_q) = if p < q { (p, q) } else { (q, p) };
        let res_rank = self.rank - 2;
        let res_size = self.dim.pow(res_rank as u32);
        let mut res_data = vec![0.0; res_size];

        // Iterate over all elements of the input tensor
        for (flat_in, &val) in self.data.iter().enumerate() {
            if val.abs() < 1e-15 {
                continue;
            }

            // Decode multi-index of flat_in
            let mut rem = flat_in;
            let mut indices = vec![0usize; self.rank];
            for k in (0..self.rank).rev() {
                indices[k] = rem % self.dim;
                rem /= self.dim;
            }

            // Only contract when indices at idx_p and idx_q match
            if indices[idx_p] == indices[idx_q] {
                // Build output multi-index by removing idx_q then idx_p
                let mut out_indices = Vec::with_capacity(res_rank);
                for (k, &v) in indices.iter().enumerate() {
                    if k != idx_p && k != idx_q {
                        out_indices.push(v);
                    }
                }

                // Encode output flat index
                let mut flat_out = 0usize;
                for &idx in &out_indices {
                    flat_out = flat_out * self.dim + idx;
                }

                if flat_out < res_size {
                    res_data[flat_out] += val;
                }
            }
        }

        Ok(Self {
            dim: self.dim,
            rank: res_rank,
            data: res_data,
        })
    }
}
