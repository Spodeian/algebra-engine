//! First-order equations the standard solver leaves unsolved: homogeneous equations `y' = F(y/x)` (the substitution
//! `v = y/x`) and exact equations `M dx + N dy = 0`.
//!
//! The standard path runs first; this method applies only when it answers with an analysis instead of a solution.

use crate::session::UraeSession;
use algebra_core::parser::ExprParser;
use std::collections::BTreeSet;

fn symbols_of(session: &UraeSession, text: &str) -> Option<BTreeSet<String>> {
    let graph = &session.graph;
    let id = ExprParser::new(graph).parse(text).ok()?;
    let mut ids = BTreeSet::new();
    crate::poly::collect_expr_symbols(graph, id, &mut ids);
    Some(
        ids.into_iter()
            .filter_map(|s| graph.symbols.resolve(s))
            .filter(|n| !matches!(n.as_str(), "pi" | "π" | "e" | "i"))
            .collect(),
    )
}

/// The value part of an `integrate` result (`F (w.r.t x)`).
fn strip_variable_note(text: &str) -> String {
    text.split(" (w.r.t")
        .next()
        .unwrap_or(text)
        .trim()
        .to_string()
}

/// `Some(result)` when `input` is a first-order `dsolve` that the standard solver does not solve and this does.
pub fn run(session: &mut UraeSession, input: &str) -> Option<Result<String, String>> {
    let t = input.trim();
    let body = t.strip_prefix("dsolve(")?.strip_suffix(')')?;
    if body.contains("''") || body.contains(", ") && body.contains("(0)") {
        return None;
    }
    let (left, right) = body.split_once('=')?;
    if right.contains('=') {
        return None;
    }
    // The dependent variable is the name before the first prime.
    let prime = body.find('\'')?;
    let name: String = body[..prime]
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if name.is_empty() {
        return None;
    }
    // Let the standard solver try first (and guard against asking ourselves).
    if session.in_first_order_fallback {
        return None;
    }
    session.in_first_order_fallback = true;
    let standard = session.eval(t);
    session.in_first_order_fallback = false;
    match &standard {
        Ok(text) if text.starts_with("Differential Equation Analysis") => {}
        _ => return None,
    }
    // E = left - right = M + N y'.
    let primed = format!("{name}'");
    let equation = format!("({left}) - ({right})").replace(&primed, "dydx");
    let derivative = session.eval(&format!("diff(({equation}), dydx)")).ok()?;
    let m = session
        .eval(&format!("subs(({equation}), dydx = 0)"))
        .ok()?;
    if derivative.trim() == "0" {
        return None;
    }
    let curvature = session.eval(&format!("diff(({derivative}), dydx)")).ok()?;
    if curvature.trim() != "0" {
        return match clairaut(session, &equation, &name) {
            Some(solution) => Some(solution),
            None => dalembert(session, &equation, &name),
        };
    }
    let n = derivative;
    let symbols = symbols_of(session, &format!("({m}) + ({n})"))?;
    let independent: Vec<&String> = symbols
        .iter()
        .filter(|s| **s != name && *s != "dydx")
        .collect();
    let [x] = independent.as_slice() else {
        return None;
    };
    let x = (*x).clone();
    let y = name;

    // Exact: M dx + N dy = 0 with dM/dy = dN/dx has the potential phi(x, y) = C1.
    let closed = session
        .eval(&format!("simplify(diff(({m}), {y}) - diff(({n}), {x}))"))
        .ok()?;
    if closed.trim() == "0" {
        let phi = potential(session, &m, &n, &x, &y)?;
        return Some(Ok(format!(
            "{phi} = C1  (implicit solution of the exact equation)"
        )));
    }

    // Homogeneous: y' = F(x, y) = -M / N with F(x, v x) free of x; v' x = F - v is separable.
    if let Some(solution) = homogeneous(session, &m, &n, &x, &y) {
        return Some(Ok(solution));
    }

    // Riccati: a quadratic right-hand side with a particular solution c x^k.
    if let Some(solution) = riccati(session, &m, &n, &x, &y) {
        return Some(solution);
    }

    // Integrating factors: mu(x) when (M_y - N_x) / N is free of y, mu(y) when (N_x - M_y) / M is free of x.
    for (divisor, other, variable, sign) in [(&n, &y, &x, ""), (&m, &x, &y, "-")] {
        let quotient = session
            .eval(&format!("simplify({sign}({closed})/({divisor}))"))
            .ok()?;
        if symbols_of(session, &quotient)?.contains(other) {
            continue;
        }
        let integral = session
            .eval(&format!("integrate(({quotient}), {variable})"))
            .ok()?;
        let integral = strip_variable_note(&integral);
        let mu = session.eval(&format!("exp({integral})")).ok()?;
        let new_m = session.eval(&format!("expand(({mu})*({m}))")).ok()?;
        let new_n = session.eval(&format!("expand(({mu})*({n}))")).ok()?;
        let check = session
            .eval(&format!(
                "simplify(diff(({new_m}), {y}) - diff(({new_n}), {x}))"
            ))
            .ok()?;
        if check.trim() != "0" {
            continue;
        }
        let phi = potential(session, &new_m, &new_n, &x, &y)?;
        return Some(Ok(format!(
            "{phi} = C1  (implicit solution; integrating factor {mu})"
        )));
    }
    None
}

/// The potential `phi` of an exact `M dx + N dy`: `phi_x = M`, `phi_y = N`.
fn potential(session: &mut UraeSession, m: &str, n: &str, x: &str, y: &str) -> Option<String> {
    let phi_x = session.eval(&format!("integrate(({m}), {x})")).ok()?;
    let phi_x = strip_variable_note(&phi_x);
    let remainder = session
        .eval(&format!("expand(({n}) - diff(({phi_x}), {y}))"))
        .ok()?;
    let h = session
        .eval(&format!("integrate(({remainder}), {y})"))
        .ok()?;
    let h = strip_variable_note(&h);
    session.eval(&format!("({phi_x}) + ({h})")).ok()
}

/// The solution of a homogeneous equation as text, or `None` when the equation is not homogeneous or cannot be integrated.
fn homogeneous(session: &mut UraeSession, m: &str, n: &str, x: &str, y: &str) -> Option<String> {
    let f = session.eval(&format!("simplify(-({m})/({n}))")).ok()?;
    let substituted = session.eval(&format!("subs(({f}), {y} = v*{x})")).ok()?;
    let g = session.eval(&format!("simplify({substituted})")).ok()?;
    if symbols_of(session, &g)?.contains(x) {
        return None;
    }
    let integrand = session
        .eval(&format!("cancel(together(1/(({g}) - v)))"))
        .ok()?;
    let integral = session.eval(&format!("integrate(({integrand}), v)")).ok()?;
    let integral = strip_variable_note(&integral);
    // Try to isolate v: G(v) = ln x + C1.
    if let Ok(solved) = session.eval(&format!("solve(({integral}) = ln({x}) + C1, v)"))
        && !solved.contains("RootOf")
        && let (Some(open), Some(close)) = (solved.find('['), solved.rfind(']'))
    {
        let roots: Vec<String> = solved[open + 1..close]
            .split(", ")
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
            .collect();
        let mut solutions = Vec::new();
        for root in &roots {
            let value = session.eval(&format!("simplify({x}*({root}))")).ok()?;
            solutions.push(format!("{y}({x}) = {value}"));
        }
        if !solutions.is_empty() {
            return Some(solutions.join("  or  "));
        }
    }
    let implicit = session
        .eval(&format!("subs(({integral}), v = {y}/{x})"))
        .ok()?;
    Some(format!(
        "{implicit} = ln({x}) + C1  (implicit solution: y cannot be isolated in closed form)"
    ))
}

/// Clairaut's equation `y = x y' + f(y')` (in any rearrangement): the lines `y = C x + f(C)` and the singular envelope
/// `x = -f'(p)`, `y = p x + f(p)`.
fn clairaut(session: &mut UraeSession, equation: &str, y: &str) -> Option<Result<String, String>> {
    let k = session.eval(&format!("diff(({equation}), {y})")).ok()?;
    let k = k.trim().to_string();
    if k == "0" || !symbols_of(session, &k)?.is_empty() {
        return None;
    }
    let symbols = symbols_of(session, equation)?;
    let independent: Vec<&String> = symbols.iter().filter(|s| *s != y && *s != "dydx").collect();
    let [x] = independent.as_slice() else {
        return None;
    };
    let x = (*x).clone();
    // E / k = y - x p - f(p), so f = y - x p - E / k must be free of x and y.
    let f = session
        .eval(&format!("simplify({y} - {x}*dydx - ({equation})/({k}))"))
        .ok()?;
    let used = symbols_of(session, &f)?;
    if used.contains(&x) || used.contains(y) {
        return None;
    }
    let general = session.eval(&format!("subs(({f}), dydx = C1)")).ok()?;
    let singular_x = session
        .eval(&format!("simplify(diff(-({f}), dydx))"))
        .ok()?;
    let singular_y = session
        .eval(&format!("simplify(dydx*({singular_x}) + ({f}))"))
        .ok()?;
    let line = format!("{y} = C1*{x} + ({general})");
    Some(Ok(format!(
        "{line}  (Clairaut equation; singular solution {x} = {singular_x}, {y} = {singular_y}, parameter dydx)"
    )))
}

/// Substitute `value` for `variable` in `expression` (`subs` cannot be nested inside other calls), then simplify.
fn substitute(
    session: &mut UraeSession,
    expression: &str,
    variable: &str,
    value: &str,
) -> Option<String> {
    // The printer writes a middle dot for multiplication, which the parser does not read back: use '*' everywhere.
    let expression = expression.replace('·', "*");
    let replaced = session
        .eval(&format!("subs(({expression}), {variable} = ({value}))"))
        .ok()?
        .replace('·', "*");
    session
        .eval(&format!("simplify({replaced})"))
        .ok()
        .map(|text| text.replace('·', "*"))
}

/// Riccati equations `y' = q0 + q1 y + q2 y^2`: a particular solution `y_p = c x^k` is searched for; then `y = y_p + 1/v` turns
/// the equation into the linear `v' = -(q1 + 2 q2 y_p) v - q2`.
fn riccati(
    session: &mut UraeSession,
    m: &str,
    n: &str,
    x: &str,
    y: &str,
) -> Option<Result<String, String>> {
    let f = session.eval(&format!("simplify(-({m})/({n}))")).ok()?;
    let first = session.eval(&format!("diff(({f}), {y})")).ok()?;
    let second = session.eval(&format!("diff(({first}), {y})")).ok()?;
    let q2 = session.eval(&format!("simplify(({second})/2)")).ok()?;
    if q2.trim() == "0" || symbols_of(session, &q2)?.contains(y) {
        return None;
    }
    let q1 = substitute(session, &first, y, "0")?;
    let q0 = substitute(session, &f, y, "0")?;
    // The equation is quadratic in y exactly when f - (q0 + q1 y + q2 y^2) vanishes; checked at exact sample points because
    // the simplifier does not always cancel equal terms written in different forms.
    let difference = format!("({f}) - (({q0}) + ({q1})*{y} + ({q2})*{y}^2)");
    for (xv, yv) in [("3", "2"), ("5", "-7"), ("2", "1/3")] {
        let at_x = substitute(session, &difference, x, xv)?;
        let at_point = substitute(session, &at_x, y, yv)?;
        if at_point.trim() != "0" {
            return None;
        }
    }
    // The residual y_p' - f(x, y_p) of a candidate, as an expression in x (and the constant c).
    let residual_of = |session: &mut UraeSession, shape: &str| -> Option<String> {
        let rhs = substitute(session, &f, y, shape)?;
        session
            .eval(&format!("simplify(diff({shape}, {x}) - ({rhs}))"))
            .ok()
    };
    let mut shapes: Vec<String> = [0i32, 1, -1, 2, -2, 3, -3]
        .iter()
        .map(|power| format!("c*{x}^({power})"))
        .collect();
    // Exponential particular solutions c e^(k x), for the exponential forcing of the coefficients.
    shapes.extend([1i32, -1, 2, -2].iter().map(|k| format!("c*exp({k}*{x})")));
    for shape in shapes {
        let residual = residual_of(session, &shape)?;
        // Exponential shapes are solved at x = 0, where every e^(kx) is 1 and the coefficients are rational.
        let point = if shape.contains("exp") { "0" } else { "2" };
        let at = substitute(session, &residual, x, point)?;
        let Ok(roots) = session.eval(&format!("solve(({at}) = 0, c)")) else {
            continue;
        };
        let Some(list) = roots.split(": [").nth(1).and_then(|r| r.strip_suffix(']')) else {
            continue;
        };
        for root in list.split(", ") {
            let root = root.trim();
            if root.is_empty() || root.contains("RootOf") || root.contains('i') {
                continue;
            }
            let particular = substitute(session, &shape, "c", root)?;
            let check = residual_of(session, &particular)?;
            if !residual_vanishes(session, &check, x)? {
                continue;
            }
            if let Some(solution) = riccati_finish(session, &particular, &q1, &q2) {
                return Some(solution);
            }
        }
    }
    // Linear polynomial particular solutions y = a x + b, for a constant q2, q1 of degree at most 1 and q0 of degree at most 2.
    // Matching powers of x in y' = q0 + q1 y + q2 y^2 gives, with q1 = p1 x + p0 and q0 = r2 x^2 + r1 x + r0:
    //   -r2 - p1 a - q2 a^2 = 0,   -r1 - p1 b - p0 a - 2 q2 a b = 0,   a - r0 - p0 b - q2 b^2 = 0.
    // The first is quadratic in a, the second fixes b; the residual check verifies the third.
    if !symbols_of(session, &q2)?.contains(x) {
        let derivative = |session: &mut UraeSession, text: &str, order: u32| -> Option<String> {
            let mut current = text.to_string();
            for _ in 0..order {
                current = session.eval(&format!("diff(({current}), {x})")).ok()?;
            }
            Some(current)
        };
        let q1_second = derivative(session, &q1, 2)?;
        let q0_third = derivative(session, &q0, 3)?;
        let zero = |session: &mut UraeSession, text: &str| -> bool {
            session
                .eval(&format!("simplify({text})"))
                .map(|v| v.trim() == "0")
                .unwrap_or(false)
        };
        if zero(session, &q1_second) && zero(session, &q0_third) {
            let p0 = substitute(session, &q1, x, "0")?;
            let q1_first = derivative(session, &q1, 1)?;
            let p1 = substitute(session, &q1_first, x, "0")?;
            let q0_first = derivative(session, &q0, 1)?;
            let r1 = substitute(session, &q0_first, x, "0")?;
            let q0_second = derivative(session, &q0, 2)?;
            let r2_value = substitute(session, &q0_second, x, "0")?;
            let r2 = session.eval(&format!("simplify(({r2_value})/2)")).ok()?;
            let roots = session
                .eval(&format!("solve(-({r2}) - ({p1})*c - ({q2})*c^2 = 0, c)"))
                .ok()?;
            let list = roots.split(": [").nth(1).and_then(|r| r.strip_suffix(']'));
            for root in list.unwrap_or("").split(", ") {
                let a = root.trim();
                if a.is_empty() || a.contains("RootOf") || a.contains('i') {
                    continue;
                }
                let Ok(b) = session.eval(&format!(
                    "simplify((({r1}) + ({p0})*({a})) / (-({p1}) - 2*({q2})*({a})))"
                )) else {
                    continue;
                };
                let Ok(particular) = session.eval(&format!("simplify(({a})*{x} + ({b}))")) else {
                    continue;
                };
                let check = residual_of(session, &particular)?;
                if !residual_vanishes(session, &check, x)? {
                    continue;
                }
                if let Some(solution) = riccati_finish(session, &particular, &q1, &q2) {
                    return Some(solution);
                }
            }
        }
    }
    None
}

/// Solve the linear equation for the constant of a Riccati equation given one particular solution, and assemble the solution.
fn riccati_finish(
    session: &mut UraeSession,
    particular: &str,
    q1: &str,
    q2: &str,
) -> Option<Result<String, String>> {
    let coefficient = session
        .eval(&format!("simplify(({q1}) + 2*({q2})*({particular}))"))
        .ok()?;
    let linear = session
        .eval(&format!("dsolve(y' = -({coefficient})*y - ({q2}))"))
        .ok()?;
    let v = linear.strip_prefix("y(x) = ")?;
    Some(Ok(format!(
        "y(x) = {particular} + 1/({v})  (Riccati equation; particular solution {particular})"
    )))
}

/// Whether a residual vanishes at three sample points: exactly, or numerically to 1e-9 (exponentials do not simplify to zero).
fn residual_vanishes(session: &mut UraeSession, check: &str, x: &str) -> Option<bool> {
    for xv in ["1/3", "2/3", "1/2"] {
        let at_point = substitute(session, check, x, xv)?;
        if at_point.trim() == "0" {
            continue;
        }
        let numeric = session.eval(&format!("evalf({at_point})")).ok();
        match numeric.and_then(|v| v.trim().parse::<f64>().ok()) {
            Some(value) if value.abs() < 1e-9 => {}
            _ => return Some(false),
        }
    }
    Some(true)
}

/// d'Alembert (Lagrange) equations `y = x F(y') + G(y')` with `F(p) != p`: differentiating and taking `p` as the independent
/// variable gives the linear `dx/dp = (F'(p) x + G'(p)) / (p - F(p))`, so the solution is parametric in `p`.
fn dalembert(session: &mut UraeSession, equation: &str, y: &str) -> Option<Result<String, String>> {
    let k = session.eval(&format!("diff(({equation}), {y})")).ok()?;
    let k = k.trim().to_string();
    if k == "0" || !symbols_of(session, &k)?.is_empty() {
        return None;
    }
    let symbols = symbols_of(session, equation)?;
    let independent: Vec<&String> = symbols.iter().filter(|s| *s != y && *s != "dydx").collect();
    let [x] = independent.as_slice() else {
        return None;
    };
    let x = (*x).clone();
    // E / k = y - x F(p) - G(p).
    let f = session
        .eval(&format!("simplify(-diff(({equation})/({k}), {x}))"))
        .ok()?;
    let used = symbols_of(session, &f)?;
    if used.contains(&x) || used.contains(y) {
        return None;
    }
    let g = session
        .eval(&format!("simplify({y} - {x}*({f}) - ({equation})/({k}))"))
        .ok()?;
    let used = symbols_of(session, &g)?;
    if used.contains(&x) || used.contains(y) {
        return None;
    }
    let gap = session.eval(&format!("simplify(dydx - ({f}))")).ok()?;
    if gap.trim() == "0" {
        return None;
    }
    let a = session
        .eval(&format!("simplify(diff(({f}), dydx)/({gap}))"))
        .ok()?;
    let b = session
        .eval(&format!("simplify(diff(({g}), dydx)/({gap}))"))
        .ok()?;
    let a = substitute(session, &a, "dydx", "x")?;
    let b = substitute(session, &b, "dydx", "x")?;
    let linear = session
        .eval(&format!("dsolve(y' = ({a})*y + ({b}))"))
        .ok()?;
    let rhs = linear.strip_prefix("y(x) = ")?;
    let x_of_p = substitute(session, rhs, "x", "dydx")?;
    let y_of_p = session
        .eval(&format!("simplify(({x_of_p})*({f}) + ({g}))"))
        .ok()?;
    // Singular solutions: a root p0 of p = F(p) gives the straight line y = p0 x + G(p0), a solution with slope p0.
    let mut singular = Vec::new();
    if let Ok(roots) = session.eval(&format!("solve(({gap}) = 0, dydx)"))
        && let Some(list) = roots.split(": [").nth(1).and_then(|r| r.strip_suffix(']'))
    {
        for root in list.split(", ") {
            let root = root.trim();
            if root.is_empty() || root.contains("RootOf") || root.contains('i') {
                continue;
            }
            if let Some(constant) = substitute(session, &g, "dydx", root)
                && let Ok(line) = session.eval(&format!("simplify(({root})*{x} + ({constant}))"))
            {
                singular.push(format!("{y} = {line}"));
            }
        }
    }
    let extra = if singular.is_empty() {
        String::new()
    } else {
        format!("; singular solutions {}", singular.join(", "))
    };
    Some(Ok(format!(
        "{x} = {x_of_p}, {y} = {y_of_p}  (d'Alembert equation; parameter dydx{extra})"
    )))
}
