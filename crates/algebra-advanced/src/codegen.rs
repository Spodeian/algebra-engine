//! # `algebra-codegen`
//!
//! AST-to-Rust/C code generation and JIT numerical closure compilation.

use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId, ExprKind, SymbolId};
use algebra_engine::numeric::{EvalContext, NumericalEval};

/// Code compilation target format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    Rust,
    C,
}

/// Compiler turning symbolic expression graphs into executable closures or code strings.
pub struct CodeGenerator;

impl CodeGenerator {
    /// Emit source code string in target language for expression `expr`.
    pub fn emit_code(
        graph: &ExprGraph,
        expr: ExprId,
        target: TargetLanguage,
    ) -> AlgebraResult<String> {
        let node = graph.get(expr);
        match &node.kind {
            ExprKind::Number(n) => match n {
                algebra_core::Number::Integer(i) => Ok(format!("{i}.0")),
                algebra_core::Number::Float(bits) => Ok(format!("{}", f64::from_bits(*bits))),
                algebra_core::Number::Rational(num, den) => Ok(format!("({num}.0 / {den}.0)")),
                algebra_core::Number::Constant(c) => match c {
                    algebra_core::Constant::Pi => Ok("std::f64::consts::PI".into()),
                    algebra_core::Constant::E => Ok("std::f64::consts::E".into()),
                    algebra_core::Constant::I => Ok("1.0".into()),
                    _ => Ok("0.0".into()),
                },
            },
            ExprKind::Symbol(s) => Ok(graph.symbols.resolve(*s).unwrap_or_default()),
            ExprKind::Add(terms) => {
                let code_terms: Vec<String> = terms
                    .iter()
                    .map(|&t| Self::emit_code(graph, t, target))
                    .collect::<AlgebraResult<_>>()?;
                Ok(format!("({})", code_terms.join(" + ")))
            }
            ExprKind::Sub(lhs, rhs) => {
                let cl = Self::emit_code(graph, *lhs, target)?;
                let cr = Self::emit_code(graph, *rhs, target)?;
                Ok(format!("({cl} - {cr})"))
            }
            ExprKind::Mul(factors) => {
                let code_factors: Vec<String> = factors
                    .iter()
                    .map(|&f| Self::emit_code(graph, f, target))
                    .collect::<AlgebraResult<_>>()?;
                Ok(format!("({})", code_factors.join(" * ")))
            }
            ExprKind::Div(num, den) => {
                let cn = Self::emit_code(graph, *num, target)?;
                let cd = Self::emit_code(graph, *den, target)?;
                Ok(format!("({cn} / {cd})"))
            }
            ExprKind::Pow(base, exp) => {
                let cb = Self::emit_code(graph, *base, target)?;
                let ce = Self::emit_code(graph, *exp, target)?;
                match target {
                    TargetLanguage::Rust => Ok(format!("{cb}.powf({ce})")),
                    TargetLanguage::C => Ok(format!("pow({cb}, {ce})")),
                }
            }
            ExprKind::Function { name, args } => {
                let fn_name = graph.symbols.resolve(*name).unwrap_or_default();
                let code_args: Vec<String> = args
                    .iter()
                    .map(|&a| Self::emit_code(graph, a, target))
                    .collect::<AlgebraResult<_>>()?;
                match target {
                    TargetLanguage::Rust => Ok(format!("{fn_name}({})", code_args.join(", "))),
                    TargetLanguage::C => Ok(format!("{fn_name}({})", code_args.join(", "))),
                }
            }
            _ => Err(AlgebraError::EvaluationError(
                "Unsupported expression for codegen".into(),
            )),
        }
    }

    /// Compile a single variable expression $f(x)$ into a fast Rust executable closure `Fn(f64) -> f64`.
    pub fn compile_fn1(graph: &ExprGraph, expr: ExprId, var: SymbolId) -> impl Fn(f64) -> f64 {
        let graph_clone = graph.clone();
        move |x_val: f64| {
            let val_node = graph_clone.float(x_val);
            let sub = graph_clone.substitute(expr, var, val_node);
            let ctx = EvalContext::new(53);
            if let Ok(big_val) = graph_clone.evalf(sub, &ctx) {
                big_val.to_f64()
            } else {
                f64::NAN
            }
        }
    }

    /// Compile a two-variable expression $f(x, y)$ into a fast Rust executable closure `Fn(f64, f64) -> f64`.
    pub fn compile_fn2(
        graph: &ExprGraph,
        expr: ExprId,
        var_x: SymbolId,
        var_y: SymbolId,
    ) -> impl Fn(f64, f64) -> f64 {
        let graph_clone = graph.clone();
        move |x_val: f64, y_val: f64| {
            let x_node = graph_clone.float(x_val);
            let y_node = graph_clone.float(y_val);
            let sub1 = graph_clone.substitute(expr, var_x, x_node);
            let sub2 = graph_clone.substitute(sub1, var_y, y_node);
            let ctx = EvalContext::new(53);
            if let Ok(big_val) = graph_clone.evalf(sub2, &ctx) {
                big_val.to_f64()
            } else {
                f64::NAN
            }
        }
    }

    /// Compile an $N$-variable expression $f(x_1, \dots, x_n)$ into a fast Rust executable closure `Fn(&[f64]) -> f64`.
    pub fn compile_fn_n(
        graph: &ExprGraph,
        expr: ExprId,
        vars: &[SymbolId],
    ) -> impl Fn(&[f64]) -> f64 {
        let graph_clone = graph.clone();
        let vars_vec = vars.to_vec();
        move |args: &[f64]| {
            let mut sub = expr;
            for (&v_sym, &val) in vars_vec.iter().zip(args.iter()) {
                let val_node = graph_clone.float(val);
                sub = graph_clone.substitute(sub, v_sym, val_node);
            }
            let ctx = EvalContext::new(53);
            if let Ok(big_val) = graph_clone.evalf(sub, &ctx) {
                big_val.to_f64()
            } else {
                f64::NAN
            }
        }
    }
}
