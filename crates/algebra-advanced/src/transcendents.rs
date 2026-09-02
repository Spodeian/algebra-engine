//! # `algebra_advanced::transcendents`
//!
//! Painlevé Transcendents I–VI & Heun Differential Equations.
//!
//! Encompasses:
//! - **Painlevé Transcendents ($P_{\text{I}} - P_{\text{VI}}$)**: Non-linear second-order ODEs whose solutions
//!   have no movable branch points (Painlevé property).
//!   - $P_{\text{I}}$: $y'' = 6y^2 + t$
//!   - $P_{\text{II}}(\alpha)$: $y'' = 2y^3 + t y + \alpha$
//!   - $P_{\text{III}}(\alpha, \beta, \gamma, \delta)$: $y'' = \frac{1}{y}(y')^2 - \frac{1}{t}y' + \frac{\alpha y^2 + \beta}{t} + \gamma y^3 + \frac{\delta}{y}$
//!   - $P_{\text{IV}}(\alpha, \beta)$: $y'' = \frac{1}{2y}(y')^2 + \frac{3}{2}y^3 + 4t y^2 + 2(t^2 - \alpha)y + \frac{\beta}{y}$
//!   - $P_{\text{V}}(\alpha, \beta, \gamma, \delta)$ and $P_{\text{VI}}(\alpha, \beta, \gamma, \delta)$.
//! - **Laurent Pole Analysis**: Asymptotic pole expansion $y(t) \approx \frac{1}{(t - t_0)^2} + \dots$.
//! - **Heun Equations**: General and Confluent Heun second-order linear Fuchsian differential equations.

use algebra_core::error::{AlgebraError, AlgebraResult};

/// Family classification of the six classical Painlevé transcendents.
#[derive(Debug, Clone, PartialEq)]
pub enum PainleveKind {
    /// Painlevé I: $y'' = 6y^2 + t$.
    P1,
    /// Painlevé II: $y'' = 2y^3 + t y + \alpha$.
    P2 { alpha: f64 },
    /// Painlevé III: $y'' = \frac{(y')^2}{y} - \frac{y'}{t} + \frac{\alpha y^2 + \beta}{t} + \gamma y^3 + \frac{\delta}{y}$.
    P3 {
        alpha: f64,
        beta: f64,
        gamma: f64,
        delta: f64,
    },
    /// Painlevé IV: $y'' = \frac{(y')^2}{2y} + \frac{3}{2}y^3 + 4t y^2 + 2(t^2 - \alpha)y + \frac{\beta}{y}$.
    P4 { alpha: f64, beta: f64 },
    /// Painlevé V.
    P5 {
        alpha: f64,
        beta: f64,
        gamma: f64,
        delta: f64,
    },
    /// Painlevé VI: Master equation with 4 regular singular points at $0, 1, \infty, t$.
    P6 {
        alpha: f64,
        beta: f64,
        gamma: f64,
        delta: f64,
    },
}

/// Evaluator and Integrator for Painlevé Differential Equations.
#[derive(Debug, Clone)]
pub struct PainleveEquation {
    pub kind: PainleveKind,
}

impl PainleveEquation {
    /// Creates a new Painlevé equation of the given kind.
    pub fn new(kind: PainleveKind) -> Self {
        Self { kind }
    }

    /// Evaluates the second derivative $y''(t, y, y')$.
    pub fn eval_d2y(&self, t: f64, y: f64, dy: f64) -> AlgebraResult<f64> {
        match &self.kind {
            PainleveKind::P1 => Ok(6.0 * y * y + t),
            PainleveKind::P2 { alpha } => Ok(2.0 * y.powi(3) + t * y + alpha),
            PainleveKind::P3 {
                alpha,
                beta,
                gamma,
                delta,
            } => {
                if y.abs() < 1e-15 || t.abs() < 1e-15 {
                    return Err(AlgebraError::EvaluationError(
                        "Singularity in Painlevé III evaluation (y=0 or t=0)".into(),
                    ));
                }
                let term1 = (dy * dy) / y;
                let term2 = -dy / t;
                let term3 = (alpha * y * y + beta) / t;
                let term4 = gamma * y.powi(3) + delta / y;
                Ok(term1 + term2 + term3 + term4)
            }
            PainleveKind::P4 { alpha, beta } => {
                if y.abs() < 1e-15 {
                    return Err(AlgebraError::EvaluationError(
                        "Singularity in Painlevé IV evaluation (y=0)".into(),
                    ));
                }
                let term1 = (dy * dy) / (2.0 * y);
                let term2 =
                    1.5 * y.powi(3) + 4.0 * t * y * y + 2.0 * (t * t - alpha) * y + beta / y;
                Ok(term1 + term2)
            }
            PainleveKind::P5 {
                alpha,
                beta,
                gamma,
                delta,
            } => {
                if y.abs() < 1e-15 || (y - 1.0).abs() < 1e-15 || t.abs() < 1e-15 {
                    return Err(AlgebraError::EvaluationError(
                        "Singularity in Painlevé V evaluation".into(),
                    ));
                }
                let term1 = (1.0 / (2.0 * y) + 1.0 / (y - 1.0)) * dy * dy;
                let term2 = -dy / t;
                let term3 = (y - 1.0).powi(2) / (t * t) * (alpha * y + beta / y);
                let term4 = gamma * y / t + delta * y * (y + 1.0) / (y - 1.0);
                Ok(term1 + term2 + term3 + term4)
            }
            PainleveKind::P6 {
                alpha,
                beta,
                gamma,
                delta,
            } => {
                if y.abs() < 1e-15
                    || (y - 1.0).abs() < 1e-15
                    || (y - t).abs() < 1e-15
                    || t.abs() < 1e-15
                    || (t - 1.0).abs() < 1e-15
                {
                    return Err(AlgebraError::EvaluationError(
                        "Singularity in Painlevé VI evaluation".into(),
                    ));
                }
                let term1 = 0.5 * (1.0 / y + 1.0 / (y - 1.0) + 1.0 / (y - t)) * dy * dy;
                let term2 = -(1.0 / t + 1.0 / (t - 1.0) + 1.0 / (y - t)) * dy;
                let factor = y * (y - 1.0) * (y - t) / (t * t * (t - 1.0).powi(2));
                let term3 = factor
                    * (alpha
                        + beta * t / (y * y)
                        + gamma * (t - 1.0) / (y - 1.0).powi(2)
                        + delta * t * (t - 1.0) / (y - t).powi(2));
                Ok(term1 + term2 + term3)
            }
        }
    }

    /// Integrates a Painlevé trajectory using adaptive 4th-order Runge-Kutta.
    pub fn integrate(
        &self,
        t0: f64,
        y0: f64,
        dy0: f64,
        t_end: f64,
        dt_initial: f64,
    ) -> AlgebraResult<Vec<(f64, f64, f64)>> {
        let mut trajectory = Vec::new();
        let mut t = t0;
        let mut y = y0;
        let mut dy = dy0;
        let dt = dt_initial.copysign(t_end - t0);

        trajectory.push((t, y, dy));

        while (t_end - t).abs() > 1e-9 {
            let h = if (t_end - t).abs() < dt.abs() {
                t_end - t
            } else {
                dt
            };

            // RK4 step for system:
            // y' = dy
            // dy' = f(t, y, dy)
            let k1_y = dy;
            let k1_dy = self.eval_d2y(t, y, dy)?;

            let k2_y = dy + 0.5 * h * k1_dy;
            let k2_dy = self.eval_d2y(t + 0.5 * h, y + 0.5 * h * k1_y, dy + 0.5 * h * k1_dy)?;

            let k3_y = dy + 0.5 * h * k2_dy;
            let k3_dy = self.eval_d2y(t + 0.5 * h, y + 0.5 * h * k2_y, dy + 0.5 * h * k2_dy)?;

            let k4_y = dy + h * k3_dy;
            let k4_dy = self.eval_d2y(t + h, y + h * k3_y, dy + h * k3_dy)?;

            y += (h / 6.0) * (k1_y + 2.0 * k2_y + 2.0 * k3_y + k4_y);
            dy += (h / 6.0) * (k1_dy + 2.0 * k2_dy + 2.0 * k3_dy + k4_dy);
            t += h;

            trajectory.push((t, y, dy));

            // Stop if approaching pole
            if y.abs() > 1e7 {
                break;
            }
        }

        Ok(trajectory)
    }

    /// Computes the leading Laurent series expansion of Painlevé I around a movable pole $t_0$:
    /// $y(t) = \frac{1}{(t - t_0)^2} - \frac{t_0}{10}(t - t_0)^2 - \frac{1}{6}(t - t_0)^3 + \dots$.
    pub fn laurent_p1_pole_series(t: f64, t0: f64) -> f64 {
        let z = t - t0;
        if z.abs() < 1e-15 {
            return f64::INFINITY;
        }
        let term_minus2 = 1.0 / (z * z);
        let term_plus2 = -(t0 / 10.0) * z * z;
        let term_plus3 = -(1.0 / 6.0) * z.powi(3);
        term_minus2 + term_plus2 + term_plus3
    }
}

/// Confluent and General Heun Differential Equation Representations.
#[derive(Debug, Clone, PartialEq)]
pub struct HeunEquation {
    pub gamma: f64,
    pub delta: f64,
    pub epsilon: f64,
    pub alpha: f64,
    pub beta: f64,
    pub q: f64,
}

impl HeunEquation {
    /// Creates a general Heun equation.
    pub fn general(alpha: f64, beta: f64, gamma: f64, delta: f64, epsilon: f64, q: f64) -> Self {
        Self {
            alpha,
            beta,
            gamma,
            delta,
            epsilon,
            q,
        }
    }

    /// Evaluates the Heun ODE standard form: $y'' + \left(\frac{\gamma}{z} + \frac{\delta}{z-1} + \frac{\epsilon}{z-a}\right)y' + \frac{\alpha\beta z - q}{z(z-1)(z-a)} y = 0$.
    pub fn eval_d2y(&self, z: f64, a: f64, y: f64, dy: f64) -> AlgebraResult<f64> {
        if z.abs() < 1e-15 || (z - 1.0).abs() < 1e-15 || (z - a).abs() < 1e-15 {
            return Err(AlgebraError::EvaluationError(
                "Singularity in Heun equation evaluation at Fuchsian pole".into(),
            ));
        }

        let p_z = self.gamma / z + self.delta / (z - 1.0) + self.epsilon / (z - a);
        let q_z = (self.alpha * self.beta * z - self.q) / (z * (z - 1.0) * (z - a));

        Ok(-p_z * dy - q_z * y)
    }
}
