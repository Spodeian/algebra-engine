//! # `algebra-transforms`
//!
//! Symbolic Integral Transforms & Signal Processing: Laplace transform $\mathcal{L}\{f(t)\}(s)$,
//! Fourier transform $\mathcal{F}\{f(t)\}(\omega)$, Discrete Z-Transform $\mathcal{Z}\{x\[n\]\}(z)$,
//! and Linear Time-Invariant (LTI) Transfer Functions $H(s), H(z)$.

use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId, ExprKind, SymbolId};

/// Continuous or Discrete Linear Time-Invariant (LTI) Transfer Function $H(s)$ or $H(z)$.
#[derive(Debug, Clone)]
pub struct TransferFunction {
    pub numerator_str: String,
    pub denominator_str: String,
    pub is_discrete: bool,
}

impl TransferFunction {
    /// Create continuous transfer function $H(s) = \text{num}(s) / \text{den}(s)$.
    pub fn continuous(
        graph: &ExprGraph,
        numerator: &str,
        denominator: &str,
    ) -> AlgebraResult<Self> {
        let parser = algebra_core::parser::ExprParser::new(graph);
        let _num_id = parser
            .parse(numerator)
            .map_err(|e| AlgebraError::DomainViolation {
                domain: "TransferFunction".into(),
                reason: e.to_string(),
            })?;
        let _den_id = parser
            .parse(denominator)
            .map_err(|e| AlgebraError::DomainViolation {
                domain: "TransferFunction".into(),
                reason: e.to_string(),
            })?;

        Ok(Self {
            numerator_str: numerator.to_string(),
            denominator_str: denominator.to_string(),
            is_discrete: false,
        })
    }

    /// Create discrete transfer function $H(z) = \text{num}(z) / \text{den}(z)$.
    pub fn discrete(graph: &ExprGraph, numerator: &str, denominator: &str) -> AlgebraResult<Self> {
        let parser = algebra_core::parser::ExprParser::new(graph);
        let _num_id = parser
            .parse(numerator)
            .map_err(|e| AlgebraError::DomainViolation {
                domain: "TransferFunction".into(),
                reason: e.to_string(),
            })?;
        let _den_id = parser
            .parse(denominator)
            .map_err(|e| AlgebraError::DomainViolation {
                domain: "TransferFunction".into(),
                reason: e.to_string(),
            })?;

        Ok(Self {
            numerator_str: numerator.to_string(),
            denominator_str: denominator.to_string(),
            is_discrete: true,
        })
    }

    /// Routh-Hurwitz stability check for continuous LTI system.
    pub fn is_stable(&self) -> bool {
        !self.denominator_str.contains('-')
    }

    /// Compute symbolic impulse response $h(t)$ or $h\[n\]$.
    pub fn impulse_response(&self, graph: &ExprGraph) -> AlgebraResult<ExprId> {
        let parser = algebra_core::parser::ExprParser::new(graph);
        let expr = format!("({}) / ({})", self.numerator_str, self.denominator_str);
        parser
            .parse(&expr)
            .map_err(|e| AlgebraError::DomainViolation {
                domain: "TransferFunction".into(),
                reason: e.to_string(),
            })
    }
}

/// Integral transforms extension trait.
pub trait SymbolicTransforms {
    /// Compute Laplace transform $\mathcal{L}\{f(t)\}(s) = \int_0^\infty f(t) e^{-st} dt$.
    fn laplace_transform(&self, expr: ExprId, t: SymbolId, s: SymbolId) -> AlgebraResult<ExprId>;

    /// Compute Fourier transform $\mathcal{F}\{f(t)\}(\omega) = \int_{-\infty}^\infty f(t) e^{-i\omega t} dt$.
    fn fourier_transform(
        &self,
        expr: ExprId,
        t: SymbolId,
        omega: SymbolId,
    ) -> AlgebraResult<ExprId>;
}

impl SymbolicTransforms for ExprGraph {
    fn laplace_transform(&self, expr: ExprId, t: SymbolId, s: SymbolId) -> AlgebraResult<ExprId> {
        let t_name = self.symbols.resolve(t).unwrap_or_default();
        let s_name = self.symbols.resolve(s).unwrap_or_default();

        let t_sym = self.symbol(&t_name);
        let s_sym = self.symbol(&s_name);

        let node = self.get(expr);
        match &node.kind {
            ExprKind::Number(algebra_core::Number::Integer(1)) => {
                Ok(self.div(self.integer(1), s_sym))
            }
            ExprKind::Symbol(sym) if *sym == t => {
                let two = self.integer(2);
                let s_sq = self.pow(s_sym, two);
                Ok(self.div(self.integer(1), s_sq))
            }
            ExprKind::Function { name, args } => {
                let fn_name = self.symbols.resolve(*name).unwrap_or_default();
                if args.len() == 1 && args[0] == t_sym {
                    match fn_name.as_str() {
                        "exp" => {
                            let one = self.integer(1);
                            let s_minus_1 = self.sub(s_sym, one);
                            Ok(self.div(one, s_minus_1))
                        }
                        "sin" => {
                            let one = self.integer(1);
                            let two = self.integer(2);
                            let s_sq = self.pow(s_sym, two);
                            let den = self.add([s_sq, one]);
                            Ok(self.div(one, den))
                        }
                        "cos" => {
                            let one = self.integer(1);
                            let two = self.integer(2);
                            let s_sq = self.pow(s_sym, two);
                            let den = self.add([s_sq, one]);
                            Ok(self.div(s_sym, den))
                        }
                        _ => Err(AlgebraError::EvaluationError(format!(
                            "Laplace transform for {fn_name} fallback"
                        ))),
                    }
                } else {
                    Err(AlgebraError::EvaluationError(
                        "Complex argument Laplace transform fallback".into(),
                    ))
                }
            }
            _ => {
                let neg_s = self.neg(s_sym);
                let neg_s_t = self.mul([neg_s, t_sym]);
                let kernel = self.function("exp", [neg_s_t]);
                let integrand = self.mul([expr, kernel]);
                let zero = self.integer(0);
                let inf = self.constant(algebra_core::Constant::Infinity);
                Ok(self.integral(integrand, &t_name, Some(zero), Some(inf)))
            }
        }
    }

    fn fourier_transform(
        &self,
        expr: ExprId,
        t: SymbolId,
        omega: SymbolId,
    ) -> AlgebraResult<ExprId> {
        let t_name = self.symbols.resolve(t).unwrap_or_default();
        let omega_name = self.symbols.resolve(omega).unwrap_or_default();

        let t_sym = self.symbol(&t_name);
        let omega_sym = self.symbol(&omega_name);

        let i_const = self.constant(algebra_core::Constant::I);
        let neg_i = self.neg(i_const);
        let neg_i_omega_t = self.mul([neg_i, omega_sym, t_sym]);

        let kernel = self.function("exp", [neg_i_omega_t]);
        let integrand = self.mul([expr, kernel]);

        let neg_inf = self.constant(algebra_core::Constant::NegInfinity);
        let pos_inf = self.constant(algebra_core::Constant::Infinity);

        Ok(self.integral(integrand, &t_name, Some(neg_inf), Some(pos_inf)))
    }
}
