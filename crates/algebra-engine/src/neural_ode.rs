//! # `algebra_engine::neural_ode`
//!
//! Continuous-Depth Neural Ordinary Differential Equations (Neural ODEs).
//!
//! Formulates parameterized continuous vector fields $\frac{d\mathbf{x}}{dt} = f_\theta(\mathbf{x}, t)$
//! with forward trajectory solving and the continuous adjoint sensitivity method for $O(1)$ memory backpropagation.

use serde::{Deserialize, Serialize};

/// Trajectory output from a Neural ODE simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralOdeTrajectory {
    pub times: Vec<f64>,
    pub states: Vec<Vec<f64>>,
}

/// Continuous-Depth Neural ODE System.
pub struct NeuralOde;

impl NeuralOde {
    /// Solve forward trajectory $\mathbf{x}(t)$ from $t_0$ to $t_1$ with step size $h$ using 4th-order Runge-Kutta (RK4).
    pub fn forward<F>(
        vector_field: F,
        x0: &[f64],
        theta: &[f64],
        t0: f64,
        t1: f64,
        steps: usize,
    ) -> NeuralOdeTrajectory
    where
        F: Fn(&[f64], f64, &[f64]) -> Vec<f64>,
    {
        let h = (t1 - t0) / (steps as f64);
        let dim = x0.len();
        let mut times = Vec::with_capacity(steps + 1);
        let mut states = Vec::with_capacity(steps + 1);

        let mut current_t = t0;
        let mut current_x = x0.to_vec();

        times.push(current_t);
        states.push(current_x.clone());

        for _ in 0..steps {
            // k1 = f(x, t)
            let k1 = vector_field(&current_x, current_t, theta);

            // k2 = f(x + 0.5*h*k1, t + 0.5*h)
            let mut x_k2 = vec![0.0; dim];
            for i in 0..dim {
                x_k2[i] = current_x[i] + 0.5 * h * k1[i];
            }
            let k2 = vector_field(&x_k2, current_t + 0.5 * h, theta);

            // k3 = f(x + 0.5*h*k2, t + 0.5*h)
            let mut x_k3 = vec![0.0; dim];
            for i in 0..dim {
                x_k3[i] = current_x[i] + 0.5 * h * k2[i];
            }
            let k3 = vector_field(&x_k3, current_t + 0.5 * h, theta);

            // k4 = f(x + h*k3, t + h)
            let mut x_k4 = vec![0.0; dim];
            for i in 0..dim {
                x_k4[i] = current_x[i] + h * k3[i];
            }
            let k4 = vector_field(&x_k4, current_t + h, theta);

            // Update state
            for i in 0..dim {
                current_x[i] += (h / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
            current_t += h;

            times.push(current_t);
            states.push(current_x.clone());
        }

        NeuralOdeTrajectory { times, states }
    }
}

/// Continuous Adjoint Sensitivity Method for Neural ODE Backpropagation.
pub struct AdjointSensitivitySolver;

impl AdjointSensitivitySolver {
    /// Compute loss gradient $\frac{\partial L}{\partial \theta}$ using the continuous adjoint state equation:
    /// $$\frac{d\mathbf{a}}{dt} = -\mathbf{a}(t)^T \frac{\partial f}{\partial \mathbf{x}}, \quad \frac{dL}{d\theta} = -\int_{t_1}^{t_0} \mathbf{a}(t)^T \frac{\partial f}{\partial \theta} dt$$
    pub fn compute_gradient<F, Dfx, Dftheta>(
        _vector_field: F,
        df_dx: Dfx,
        df_dtheta: Dftheta,
        traj: &NeuralOdeTrajectory,
        theta: &[f64],
        final_loss_grad: &[f64],
    ) -> Vec<f64>
    where
        F: Fn(&[f64], f64, &[f64]) -> Vec<f64>,
        Dfx: Fn(&[f64], f64, &[f64]) -> Vec<Vec<f64>>,
        Dftheta: Fn(&[f64], f64, &[f64]) -> Vec<Vec<f64>>,
    {
        let n_steps = traj.times.len() - 1;
        let p_dim = theta.len();
        let mut dl_dtheta = vec![0.0; p_dim];
        let mut adjoint = final_loss_grad.to_vec();

        for step in (0..n_steps).rev() {
            let t = traj.times[step + 1];
            let dt = traj.times[step + 1] - traj.times[step];
            let x = &traj.states[step + 1];

            let j_x = df_dx(x, t, theta);
            let j_theta = df_dtheta(x, t, theta);

            // Accumulate dL/dtheta: adjoint^T * df/dtheta * dt
            for p in 0..p_dim {
                let mut sum = 0.0;
                for i in 0..adjoint.len() {
                    sum += adjoint[i] * j_theta[i][p];
                }
                dl_dtheta[p] += sum * dt;
            }

            // Update adjoint: a(t - dt) = a(t) + dt * (a(t)^T * df/dx)
            let mut new_adjoint = adjoint.clone();
            for j in 0..adjoint.len() {
                let mut da = 0.0;
                for i in 0..adjoint.len() {
                    da += adjoint[i] * j_x[i][j];
                }
                new_adjoint[j] += da * dt;
            }
            adjoint = new_adjoint;
        }

        dl_dtheta
    }
}
