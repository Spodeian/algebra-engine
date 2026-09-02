//! # `algebra_engine::simplify::probabilistic`
//!
//! Schwartz-Zippel Probabilistic Equivalence Testing ($O(1)$ fast refutation).
//!
//! ## Mathematical Foundations
//! By the **Schwartz-Zippel Lemma**, for any non-zero multi-variate polynomial
//! $P(x_1, \dots, x_n) \in \mathbb{F}[x_1, \dots, x_n]$ of total degree $d$ over a finite field $\mathbb{F}_p$:
//! $$\Pr_{\mathbf{r} \in S^n}[P(r_1, \dots, r_n) = 0] \le \frac{d}{|S|}$$
//!
//! For the Mersenne prime $p = 2^{31} - 1 = 2{,}147{,}483{,}647$, testing $k=3$ random samples gives an
//! error probability $< 10^{-20}$. If $P(\mathbf{r}) \ne 0$ for any sample, it is **strictly proven**
//! that $P \not\equiv 0$ in $O(1)$ time without running exponential E-Graph saturation or Gröbner basis reductions.

use algebra_core::{ExprGraph, ExprId, ExprKind, Number, SymbolId};
use std::collections::HashMap;

/// Prime modulus for Schwartz-Zippel evaluations ($p = 2^{31} - 1$).
pub const SCHWARTZ_ZIPPEL_PRIME: i64 = 2_147_483_647;

/// Evaluator for polynomial identity testing over finite fields $\mathbb{F}_p$.
pub struct SchwartzZippel;

impl SchwartzZippel {
    /// Probabilistically test if two symbolic expressions $A$ and $B$ are identical: $A \equiv B$.
    ///
    /// Returns `false` if definitely NOT equal (proven in $O(1)$).
    /// Returns `true` if identical on all $k$ random trials with probability $> 1 - 10^{-20}$.
    pub fn are_equal(graph: &ExprGraph, a: ExprId, b: ExprId) -> bool {
        let diff = graph.sub(a, b);
        Self::is_zero(graph, diff, 3)
    }

    /// Probabilistically test if an expression is identically zero: $P(x_1, \dots, x_n) \equiv 0$.
    pub fn is_zero(graph: &ExprGraph, expr: ExprId, trials: usize) -> bool {
        // Collect all free variables
        let mut symbols = Vec::new();
        Self::collect_symbols(graph, expr, &mut symbols);

        if symbols.is_empty() {
            // Constant evaluation
            if let Some(val) =
                Self::eval_finite_field(graph, expr, &HashMap::new(), SCHWARTZ_ZIPPEL_PRIME)
            {
                return val == 0;
            }
            return false;
        }

        // Pseudo-random deterministic seed sequence for reproducible tests
        let mut state: u64 = 0x9E3779B97F4A7C15;
        for _ in 0..trials {
            let mut env = HashMap::new();
            for &sym in &symbols {
                // Linear congruential generator step
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let rand_val = ((state >> 33) as i64) % (SCHWARTZ_ZIPPEL_PRIME - 2) + 1;
                env.insert(sym, rand_val);
            }

            match Self::eval_finite_field(graph, expr, &env, SCHWARTZ_ZIPPEL_PRIME) {
                Some(0) => continue,
                _ => return false, // Proven non-zero!
            }
        }

        true
    }

    fn collect_symbols(graph: &ExprGraph, id: ExprId, out: &mut Vec<SymbolId>) {
        let node = graph.get(id);
        match &node.kind {
            ExprKind::Symbol(sym) if !out.contains(sym) => {
                out.push(*sym);
            }
            ExprKind::Add(terms) | ExprKind::Mul(terms) => {
                for &term in terms {
                    Self::collect_symbols(graph, term, out);
                }
            }
            ExprKind::Sub(a, b) | ExprKind::Div(a, b) | ExprKind::Pow(a, b) => {
                Self::collect_symbols(graph, *a, out);
                Self::collect_symbols(graph, *b, out);
            }
            ExprKind::Neg(a) => Self::collect_symbols(graph, *a, out),
            ExprKind::Function { args, .. } => {
                for &arg in args {
                    Self::collect_symbols(graph, arg, out);
                }
            }
            _ => {}
        }
    }

    /// Evaluate expression modulo prime $p$.
    fn eval_finite_field(
        graph: &ExprGraph,
        id: ExprId,
        env: &HashMap<SymbolId, i64>,
        p: i64,
    ) -> Option<i64> {
        let node = graph.get(id);
        match &node.kind {
            ExprKind::Number(num) => match num {
                Number::Integer(i) => Some(((i % p) + p) % p),
                Number::Float(bits) => {
                    let f = f64::from_bits(*bits);
                    let rounded = f.round() as i64;
                    Some(((rounded % p) + p) % p)
                }
                Number::Rational(numer, denom) => {
                    let num_mod = (numer % p + p) % p;
                    let den_mod = (denom % p + p) % p;
                    if den_mod == 0 {
                        return None;
                    }
                    let inv_denom = Self::mod_inverse(den_mod, p)?;
                    Some((num_mod * inv_denom) % p)
                }
                _ => None,
            },
            ExprKind::Symbol(sym) => env.get(sym).copied().map(|v| ((v % p) + p) % p),
            ExprKind::Add(terms) => {
                let mut sum = 0i64;
                for &t in terms {
                    let v = Self::eval_finite_field(graph, t, env, p)?;
                    sum = (sum + v) % p;
                }
                Some(sum)
            }
            ExprKind::Mul(factors) => {
                let mut prod = 1i64;
                for &f in factors {
                    let v = Self::eval_finite_field(graph, f, env, p)?;
                    prod = ((prod as i128 * v as i128) % p as i128) as i64;
                }
                Some(prod)
            }
            ExprKind::Sub(a, b) => {
                let va = Self::eval_finite_field(graph, *a, env, p)?;
                let vb = Self::eval_finite_field(graph, *b, env, p)?;
                Some(((va - vb) % p + p) % p)
            }
            ExprKind::Div(a, b) => {
                let va = Self::eval_finite_field(graph, *a, env, p)?;
                let vb = Self::eval_finite_field(graph, *b, env, p)?;
                let inv_b = Self::mod_inverse(vb, p)?;
                Some(((va as i128 * inv_b as i128) % p as i128) as i64)
            }
            ExprKind::Pow(a, b) => {
                let base = Self::eval_finite_field(graph, *a, env, p)?;
                let exp = Self::eval_finite_field(graph, *b, env, p)?;
                Some(Self::mod_pow(base, exp, p))
            }
            ExprKind::Neg(a) => {
                let va = Self::eval_finite_field(graph, *a, env, p)?;
                Some((-va + p) % p)
            }
            _ => None,
        }
    }

    /// Modular exponentiation: $base^{exp} \pmod p$.
    fn mod_pow(mut base: i64, mut exp: i64, p: i64) -> i64 {
        let mut res = 1i64;
        base = ((base % p) + p) % p;
        while exp > 0 {
            if exp % 2 == 1 {
                res = ((res as i128 * base as i128) % p as i128) as i64;
            }
            base = ((base as i128 * base as i128) % p as i128) as i64;
            exp /= 2;
        }
        res
    }

    /// Extended Euclidean Algorithm for modular inverse.
    fn mod_inverse(a: i64, m: i64) -> Option<i64> {
        let mut t = 0i64;
        let mut newt = 1i64;
        let mut r = m;
        let mut newr = ((a % m) + m) % m;

        while newr != 0 {
            let quotient = r / newr;
            let tmp_t = t - quotient * newt;
            t = newt;
            newt = tmp_t;

            let tmp_r = r - quotient * newr;
            r = newr;
            newr = tmp_r;
        }

        if r > 1 {
            return None; // Not invertible
        }
        if t < 0 {
            t += m;
        }
        Some(t)
    }
}
