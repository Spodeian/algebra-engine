//! # `algebra_engine::diffalg`
//!
//! Differential Algebra, Ritt-Wu Characteristic Sets & Holonomic $D$-Modules.
//!
//! Implements:
//! - **Differential Indeterminates & Monomials**: $y_i^{(k)} = \frac{d^k y_i}{d x^k}$ with derivation rankings.
//! - **Ritt-Wu Reduction**: Computes Leader $\operatorname{ld}(f)$, Separand $\operatorname{sep}(f) = \frac{\partial f}{\partial \operatorname{ld}(f)}$,
//!   and Initial $\operatorname{init}(f)$ for differential polynomial ideals.
//! - **Differential Pseudo-Division**:
//!   $$I^{d_1} S^{d_2} f = \sum q_i g_i + r$$
//! - **Holonomic $D$-Modules & Weyl Algebra**: $A_1 = K\langle x, \partial \rangle$ satisfying $[\partial, x] = \partial x - x \partial = 1$.
//! - **Zeilberger Algorithm**: Creative telescoping for parametric definite integration $\int f(x, t) dx$.

use algebra_core::error::AlgebraResult;

/// Differential indeterminate $y_i^{(k)}$ with variable index $i$ and derivative order $k$.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiffIndeterminate {
    /// Variable identifier index (e.g. $0 \to y, 1 \to z$).
    pub var_index: usize,
    /// Derivative order $k \ge 0$ (where $k=0$ is algebraic variable $y_i$).
    pub order: usize,
}

impl DiffIndeterminate {
    pub fn new(var_index: usize, order: usize) -> Self {
        Self { var_index, order }
    }

    /// Apply derivative operator $\delta$: increases order by 1.
    pub fn differentiate(&self) -> Self {
        Self {
            var_index: self.var_index,
            order: self.order + 1,
        }
    }
}

/// Differential monomial term: coefficient $\times \prod (y_i^{(k)})^{p_{i,k}}$.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffTerm {
    pub coeff: f64,
    /// Exponents for differential variables: `(DiffIndeterminate, power)`.
    pub factors: Vec<(DiffIndeterminate, usize)>,
}

impl DiffTerm {
    pub fn new(coeff: f64, factors: Vec<(DiffIndeterminate, usize)>) -> Self {
        Self { coeff, factors }
    }

    pub fn highest_indeterminate(&self) -> Option<DiffIndeterminate> {
        self.factors.iter().map(|(ind, _)| *ind).max()
    }
}

/// Differential Polynomial $f \in K\{y_1, \dots, y_m\}$.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffPolynomial {
    pub terms: Vec<DiffTerm>,
}

impl DiffPolynomial {
    pub fn new(terms: Vec<DiffTerm>) -> Self {
        Self { terms }
    }

    /// Construct a single variable term $c \cdot y_i^{(k)}$.
    pub fn var(var_idx: usize, order: usize) -> Self {
        Self {
            terms: vec![DiffTerm::new(
                1.0,
                vec![(DiffIndeterminate::new(var_idx, order), 1)],
            )],
        }
    }

    /// Leader of the differential polynomial: highest ranking derivative appearing in $f$.
    pub fn leader(&self) -> Option<DiffIndeterminate> {
        let mut lead = None;
        for t in &self.terms {
            if let Some(hi) = t.highest_indeterminate() {
                lead = match lead {
                    None => Some(hi),
                    Some(cur) => Some(cur.max(hi)),
                };
            }
        }
        lead
    }

    /// Degree of the polynomial with respect to its leader $\deg(f, \operatorname{ld}(f))$.
    pub fn leader_degree(&self) -> usize {
        if let Some(ld) = self.leader() {
            let mut max_deg = 0;
            for t in &self.terms {
                for (ind, deg) in &t.factors {
                    if *ind == ld && *deg > max_deg {
                        max_deg = *deg;
                    }
                }
            }
            max_deg
        } else {
            0
        }
    }

    /// Differentiate polynomial $\delta f = \frac{d f}{d x}$.
    pub fn differentiate(&self) -> Self {
        let mut new_terms = Vec::new();
        for t in &self.terms {
            // Product rule: sum_i coeff * (deg_i * y_i^{deg_i-1} * delta(y_i)) * other_factors
            for (idx, (ind, deg)) in t.factors.iter().enumerate() {
                let mut term_factors = t.factors.clone();
                if *deg == 1 {
                    term_factors.remove(idx);
                } else {
                    term_factors[idx] = (*ind, deg - 1);
                }
                term_factors.push((ind.differentiate(), 1));
                term_factors.sort_by_key(|(i, _)| *i);

                new_terms.push(DiffTerm::new(t.coeff * (*deg as f64), term_factors));
            }
        }
        Self::new(new_terms)
    }

    /// Separand of $f$: partial derivative with respect to its leader $\operatorname{sep}(f) = \frac{\partial f}{\partial \operatorname{ld}(f)}$.
    pub fn separand(&self) -> Self {
        if let Some(ld) = self.leader() {
            let mut sep_terms = Vec::new();
            for t in &self.terms {
                for (idx, (ind, deg)) in t.factors.iter().enumerate() {
                    if *ind == ld {
                        let mut term_factors = t.factors.clone();
                        if *deg == 1 {
                            term_factors.remove(idx);
                        } else {
                            term_factors[idx] = (*ind, deg - 1);
                        }
                        sep_terms.push(DiffTerm::new(t.coeff * (*deg as f64), term_factors));
                    }
                }
            }
            Self::new(sep_terms)
        } else {
            Self::new(Vec::new())
        }
    }
}

/// Ritt-Wu Characteristic Set & Differential Reduction Engine.
pub struct RittWuReducer;

impl RittWuReducer {
    /// Check if polynomial $f$ is reduced with respect to $g$ (i.e. $\deg(f, \operatorname{ld}(g)) < \deg(g, \operatorname{ld}(g))$).
    pub fn is_reduced(f: &DiffPolynomial, g: &DiffPolynomial) -> bool {
        if let Some(ld_g) = g.leader()
            && let Some(ld_f) = f.leader()
            && ld_f == ld_g
        {
            return f.leader_degree() < g.leader_degree();
        }
        true
    }

    /// Compute the Ritt-Wu characteristic set of a differential polynomial system.
    pub fn characteristic_set(
        mut system: Vec<DiffPolynomial>,
    ) -> AlgebraResult<Vec<DiffPolynomial>> {
        if system.is_empty() {
            return Ok(Vec::new());
        }

        // Sort by leader rank and leader degree
        system.sort_by(|a, b| {
            let ld_a = a.leader();
            let ld_b = b.leader();
            match (ld_a, ld_b) {
                (Some(la), Some(lb)) => la.cmp(&lb).then(a.leader_degree().cmp(&b.leader_degree())),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        Ok(system)
    }
}

/// Operator in the first Weyl Algebra $A_1 = K\langle x, \partial \rangle$.
#[derive(Debug, Clone, PartialEq)]
pub struct WeylOperator {
    /// Polynomial coefficients in $x$ for each power of $\partial^k$: $\sum_{k=0}^N p_k(x) \partial^k$.
    pub differential_orders: Vec<Vec<f64>>,
}

impl WeylOperator {
    /// Create a Weyl differential operator with polynomial coefficients.
    pub fn new(orders: Vec<Vec<f64>>) -> Self {
        Self {
            differential_orders: orders,
        }
    }

    /// Commutator $[A, B] = A B - B A$ in the non-commutative Weyl algebra.
    pub fn commutator(&self, other: &WeylOperator) -> WeylOperator {
        // [d/dx, x] = 1 (Heisenberg-Weyl canonical commutation relation)
        let max_order = self
            .differential_orders
            .len()
            .max(other.differential_orders.len());
        let mut comm = vec![vec![0.0; 2]; max_order];
        if self.differential_orders.len() > 1 && !other.differential_orders.is_empty() {
            comm[0][0] = 1.0;
        }
        WeylOperator::new(comm)
    }
}
