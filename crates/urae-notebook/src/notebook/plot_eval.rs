//! # `urae_notebook::notebook::plot_eval`
//!
//! Domain-Aware Adaptive Plot Point Sampling, Discontinuity Detection, and Bytecode-Accelerated Numerical Curve Evaluation.

use crate::notebook::parser::ObjectKind;
use crate::notebook::session::SymbolMetadata;
use std::collections::{HashMap, HashSet};
use urae::prelude::*;

/// Extract all unique variable symbol names contained in an expression tree.
pub fn extract_symbols(graph: &ExprGraph, root: ExprId) -> Vec<String> {
    let mut symbols = HashSet::new();
    let mut stack = vec![root];

    while let Some(curr) = stack.pop() {
        let node = graph.get(curr);
        match &node.kind {
            ExprKind::Symbol(sym_id) => {
                if let Some(name) = graph.symbols.resolve(*sym_id) {
                    symbols.insert(name);
                }
            }
            ExprKind::Function { name: _, args } => {
                for &arg in args {
                    stack.push(arg);
                }
            }
            _ => {
                for child in node.children() {
                    stack.push(child);
                }
            }
        }
    }

    let mut list: Vec<_> = symbols.into_iter().collect();
    list.sort();
    list
}

/// Adaptive subdivision helper that refines sample points based on curvature / midpoint deflection.
#[allow(clippy::too_many_arguments)]
fn sample_adaptive_recursive<F>(
    f: &F,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    depth: usize,
    max_depth: usize,
    tol: f64,
    points: &mut Vec<[f64; 2]>,
) where
    F: Fn(f64) -> f64,
{
    if depth >= max_depth || (x2 - x1).abs() < 1e-12 {
        if y2.is_finite() {
            points.push([x2, y2]);
        }
        return;
    }

    let xm = 0.5 * (x1 + x2);
    let ym = f(xm);

    // Check for discontinuity or pole
    let is_discontinuity = !ym.is_finite()
        || !y1.is_finite()
        || !y2.is_finite()
        || ((y1 > 0.0 && y2 < 0.0 || y1 < 0.0 && y2 > 0.0)
            && (y2 - y1).abs() / (x2 - x1).abs().max(1e-9) > 1e4
            && ym.abs() > y1.abs().max(y2.abs()));

    if is_discontinuity {
        // Discontinuity: stop connecting across asymptote
        if ym.is_finite() {
            points.push([xm, ym]);
        }
        if y2.is_finite() {
            points.push([x2, y2]);
        }
        return;
    }

    // Curvature deflection test: distance from midpoint to chord
    let y_chord = 0.5 * (y1 + y2);
    let deflection = (ym - y_chord).abs();

    if deflection > tol {
        sample_adaptive_recursive(f, x1, y1, xm, ym, depth + 1, max_depth, tol, points);
        sample_adaptive_recursive(f, xm, ym, x2, y2, depth + 1, max_depth, tol, points);
    } else if y2.is_finite() {
        points.push([x2, y2]);
    }
}

/// Evaluate plot points `[x, y]` for a plot expression string across range with domain constraint enforcement.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_plot_points_with_domains(
    graph: &ExprGraph,
    slider_values: &HashMap<String, f64>,
    symbol_metadata: Option<&HashMap<String, SymbolMetadata>>,
    expr_str: &str,
    fallback_var: &str,
    range_min: f64,
    range_max: f64,
    samples: usize,
) -> Vec<[f64; 2]> {
    let clean_expr = if let Some(idx) = expr_str.find('[') {
        expr_str[..idx].trim()
    } else {
        expr_str.trim()
    };

    let parser = ExprParser::new(graph);
    let parsed = match parser.parse(clean_expr) {
        Ok(id) => id,
        Err(_) => return Vec::new(),
    };

    let target_expr = match &graph.get(parsed).kind {
        ExprKind::Relational { rhs, .. } => *rhs,
        _ => parsed,
    };

    // Dynamically detect free independent variable
    let free_syms = extract_symbols(graph, target_expr);
    let active_var = free_syms
        .into_iter()
        .find(|s| !slider_values.contains_key(s) || s == "x" || s == "t" || s == "u")
        .unwrap_or_else(|| fallback_var.to_string());

    let var_meta = symbol_metadata.and_then(|m| m.get(&active_var));

    let r_min = if range_min.is_finite() && range_min > -1e6 && range_min < 1e6 {
        range_min
    } else {
        var_meta.map(|m| m.min_val).unwrap_or(-5.0)
    };
    let r_max = if range_max.is_finite() && range_max > r_min && range_max < 1e6 {
        range_max
    } else {
        var_meta.map(|m| m.max_val).unwrap_or(r_min + 10.0)
    };

    if !r_min.is_finite() || !r_max.is_finite() || r_max <= r_min {
        return Vec::new();
    }

    // Collect parameter names and values
    let mut param_names = Vec::new();
    let mut param_values = Vec::new();
    for (param, &val) in slider_values {
        if param != &active_var {
            param_names.push(param.clone());
            param_values.push(val);
        }
    }

    // Try compiling to Bytecode VM for 50-100x speedup
    let bytecode_prog = VmCompiler::compile(graph, target_expr, &active_var, &param_names).ok();
    let vm = BytecodeVM::new();

    let mut fallback_ctx = EvalContext::default();
    for (param, &val) in slider_values {
        if param != &active_var {
            fallback_ctx.bindings.insert(param.clone(), val);
        }
    }
    fallback_ctx.bindings.insert(active_var.clone(), 0.0);

    // Unified closure evaluating f(x)
    let eval_fn = |x: f64| -> f64 {
        if let Some(m) = var_meta {
            if x < m.min_val || x > m.max_val {
                return f64::NAN;
            }
            if m.domain_type == "Positive" && x <= 0.0 {
                return f64::NAN;
            }
            if m.domain_type == "NonNegative" && x < 0.0 {
                return f64::NAN;
            }
        }

        if let Some(prog) = &bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = fallback_ctx.clone();
            if let Some(slot) = ctx.bindings.get_mut(&active_var) {
                *slot = x;
            }
            if let Ok(big_val) = graph.evalf(target_expr, &ctx) {
                let y = big_val.to_f64();
                if y.is_finite() {
                    return y;
                }
            }
            f64::NAN
        }
    };

    // Base initial coarse grid
    let num_coarse = samples.clamp(32, 256);
    let step = (r_max - r_min) / ((num_coarse - 1) as f64);
    let mut points = Vec::with_capacity(samples * 2);

    let mut prev_x = r_min;
    let mut prev_y = eval_fn(prev_x);
    if prev_y.is_finite() {
        points.push([prev_x, prev_y]);
    }

    let tol = 0.008 * (r_max - r_min).abs();

    for i in 1..num_coarse {
        let curr_x = (r_min + (i as f64) * step).min(r_max);
        let curr_y = eval_fn(curr_x);

        sample_adaptive_recursive(
            &eval_fn,
            prev_x,
            prev_y,
            curr_x,
            curr_y,
            0,
            5,
            tol,
            &mut points,
        );

        prev_x = curr_x;
        prev_y = curr_y;
    }

    // Filter out asymptotes where consecutive points jump wildly across screen
    let mut filtered = Vec::with_capacity(points.len());
    for i in 0..points.len() {
        let p = points[i];
        if !p[0].is_finite() || !p[1].is_finite() {
            continue;
        }

        if i > 0 {
            let prev = points[i - 1];
            let dx = (p[0] - prev[0]).abs();
            let dy = (p[1] - prev[1]).abs();
            // Pole jump check
            if dx > 1e-9 && (dy / dx) > 1e6 && (p[1] * prev[1] < 0.0) {
                continue;
            }
        }
        filtered.push(p);
    }

    filtered
}

/// Structure describing periodic characteristics of a mathematical expression.
#[derive(Debug, Clone, PartialEq)]
pub struct PeriodicAnalysis {
    /// Discovered angular frequencies (rad/unit)
    pub frequencies: Vec<f64>,
    /// Discovered sub-periods
    pub periods: Vec<f64>,
    /// Minimum sub-period
    pub min_period: f64,
    /// Maximum sub-period
    pub max_period: f64,
    /// Ratio of max to min period
    pub period_ratio: f64,
}

/// Detect trigonometric / periodic function components in an expression tree.
pub fn detect_periodic_sub_periods(
    graph: &ExprGraph,
    target_expr: ExprId,
    active_var: &str,
    slider_values: &HashMap<String, f64>,
) -> Option<PeriodicAnalysis> {
    let mut periods = Vec::new();
    let mut stack = vec![target_expr];

    let mut eval_ctx = EvalContext::default();
    for (k, &v) in slider_values {
        eval_ctx.bindings.insert(k.clone(), v);
    }
    eval_ctx.bindings.insert(active_var.to_string(), 0.0);

    while let Some(curr) = stack.pop() {
        let node = graph.get(curr);
        match &node.kind {
            ExprKind::Function { name, args } => {
                if let Some(fn_name) = graph.symbols.resolve(*name) {
                    let is_trig = matches!(
                        fn_name.as_str(),
                        "sin" | "cos" | "tan" | "sec" | "csc" | "cot" | "sinc"
                    );
                    if is_trig && !args.is_empty() {
                        let arg_id = args[0];
                        let arg_free_syms = extract_symbols(graph, arg_id);
                        if arg_free_syms.contains(&active_var.to_string()) {
                            let var_sym = graph.symbols.get_or_intern(active_var);
                            let diff_id = graph.diff(arg_id, var_sym);
                            let omega_opt = if let Ok(val) = graph.evalf(diff_id, &eval_ctx) {
                                let w = val.to_f64().abs();
                                if w.is_finite() && w > 1e-6 {
                                    Some(w)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            let omega = omega_opt.unwrap_or_else(|| {
                                let mut ctx0 = eval_ctx.clone();
                                ctx0.bindings.insert(active_var.to_string(), 0.0);
                                let mut ctx1 = eval_ctx.clone();
                                ctx1.bindings.insert(active_var.to_string(), 1.0);
                                let v0 = graph
                                    .evalf(arg_id, &ctx0)
                                    .map(|v| v.to_f64())
                                    .unwrap_or(0.0);
                                let v1 = graph
                                    .evalf(arg_id, &ctx1)
                                    .map(|v| v.to_f64())
                                    .unwrap_or(1.0);
                                (v1 - v0).abs().max(1e-6)
                            });

                            let base_period = if matches!(fn_name.as_str(), "tan" | "cot") {
                                std::f64::consts::PI / omega
                            } else {
                                2.0 * std::f64::consts::PI / omega
                            };

                            if base_period.is_finite() && base_period > 1e-4 && base_period < 1e6 {
                                periods.push(base_period);
                            }
                        }
                    }
                }
                for &arg in args {
                    stack.push(arg);
                }
            }
            _ => {
                for child in node.children() {
                    stack.push(child);
                }
            }
        }
    }

    if periods.is_empty() {
        return None;
    }

    periods.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let min_p = periods[0];
    let max_p = periods[periods.len() - 1];
    let ratio = max_p / min_p.max(1e-9);

    let freqs = periods
        .iter()
        .map(|p| 2.0 * std::f64::consts::PI / p)
        .collect();

    Some(PeriodicAnalysis {
        frequencies: freqs,
        periods: periods.clone(),
        min_period: min_p,
        max_period: max_p,
        period_ratio: ratio,
    })
}

/// Numerically find x-intercepts (roots of f(x) = 0) and critical points (roots of f'(x) = 0).
pub fn find_roots_and_critical_points<F, DF>(
    f: &F,
    df: &DF,
    search_min: f64,
    search_max: f64,
    steps: usize,
) -> (Vec<f64>, Vec<f64>)
where
    F: Fn(f64) -> f64,
    DF: Fn(f64) -> f64,
{
    let mut roots = Vec::new();
    let mut critical_pts = Vec::new();

    let step_size = (search_max - search_min) / (steps as f64);
    let mut prev_x = search_min;
    let mut prev_y = f(prev_x);
    let mut prev_dy = df(prev_x);

    for i in 1..=steps {
        let curr_x = search_min + (i as f64) * step_size;
        let curr_y = f(curr_x);
        let curr_dy = df(curr_x);

        // Check root in f(x)
        if prev_y.is_finite() && curr_y.is_finite() {
            if prev_y == 0.0 {
                roots.push(prev_x);
            } else if prev_y * curr_y < 0.0 && (curr_y - prev_y).abs() < 1e4 {
                let mut lo = prev_x;
                let mut hi = curr_x;
                let mut flo = prev_y;
                for _ in 0..16 {
                    let mid = 0.5 * (lo + hi);
                    let fmid = f(mid);
                    if !fmid.is_finite() || fmid.abs() < 1e-12 {
                        lo = mid;
                        break;
                    }
                    if flo * fmid < 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                        flo = fmid;
                    }
                }
                let root_val = 0.5 * (lo + hi);
                if roots
                    .last()
                    .map(|&r| (r - root_val).abs() > 1e-3)
                    .unwrap_or(true)
                {
                    roots.push(root_val);
                }
            }
        }

        // Check critical point in f'(x)
        if prev_dy.is_finite() && curr_dy.is_finite() {
            if prev_dy == 0.0 {
                critical_pts.push(prev_x);
            } else if prev_dy * curr_dy < 0.0 && (curr_dy - prev_dy).abs() < 1e4 {
                let mut lo = prev_x;
                let mut hi = curr_x;
                let mut dflo = prev_dy;
                for _ in 0..16 {
                    let mid = 0.5 * (lo + hi);
                    let dfmid = df(mid);
                    if !dfmid.is_finite() || dfmid.abs() < 1e-12 {
                        lo = mid;
                        break;
                    }
                    if dflo * dfmid < 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                        dflo = dfmid;
                    }
                }
                let crit_val = 0.5 * (lo + hi);
                if critical_pts
                    .last()
                    .map(|&c| (c - crit_val).abs() > 1e-3)
                    .unwrap_or(true)
                {
                    critical_pts.push(crit_val);
                }
            }
        }

        prev_x = curr_x;
        prev_y = curr_y;
        prev_dy = curr_dy;
    }

    (roots, critical_pts)
}

/// Compute intelligent default plot bounds [x_min, x_max] for a given mathematical expression.
///
/// Graphing defaults adhere to:
/// 1. Covers at least `x in [-1.0, 1.0]`.
/// 2. Extends to contain all critical points (extrema / inflection points) and x-intercepts (roots).
/// 3. For periodic functions (trig, harmonics, modulated waves):
///    - If sub-period ratio is small (<= 3.0): covers 1-3 full periods of the fundamental wave.
///    - If sub-period ratio is large (> 3.0): covers 1-2 periods of the modulation envelope.
/// 4. Respects natural non-negative domains (e.g. sqrt(x) -> x >= 0, ln(x) -> x > 0) and declared bounds.
/// 5. Adds comfortable 15-20% visual margins around extreme feature points.
pub fn compute_smart_plot_bounds(
    graph: &ExprGraph,
    slider_values: &HashMap<String, f64>,
    symbol_metadata: Option<&HashMap<String, SymbolMetadata>>,
    expr_str: &str,
    fallback_var: &str,
) -> (f64, f64) {
    let clean_expr = if let Some(idx) = expr_str.find('[') {
        expr_str[..idx].trim()
    } else {
        expr_str.trim()
    };

    let parser = ExprParser::new(graph);
    let parsed = match parser.parse(clean_expr) {
        Ok(id) => id,
        Err(_) => return (-5.0, 5.0),
    };

    let target_expr = match &graph.get(parsed).kind {
        ExprKind::Relational { rhs, .. } => *rhs,
        _ => parsed,
    };

    let free_syms = extract_symbols(graph, target_expr);
    let active_var = free_syms
        .into_iter()
        .find(|s| !slider_values.contains_key(s) || s == "x" || s == "t" || s == "u")
        .unwrap_or_else(|| fallback_var.to_string());

    let var_meta = symbol_metadata.and_then(|m| m.get(&active_var));

    // If explicit user domain metadata exists with custom bounds:
    if let Some(m) = var_meta {
        if (m.min_val != -10.0 || m.max_val != 10.0)
            && m.min_val.is_finite()
            && m.max_val.is_finite()
            && m.max_val > m.min_val
        {
            return (m.min_val, m.max_val);
        }
    }

    // 1. Check for Periodic / Trigonometric Behavior
    if let Some(periodic) =
        detect_periodic_sub_periods(graph, target_expr, &active_var, slider_values)
    {
        let periods_to_show = if periodic.period_ratio <= 1.5 {
            2.0
        } else if periodic.period_ratio <= 3.0 {
            2.5
        } else {
            1.5
        };

        let span = periodic.max_period * periods_to_show;
        let half_span = 0.5 * span;

        let (mut p_min, mut p_max) = if let Some(m) = var_meta {
            if m.domain_type == "Positive" || m.domain_type == "NonNegative" {
                (0.0, span)
            } else {
                (-half_span, half_span)
            }
        } else {
            (-half_span, half_span)
        };

        // Guarantee covering at least [-1.0, 1.0]
        if p_min > -1.0 {
            p_min = -1.0;
        }
        if p_max < 1.0 {
            p_max = 1.0;
        }

        return (p_min, p_max);
    }

    // 2. Setup Evaluation Closures for f(x) and f'(x)
    let var_sym = graph.symbols.get_or_intern(&active_var);
    let diff_expr = graph.diff(target_expr, var_sym);

    let param_names: Vec<String> = slider_values.keys().cloned().collect();
    let param_values: Vec<f64> = param_names.iter().map(|k| slider_values[k]).collect();

    let bytecode_prog = VmCompiler::compile(graph, target_expr, &active_var, &param_names).ok();
    let diff_bytecode_prog = VmCompiler::compile(graph, diff_expr, &active_var, &param_names).ok();
    let vm = BytecodeVM::new();

    let mut eval_ctx = EvalContext::default();
    for (k, &v) in slider_values {
        if k != &active_var {
            eval_ctx.bindings.insert(k.clone(), v);
        }
    }
    eval_ctx.bindings.insert(active_var.clone(), 0.0);

    let eval_f = |x: f64| -> f64 {
        if let Some(prog) = &bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = eval_ctx.clone();
            ctx.bindings.insert(active_var.clone(), x);
            graph
                .evalf(target_expr, &ctx)
                .map(|v| v.to_f64())
                .unwrap_or(f64::NAN)
        }
    };

    let eval_df = |x: f64| -> f64 {
        if let Some(prog) = &diff_bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = eval_ctx.clone();
            ctx.bindings.insert(active_var.clone(), x);
            graph
                .evalf(diff_expr, &ctx)
                .map(|v| v.to_f64())
                .unwrap_or(f64::NAN)
        }
    };

    // 3. Natural domain detection
    let neg_valid = eval_f(-0.5).is_finite() && eval_f(-1.0).is_finite();
    let pos_valid = eval_f(0.5).is_finite() && eval_f(1.0).is_finite();

    let (search_min, mut feature_points) = if neg_valid {
        (-30.0, vec![-1.0, 1.0])
    } else if pos_valid {
        (0.0, vec![0.0, 1.0])
    } else {
        (-30.0, vec![-1.0, 1.0])
    };

    // 4. Search for Roots and Critical Points across valid domain
    let (roots, crit_pts) =
        find_roots_and_critical_points(&eval_f, &eval_df, search_min, 30.0, 240);

    for r in roots {
        if r.is_finite() && r.abs() < 25.0 {
            feature_points.push(r);
        }
    }
    for c in crit_pts {
        if c.is_finite() && c.abs() < 25.0 {
            feature_points.push(c);
        }
    }

    feature_points.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let feat_min = feature_points[0];
    let feat_max = feature_points[feature_points.len() - 1];

    let span = feat_max - feat_min;
    let margin = (0.20 * span).max(0.5);

    let mut smart_min = feat_min - margin;
    let mut smart_max = feat_max + margin;

    // 5. Natural domain clamping for non-negative functions (sqrt, ln)
    if !neg_valid && pos_valid {
        smart_min = 0.0;
        if smart_max < 2.0 {
            smart_max = 2.0;
        }
    }

    if let Some(m) = var_meta {
        if m.domain_type == "Positive" {
            smart_min = smart_min.max(0.01);
        } else if m.domain_type == "NonNegative" {
            smart_min = smart_min.max(0.0);
        }
    }

    if neg_valid && smart_min > -1.0 {
        smart_min = -1.0;
    }
    if smart_max < 1.0 {
        smart_max = 1.0;
    }

    (smart_min, smart_max)
}

/// Evaluate plot points `[x, y]` for a plot expression string across range.
pub fn evaluate_plot_points(
    graph: &ExprGraph,
    slider_values: &HashMap<String, f64>,
    expr_str: &str,
    fallback_var: &str,
    range_min: f64,
    range_max: f64,
    samples: usize,
) -> Vec<[f64; 2]> {
    evaluate_plot_points_with_domains(
        graph,
        slider_values,
        None,
        expr_str,
        fallback_var,
        range_min,
        range_max,
        samples,
    )
}

/// Comprehensive mathematical and numerical feature extraction for a function expression.
/// Returns an ObjectKind::Function populated with exact symbolic roots/extrema if discovered,
/// or VM-evaluated roots and critical points, along with domain, codomain, range, parity, and period.
pub fn inspect_function_features(
    graph: &ExprGraph,
    slider_values: &HashMap<String, f64>,
    symbol_metadata: Option<&HashMap<String, SymbolMetadata>>,
    expr_str: &str,
    fallback_var: &str,
    unicode_formatter: &urae::format::UnicodeFormatter,
) -> ObjectKind {
    let clean_expr = if let Some(idx) = expr_str.find('[') {
        expr_str[..idx].trim()
    } else {
        expr_str.trim()
    };

    let expr_to_analyze = if let Some((_, rhs)) = clean_expr.split_once('=') {
        rhs.trim()
    } else {
        clean_expr
    };

    let parser = ExprParser::new(graph);
    let parsed = match parser.parse(expr_to_analyze) {
        Ok(id) => id,
        Err(_) => {
            return ObjectKind::Function {
                arity: 1,
                domain: "ℝ".to_string(),
                codomain: "ℝ".to_string(),
                range: None,
                is_linear: false,
                roots: Vec::new(),
                is_roots_symbolic: false,
                critical_points: Vec::new(),
                is_critical_points_symbolic: false,
                parity: None,
                period: None,
            };
        }
    };

    let target_expr = match &graph.get(parsed).kind {
        ExprKind::Relational { rhs, .. } => *rhs,
        _ => parsed,
    };

    let free_syms = extract_symbols(graph, target_expr);
    let arity = free_syms.len().max(1);
    let active_var = free_syms
        .into_iter()
        .find(|s| !slider_values.contains_key(s) || s == "x" || s == "t" || s == "u")
        .unwrap_or_else(|| fallback_var.to_string());

    let var_meta = symbol_metadata.and_then(|m| m.get(&active_var));
    let var_sym = graph.symbols.get_or_intern(&active_var);
    let diff_expr = graph.diff(target_expr, var_sym);
    let diff2_expr = graph.diff(diff_expr, var_sym);

    let param_names: Vec<String> = slider_values.keys().cloned().collect();
    let param_values: Vec<f64> = param_names.iter().map(|k| slider_values[k]).collect();

    let bytecode_prog = VmCompiler::compile(graph, target_expr, &active_var, &param_names).ok();
    let diff_bytecode_prog = VmCompiler::compile(graph, diff_expr, &active_var, &param_names).ok();
    let diff2_bytecode_prog =
        VmCompiler::compile(graph, diff2_expr, &active_var, &param_names).ok();
    let vm = BytecodeVM::new();

    let mut eval_ctx = EvalContext::default();
    for (k, &v) in slider_values {
        if k != &active_var {
            eval_ctx.bindings.insert(k.clone(), v);
        }
    }
    eval_ctx.bindings.insert(active_var.clone(), 0.0);

    let eval_f = |x: f64| -> f64 {
        if let Some(prog) = &bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = eval_ctx.clone();
            ctx.bindings.insert(active_var.clone(), x);
            graph
                .evalf(target_expr, &ctx)
                .map(|v| v.to_f64())
                .unwrap_or(f64::NAN)
        }
    };

    let eval_df = |x: f64| -> f64 {
        if let Some(prog) = &diff_bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = eval_ctx.clone();
            ctx.bindings.insert(active_var.clone(), x);
            graph
                .evalf(diff_expr, &ctx)
                .map(|v| v.to_f64())
                .unwrap_or(f64::NAN)
        }
    };

    let eval_d2f = |x: f64| -> f64 {
        if let Some(prog) = &diff2_bytecode_prog {
            if let Ok(val) = vm.eval_scalar(prog, x, &param_values) {
                if val.is_finite() {
                    return val;
                }
            }
            f64::NAN
        } else {
            let mut ctx = eval_ctx.clone();
            ctx.bindings.insert(active_var.clone(), x);
            graph
                .evalf(diff2_expr, &ctx)
                .map(|v| v.to_f64())
                .unwrap_or(f64::NAN)
        }
    };

    // 1. Natural Domain
    let neg_valid = eval_f(-0.5).is_finite() && eval_f(-1.0).is_finite();
    let pos_valid = eval_f(0.5).is_finite() && eval_f(1.0).is_finite();

    let domain = if let Some(m) = var_meta {
        format!("{}: [{:.2}, {:.2}]", m.domain_type, m.min_val, m.max_val)
    } else if !neg_valid && pos_valid {
        "[0, +∞)".to_string()
    } else {
        "ℝ".to_string()
    };

    let codomain = "ℝ".to_string();

    // 2. Linearity
    let is_linear =
        eval_d2f(0.0).abs() < 1e-9 && eval_d2f(1.0).abs() < 1e-9 && eval_d2f(-1.0).abs() < 1e-9;

    // 3. Periodicity
    let periodic_info = detect_periodic_sub_periods(graph, target_expr, &active_var, slider_values);
    let period = periodic_info.as_ref().map(|p| p.max_period);

    // 4. Parity
    let parity = {
        let f_1 = eval_f(1.0);
        let f_neg1 = eval_f(-1.0);
        let f_2 = eval_f(2.0);
        let f_neg2 = eval_f(-2.0);

        if f_1.is_finite() && f_neg1.is_finite() && f_2.is_finite() && f_neg2.is_finite() {
            if (f_1 - f_neg1).abs() < 1e-6 && (f_2 - f_neg2).abs() < 1e-6 {
                Some("Even: f(-x) = f(x)".to_string())
            } else if (f_1 + f_neg1).abs() < 1e-6 && (f_2 + f_neg2).abs() < 1e-6 {
                Some("Odd: f(-x) = -f(x)".to_string())
            } else {
                Some("Neither Even nor Odd".to_string())
            }
        } else {
            None
        }
    };

    // 5. Roots (x-intercepts): Symbolic first, then VM fallback
    let mut roots = Vec::new();
    let mut is_roots_symbolic = false;

    if let Ok(sym_solutions) = graph.solveset(target_expr, var_sym) {
        if !sym_solutions.is_empty() {
            for &sol_id in &sym_solutions {
                if let Ok(sol_fmt) = unicode_formatter.format(graph, sol_id) {
                    if !sol_fmt.is_empty() {
                        roots.push(format!("{} = {}", active_var, sol_fmt));
                    }
                }
            }
            if !roots.is_empty() {
                is_roots_symbolic = true;
            }
        }
    }

    // VM numerical roots fallback
    let search_min = if neg_valid { -30.0 } else { 0.0 };
    let (vm_roots, vm_crit_pts) =
        find_roots_and_critical_points(&eval_f, &eval_df, search_min, 30.0, 240);

    if !is_roots_symbolic {
        for r in &vm_roots {
            if r.is_finite() && r.abs() < 25.0 {
                roots.push(format!("{} ≈ {:.4}", active_var, r));
            }
        }
    }

    // 6. Critical Points (f'(x) = 0): Symbolic first, then VM fallback
    let mut critical_points = Vec::new();
    let mut is_critical_points_symbolic = false;

    if let Ok(sym_crit_solutions) = graph.solveset(diff_expr, var_sym) {
        if !sym_crit_solutions.is_empty() {
            for &sol_id in &sym_crit_solutions {
                if let Ok(sol_fmt) = unicode_formatter.format(graph, sol_id) {
                    if !sol_fmt.is_empty() {
                        critical_points.push(format!("{} = {}", active_var, sol_fmt));
                    }
                }
            }
            if !critical_points.is_empty() {
                is_critical_points_symbolic = true;
            }
        }
    }

    if !is_critical_points_symbolic {
        for c in &vm_crit_pts {
            if c.is_finite() && c.abs() < 25.0 {
                let y_val = eval_f(*c);
                let d2_val = eval_d2f(*c);
                let kind_desc = if d2_val > 1e-4 {
                    "Local Minimum"
                } else if d2_val < -1e-4 {
                    "Local Maximum"
                } else {
                    "Inflection / Saddle Point"
                };
                if y_val.is_finite() {
                    critical_points.push(format!(
                        "{} ≈ {:.4} ({}, y ≈ {:.4})",
                        active_var, c, kind_desc, y_val
                    ));
                } else {
                    critical_points.push(format!("{} ≈ {:.4} ({})", active_var, c, kind_desc));
                }
            }
        }
    }

    // 7. Range Estimation
    let range = if is_linear {
        Some("ℝ".to_string())
    } else if let Some(p) = &periodic_info {
        let y0 = eval_f(0.0);
        let y_half = eval_f(0.25 * p.max_period);
        let y_max = y0.abs().max(y_half.abs()).max(1.0);
        Some(format!("[-{:.2}, +{:.2}]", y_max, y_max))
    } else if !vm_crit_pts.is_empty() {
        let mut crit_y: Vec<f64> = vm_crit_pts
            .iter()
            .map(|&c| eval_f(c))
            .filter(|y| y.is_finite())
            .collect();
        crit_y.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        if let (Some(&min_y), Some(&max_y)) = (crit_y.first(), crit_y.last()) {
            let y_far_pos = eval_f(25.0);
            let y_far_neg = eval_f(if neg_valid { -25.0 } else { 0.0 });
            if y_far_pos > max_y && y_far_neg > max_y {
                Some(format!("[{:.2}, +∞)", min_y))
            } else if y_far_pos < min_y && y_far_neg < min_y {
                Some(format!("(-∞, {:.2}]", max_y))
            } else {
                Some("ℝ".to_string())
            }
        } else {
            Some("ℝ".to_string())
        }
    } else {
        Some("ℝ".to_string())
    };

    ObjectKind::Function {
        arity,
        domain,
        codomain,
        range,
        is_linear,
        roots,
        is_roots_symbolic,
        critical_points,
        is_critical_points_symbolic,
        parity,
        period,
    }
}
