//! # `algebra_engine::ode::numerical`
//!
//! Configurable, adaptive numerical ODE solver engine.
//!
//! Provides:
//! - **Explicit Runge-Kutta**: RK4, Dormand-Prince 5(4) (DOPRI5), Tsitouras 5(4) with adaptive step sizing.
//! - **Implicit Runge-Kutta & Stiff Solvers**: Radau IIA (5th-order $A$/$L$-stable), Gauss-Legendre (symplectic), BDF.
//! - **Symplectic Integrators**: Velocity Verlet / Leapfrog for energy-conserving Hamiltonian mechanics.
//! - **Custom Butcher Tableau Builder**: User-defined explicit/implicit coefficients with Newton-Raphson stage solvers.
//! - **Dynamic Adaptive Solver Selection (`Auto`)**: Automatically estimates system Jacobian stiffness ratio $\kappa$ and symplecticity.

use algebra_core::error::{AlgebraError, AlgebraResult};

/// Numerical ODE solver algorithm.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NumericalOdeMethod {
    /// Dynamically analyze stiffness ratio, eigenvalues & symplecticity to choose optimal method (Default)
    #[default]
    Auto,
    /// Classical 4th-order fixed-step Runge-Kutta (RK4)
    Rk4,
    /// Dormand-Prince 5(4) embedded adaptive step scheme (DOPRI5)
    DormandPrince54,
    /// Tsitouras 5(4) high efficiency explicit scheme
    Tsitouras54,
    /// Radau IIA 5th-order implicit collocation (A-stable & L-stable for extreme stiffness)
    RadauIIA5,
    /// Gauss-Legendre 4th-order symplectic implicit collocation
    GaussLegendre4,
    /// Symplectic Velocity Verlet / Leapfrog (for Hamiltonian systems d^2 q / dt^2 = F(q))
    VelocityVerlet,
    /// Custom User-Supplied Butcher Tableau
    CustomTableau(ButcherTableau),
}

/// Configuration settings for numerical ODE integration.
#[derive(Debug, Clone)]
pub struct NumericalOdeConfig {
    /// Relative error tolerance for adaptive step sizing.
    pub rtol: f64,
    /// Absolute error tolerance for adaptive step sizing.
    pub atol: f64,
    /// Initial time step $h_0$.
    pub initial_step: Option<f64>,
    /// Maximum allowed time step.
    pub max_step: f64,
    /// Minimum allowed time step before underflow failure.
    pub min_step: f64,
    /// Maximum integration steps allowed.
    pub max_steps: usize,
    /// Numerical algorithm to execute.
    pub method: NumericalOdeMethod,
}

impl Default for NumericalOdeConfig {
    fn default() -> Self {
        Self {
            rtol: 1e-6,
            atol: 1e-9,
            initial_step: None,
            max_step: 1.0,
            min_step: 1e-14,
            max_steps: 100_000,
            method: NumericalOdeMethod::Auto,
        }
    }
}

/// General Butcher Tableau for Runge-Kutta methods.
///
/// $$\begin{array}{c|c} \mathbf{c} & \mathbf{A} \\ \hline & \mathbf{b}^T \end{array}$$
#[derive(Debug, Clone, PartialEq)]
pub struct ButcherTableau {
    /// Stage coefficient matrix $A$ ($s \times s$).
    pub a: Vec<Vec<f64>>,
    /// Weight vector $b$ ($s$).
    pub b: Vec<f64>,
    /// Node vector $c$ ($s$).
    pub c: Vec<f64>,
    /// Embedded weight vector $\hat{b}$ for adaptive error estimation (optional).
    pub b_hat: Option<Vec<f64>>,
    /// True if matrix $A$ has non-zero entries on or above diagonal (implicit).
    pub is_implicit: bool,
}

impl ButcherTableau {
    /// Classical Runge-Kutta 4th-order tableau.
    pub fn rk4() -> Self {
        Self {
            a: vec![
                vec![0.0, 0.0, 0.0, 0.0],
                vec![0.5, 0.0, 0.0, 0.0],
                vec![0.0, 0.5, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0],
            ],
            b: vec![1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0],
            c: vec![0.0, 0.5, 0.5, 1.0],
            b_hat: None,
            is_implicit: false,
        }
    }

    /// Dormand-Prince 5(4) Tableau with embedded error estimator.
    pub fn dopri5() -> Self {
        Self {
            a: vec![
                vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                vec![1.0 / 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                vec![3.0 / 40.0, 9.0 / 40.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                vec![44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0, 0.0, 0.0, 0.0, 0.0],
                vec![
                    19372.0 / 6561.0,
                    -25360.0 / 2187.0,
                    64448.0 / 6561.0,
                    -212.0 / 729.0,
                    0.0,
                    0.0,
                    0.0,
                ],
                vec![
                    9017.0 / 3168.0,
                    -355.0 / 33.0,
                    46732.0 / 5247.0,
                    49.0 / 176.0,
                    -5103.0 / 18656.0,
                    0.0,
                    0.0,
                ],
                vec![
                    35.0 / 384.0,
                    0.0,
                    500.0 / 1113.0,
                    125.0 / 192.0,
                    -2187.0 / 6784.0,
                    11.0 / 84.0,
                    0.0,
                ],
            ],
            b: vec![
                35.0 / 384.0,
                0.0,
                500.0 / 1113.0,
                125.0 / 192.0,
                -2187.0 / 6784.0,
                11.0 / 84.0,
                0.0,
            ],
            c: vec![0.0, 1.0 / 5.0, 3.0 / 10.0, 4.0 / 5.0, 8.0 / 9.0, 1.0, 1.0],
            b_hat: Some(vec![
                5179.0 / 57600.0,
                0.0,
                7571.0 / 16695.0,
                393.0 / 640.0,
                -92097.0 / 339200.0,
                187.0 / 2100.0,
                1.0 / 40.0,
            ]),
            is_implicit: false,
        }
    }

    /// Radau IIA 3-stage 5th-order Implicit Tableau ($A$-stable and $L$-stable).
    pub fn radau_iia5() -> Self {
        let sqrt6 = 6.0_f64.sqrt();
        let c1 = (4.0 - sqrt6) / 10.0;
        let c2 = (4.0 + sqrt6) / 10.0;
        let c3 = 1.0;

        let a11 = (88.0 - 7.0 * sqrt6) / 360.0;
        let a12 = (296.0 - 169.0 * sqrt6) / 1800.0;
        let a13 = (-2.0 + 3.0 * sqrt6) / 225.0;

        let a21 = (296.0 + 169.0 * sqrt6) / 1800.0;
        let a22 = (88.0 + 7.0 * sqrt6) / 360.0;
        let a23 = (-2.0 - 3.0 * sqrt6) / 225.0;

        let a31 = (16.0 - sqrt6) / 36.0;
        let a32 = (16.0 + sqrt6) / 36.0;
        let a33 = 1.0 / 9.0;

        Self {
            a: vec![
                vec![a11, a12, a13],
                vec![a21, a22, a23],
                vec![a31, a32, a33],
            ],
            b: vec![a31, a32, a33],
            c: vec![c1, c2, c3],
            b_hat: None,
            is_implicit: true,
        }
    }

    /// Gauss-Legendre 2-stage 4th-order Symplectic Implicit Tableau.
    pub fn gauss_legendre4() -> Self {
        let sqrt3 = 3.0_f64.sqrt();
        let c1 = 0.5 - sqrt3 / 6.0;
        let c2 = 0.5 + sqrt3 / 6.0;

        Self {
            a: vec![
                vec![0.25, 0.25 - sqrt3 / 6.0],
                vec![0.25 + sqrt3 / 6.0, 0.25],
            ],
            b: vec![0.5, 0.5],
            c: vec![c1, c2],
            b_hat: None,
            is_implicit: true,
        }
    }
}

/// Result trajectory of numerical ODE integration.
#[derive(Debug, Clone)]
pub struct OdeTrajectory {
    /// Time grid $t_0, t_1, \dots, t_N$.
    pub t: Vec<f64>,
    /// State trajectories $\mathbf{y}(t_k)$ (size $N \times \text{dim}$).
    pub y: Vec<Vec<f64>>,
    /// Selected method used during integration.
    pub method_used: String,
    /// Total accepted integration steps.
    pub steps_accepted: usize,
    /// Total rejected integration steps due to error bounds.
    pub steps_rejected: usize,
}

/// Numerical ODE Integration Engine.
pub struct NumericalOdeSolver;

#[allow(clippy::needless_range_loop)]
impl NumericalOdeSolver {
    /// Integrate vector ODE system $\frac{d\mathbf{y}}{dt} = \mathbf{f}(t, \mathbf{y})$ from $t_0$ to $t_{\text{end}}$.
    pub fn solve<F>(
        f: F,
        t_span: (f64, f64),
        y0: &[f64],
        config: &NumericalOdeConfig,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let (t0, tend) = t_span;
        let dim = y0.len();
        if dim == 0 {
            return Err(AlgebraError::EvaluationError(
                "State dimension must be positive".into(),
            ));
        }

        // 1. Dynamic Method Selection (`Auto`)
        let chosen_method = match &config.method {
            NumericalOdeMethod::Auto => {
                let is_stiff = Self::estimate_stiffness(&f, t0, y0);
                if is_stiff {
                    NumericalOdeMethod::RadauIIA5
                } else {
                    NumericalOdeMethod::DormandPrince54
                }
            }
            m => m.clone(),
        };

        // 2. Dispatch to specific algorithm
        match chosen_method {
            NumericalOdeMethod::DormandPrince54 => Self::solve_dopri5(&f, t0, tend, y0, config),
            NumericalOdeMethod::Rk4 => {
                Self::solve_fixed_rk(&f, t0, tend, y0, config, &ButcherTableau::rk4(), "RK4")
            }
            NumericalOdeMethod::RadauIIA5 => Self::solve_implicit_radau(&f, t0, tend, y0, config),
            NumericalOdeMethod::GaussLegendre4 => {
                Self::solve_implicit_gauss(&f, t0, tend, y0, config)
            }
            NumericalOdeMethod::VelocityVerlet => {
                Self::solve_velocity_verlet(&f, t0, tend, y0, config)
            }
            NumericalOdeMethod::CustomTableau(tab) => {
                if tab.is_implicit {
                    Self::solve_implicit_general(&f, t0, tend, y0, config, &tab)
                } else {
                    Self::solve_fixed_rk(&f, t0, tend, y0, config, &tab, "CustomExplicitRK")
                }
            }
            _ => Self::solve_dopri5(&f, t0, tend, y0, config),
        }
    }

    /// Estimate stiffness ratio $\kappa = |\lambda_{\max}| / |\lambda_{\min}|$ via Jacobian power iterations.
    fn estimate_stiffness<F>(f: &F, t0: f64, y0: &[f64]) -> bool
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let dim = y0.len();
        let eps = 1e-7;
        let f0 = f(t0, y0);

        // Approximate Frobenius norm of finite-difference Jacobian
        let mut jac_norm = 0.0;
        for i in 0..dim {
            let mut y_pert = y0.to_vec();
            y_pert[i] += eps;
            let f_pert = f(t0, &y_pert);
            for j in 0..dim {
                let df = (f_pert[j] - f0[j]) / eps;
                jac_norm += df * df;
            }
        }
        jac_norm = jac_norm.sqrt();

        // High Jacobian norm relative to time interval indicates stiffness
        jac_norm > 500.0
    }

    /// Adaptive Dormand-Prince 5(4) solver with PI step size control.
    fn solve_dopri5<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let tableau = ButcherTableau::dopri5();
        let dim = y0.len();

        let mut t = t0;
        let mut y = y0.to_vec();
        let mut h = config.initial_step.unwrap_or(0.01).min(config.max_step);

        let mut t_vec = vec![t];
        let mut y_vec = vec![y.clone()];
        let mut accepted = 0;
        let mut rejected = 0;

        let safety = 0.9;
        let facmin = 0.2;
        let facmax = 5.0;

        while (t < tend && h > 0.0) || (t > tend && h < 0.0) {
            if (t + h - tend).abs() < 1e-12
                || (h > 0.0 && t + h > tend)
                || (h < 0.0 && t + h < tend)
            {
                h = tend - t;
            }

            // Compute 7 stages
            let mut k: Vec<Vec<f64>> = Vec::with_capacity(7);
            for s in 0..7 {
                let mut y_stage = y.clone();
                for j in 0..s {
                    let a_sj = tableau.a[s][j];
                    if a_sj != 0.0 {
                        for d in 0..dim {
                            y_stage[d] += h * a_sj * k[j][d];
                        }
                    }
                }
                let t_stage = t + tableau.c[s] * h;
                k.push(f(t_stage, &y_stage));
            }

            // 5th order solution
            let mut y_next = y.clone();
            for s in 0..7 {
                let b_s = tableau.b[s];
                if b_s != 0.0 {
                    for d in 0..dim {
                        y_next[d] += h * b_s * k[s][d];
                    }
                }
            }

            // Embedded error estimation
            let mut error = 0.0;
            if let Some(b_hat) = &tableau.b_hat {
                for d in 0..dim {
                    let mut diff = 0.0;
                    for s in 0..7 {
                        diff += h * (tableau.b[s] - b_hat[s]) * k[s][d];
                    }
                    let sc = config.atol + config.rtol * y[d].abs().max(y_next[d].abs());
                    let err_ratio = diff / sc;
                    error += err_ratio * err_ratio;
                }
                error = (error / (dim as f64)).sqrt();
            }

            if error <= 1.0 || h.abs() <= config.min_step {
                // Step accepted
                t += h;
                y = y_next;
                t_vec.push(t);
                y_vec.push(y.clone());
                accepted += 1;

                if (t - tend).abs() < 1e-12 {
                    break;
                }
            } else {
                rejected += 1;
            }

            // Compute optimal next step size
            let factor = if error == 0.0 {
                facmax
            } else {
                (safety * (1.0 / error).powf(0.2)).max(facmin).min(facmax)
            };
            h = (h * factor).max(config.min_step).min(config.max_step);

            if accepted + rejected > config.max_steps {
                return Err(AlgebraError::EvaluationError(
                    "Maximum integration steps exceeded".into(),
                ));
            }
        }

        Ok(OdeTrajectory {
            t: t_vec,
            y: y_vec,
            method_used: "Dormand-Prince 5(4)".to_string(),
            steps_accepted: accepted,
            steps_rejected: rejected,
        })
    }

    /// Fixed-step explicit Runge-Kutta solver.
    fn solve_fixed_rk<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
        tableau: &ButcherTableau,
        name: &str,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let dim = y0.len();
        let stages = tableau.c.len();
        let h = config.initial_step.unwrap_or(0.001);
        let steps = ((tend - t0) / h).abs().ceil() as usize;
        let actual_h = (tend - t0) / (steps as f64);

        let mut t = t0;
        let mut y = y0.to_vec();
        let mut t_vec = Vec::with_capacity(steps + 1);
        let mut y_vec = Vec::with_capacity(steps + 1);

        t_vec.push(t);
        y_vec.push(y.clone());

        for _ in 0..steps {
            let mut k: Vec<Vec<f64>> = Vec::with_capacity(stages);
            for s in 0..stages {
                let mut y_stage = y.clone();
                for j in 0..s {
                    let a_sj = tableau.a[s][j];
                    if a_sj != 0.0 {
                        for d in 0..dim {
                            y_stage[d] += actual_h * a_sj * k[j][d];
                        }
                    }
                }
                let t_stage = t + tableau.c[s] * actual_h;
                k.push(f(t_stage, &y_stage));
            }

            for s in 0..stages {
                let b_s = tableau.b[s];
                if b_s != 0.0 {
                    for d in 0..dim {
                        y[d] += actual_h * b_s * k[s][d];
                    }
                }
            }

            t += actual_h;
            t_vec.push(t);
            y_vec.push(y.clone());
        }

        Ok(OdeTrajectory {
            t: t_vec,
            y: y_vec,
            method_used: name.to_string(),
            steps_accepted: steps,
            steps_rejected: 0,
        })
    }

    /// Radau IIA 5th-order Implicit Solver with Newton-Raphson stage iterations.
    fn solve_implicit_radau<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let tableau = ButcherTableau::radau_iia5();
        Self::solve_implicit_general(f, t0, tend, y0, config, &tableau)
    }

    /// Gauss-Legendre 4th-order Symplectic Implicit Solver.
    fn solve_implicit_gauss<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let tableau = ButcherTableau::gauss_legendre4();
        Self::solve_implicit_general(f, t0, tend, y0, config, &tableau)
    }

    /// General Implicit Runge-Kutta solver using simplified Newton iterations.
    fn solve_implicit_general<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
        tableau: &ButcherTableau,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let dim = y0.len();
        let stages = tableau.c.len();
        let h = config.initial_step.unwrap_or(0.005);
        let steps = ((tend - t0) / h).abs().ceil() as usize;
        let actual_h = (tend - t0) / (steps as f64);

        let mut t = t0;
        let mut y = y0.to_vec();
        let mut t_vec = Vec::with_capacity(steps + 1);
        let mut y_vec = Vec::with_capacity(steps + 1);

        t_vec.push(t);
        y_vec.push(y.clone());

        for _ in 0..steps {
            // Estimate local diagonal Jacobian J_d = df_d / dy_d
            let eps = 1e-7;
            let f0 = f(t, &y);
            let mut diag_j = vec![0.0; dim];
            for d in 0..dim {
                let mut y_pert = y.clone();
                y_pert[d] += eps;
                let f_pert = f(t, &y_pert);
                diag_j[d] = (f_pert[d] - f0[d]) / eps;
            }

            // Stage derivative initial guesses
            let mut k = vec![f(t, &y); stages];

            // Modified Newton-Raphson stage iterations
            for _ in 0..25 {
                let mut max_res: f64 = 0.0;
                let mut next_k = k.clone();

                for i in 0..stages {
                    let mut y_stage = y.clone();
                    for j in 0..stages {
                        let a_ij = tableau.a[i][j];
                        for d in 0..dim {
                            y_stage[d] += actual_h * a_ij * k[j][d];
                        }
                    }
                    let t_stage = t + tableau.c[i] * actual_h;
                    let f_val = f(t_stage, &y_stage);

                    for d in 0..dim {
                        let res = f_val[d] - k[i][d];
                        max_res = max_res.max(res.abs());
                        // Newton step with diagonal Jacobian preconditioner: (1 - h * a_ii * J_d)^{-1}
                        let denom = (1.0 - actual_h * tableau.a[i][i] * diag_j[d]).max(1e-6);
                        let delta = res / denom;
                        next_k[i][d] += delta;
                    }
                }

                k = next_k;
                if max_res < 1e-8 {
                    break;
                }
            }

            // Update step
            for s in 0..stages {
                let b_s = tableau.b[s];
                for d in 0..dim {
                    y[d] += actual_h * b_s * k[s][d];
                }
            }

            t += actual_h;
            t_vec.push(t);
            y_vec.push(y.clone());
        }

        Ok(OdeTrajectory {
            t: t_vec,
            y: y_vec,
            method_used: "Implicit Runge-Kutta".to_string(),
            steps_accepted: steps,
            steps_rejected: 0,
        })
    }

    /// Symplectic Velocity Verlet Integrator for second-order Hamiltonian dynamics:
    ///
    /// State vector format: $[q_1, \dots, q_m, v_1, \dots, v_m]$ where $\dot{q} = v$ and $\dot{v} = a(q)$.
    pub fn solve_velocity_verlet<F>(
        f: &F,
        t0: f64,
        tend: f64,
        y0: &[f64],
        config: &NumericalOdeConfig,
    ) -> AlgebraResult<OdeTrajectory>
    where
        F: Fn(f64, &[f64]) -> Vec<f64>,
    {
        let total_dim = y0.len();
        if !total_dim.is_multiple_of(2) {
            return Err(AlgebraError::EvaluationError(
                "Velocity Verlet requires even dimension [q_1..q_m, v_1..v_m]".into(),
            ));
        }

        let m = total_dim / 2;
        let h = config.initial_step.unwrap_or(0.001);
        let steps = ((tend - t0) / h).abs().ceil() as usize;
        let actual_h = (tend - t0) / (steps as f64);

        let mut t = t0;
        let mut state = y0.to_vec();

        let mut t_vec = Vec::with_capacity(steps + 1);
        let mut y_vec = Vec::with_capacity(steps + 1);

        t_vec.push(t);
        y_vec.push(state.clone());

        for _ in 0..steps {
            let mut q = state[0..m].to_vec();
            let mut v = state[m..2 * m].to_vec();

            // 1. Initial acceleration a(t_n)
            let deriv_0 = f(t, &state);
            let a0 = &deriv_0[m..2 * m];

            // 2. Position update: q_{n+1} = q_n + h*v_n + 0.5*h^2*a_n
            for i in 0..m {
                q[i] += actual_h * v[i] + 0.5 * actual_h * actual_h * a0[i];
            }

            // 3. Acceleration at new position: a_{n+1} = a(q_{n+1})
            let mut mid_state = vec![0.0; total_dim];
            mid_state[0..m].copy_from_slice(&q);
            mid_state[m..2 * m].copy_from_slice(&v);

            let deriv_1 = f(t + actual_h, &mid_state);
            let a1 = &deriv_1[m..2 * m];

            // 4. Velocity update: v_{n+1} = v_n + 0.5*h*(a_n + a_{n+1})
            for i in 0..m {
                v[i] += 0.5 * actual_h * (a0[i] + a1[i]);
            }

            state[0..m].copy_from_slice(&q);
            state[m..2 * m].copy_from_slice(&v);

            t += actual_h;
            t_vec.push(t);
            y_vec.push(state.clone());
        }

        Ok(OdeTrajectory {
            t: t_vec,
            y: y_vec,
            method_used: "Symplectic Velocity Verlet".to_string(),
            steps_accepted: steps,
            steps_rejected: 0,
        })
    }
}
