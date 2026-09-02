//! Algebraic Format Transformations (`expand`, `collect`, `together`, `cancel`).

use algebra_core::{ExprGraph, ExprId, ExprKind, SymbolId};

/// Algebraic Format Transformer.
pub struct AlgebraicTransformations;

impl AlgebraicTransformations {
    /// Expand products and polynomial powers: (a + b)(c + d) -> a*c + a*d + b*c + b*d, (a + b)^2 -> a^2 + 2*a*b + b^2.
    pub fn expand(graph: &ExprGraph, expr: ExprId) -> ExprId {
        tracing::debug!(expr = ?expr, "Performing algebraic expansion");
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Mul(factors) => {
                if factors.len() == 2 {
                    let f0 = Self::expand(graph, factors[0]);
                    let f1 = Self::expand(graph, factors[1]);

                    let node0 = graph.get(f0);
                    let node1 = graph.get(f1);

                    match (&node0.kind, &node1.kind) {
                        (ExprKind::Add(t0), ExprKind::Add(t1)) => {
                            let mut expanded = Vec::new();
                            for &a in t0.as_slice() {
                                for &b in t1.as_slice() {
                                    let prod = graph.mul([a, b]);
                                    expanded.push(Self::expand(graph, prod));
                                }
                            }
                            graph.add(expanded)
                        }
                        (ExprKind::Add(t0), _) => {
                            let mut expanded = Vec::new();
                            for &a in t0.as_slice() {
                                let prod = graph.mul([a, f1]);
                                expanded.push(Self::expand(graph, prod));
                            }
                            graph.add(expanded)
                        }
                        (_, ExprKind::Add(t1)) => {
                            let mut expanded = Vec::new();
                            for &b in t1.as_slice() {
                                let prod = graph.mul([f0, b]);
                                expanded.push(Self::expand(graph, prod));
                            }
                            graph.add(expanded)
                        }
                        _ => graph.mul([f0, f1]),
                    }
                } else {
                    expr
                }
            }
            ExprKind::Pow(base, exp) => {
                let exp_node = graph.get(*exp);
                if let ExprKind::Number(algebra_core::Number::Integer(2)) = exp_node.kind {
                    let expanded_base = Self::expand(graph, *base);
                    let base_node = graph.get(expanded_base);
                    if let ExprKind::Add(terms) = &base_node.kind {
                        if terms.len() == 2 {
                            let a = terms[0];
                            let b = terms[1];
                            let two = graph.integer(2);
                            let a_sq = graph.pow(a, two);
                            let two_ab = graph.mul([two, a, b]);
                            let b_sq = graph.pow(b, two);
                            return graph.add([a_sq, two_ab, b_sq]);
                        }
                    }
                }
                expr
            }
            ExprKind::Add(terms) => {
                let expanded_terms: Vec<ExprId> =
                    terms.iter().map(|&t| Self::expand(graph, t)).collect();
                graph.add(expanded_terms)
            }
            ExprKind::Sub(l, r) => {
                let el = Self::expand(graph, *l);
                let er = Self::expand(graph, *r);
                graph.sub(el, er)
            }
            _ => expr,
        }
    }

    /// Collect terms with respect to target symbol $x$: a*x + b*x + c -> (a + b)*x + c.
    pub fn collect(graph: &ExprGraph, expr: ExprId, target: SymbolId) -> ExprId {
        tracing::debug!(expr = ?expr, target = ?target, "Performing term collection");
        let node = graph.get(expr);
        let target_sym = graph.symbol(&graph.symbols.resolve(target).unwrap_or_default());

        match &node.kind {
            ExprKind::Add(terms) => {
                let mut target_coeffs = Vec::new();
                let mut rest_terms = Vec::new();

                for &t in terms.as_slice() {
                    let t_node = graph.get(t);
                    match &t_node.kind {
                        ExprKind::Mul(factors) => {
                            if factors.contains(&target_sym) {
                                let other_factors: Vec<ExprId> = factors
                                    .iter()
                                    .copied()
                                    .filter(|&f| f != target_sym)
                                    .collect();
                                if other_factors.is_empty() {
                                    target_coeffs.push(graph.integer(1));
                                } else {
                                    target_coeffs.push(graph.mul(other_factors));
                                }
                            } else {
                                rest_terms.push(t);
                            }
                        }
                        ExprKind::Symbol(s) if *s == target => {
                            target_coeffs.push(graph.integer(1));
                        }
                        _ => rest_terms.push(t),
                    }
                }

                if !target_coeffs.is_empty() {
                    let coeff_sum = graph.add(target_coeffs);
                    let collected_term = graph.mul([coeff_sum, target_sym]);
                    rest_terms.push(collected_term);
                    graph.add(rest_terms)
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }

    /// Combine sums of rational expressions into a single common fraction: a/b + c/d -> (a*d + b*c) / (b*d).
    pub fn together(graph: &ExprGraph, expr: ExprId) -> ExprId {
        tracing::debug!(expr = ?expr, "Performing rational fraction combination (together)");
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Add(terms) => {
                if terms.len() == 2 {
                    let t0 = graph.get(terms[0]);
                    let t1 = graph.get(terms[1]);

                    match (&t0.kind, &t1.kind) {
                        (ExprKind::Div(a, b), ExprKind::Div(c, d)) => {
                            let ad = graph.mul([*a, *d]);
                            let bc = graph.mul([*b, *c]);
                            let num = graph.add([ad, bc]);
                            let den = graph.mul([*b, *d]);
                            graph.div(num, den)
                        }
                        (ExprKind::Div(a, b), _) => {
                            let b_t1 = graph.mul([*b, terms[1]]);
                            let num = graph.add([*a, b_t1]);
                            graph.div(num, *b)
                        }
                        (_, ExprKind::Div(c, d)) => {
                            let d_t0 = graph.mul([*d, terms[0]]);
                            let num = graph.add([d_t0, *c]);
                            graph.div(num, *d)
                        }
                        _ => expr,
                    }
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }

    /// Cancel common factors in fraction numerator and denominator.
    pub fn cancel(graph: &ExprGraph, expr: ExprId) -> ExprId {
        tracing::debug!(expr = ?expr, "Performing fraction factor cancellation");
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Div(num, den) => {
                let simp_num = crate::Simplifier::simplify(graph, *num).unwrap_or(*num);
                let simp_den = crate::Simplifier::simplify(graph, *den).unwrap_or(*den);
                if simp_num == simp_den {
                    graph.integer(1)
                } else {
                    graph.div(simp_num, simp_den)
                }
            }
            _ => expr,
        }
    }

    /// Convert polynomial into Horner's nested multiplication form: a*x^3 + b*x^2 + c*x + d -> ((a*x + b)*x + c)*x + d.
    pub fn horner(graph: &ExprGraph, expr: ExprId, target: SymbolId) -> ExprId {
        use crate::calculus::SymbolicCalculus;
        use crate::numeric::NumericalEval;

        tracing::debug!(expr = ?expr, target = ?target, "Converting polynomial to Horner form");

        let target_sym = graph.symbol(&graph.symbols.resolve(target).unwrap_or_default());
        let zero = graph.integer(0);
        let ctx = crate::numeric::EvalContext::default();

        let dt = graph.diff(expr, target);
        let d2t = graph.diff(dt, target);

        let f0 = graph.substitute(expr, target, zero);
        let f1 = graph.substitute(dt, target, zero);
        let f2 = graph.substitute(d2t, target, zero);

        let c = graph.evalf(f0, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
        let b = graph.evalf(f1, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
        let a = graph.evalf(f2, &ctx).map(|v| v.to_f64()).unwrap_or(0.0) / 2.0;

        if a.abs() > 1e-12 {
            let a_node = if a.fract() == 0.0 {
                graph.integer(a as i64)
            } else {
                graph.float(a)
            };
            let b_node = if b.fract() == 0.0 {
                graph.integer(b as i64)
            } else {
                graph.float(b)
            };
            let c_node = if c.fract() == 0.0 {
                graph.integer(c as i64)
            } else {
                graph.float(c)
            };

            let ax_plus_b = graph.add([graph.mul([a_node, target_sym]), b_node]);
            let horner_inner = graph.mul([ax_plus_b, target_sym]);
            graph.add([horner_inner, c_node])
        } else {
            expr
        }
    }

    /// Factor quadratic polynomial into irreducible factors: a*x^2 + b*x + c -> a * (x - r1) * (x - r2).
    pub fn factor(graph: &ExprGraph, expr: ExprId) -> ExprId {
        use crate::calculus::SymbolicCalculus;
        use crate::numeric::NumericalEval;

        tracing::debug!(expr = ?expr, "Factoring polynomial expression");

        let x_sym = graph.symbols.get_or_intern("x");
        let target_sym = graph.symbol("x");
        let zero = graph.integer(0);
        let ctx = crate::numeric::EvalContext::default();

        let dt = graph.diff(expr, x_sym);
        let d2t = graph.diff(dt, x_sym);

        let f0 = graph.substitute(expr, x_sym, zero);
        let f1 = graph.substitute(dt, x_sym, zero);
        let f2 = graph.substitute(d2t, x_sym, zero);

        let c_val = graph.evalf(f0, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
        let b_val = graph.evalf(f1, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
        let a_val = graph.evalf(f2, &ctx).map(|v| v.to_f64()).unwrap_or(0.0) / 2.0;

        if a_val.abs() > 1e-12 {
            let disc = b_val * b_val - 4.0 * a_val * c_val;
            if disc >= 0.0 {
                let r1 = (-b_val + disc.sqrt()) / (2.0 * a_val);
                let r2 = (-b_val - disc.sqrt()) / (2.0 * a_val);

                let r1_node = if r1.fract() == 0.0 {
                    graph.integer(r1 as i64)
                } else {
                    graph.float(r1)
                };
                let r2_node = if r2.fract() == 0.0 {
                    graph.integer(r2 as i64)
                } else {
                    graph.float(r2)
                };

                let f1 = graph.sub(target_sym, r1_node);
                let f2 = graph.sub(target_sym, r2_node);

                if (a_val - 1.0).abs() < 1e-12 {
                    graph.mul([f1, f2])
                } else {
                    let a_node = if a_val.fract() == 0.0 {
                        graph.integer(a_val as i64)
                    } else {
                        graph.float(a_val)
                    };
                    graph.mul([a_node, f1, f2])
                }
            } else {
                expr
            }
        } else {
            expr
        }
    }

    /// Decompose rational function into partial fractions: P(x) / Q(x) -> A / (x - r1) + B / (x - r2).
    pub fn partial_fractions(graph: &ExprGraph, expr: ExprId, target: SymbolId) -> ExprId {
        use crate::calculus::SymbolicCalculus;
        use crate::numeric::NumericalEval;

        tracing::debug!(expr = ?expr, target = ?target, "Performing partial fractions decomposition");

        let node = graph.get(expr);
        let target_sym = graph.symbol(&graph.symbols.resolve(target).unwrap_or_default());

        match &node.kind {
            ExprKind::Div(num, den) => {
                let zero = graph.integer(0);
                let ctx = crate::numeric::EvalContext::default();

                let dt = graph.diff(*den, target);
                let d2t = graph.diff(dt, target);

                let f0 = graph.substitute(*den, target, zero);
                let f1 = graph.substitute(dt, target, zero);
                let f2 = graph.substitute(d2t, target, zero);

                let c_val = graph.evalf(f0, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
                let b_val = graph.evalf(f1, &ctx).map(|v| v.to_f64()).unwrap_or(0.0);
                let a_val = graph.evalf(f2, &ctx).map(|v| v.to_f64()).unwrap_or(0.0) / 2.0;

                if a_val.abs() > 1e-12 {
                    let disc = b_val * b_val - 4.0 * a_val * c_val;
                    if disc > 1e-12 {
                        let r1 = (-b_val + disc.sqrt()) / (2.0 * a_val);
                        let r2 = (-b_val - disc.sqrt()) / (2.0 * a_val);

                        let r1_node = if r1.fract() == 0.0 {
                            graph.integer(r1 as i64)
                        } else {
                            graph.float(r1)
                        };
                        let r2_node = if r2.fract() == 0.0 {
                            graph.integer(r2 as i64)
                        } else {
                            graph.float(r2)
                        };

                        let num_r1 = graph
                            .evalf(graph.substitute(*num, target, r1_node), &ctx)
                            .map(|v| v.to_f64())
                            .unwrap_or(1.0);
                        let num_r2 = graph
                            .evalf(graph.substitute(*num, target, r2_node), &ctx)
                            .map(|v| v.to_f64())
                            .unwrap_or(1.0);

                        let a_coeff = num_r1 / (a_val * (r1 - r2));
                        let b_coeff = num_r2 / (a_val * (r2 - r1));

                        let a_node = if a_coeff.fract() == 0.0 {
                            graph.integer(a_coeff as i64)
                        } else {
                            graph.float(a_coeff)
                        };
                        let b_node = if b_coeff.fract() == 0.0 {
                            graph.integer(b_coeff as i64)
                        } else {
                            graph.float(b_coeff)
                        };

                        let term1 = graph.div(a_node, graph.sub(target_sym, r1_node));
                        let term2 = graph.div(b_node, graph.sub(target_sym, r2_node));

                        graph.add([term1, term2])
                    } else {
                        expr
                    }
                } else {
                    expr
                }
            }
            _ => expr,
        }
    }
}
