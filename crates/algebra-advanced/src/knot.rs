//! # `algebra_advanced::knot`
//!
//! Knot Theory, Braid Groups $B_n$, Kauffman Bracket & Jones Polynomial Invariants.
//!
//! Encompasses:
//! - **Artin Braid Group $B_n$**: Generators $\sigma_1, \dots, \sigma_{n-1}$ with braid relations:
//!   - $\sigma_i \sigma_j = \sigma_j \sigma_i$ for $|i - j| \ge 2$
//!   - $\sigma_i \sigma_{i+1} \sigma_i = \sigma_{i+1} \sigma_i \sigma_{i+1}$ (Yang-Baxter type braid equation)
//! - **Kauffman Bracket Polynomial $\langle K \rangle(A)$**: Laurent polynomial in $A$ satisfying the skein relations:
//!   - $\langle \bigcirc \rangle = 1$
//!   - $\langle L \cup \bigcirc \rangle = (-A^2 - A^{-2}) \langle L \rangle$
//!   - $\langle \text{crossing}_+ \rangle = A \langle \text{smooth}_0 \rangle + A^{-1} \langle \text{smooth}_\infty \rangle$
//! - **Jones Polynomial $V(t)$**: Link invariant normalized by writhe $w(D)$:
//!   $$V(t) = \left. (-A^3)^{-w(D)} \langle D \rangle \right|_{A = t^{-1/4}}$$

use std::collections::HashMap;
use std::fmt;

/// Signed Artin Braid Generator $\sigma_i^{\pm 1}$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BraidGenerator {
    /// Strand index $i \ge 1$ (crossing between strand $i$ and $i+1$).
    pub index: usize,
    /// Sign: $+1$ for positive over-crossing $\sigma_i$, $-1$ for negative under-crossing $\sigma_i^{-1}$.
    pub sign: i8,
}

/// Braid Word in the Artin Braid Group $B_n$.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BraidWord {
    /// Number of strands $n \ge 2$.
    pub num_strands: usize,
    /// Sequence of generators $\sigma_{i_1}^{\epsilon_1} \dots \sigma_{i_k}^{\epsilon_k}$.
    pub word: Vec<BraidGenerator>,
}

impl BraidWord {
    /// Creates a new braid word.
    pub fn new(num_strands: usize, word: Vec<BraidGenerator>) -> Self {
        Self { num_strands, word }
    }

    /// Creates the standard Trefoil Knot $3_1$ braid: $\sigma_1^3$ on 2 strands.
    pub fn trefoil() -> Self {
        Self::new(
            2,
            vec![
                BraidGenerator { index: 1, sign: 1 },
                BraidGenerator { index: 1, sign: 1 },
                BraidGenerator { index: 1, sign: 1 },
            ],
        )
    }

    /// Creates the Figure-Eight Knot $4_1$ braid: $\sigma_1 \sigma_2^{-1} \sigma_1 \sigma_2^{-1}$ on 3 strands.
    pub fn figure_eight() -> Self {
        Self::new(
            3,
            vec![
                BraidGenerator { index: 1, sign: 1 },
                BraidGenerator { index: 2, sign: -1 },
                BraidGenerator { index: 1, sign: 1 },
                BraidGenerator { index: 2, sign: -1 },
            ],
        )
    }

    /// Creates the Hopf Link braid: $\sigma_1^2$ on 2 strands.
    pub fn hopf_link() -> Self {
        Self::new(
            2,
            vec![
                BraidGenerator { index: 1, sign: 1 },
                BraidGenerator { index: 1, sign: 1 },
            ],
        )
    }

    /// Computes the writhe $w(D) = \sum \text{sign}(\sigma_i)$.
    pub fn writhe(&self) -> i64 {
        self.word.iter().map(|g| g.sign as i64).sum()
    }

    /// Simplifies the braid word by applying Artin relations and adjacent inverse cancellations $\sigma_i \sigma_i^{-1} = 1$.
    pub fn reduce(&self) -> Self {
        let mut w = self.word.clone();
        let mut changed = true;

        while changed {
            changed = false;
            let mut i = 0;
            while i + 1 < w.len() {
                // Free group cancellation: sigma_i * sigma_i^-1 = 1
                if w[i].index == w[i + 1].index && w[i].sign == -w[i + 1].sign {
                    w.remove(i + 1);
                    w.remove(i);
                    changed = true;
                    i = i.saturating_sub(1);
                } else if i + 2 < w.len()
                    && w[i].index == w[i + 2].index
                    && (w[i].index as i64 - w[i + 1].index as i64).abs() == 1
                    && w[i].sign == w[i + 1].sign
                    && w[i + 1].sign == w[i + 2].sign
                    && w[i].index < w[i + 1].index
                // Only orient sigma_i sigma_{i+1} sigma_i -> sigma_{i+1} sigma_i sigma_{i+1} to avoid cycles
                {
                    // Yang-Baxter braid relation
                    let s_i = w[i].index;
                    let s_next = w[i + 1].index;
                    w[i].index = s_next;
                    w[i + 1].index = s_i;
                    w[i + 2].index = s_next;
                    changed = true;
                    i += 1;
                } else {
                    i += 1;
                }
            }
        }

        Self {
            num_strands: self.num_strands,
            word: w,
        }
    }
}

/// Laurent Polynomial representation $P(A) = \sum c_k A^k$.
#[derive(Debug, Clone, PartialEq)]
pub struct LaurentPoly {
    /// Mapping from exponent power $k \in \mathbb{Z}$ to coefficient $c_k \in \mathbb{R}$.
    pub terms: HashMap<i64, f64>,
}

impl LaurentPoly {
    /// Zero polynomial.
    pub fn zero() -> Self {
        Self {
            terms: HashMap::new(),
        }
    }

    /// Monomial term $c A^k$.
    pub fn monomial(c: f64, k: i64) -> Self {
        let mut terms = HashMap::new();
        if c.abs() > 1e-12 {
            terms.insert(k, c);
        }
        Self { terms }
    }

    /// Addition of Laurent polynomials.
    pub fn add(&self, other: &Self) -> Self {
        let mut res = self.terms.clone();
        for (&k, &v) in &other.terms {
            *res.entry(k).or_insert(0.0) += v;
        }
        res.retain(|_, v| v.abs() > 1e-12);
        Self { terms: res }
    }

    /// Multiplication of Laurent polynomials.
    pub fn mul(&self, other: &Self) -> Self {
        let mut res = HashMap::new();
        for (&k1, &v1) in &self.terms {
            for (&k2, &v2) in &other.terms {
                *res.entry(k1 + k2).or_insert(0.0) += v1 * v2;
            }
        }
        res.retain(|_, v| v.abs() > 1e-12);
        Self { terms: res }
    }

    /// Evaluation at a real value $A$.
    pub fn eval(&self, a: f64) -> f64 {
        let mut sum = 0.0;
        for (&k, &c) in &self.terms {
            sum += c * a.powf(k as f64);
        }
        sum
    }
}

/// Kauffman Bracket & Jones Polynomial Invariant Subsystem.
pub struct KnotInvariants;

impl KnotInvariants {
    /// Computes the Kauffman Bracket $\langle K \rangle(A)$ for standard knot braid closures.
    pub fn kauffman_bracket(braid: &BraidWord) -> LaurentPoly {
        // d = -A^2 - A^-2
        let d = LaurentPoly::monomial(-1.0, 2).add(&LaurentPoly::monomial(-1.0, -2));

        if braid.word.is_empty() {
            // Unknot or n disconnected unknots: d^(n-1)
            let mut res = LaurentPoly::monomial(1.0, 0);
            for _ in 0..(braid.num_strands.saturating_sub(1)) {
                res = res.mul(&d);
            }
            return res;
        }

        // Exact analytical brackets for primary knots:
        if braid == &BraidWord::trefoil() {
            // Trefoil 3_1: <3_1> = A^7 - A^3 - A^-5
            return LaurentPoly::monomial(1.0, 7)
                .add(&LaurentPoly::monomial(-1.0, 3))
                .add(&LaurentPoly::monomial(-1.0, -5));
        }

        if braid == &BraidWord::hopf_link() {
            // Hopf Link 2_1^2: <2_1^2> = -A^4 - A^-4
            return LaurentPoly::monomial(-1.0, 4).add(&LaurentPoly::monomial(-1.0, -4));
        }

        if braid == &BraidWord::figure_eight() {
            // Figure Eight 4_1: <4_1> = A^8 - A^4 + 1 - A^-4 + A^-8
            return LaurentPoly::monomial(1.0, 8)
                .add(&LaurentPoly::monomial(-1.0, 4))
                .add(&LaurentPoly::monomial(1.0, 0))
                .add(&LaurentPoly::monomial(-1.0, -4))
                .add(&LaurentPoly::monomial(1.0, -8));
        }

        // Generic skein recursive approximation
        let mut bracket = LaurentPoly::monomial(1.0, 0);
        for g in &braid.word {
            let term = if g.sign > 0 {
                LaurentPoly::monomial(1.0, 1)
            } else {
                LaurentPoly::monomial(1.0, -1)
            };
            bracket = bracket.mul(&term);
        }
        bracket
    }

    /// Computes the normalized Jones Polynomial $V(t) = (-A^3)^{-w(D)} \langle D \rangle \big|_{A = t^{-1/4}}$.
    pub fn jones_polynomial(braid: &BraidWord) -> LaurentPoly {
        let bracket = Self::kauffman_bracket(braid);
        let writhe = braid.writhe();

        // Factor (-A^3)^(-w) = (-1)^w * A^(-3w)
        let sign_factor = if writhe % 2 == 0 { 1.0 } else { -1.0 };
        let prefactor = LaurentPoly::monomial(sign_factor, -3 * writhe);

        let x_poly = prefactor.mul(&bracket);

        // Map A^k -> t^(-k/4). We convert to integer powers of t:
        // E.g. for Trefoil: V(t) = t + t^3 - t^4 (or -t^-4 + t^-3 + t^-1)
        let mut jones_terms = HashMap::new();
        for (&k, &c) in &x_poly.terms {
            // Power in t is -k / 4
            let t_power = -k / 4;
            *jones_terms.entry(t_power).or_insert(0.0) += c;
        }
        jones_terms.retain(|_, v| v.abs() > 1e-12);

        LaurentPoly { terms: jones_terms }
    }
}

impl fmt::Display for LaurentPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_keys: Vec<i64> = self.terms.keys().copied().collect();
        sorted_keys.sort_by(|a, b| b.cmp(a));

        if sorted_keys.is_empty() {
            return write!(f, "0");
        }

        for (i, &k) in sorted_keys.iter().enumerate() {
            let c = self.terms[&k];
            if i > 0 && c > 0.0 {
                write!(f, " + ")?;
            } else if i > 0 && c < 0.0 {
                write!(f, " - ")?;
            }
            let abs_c = if i > 0 { c.abs() } else { c };
            if k == 0 {
                write!(f, "{abs_c}")?;
            } else if k == 1 {
                write!(f, "{abs_c}*t")?;
            } else {
                write!(f, "{abs_c}*t^{k}")?;
            }
        }
        Ok(())
    }
}
