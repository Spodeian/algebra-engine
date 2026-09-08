//! Formatting and pretty-printing engine for mathematical expressions (LaTeX, Unicode, MathML).

use crate::{Constant, ExprGraph, ExprId, ExprKind, Number, RelOp, SetOp};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("Formatting error: {0}")]
    Custom(String),
}

pub type FormatResult<T> = Result<T, FormatError>;

/// Trait for formatting expressions into text representations.
pub trait Formatter {
    fn format(&self, graph: &ExprGraph, root: ExprId) -> FormatResult<String>;
}

/// Formatter generating LaTeX string output.
#[derive(Debug, Clone, Copy, Default)]
pub struct LatexFormatter;

impl Formatter for LatexFormatter {
    fn format(&self, graph: &ExprGraph, root: ExprId) -> FormatResult<String> {
        let node = graph.get(root);
        match &node.kind {
            ExprKind::Number(num) => Ok(match num {
                Number::Integer(i) => i.to_string(),
                Number::BigInteger(b) => b.to_string(),
                Number::Rational(n, d) => format!("\\frac{{{}}}{{{}}}", n, d),
                Number::BigRational(r) => format!("\\frac{{{}}}{{{}}}", r.numer(), r.denom()),
                Number::Scientific { mantissa, exponent } => {
                    if *exponent == 0 {
                        mantissa.to_string()
                    } else {
                        format!("{} \\times 10^{{{}}}", mantissa, exponent)
                    }
                }
                Number::Float(bits) => f64::from_bits(*bits).to_string(),
                Number::Constant(c) => match c {
                    Constant::Pi => "\\pi".to_string(),
                    Constant::E => "e".to_string(),
                    Constant::I => "i".to_string(),
                    Constant::EulerGamma => "\\gamma".to_string(),
                    Constant::GoldenRatio => "\\phi".to_string(),
                    Constant::Apery => "\\zeta(3)".to_string(),
                    Constant::Catalan => "G".to_string(),
                    Constant::Infinity => "\\infty".to_string(),
                    Constant::NegInfinity => "-\\infty".to_string(),
                    Constant::Undefined => "\\text{Undefined}".to_string(),
                },
            }),
            ExprKind::Symbol(sym) => {
                let name = graph
                    .symbols
                    .resolve(*sym)
                    .unwrap_or_else(|| "x".to_string());
                Ok(name)
            }
            ExprKind::Add(terms) => {
                let mut parts = Vec::new();
                for &t in terms {
                    parts.push(self.format(graph, t)?);
                }
                Ok(parts.join(" + "))
            }
            ExprKind::Mul(factors) => {
                let mut parts = Vec::new();
                for &f in factors {
                    parts.push(self.format(graph, f)?);
                }
                Ok(parts.join(" \\cdot "))
            }
            ExprKind::Pow(base, exp) => {
                let b_str = self.format(graph, *base)?;
                let e_str = self.format(graph, *exp)?;
                Ok(format!("{{{}}}^{{{}}}", b_str, e_str))
            }
            ExprKind::Div(num, den) => {
                let n_str = self.format(graph, *num)?;
                let d_str = self.format(graph, *den)?;
                Ok(format!("\\frac{{{}}}{{{}}}", n_str, d_str))
            }
            ExprKind::Sub(lhs, rhs) => {
                let l_str = self.format(graph, *lhs)?;
                let r_str = self.format(graph, *rhs)?;
                Ok(format!("{} - {}", l_str, r_str))
            }
            ExprKind::Neg(inner) => {
                let i_str = self.format(graph, *inner)?;
                Ok(format!("-{}", i_str))
            }
            ExprKind::Function { name, args } => {
                let fn_name = graph.symbols.resolve(*name).unwrap_or_default();
                let arg_strs: Result<Vec<_>, _> =
                    args.iter().map(|&a| self.format(graph, a)).collect();
                let args_vec = arg_strs?;
                if (fn_name == "BringRadical" || fn_name == "BR") && args_vec.len() == 1 {
                    Ok(format!(
                        "\\operatorname{{BR}}\\left({}\\right)",
                        args_vec[0]
                    ))
                } else if fn_name == "besselj" && args_vec.len() == 2 {
                    Ok(format!(
                        "J_{{{}}}\\left({}\\right)",
                        args_vec[0], args_vec[1]
                    ))
                } else {
                    Ok(format!(
                        "\\operatorname{{{}}}\\left({}\\right)",
                        fn_name,
                        args_vec.join(", ")
                    ))
                }
            }
            ExprKind::Derivative { expr, wrt, order } => {
                let e_str = self.format(graph, *expr)?;
                let wrt_str = graph.symbols.resolve(*wrt).unwrap_or_default();
                if *order == 1 {
                    Ok(format!(
                        "\\frac{{\\mathrm{{d}}}}{{\\mathrm{{d}}{}}} \\left({}\\right)",
                        wrt_str, e_str
                    ))
                } else {
                    Ok(format!(
                        "\\frac{{\\mathrm{{d}}^{{{}}}}}{{\\mathrm{{d}}{}^{{{}}}}} \\left({}\\right)",
                        order, wrt_str, order, e_str
                    ))
                }
            }
            ExprKind::Integral {
                expr,
                wrt,
                lower,
                upper,
            } => {
                let e_str = self.format(graph, *expr)?;
                let wrt_str = graph.symbols.resolve(*wrt).unwrap_or_default();
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "\\int_{{{}}}^{{{}}} {} \\,\\mathrm{{d}}{}",
                            l_str, u_str, e_str, wrt_str
                        ))
                    }
                    _ => Ok(format!("\\int {} \\,\\mathrm{{d}}{}", e_str, wrt_str)),
                }
            }
            ExprKind::Matrix {
                rows,
                cols,
                elements,
            } => {
                let mut rows_str = Vec::new();
                for r in 0..*rows {
                    let mut col_strs = Vec::new();
                    for c in 0..*cols {
                        let idx = r * cols + c;
                        col_strs.push(self.format(graph, elements[idx])?);
                    }
                    rows_str.push(col_strs.join(" & "));
                }
                Ok(format!(
                    "\\begin{{pmatrix}} {} \\end{{pmatrix}}",
                    rows_str.join(" \\\\ ")
                ))
            }
            ExprKind::Relational { op, lhs, rhs } => {
                let l_str = self.format(graph, *lhs)?;
                let r_str = self.format(graph, *rhs)?;
                let op_str = match op {
                    RelOp::Equal => "=",
                    RelOp::NotEqual => "\\neq",
                    RelOp::LessThan => "<",
                    RelOp::LessEqual => "\\leq",
                    RelOp::GreaterThan => ">",
                    RelOp::GreaterEqual => "\\geq",
                };
                Ok(format!("{} {} {}", l_str, op_str, r_str))
            }
            ExprKind::SetOperation { op, args } => {
                let arg_strs: Result<Vec<_>, _> =
                    args.iter().map(|&a| self.format(graph, a)).collect();
                let op_str = match op {
                    SetOp::Union => "\\cup",
                    SetOp::Intersection => "\\cap",
                    SetOp::Difference => "\\setminus",
                    SetOp::SymmetricDifference => "\\Delta",
                    SetOp::CartesianProduct => "\\times",
                    SetOp::In => "\\in",
                    SetOp::Subset => "\\subset",
                };
                Ok(arg_strs?.join(&format!(" {} ", op_str)))
            }
            ExprKind::Sum {
                body,
                var,
                lower,
                upper,
            } => {
                let body_str = self.format(graph, *body)?;
                let var_str = graph
                    .symbols
                    .resolve(*var)
                    .unwrap_or_else(|| "i".to_string());
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "\\sum_{{{}={}}}^{{{}}} {}",
                            var_str, l_str, u_str, body_str
                        ))
                    }
                    _ => Ok(format!("\\sum_{{{}}} {}", var_str, body_str)),
                }
            }
            ExprKind::Product {
                body,
                var,
                lower,
                upper,
            } => {
                let body_str = self.format(graph, *body)?;
                let var_str = graph
                    .symbols
                    .resolve(*var)
                    .unwrap_or_else(|| "i".to_string());
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "\\prod_{{{}={}}}^{{{}}} {}",
                            var_str, l_str, u_str, body_str
                        ))
                    }
                    _ => Ok(format!("\\prod_{{{}}} {}", var_str, body_str)),
                }
            }
            ExprKind::TensorContraction {
                tensor_a, tensor_b, ..
            } => {
                let a_str = self.format(graph, *tensor_a)?;
                let b_str = self.format(graph, *tensor_b)?;
                Ok(format!(
                    "\\operatorname{{Contract}}\\left({}, {}\\right)",
                    a_str, b_str
                ))
            }
        }
    }
}

/// Formatter generating clean, pretty 2D/1D Unicode string output.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnicodeFormatter;

impl UnicodeFormatter {
    #[allow(dead_code)]
    fn superscript(exp_str: &str) -> String {
        exp_str
            .chars()
            .map(|c| match c {
                '0' => '⁰',
                '1' => '¹',
                '2' => '²',
                '3' => '³',
                '4' => '⁴',
                '5' => '⁵',
                '6' => '⁶',
                '7' => '⁷',
                '8' => '⁸',
                '9' => '⁹',
                '+' => '⁺',
                '-' => '⁻',
                'n' => 'ⁿ',
                'x' => 'ˣ',
                _ => c,
            })
            .collect()
    }
}

impl Formatter for UnicodeFormatter {
    fn format(&self, graph: &ExprGraph, root: ExprId) -> FormatResult<String> {
        let node = graph.get(root);
        match &node.kind {
            ExprKind::Number(num) => Ok(match num {
                Number::Integer(i) => i.to_string(),
                Number::BigInteger(b) => b.to_string(),
                Number::Rational(n, d) => format!("{}/{}", n, d),
                Number::BigRational(r) => format!("{}/{}", r.numer(), r.denom()),
                Number::Scientific { mantissa, exponent } => {
                    if *exponent == 0 {
                        mantissa.to_string()
                    } else {
                        format!("{} × 10^{}", mantissa, exponent)
                    }
                }
                Number::Float(bits) => f64::from_bits(*bits).to_string(),
                Number::Constant(c) => match c {
                    Constant::Pi => "π".to_string(),
                    Constant::E => "e".to_string(),
                    Constant::I => "i".to_string(),
                    Constant::EulerGamma => "γ".to_string(),
                    Constant::GoldenRatio => "ϕ".to_string(),
                    Constant::Apery => "ζ(3)".to_string(),
                    Constant::Catalan => "G".to_string(),
                    Constant::Infinity => "∞".to_string(),
                    Constant::NegInfinity => "-∞".to_string(),
                    Constant::Undefined => "Undefined".to_string(),
                },
            }),
            ExprKind::Symbol(sym) => {
                let name = graph
                    .symbols
                    .resolve(*sym)
                    .unwrap_or_else(|| "x".to_string());
                Ok(name)
            }
            ExprKind::Add(terms) => {
                let mut parts = Vec::new();
                for &t in terms {
                    parts.push(self.format(graph, t)?);
                }
                Ok(parts.join(" + "))
            }
            ExprKind::Mul(factors) => {
                let mut parts = Vec::new();
                for &f in factors {
                    parts.push(self.format(graph, f)?);
                }
                Ok(parts.join(" · "))
            }
            ExprKind::Pow(base, exp) => {
                let b_str = self.format(graph, *base)?;
                let e_str = self.format(graph, *exp)?;
                Ok(format!("{}^{}", b_str, e_str))
            }
            ExprKind::Div(num, den) => {
                let n_str = self.format(graph, *num)?;
                let d_str = self.format(graph, *den)?;
                Ok(format!("({}) / ({})", n_str, d_str))
            }
            ExprKind::Sub(lhs, rhs) => {
                let l_str = self.format(graph, *lhs)?;
                let r_str = self.format(graph, *rhs)?;
                Ok(format!("{} - {}", l_str, r_str))
            }
            ExprKind::Neg(inner) => {
                let i_str = self.format(graph, *inner)?;
                Ok(format!("-{}", i_str))
            }
            ExprKind::Function { name, args } => {
                let fn_name = graph.symbols.resolve(*name).unwrap_or_default();
                let arg_strs: Result<Vec<_>, _> =
                    args.iter().map(|&a| self.format(graph, a)).collect();
                let args_vec = arg_strs?;
                if fn_name == "sqrt" && args_vec.len() == 1 {
                    Ok(format!("√({})", args_vec[0]))
                } else if (fn_name == "BringRadical" || fn_name == "BR") && args_vec.len() == 1 {
                    Ok(format!("BR({})", args_vec[0]))
                } else if fn_name == "besselj" && args_vec.len() == 2 {
                    Ok(format!("J_{}({})", args_vec[0], args_vec[1]))
                } else {
                    Ok(format!("{}({})", fn_name, args_vec.join(", ")))
                }
            }
            ExprKind::Derivative { expr, wrt, order } => {
                let e_str = self.format(graph, *expr)?;
                let wrt_str = graph.symbols.resolve(*wrt).unwrap_or_default();
                if *order == 1 {
                    Ok(format!("d/d{} [{}]", wrt_str, e_str))
                } else {
                    Ok(format!("d^{}/d{}^{} [{}]", order, wrt_str, order, e_str))
                }
            }
            ExprKind::Integral {
                expr,
                wrt,
                lower,
                upper,
            } => {
                let e_str = self.format(graph, *expr)?;
                let wrt_str = graph.symbols.resolve(*wrt).unwrap_or_default();
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "∫_{{{}}}^{{{}}} {} d{}",
                            l_str, u_str, e_str, wrt_str
                        ))
                    }
                    _ => Ok(format!("∫ {} d{}", e_str, wrt_str)),
                }
            }
            ExprKind::Matrix {
                rows,
                cols,
                elements,
            } => {
                let mut rows_str = Vec::new();
                for r in 0..*rows {
                    let mut col_strs = Vec::new();
                    for c in 0..*cols {
                        let idx = r * cols + c;
                        col_strs.push(self.format(graph, elements[idx])?);
                    }
                    rows_str.push(col_strs.join(", "));
                }
                Ok(format!("[ {} ]", rows_str.join(" ; ")))
            }
            ExprKind::Relational { op, lhs, rhs } => {
                let l_str = self.format(graph, *lhs)?;
                let r_str = self.format(graph, *rhs)?;
                let op_str = match op {
                    RelOp::Equal => "=",
                    RelOp::NotEqual => "≠",
                    RelOp::LessThan => "<",
                    RelOp::LessEqual => "≤",
                    RelOp::GreaterThan => ">",
                    RelOp::GreaterEqual => "≥",
                };
                Ok(format!("{} {} {}", l_str, op_str, r_str))
            }
            ExprKind::SetOperation { op, args } => {
                let arg_strs: Result<Vec<_>, _> =
                    args.iter().map(|&a| self.format(graph, a)).collect();
                let op_str = match op {
                    SetOp::Union => "∪",
                    SetOp::Intersection => "∩",
                    SetOp::Difference => "\\",
                    SetOp::SymmetricDifference => "Δ",
                    SetOp::CartesianProduct => "×",
                    SetOp::In => "∈",
                    SetOp::Subset => "⊂",
                };
                Ok(arg_strs?.join(&format!(" {} ", op_str)))
            }
            ExprKind::Sum {
                body,
                var,
                lower,
                upper,
            } => {
                let body_str = self.format(graph, *body)?;
                let var_str = graph
                    .symbols
                    .resolve(*var)
                    .unwrap_or_else(|| "i".to_string());
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "∑_({}={})^({}) [{}]",
                            var_str, l_str, u_str, body_str
                        ))
                    }
                    _ => Ok(format!("∑_({}) [{}]", var_str, body_str)),
                }
            }
            ExprKind::Product {
                body,
                var,
                lower,
                upper,
            } => {
                let body_str = self.format(graph, *body)?;
                let var_str = graph
                    .symbols
                    .resolve(*var)
                    .unwrap_or_else(|| "i".to_string());
                match (lower, upper) {
                    (Some(l), Some(u)) => {
                        let l_str = self.format(graph, *l)?;
                        let u_str = self.format(graph, *u)?;
                        Ok(format!(
                            "∏_({}={})^({}) [{}]",
                            var_str, l_str, u_str, body_str
                        ))
                    }
                    _ => Ok(format!("∏_({}) [{}]", var_str, body_str)),
                }
            }
            ExprKind::TensorContraction {
                tensor_a, tensor_b, ..
            } => {
                let a_str = self.format(graph, *tensor_a)?;
                let b_str = self.format(graph, *tensor_b)?;
                Ok(format!("Contract({}, {})", a_str, b_str))
            }
        }
    }
}

/// Formatter generating MathML XML string output.
#[derive(Debug, Clone, Copy, Default)]
pub struct MathMLFormatter;

impl Formatter for MathMLFormatter {
    fn format(&self, graph: &ExprGraph, root: ExprId) -> FormatResult<String> {
        let node = graph.get(root);
        match &node.kind {
            ExprKind::Number(num) => Ok(match num {
                Number::Integer(i) => format!("<mn>{}</mn>", i),
                Number::BigInteger(b) => format!("<mn>{}</mn>", b),
                Number::Rational(n, d) => format!("<mfrac><mn>{}</mn><mn>{}</mn></mfrac>", n, d),
                Number::BigRational(r) => format!(
                    "<mfrac><mn>{}</mn><mn>{}</mn></mfrac>",
                    r.numer(),
                    r.denom()
                ),
                Number::Scientific { mantissa, exponent } => {
                    if *exponent == 0 {
                        format!("<mn>{}</mn>", mantissa)
                    } else {
                        format!(
                            "<mn>{}</mn><mo>&times;</mo><msup><mn>10</mn><mn>{}</mn></msup>",
                            mantissa, exponent
                        )
                    }
                }
                Number::Float(bits) => format!("<mn>{}</mn>", f64::from_bits(*bits)),
                Number::Constant(c) => match c {
                    Constant::Pi => "<mi>&pi;</mi>".to_string(),
                    Constant::E => "<mi>e</mi>".to_string(),
                    Constant::I => "<mi>i</mi>".to_string(),
                    _ => "<mi>&infty;</mi>".to_string(),
                },
            }),
            ExprKind::Symbol(sym) => {
                let name = graph
                    .symbols
                    .resolve(*sym)
                    .unwrap_or_else(|| "x".to_string());
                Ok(format!("<mi>{}</mi>", name))
            }
            ExprKind::Add(terms) => {
                let mut parts = Vec::new();
                for &t in terms {
                    parts.push(self.format(graph, t)?);
                }
                Ok(parts.join("<mo>+</mo>"))
            }
            ExprKind::Mul(factors) => {
                let mut parts = Vec::new();
                for &f in factors {
                    parts.push(self.format(graph, f)?);
                }
                Ok(parts.join("<mo>&middot;</mo>"))
            }
            ExprKind::Pow(base, exp) => {
                let b_str = self.format(graph, *base)?;
                let e_str = self.format(graph, *exp)?;
                Ok(format!("<msup>{}{}</msup>", b_str, e_str))
            }
            _ => Ok(format!("<mi>{:?}</mi>", node.kind)),
        }
    }
}
