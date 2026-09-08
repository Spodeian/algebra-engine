//! # `algebra-special`
//!
//! Symbolic special functions including Gamma $\Gamma(x)$, Bessel functions $J_\nu(x)$,
//! Riemann Zeta $\zeta(s)$, Dirac Delta $\delta(x)$, and Error function $\text{erf}(x)$.

pub use crate::weyl_dmodules::{BesselJ, ErrorFunctionErf, HermiteH, HolonomicFunction, LegendreP};
use algebra_core::{AlgebraResult, ExprGraph, ExprId};

/// Special functions constructors for symbolic expressions.
pub trait SpecialFunctions {
    /// Construct Gamma function $\Gamma(x)$ node.
    fn gamma(&self, arg: ExprId) -> ExprId;

    /// Construct Bessel function of the first kind $J_\nu(x)$ node.
    fn bessel_j(&self, order: ExprId, arg: ExprId) -> ExprId;

    /// Construct Riemann Zeta function $\zeta(s)$ node.
    fn zeta(&self, arg: ExprId) -> ExprId;

    /// Construct Dirac Delta distribution $\delta(x)$ node.
    fn dirac_delta(&self, arg: ExprId) -> ExprId;

    /// Construct Error function $\text{erf}(x)$ node.
    fn erf(&self, arg: ExprId) -> ExprId;

    /// Evaluate exact values for integer and half-integer special function arguments.
    fn evaluate_special(&self, expr: ExprId) -> AlgebraResult<ExprId>;
}

impl SpecialFunctions for ExprGraph {
    fn gamma(&self, arg: ExprId) -> ExprId {
        self.function("gamma", [arg])
    }

    fn bessel_j(&self, order: ExprId, arg: ExprId) -> ExprId {
        self.function("bessel_j", [order, arg])
    }

    fn zeta(&self, arg: ExprId) -> ExprId {
        self.function("zeta", [arg])
    }

    fn dirac_delta(&self, arg: ExprId) -> ExprId {
        self.function("dirac_delta", [arg])
    }

    fn erf(&self, arg: ExprId) -> ExprId {
        self.function("erf", [arg])
    }

    fn evaluate_special(&self, expr: ExprId) -> AlgebraResult<ExprId> {
        let node = self.get(expr);
        if let algebra_core::ExprKind::Function { name, args } = &node.kind {
            let fn_name = self.symbols.resolve(*name).unwrap_or_default();
            if fn_name == "gamma" && args.len() == 1 {
                let arg_node = self.get(args[0]);
                if let algebra_core::ExprKind::Number(algebra_core::Number::Integer(n)) =
                    arg_node.kind
                {
                    if n > 0 {
                        // Gamma(n) = (n - 1)!
                        let mut fact = 1i64;
                        for i in 1..n {
                            fact *= i;
                        }
                        return Ok(self.integer(fact));
                    }
                }
            } else if fn_name == "zeta" && args.len() == 1 {
                let arg_node = self.get(args[0]);
                if let algebra_core::ExprKind::Number(algebra_core::Number::Integer(2)) =
                    arg_node.kind
                {
                    // zeta(2) = pi^2 / 6
                    let pi = self.symbol("pi");
                    let pi_sq = self.pow(pi, self.integer(2));
                    return Ok(self.div(pi_sq, self.integer(6)));
                }
            }
        }
        Ok(expr)
    }
}
