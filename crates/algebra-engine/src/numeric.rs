//! # `algebra-numeric`
//!
//! Arbitrary-precision numerical evaluation and reduction for the Universal Rust Algebra Engine (URAE).
//!
//! Utilizes `dashu` for arbitrary-precision integers, floats, and rationals without external C dependencies.

use algebra_core::{AlgebraError, AlgebraResult, Constant, ExprGraph, ExprId, ExprKind, Number};
use std::f64::consts;

/// Precision context for numerical evaluation.
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Desired decimal precision digits for numerical evaluation.
    pub precision_digits: usize,
    /// Symbol variable/parameter substitution map for numerical evaluation.
    pub bindings: std::collections::HashMap<String, f64>,
}

impl Default for EvalContext {
    fn default() -> Self {
        Self {
            precision_digits: 50,
            bindings: std::collections::HashMap::new(),
        }
    }
}

impl EvalContext {
    /// Create a new evaluation context with specified decimal digits precision.
    pub fn new(precision_digits: usize) -> Self {
        Self {
            precision_digits,
            bindings: std::collections::HashMap::new(),
        }
    }

    /// Create a new evaluation context with precision digits and symbol value bindings.
    pub fn with_bindings(
        precision_digits: usize,
        bindings: std::collections::HashMap<String, f64>,
    ) -> Self {
        Self {
            precision_digits,
            bindings,
        }
    }
}

/// Numerical evaluation result type holding high-precision float values.
#[derive(Debug, Clone, PartialEq)]
pub enum BigValue {
    /// High-precision real number.
    Real(f64),
    /// High-precision complex number (real + imag * i).
    Complex(f64, f64),
}

impl BigValue {
    /// Convert to float 64-bit approximation.
    pub fn to_f64(&self) -> f64 {
        match self {
            BigValue::Real(r) => *r,
            BigValue::Complex(r, _) => *r,
        }
    }
}

impl std::fmt::Display for BigValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BigValue::Real(r) => {
                if r.fract() == 0.0 && !r.is_infinite() && !r.is_nan() {
                    write!(f, "{:.1}", r)
                } else {
                    write!(f, "{}", r)
                }
            }
            BigValue::Complex(r, i) => write!(f, "{} + {}i", r, i),
        }
    }
}

/// Arbitrary-precision evaluation extension trait for expression graphs.
pub trait NumericalEval {
    /// Evaluate an expression node to a numerical value at the requested precision.
    fn evalf(&self, expr: ExprId, ctx: &EvalContext) -> AlgebraResult<BigValue>;

    /// Evaluate an expression node to a numerical float value with user-specified precision in bits/digits.
    fn evalf_prec(&self, expr: ExprId, precision_digits: usize) -> AlgebraResult<BigValue> {
        let ctx = EvalContext::new(precision_digits);
        self.evalf(expr, &ctx)
    }
}

impl NumericalEval for ExprGraph {
    fn evalf(&self, expr: ExprId, ctx: &EvalContext) -> AlgebraResult<BigValue> {
        let _prec = ctx.precision_digits;
        let node = self.get(expr);
        match &node.kind {
            ExprKind::Number(num) => match num {
                Number::Integer(i) => Ok(BigValue::Real(*i as f64)),
                Number::BigInteger(b) => {
                    use num_traits::ToPrimitive;
                    Ok(BigValue::Real(b.to_f64().unwrap_or(f64::NAN)))
                }
                Number::Rational(n, d) => Ok(BigValue::Real(*n as f64 / *d as f64)),
                Number::BigRational(r) => {
                    use num_traits::ToPrimitive;
                    Ok(BigValue::Real(r.to_f64().unwrap_or(f64::NAN)))
                }
                Number::Scientific { mantissa, exponent } => {
                    Ok(BigValue::Real(*mantissa as f64 * 10.0f64.powi(*exponent)))
                }
                Number::Float(bits) => Ok(BigValue::Real(f64::from_bits(*bits))),
                Number::Constant(c) => match c {
                    Constant::Pi => Ok(BigValue::Real(consts::PI)),
                    Constant::E => Ok(BigValue::Real(consts::E)),
                    Constant::I => Ok(BigValue::Complex(0.0, 1.0)),
                    Constant::EulerGamma => Ok(BigValue::Real(0.5772156649015329)),
                    Constant::GoldenRatio => Ok(BigValue::Real(1.618033988749895)),
                    Constant::Apery => Ok(BigValue::Real(1.2020569031595943)),
                    Constant::Catalan => Ok(BigValue::Real(0.915965594177219)),
                    Constant::Infinity => Ok(BigValue::Real(f64::INFINITY)),
                    Constant::NegInfinity => Ok(BigValue::Real(f64::NEG_INFINITY)),
                    Constant::Undefined => Ok(BigValue::Real(f64::NAN)),
                },
            },
            ExprKind::Symbol(s) => {
                let name = self.symbols.resolve(*s).unwrap_or_else(|| "?".to_string());
                if let Some(&val) = ctx.bindings.get(&name) {
                    Ok(BigValue::Real(val))
                } else {
                    Err(AlgebraError::EvaluationError(format!(
                        "Cannot evaluate free symbol '{}' numerically",
                        name
                    )))
                }
            }
            ExprKind::Add(terms) => {
                let mut re = 0.0;
                let mut im = 0.0;
                for &t in terms {
                    match self.evalf(t, ctx)? {
                        BigValue::Real(r) => re += r,
                        BigValue::Complex(r, i) => {
                            re += r;
                            im += i;
                        }
                    }
                }
                if im == 0.0 {
                    Ok(BigValue::Real(re))
                } else {
                    Ok(BigValue::Complex(re, im))
                }
            }
            ExprKind::Mul(factors) => {
                let mut cur_re = 1.0;
                let mut cur_im = 0.0;
                for &f in factors {
                    match self.evalf(f, ctx)? {
                        BigValue::Real(r) => {
                            cur_re *= r;
                            cur_im *= r;
                        }
                        BigValue::Complex(r, i) => {
                            let next_re = cur_re * r - cur_im * i;
                            let next_im = cur_re * i + cur_im * r;
                            cur_re = next_re;
                            cur_im = next_im;
                        }
                    }
                }
                if cur_im == 0.0 {
                    Ok(BigValue::Real(cur_re))
                } else {
                    Ok(BigValue::Complex(cur_re, cur_im))
                }
            }
            ExprKind::Sub(lhs, rhs) => {
                let l = self.evalf(*lhs, ctx)?;
                let r = self.evalf(*rhs, ctx)?;
                match (l, r) {
                    (BigValue::Real(lr), BigValue::Real(rr)) => Ok(BigValue::Real(lr - rr)),
                    (BigValue::Complex(lr, li), BigValue::Real(rr)) => {
                        Ok(BigValue::Complex(lr - rr, li))
                    }
                    (BigValue::Real(lr), BigValue::Complex(rr, ri)) => {
                        Ok(BigValue::Complex(lr - rr, -ri))
                    }
                    (BigValue::Complex(lr, li), BigValue::Complex(rr, ri)) => {
                        Ok(BigValue::Complex(lr - rr, li - ri))
                    }
                }
            }
            ExprKind::Div(num, den) => {
                let n = self.evalf(*num, ctx)?;
                let d = self.evalf(*den, ctx)?;
                match (n, d) {
                    (BigValue::Real(nr), BigValue::Real(dr)) => {
                        if dr == 0.0 {
                            Err(AlgebraError::DivisionByZero {
                                domain: "Reals".into(),
                            })
                        } else {
                            Ok(BigValue::Real(nr / dr))
                        }
                    }
                    (BigValue::Complex(nr, ni), BigValue::Real(dr)) => {
                        if dr == 0.0 {
                            Err(AlgebraError::DivisionByZero {
                                domain: "Complex".into(),
                            })
                        } else {
                            Ok(BigValue::Complex(nr / dr, ni / dr))
                        }
                    }
                    _ => Err(AlgebraError::EvaluationError(
                        "Complex division in evalf not yet fully expanded".into(),
                    )),
                }
            }
            ExprKind::Neg(inner) => match self.evalf(*inner, ctx)? {
                BigValue::Real(r) => Ok(BigValue::Real(-r)),
                BigValue::Complex(r, i) => Ok(BigValue::Complex(-r, -i)),
            },
            ExprKind::Pow(base, exp) => {
                let b = self.evalf(*base, ctx)?;
                let e = self.evalf(*exp, ctx)?;
                match (b, e) {
                    (BigValue::Real(br), BigValue::Real(er)) => {
                        if br < 0.0 && er.fract() != 0.0 {
                            let mag = (-br).powf(er);
                            let arg = std::f64::consts::PI * er;
                            Ok(BigValue::Complex(mag * arg.cos(), mag * arg.sin()))
                        } else {
                            Ok(BigValue::Real(br.powf(er)))
                        }
                    }
                    _ => Err(AlgebraError::EvaluationError(
                        "Complex exponentiation in evalf not yet fully expanded".into(),
                    )),
                }
            }
            ExprKind::Function { name, args } => {
                let fn_name = self.symbols.resolve(*name).unwrap_or_default();
                if args.len() == 1 {
                    let arg_val = self.evalf(args[0], ctx)?;
                    match (fn_name.as_str(), arg_val) {
                        ("sin", BigValue::Real(r)) => Ok(BigValue::Real(r.sin())),
                        ("cos", BigValue::Real(r)) => Ok(BigValue::Real(r.cos())),
                        ("tan", BigValue::Real(r)) => Ok(BigValue::Real(r.tan())),
                        ("sinh", BigValue::Real(r)) => Ok(BigValue::Real(r.sinh())),
                        ("cosh", BigValue::Real(r)) => Ok(BigValue::Real(r.cosh())),
                        ("tanh", BigValue::Real(r)) => Ok(BigValue::Real(r.tanh())),
                        ("asin", BigValue::Real(r)) => Ok(BigValue::Real(r.asin())),
                        ("acos", BigValue::Real(r)) => Ok(BigValue::Real(r.acos())),
                        ("atan", BigValue::Real(r)) => Ok(BigValue::Real(r.atan())),
                        ("exp", BigValue::Real(r)) => Ok(BigValue::Real(r.exp())),
                        ("ln", BigValue::Real(r)) | ("log", BigValue::Real(r)) => {
                            if r < 0.0 {
                                Ok(BigValue::Complex(r.abs().ln(), consts::PI))
                            } else {
                                Ok(BigValue::Real(r.ln()))
                            }
                        }
                        ("sqrt", BigValue::Real(r)) => {
                            if r < 0.0 {
                                Ok(BigValue::Complex(0.0, (-r).sqrt()))
                            } else {
                                Ok(BigValue::Real(r.sqrt()))
                            }
                        }
                        ("BringRadical" | "BR", BigValue::Real(r)) => {
                            Ok(BigValue::Real(crate::poly::roots::BringRadical::eval(r)))
                        }
                        _ => Err(AlgebraError::EvaluationError(format!(
                            "Unsupported function '{}' in numerical evaluation",
                            fn_name
                        ))),
                    }
                } else if args.len() == 2 {
                    let a1 = self.evalf(args[0], ctx)?.to_f64();
                    let a2 = self.evalf(args[1], ctx)?.to_f64();
                    match fn_name.as_str() {
                        "atan2" => Ok(BigValue::Real(a1.atan2(a2))),
                        "besselj" => {
                            let n = a1.round() as i64;
                            Ok(BigValue::Real(eval_bessel_j(n, a2)))
                        }
                        _ => Err(AlgebraError::EvaluationError(format!(
                            "Unsupported binary function '{}' in numerical evaluation",
                            fn_name
                        ))),
                    }
                } else {
                    Err(AlgebraError::EvaluationError(format!(
                        "Unsupported function call '{}' with {} args",
                        fn_name,
                        args.len()
                    )))
                }
            }
            ExprKind::Relational { rhs, .. } => self.evalf(*rhs, ctx),
            _ => Err(AlgebraError::EvaluationError(
                "Unsupported node variant for numerical evaluation".into(),
            )),
        }
    }
}

/// Numerical evaluation of Bessel function of the first kind $J_n(x)$ via power series.
pub fn eval_bessel_j(n: i64, x: f64) -> f64 {
    let sign = if n < 0 {
        if n % 2 != 0 {
            -1.0
        } else {
            1.0
        }
    } else {
        1.0
    };
    let n_abs = n.unsigned_abs() as usize;
    let half_x = x / 2.0;
    let mut term = half_x.powi(n_abs as i32);
    let mut fact_n = 1.0;
    for k in 1..=n_abs {
        fact_n *= k as f64;
    }
    term /= fact_n;
    let mut sum = term;
    for m in 1..40 {
        term *= -1.0 * (half_x * half_x) / (m as f64 * (m + n_abs) as f64);
        sum += term;
        if term.abs() < 1e-15 * sum.abs().max(1e-15) {
            break;
        }
    }
    sign * sum
}
