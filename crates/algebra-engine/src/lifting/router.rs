//! # `algebra_engine::lifting::router`
//!
//! Functorial Domain Lifting Router & Compatibility Validator.
//!
//! Provides automatic problem transformation from difficult source domains $\mathcal{D}_{\text{src}}$
//! into transformed domains $\mathcal{D}_{\text{lift}}$ (Euler complex exponentials, Weierstrass
//! rational functions, Laplace frequency domain, Nilpotent duals, Clifford rotors), solves the problem
//! algebraically, and retracts back using [`RetractionCleaner`].

use super::cleaner::RetractionCleaner;
use crate::simplify::Simplifier;
use algebra_core::domain::GoalDomain;
use algebra_core::error::{AlgebraError, AlgebraResult};
use algebra_core::{ExprGraph, ExprId, ExprKind, SymbolId};

/// Trait defining a bidirectional Functorial Domain Lifting & Retraction Channel.
pub trait LiftingFunctor: Send + Sync {
    /// Human-readable identifier of the lifting channel.
    fn name(&self) -> &'static str;

    /// Checks if this functor can meaningfully lift the given expression.
    fn can_lift(&self, graph: &ExprGraph, expr: ExprId) -> bool;

    /// Lift an expression from $\mathcal{D}_{\text{src}}$ to $\mathcal{D}_{\text{lift}}$.
    fn lift(&self, graph: &ExprGraph, expr: ExprId, var: SymbolId) -> AlgebraResult<ExprId>;

    /// Retract the solution from $\mathcal{D}_{\text{lift}}$ back to $\mathcal{D}_{\text{src}}$.
    fn retract(&self, graph: &ExprGraph, expr: ExprId, var: SymbolId) -> AlgebraResult<ExprId>;
}

/// Euler Complex Exponential Lifting Functor: $\mathbb{R}[\sin, \cos] \to \mathbb{C}[\exp]$.
pub struct EulerLiftingFunctor;

impl LiftingFunctor for EulerLiftingFunctor {
    fn name(&self) -> &'static str {
        "EulerComplexLifting"
    }

    fn can_lift(&self, graph: &ExprGraph, expr: ExprId) -> bool {
        contains_trig_functions(graph, expr)
    }

    fn lift(&self, graph: &ExprGraph, expr: ExprId, _var: SymbolId) -> AlgebraResult<ExprId> {
        let i_sym = graph.symbol("i");
        let two = graph.integer(2);
        let two_i = graph.mul([two, i_sym]);

        let lifted = Self::transform_trig_to_euler(graph, expr, i_sym, two, two_i);
        Ok(Simplifier::simplify(graph, lifted).unwrap_or(lifted))
    }

    fn retract(&self, graph: &ExprGraph, expr: ExprId, var: SymbolId) -> AlgebraResult<ExprId> {
        Ok(RetractionCleaner::retract_euler_to_trig(graph, expr, var))
    }
}

impl EulerLiftingFunctor {
    fn transform_trig_to_euler(
        graph: &ExprGraph,
        expr: ExprId,
        i_sym: ExprId,
        two: ExprId,
        two_i: ExprId,
    ) -> ExprId {
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Function { name, args } => {
                let fn_str = graph.symbols.resolve(*name).unwrap_or_default();
                if args.len() == 1 {
                    let arg = Self::transform_trig_to_euler(graph, args[0], i_sym, two, two_i);
                    let ix = graph.mul([i_sym, arg]);
                    let neg_ix = graph.neg(ix);
                    let exp_ix = graph.function("exp", [ix]);
                    let exp_neg_ix = graph.function("exp", [neg_ix]);

                    match fn_str.as_str() {
                        "cos" => {
                            let sum = graph.add([exp_ix, exp_neg_ix]);
                            graph.div(sum, two)
                        }
                        "sin" => {
                            let diff = graph.sub(exp_ix, exp_neg_ix);
                            graph.div(diff, two_i)
                        }
                        "tan" => {
                            let diff = graph.sub(exp_ix, exp_neg_ix);
                            let sum = graph.add([exp_ix, exp_neg_ix]);
                            let sin_term = graph.div(diff, two_i);
                            let cos_term = graph.div(sum, two);
                            graph.div(sin_term, cos_term)
                        }
                        _ => {
                            let new_args = vec![arg];
                            graph.function(&fn_str, new_args)
                        }
                    }
                } else {
                    let new_args: Vec<ExprId> = args
                        .iter()
                        .map(|&a| Self::transform_trig_to_euler(graph, a, i_sym, two, two_i))
                        .collect();
                    graph.function(&fn_str, new_args)
                }
            }
            ExprKind::Add(terms) => {
                let new_terms: Vec<ExprId> = terms
                    .iter()
                    .map(|&t| Self::transform_trig_to_euler(graph, t, i_sym, two, two_i))
                    .collect();
                graph.add(new_terms)
            }
            ExprKind::Sub(lhs, rhs) => {
                let l = Self::transform_trig_to_euler(graph, *lhs, i_sym, two, two_i);
                let r = Self::transform_trig_to_euler(graph, *rhs, i_sym, two, two_i);
                graph.sub(l, r)
            }
            ExprKind::Mul(factors) => {
                let new_factors: Vec<ExprId> = factors
                    .iter()
                    .map(|&f| Self::transform_trig_to_euler(graph, f, i_sym, two, two_i))
                    .collect();
                graph.mul(new_factors)
            }
            ExprKind::Div(num, den) => {
                let n = Self::transform_trig_to_euler(graph, *num, i_sym, two, two_i);
                let d = Self::transform_trig_to_euler(graph, *den, i_sym, two, two_i);
                graph.div(n, d)
            }
            ExprKind::Neg(inner) => {
                let i = Self::transform_trig_to_euler(graph, *inner, i_sym, two, two_i);
                graph.neg(i)
            }
            ExprKind::Pow(b, e) => {
                let new_b = Self::transform_trig_to_euler(graph, *b, i_sym, two, two_i);
                let new_e = Self::transform_trig_to_euler(graph, *e, i_sym, two, two_i);
                graph.pow(new_b, new_e)
            }
            _ => expr,
        }
    }
}

/// Helper to scan for trigonometric functions in an expression DAG.
fn contains_trig_functions(graph: &ExprGraph, root: ExprId) -> bool {
    let mut stack = vec![root];
    while let Some(curr) = stack.pop() {
        let node = graph.get(curr);
        match &node.kind {
            ExprKind::Function { name, args } => {
                let s = graph.symbols.resolve(*name).unwrap_or_default();
                if s == "sin" || s == "cos" || s == "tan" || s == "sec" || s == "csc" || s == "cot"
                {
                    return true;
                }
                stack.extend(args);
            }
            ExprKind::Add(terms) | ExprKind::Mul(terms) => {
                stack.extend(terms);
            }
            ExprKind::Sub(l, r) | ExprKind::Div(l, r) | ExprKind::Pow(l, r) => {
                stack.push(*l);
                stack.push(*r);
            }
            ExprKind::Neg(inner) => {
                stack.push(*inner);
            }
            _ => {}
        }
    }
    false
}

/// Weierstrass Rational Lifting Functor: $\int R(\sin x, \cos x) dx \to \int \frac{P(t)}{Q(t)} dt$ via $t = \tan(x/2)$.
pub struct WeierstrassLiftingFunctor;

impl LiftingFunctor for WeierstrassLiftingFunctor {
    fn name(&self) -> &'static str {
        "WeierstrassRationalLifting"
    }

    fn can_lift(&self, graph: &ExprGraph, expr: ExprId) -> bool {
        contains_trig_functions(graph, expr)
    }

    fn lift(&self, graph: &ExprGraph, expr: ExprId, _var: SymbolId) -> AlgebraResult<ExprId> {
        let t_node = graph.symbol("t");
        let one = graph.integer(1);
        let two = graph.integer(2);
        let t_sq = graph.pow(t_node, two);
        let one_plus_t_sq = graph.add([one, t_sq]);
        let one_minus_t_sq = graph.sub(one, t_sq);

        let sin_t = graph.div(graph.mul([two, t_node]), one_plus_t_sq);
        let cos_t = graph.div(one_minus_t_sq, one_plus_t_sq);

        let lifted = Self::transform_trig_to_weierstrass(graph, expr, sin_t, cos_t);
        Ok(Simplifier::simplify(graph, lifted).unwrap_or(lifted))
    }

    fn retract(&self, graph: &ExprGraph, expr: ExprId, var: SymbolId) -> AlgebraResult<ExprId> {
        Ok(RetractionCleaner::retract_weierstrass_to_trig(
            graph, expr, var,
        ))
    }
}

impl WeierstrassLiftingFunctor {
    fn transform_trig_to_weierstrass(
        graph: &ExprGraph,
        expr: ExprId,
        sin_t: ExprId,
        cos_t: ExprId,
    ) -> ExprId {
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Function { name, args } => {
                let fn_str = graph.symbols.resolve(*name).unwrap_or_default();
                if args.len() == 1 {
                    match fn_str.as_str() {
                        "sin" => sin_t,
                        "cos" => cos_t,
                        "tan" => graph.div(sin_t, cos_t),
                        _ => {
                            let new_args: Vec<ExprId> = args
                                .iter()
                                .map(|&a| {
                                    Self::transform_trig_to_weierstrass(graph, a, sin_t, cos_t)
                                })
                                .collect();
                            graph.function(&fn_str, new_args)
                        }
                    }
                } else {
                    let new_args: Vec<ExprId> = args
                        .iter()
                        .map(|&a| Self::transform_trig_to_weierstrass(graph, a, sin_t, cos_t))
                        .collect();
                    graph.function(&fn_str, new_args)
                }
            }
            ExprKind::Add(terms) => {
                let new_terms: Vec<ExprId> = terms
                    .iter()
                    .map(|&t| Self::transform_trig_to_weierstrass(graph, t, sin_t, cos_t))
                    .collect();
                graph.add(new_terms)
            }
            ExprKind::Sub(lhs, rhs) => {
                let l = Self::transform_trig_to_weierstrass(graph, *lhs, sin_t, cos_t);
                let r = Self::transform_trig_to_weierstrass(graph, *rhs, sin_t, cos_t);
                graph.sub(l, r)
            }
            ExprKind::Mul(factors) => {
                let new_factors: Vec<ExprId> = factors
                    .iter()
                    .map(|&f| Self::transform_trig_to_weierstrass(graph, f, sin_t, cos_t))
                    .collect();
                graph.mul(new_factors)
            }
            ExprKind::Div(num, den) => {
                let n = Self::transform_trig_to_weierstrass(graph, *num, sin_t, cos_t);
                let d = Self::transform_trig_to_weierstrass(graph, *den, sin_t, cos_t);
                graph.div(n, d)
            }
            _ => expr,
        }
    }
}

/// Central Domain Lift Router coordinating dynamic elevation and retraction.
pub struct DomainLiftRouter;

impl DomainLiftRouter {
    /// Elevate an expression using the specified `GoalDomain`, execute a solver closure in that domain,
    /// and retract the solution back to the base domain with artifact clean-up.
    pub fn roundtrip<F>(
        graph: &ExprGraph,
        expr: ExprId,
        var: SymbolId,
        goal: GoalDomain,
        solver: F,
    ) -> AlgebraResult<ExprId>
    where
        F: FnOnce(&ExprGraph, ExprId, SymbolId) -> AlgebraResult<ExprId>,
    {
        match goal {
            GoalDomain::ComplexExponential => {
                let functor = EulerLiftingFunctor;
                let lifted = functor.lift(graph, expr, var)?;
                let solved_in_c = solver(graph, lifted, var)?;
                functor.retract(graph, solved_in_c, var)
            }
            GoalDomain::RationalWeierstrass => {
                let functor = WeierstrassLiftingFunctor;
                let lifted = functor.lift(graph, expr, var)?;
                let solved_in_rat = solver(graph, lifted, var)?;
                functor.retract(graph, solved_in_rat, var)
            }
            _ => {
                // Default solve directly in base domain
                solver(graph, expr, var)
            }
        }
    }
}

/// Domain compatibility checker validating operations across disparate algebraic systems.
pub struct DomainCompatibilityChecker;

impl DomainCompatibilityChecker {
    /// Validates if two goal domains or contexts can be combined safely without contradiction.
    pub fn validate_compatibility(d1: GoalDomain, d2: GoalDomain) -> AlgebraResult<()> {
        if d1 == d2 || d1 == GoalDomain::ConserveOriginal || d2 == GoalDomain::ConserveOriginal {
            return Ok(());
        }

        // Catch incompatible mathematical categories
        match (d1, d2) {
            (GoalDomain::RationalWeierstrass, GoalDomain::CliffordBlade) => {
                Err(AlgebraError::EvaluationError(
                    "Incompatible category: Weierstrass rational fraction cannot directly tensor with Clifford blades".into(),
                ))
            }
            (GoalDomain::NilpotentDual, GoalDomain::AdeleRepresentation) => {
                Err(AlgebraError::EvaluationError(
                    "Incompatible category: Nilpotent dual numbers incompatible with Adele ring valuations".into(),
                ))
            }
            _ => Ok(()),
        }
    }
}
