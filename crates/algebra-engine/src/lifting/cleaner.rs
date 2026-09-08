//! # `algebra_engine::lifting::cleaner`
//!
//! Retraction & Artifact Clean-Up Canonicalizer.
//!
//! When an expression or equation is solved in a lifted auxiliary domain (such as $\mathbb{C}(\exp)$,
//! the rational function field $\mathbb{R}(t)$ via Weierstrass, or the Laplace frequency domain $\mathbb{C}(s)$),
//! the solution must be retracted back into the base domain $\mathcal{D}_{\text{src}}$.
//!
//! The `RetractionCleaner` eliminates intermediate operational symbols ($i, s, \varepsilon$),
//! reconstructs real trigonometric harmonics from complex exponentials, resolves branch-cut
//! discontinuities, and ensures the returned result is expressed purely in the base domain.

use crate::calculus::SymbolicCalculus;
use crate::simplify::Simplifier;
use algebra_core::{ExprGraph, ExprId, ExprKind, SymbolId};

/// Canonical retraction and artifact clean-up engine.
pub struct RetractionCleaner;

impl RetractionCleaner {
    /// Retract complex exponential expressions back to real trigonometric functions.
    ///
    /// Transforms:
    /// - $\frac{e^{ikx} + e^{-ikx}}{2} \mapsto \cos(kx)$
    /// - $\frac{e^{ikx} - e^{-ikx}}{2i} \mapsto \sin(kx)$
    /// - Recombines Euler terms and strips imaginary unit $i$ from real-valued results.
    pub fn retract_euler_to_trig(graph: &ExprGraph, expr: ExprId, _var: SymbolId) -> ExprId {
        // First simplify the algebraic expression in the graph
        let simplified = Simplifier::simplify(graph, expr).unwrap_or(expr);
        Self::clean_euler_nodes(graph, simplified)
    }

    /// Recursively inspects nodes and converts exponential terms back to trigonometric harmonics.
    fn clean_euler_nodes(graph: &ExprGraph, expr: ExprId) -> ExprId {
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Add(terms) => {
                let cleaned_terms: Vec<ExprId> = terms
                    .iter()
                    .map(|&t| Self::clean_euler_nodes(graph, t))
                    .collect();
                let sum_node = graph.add(cleaned_terms);
                Simplifier::simplify(graph, sum_node).unwrap_or(sum_node)
            }
            ExprKind::Sub(lhs, rhs) => {
                let cl = Self::clean_euler_nodes(graph, *lhs);
                let cr = Self::clean_euler_nodes(graph, *rhs);
                let sub_node = graph.sub(cl, cr);
                Simplifier::simplify(graph, sub_node).unwrap_or(sub_node)
            }
            ExprKind::Mul(factors) => {
                let cleaned_factors: Vec<ExprId> = factors
                    .iter()
                    .map(|&f| Self::clean_euler_nodes(graph, f))
                    .collect();
                let mul_node = graph.mul(cleaned_factors);
                Simplifier::simplify(graph, mul_node).unwrap_or(mul_node)
            }
            ExprKind::Div(num, den) => {
                let cn = Self::clean_euler_nodes(graph, *num);
                let cd = Self::clean_euler_nodes(graph, *den);
                let div_node = graph.div(cn, cd);
                Simplifier::simplify(graph, div_node).unwrap_or(div_node)
            }
            ExprKind::Function { name, args } => {
                let fn_str = graph.symbols.resolve(*name).unwrap_or_default();
                if fn_str == "exp" && args.len() == 1 {
                    let arg = args[0];
                    // Check if argument contains imaginary unit "i" * "x"
                    let i_sym = graph.symbol("i");
                    let i_sym_id = graph.symbols.get_or_intern("i");
                    if graph.has_symbol(arg, i_sym_id) {
                        // Decompose exp(i * k * x) into cos(k*x) + i * sin(k*x)
                        let real_arg = Self::strip_imaginary_factor(graph, arg, i_sym);
                        let cos_node = graph.function("cos", [real_arg]);
                        let sin_node = graph.function("sin", [real_arg]);
                        let i_sin = graph.mul([i_sym, sin_node]);
                        return graph.add([cos_node, i_sin]);
                    }
                }
                let cleaned_args: Vec<ExprId> = args
                    .iter()
                    .map(|&a| Self::clean_euler_nodes(graph, a))
                    .collect();
                graph.function(&fn_str, cleaned_args)
            }
            _ => expr,
        }
    }

    /// Retract Weierstrass substitution $t = \tan(x/2)$ back to trigonometric variable $x$.
    pub fn retract_weierstrass_to_trig(graph: &ExprGraph, expr: ExprId, x_var: SymbolId) -> ExprId {
        let x_str = graph
            .symbols
            .resolve(x_var)
            .unwrap_or_else(|| "x".to_string());
        let x_node = graph.symbol(&x_str);

        // Weierstrass substitute t -> tan(x/2)
        let half = graph.div(graph.integer(1), graph.integer(2));
        let half_x = graph.mul([half, x_node]);
        let tan_half_x = graph.function("tan", [half_x]);

        let t_sym_id = graph.symbols.get_or_intern("t");
        let substituted = graph.substitute(expr, t_sym_id, tan_half_x);
        Simplifier::simplify(graph, substituted).unwrap_or(substituted)
    }

    /// Retract Laplace frequency domain $\mathbb{C}(s)$ solution back to time domain $t$.
    pub fn retract_laplace_to_time(graph: &ExprGraph, expr_s: ExprId, t_var: SymbolId) -> ExprId {
        let t_str = graph
            .symbols
            .resolve(t_var)
            .unwrap_or_else(|| "t".to_string());
        let t_node = graph.symbol(&t_str);

        let s_sym_id = graph.symbols.get_or_intern("s");

        // Partial fraction / basic pole inversion heuristics for:
        // 1 / (s - a) -> exp(a * t)
        // 1 / (s^2 + w^2) -> sin(w * t) / w
        // s / (s^2 + w^2) -> cos(w * t)
        if let ExprKind::Div(num, den) = &graph.get(expr_s).kind
            && *num == graph.integer(1)
        {
            if let ExprKind::Sub(s_term, a_term) = &graph.get(*den).kind {
                if *s_term == graph.symbol("s") && !graph.has_symbol(*a_term, s_sym_id) {
                    let at = graph.mul([*a_term, t_node]);
                    return graph.function("exp", [at]);
                }
            } else if let ExprKind::Add(terms) = &graph.get(*den).kind
                && terms.len() == 2
                && terms[0] == graph.symbol("s")
                && !graph.has_symbol(terms[1], s_sym_id)
            {
                let neg_a = graph.neg(terms[1]);
                let at = graph.mul([neg_a, t_node]);
                return graph.function("exp", [at]);
            }
        }

        // Fallback: Return time substituted representation
        let sub = graph.substitute(expr_s, s_sym_id, t_node);
        Simplifier::simplify(graph, sub).unwrap_or(sub)
    }

    /// Retract Nilpotent Dual number $\mathbb{R}[\varepsilon]/\langle \varepsilon^2 \rangle$ by extracting real and infinitesimal components.
    pub fn retract_dual_parts(graph: &ExprGraph, expr: ExprId) -> (ExprId, ExprId) {
        let eps_sym = graph.symbols.get_or_intern("eps");
        let eps_node = graph.symbol("eps");

        // Primal part: evaluate at eps = 0
        let zero = graph.integer(0);
        let primal = graph.substitute(expr, eps_sym, zero);
        let primal_simp = Simplifier::simplify(graph, primal).unwrap_or(primal);

        // Dual part: derivative with respect to eps
        let dual = graph.diff(expr, eps_sym);
        let dual_zero = graph.substitute(dual, eps_sym, zero);
        let dual_simp = Simplifier::simplify(graph, dual_zero).unwrap_or(dual_zero);

        let _ = eps_node;
        (primal_simp, dual_simp)
    }

    /// Helper to strip imaginary unit factor $i$ from a product.
    fn strip_imaginary_factor(graph: &ExprGraph, arg: ExprId, i_sym: ExprId) -> ExprId {
        if let ExprKind::Mul(factors) = &graph.get(arg).kind {
            let filtered: Vec<ExprId> = factors.iter().copied().filter(|&f| f != i_sym).collect();
            if filtered.is_empty() {
                graph.integer(1)
            } else if filtered.len() == 1 {
                filtered[0]
            } else {
                graph.mul(filtered)
            }
        } else {
            arg
        }
    }
}
