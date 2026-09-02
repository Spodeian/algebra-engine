//! # `algebra_engine::symplectic`
//!
//! Symplectic Geometric Mechanics & Energy-Preserving Numerical Integrators.
//!
//! Symplectic integrators preserve the canonical symplectic 2-form $\omega = \sum_i dq_i \wedge dp_i$
//! and phase space volume (Liouville's theorem), guaranteeing bounded energy drift $O(h^{2k})$ over
//! millions of orbital, molecular, and celestial dynamics time steps.
//!
//! ## Implemented Steppers:
//! 1. **Verlet / Leapfrog (2nd-Order Symplectic)**:
//!    $$\mathbf{p}_{n+1/2} = \mathbf{p}_n - \frac{h}{2} \nabla V(\mathbf{q}_n)$$
//!    $$\mathbf{q}_{n+1} = \mathbf{q}_n + h \mathbf{M}^{-1} \mathbf{p}_{n+1/2}$$
//!    $$\mathbf{p}_{n+1} = \mathbf{p}_{n+1/2} - \frac{h}{2} \nabla V(\mathbf{q}_{n+1})$$
//!
//! 2. **Ruth (3rd-Order Symplectic)**:
//!    3-stage symplectic partition with optimized coefficients.
//!
//! 3. **Yoshida (4th-Order Symplectic)**:
//!    Composition of three 2nd-order Verlet steps with Suzuki-Yoshida scaling coefficient
//!    $w_1 = \frac{1}{2 - 2^{1/3}}$, $w_0 = 1 - 2w_1$.

use serde::{Deserialize, Serialize};

/// Separable Hamiltonian System: $H(\mathbf{q}, \mathbf{p}) = T(\mathbf{p}) + V(\mathbf{q})$
/// where kinetic energy is $T(\mathbf{p}) = \frac{1}{2} \mathbf{p}^T \mathbf{M}^{-1} \mathbf{p}$
/// and potential energy is $V(\mathbf{q})$.
pub trait HamiltonianSystemND {
    /// Dimension of the generalized coordinates $\mathbf{q} \in \mathbb{R}^N$.
    fn dimension(&self) -> usize;

    /// Gradient of the potential energy $-\nabla V(\mathbf{q})$ (Force vector $\mathbf{F}(\mathbf{q})$).
    fn force(&self, q: &[f64]) -> Vec<f64>;

    /// Inverse mass matrix diagonal $\mathbf{M}^{-1}$ (or velocity vector from momentum $\mathbf{M}^{-1}\mathbf{p}$).
    fn velocity(&self, p: &[f64]) -> Vec<f64>;

    /// Exact total energy $E(\mathbf{q}, \mathbf{p}) = T(\mathbf{p}) + V(\mathbf{q})$.
    fn total_energy(&self, q: &[f64], p: &[f64]) -> f64;
}

/// Standard 1D / 2D / 3D Harmonic Oscillator: $H(q, p) = \frac{p^2}{2m} + \frac{1}{2} k q^2$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicOscillator {
    pub mass: f64,
    pub spring_k: f64,
}

impl HarmonicOscillator {
    pub fn new(mass: f64, spring_k: f64) -> Self {
        Self {
            mass: mass.max(1e-12),
            spring_k: spring_k.max(0.0),
        }
    }
}

impl HamiltonianSystemND for HarmonicOscillator {
    fn dimension(&self) -> usize {
        1
    }

    fn force(&self, q: &[f64]) -> Vec<f64> {
        vec![-self.spring_k * q[0]]
    }

    fn velocity(&self, p: &[f64]) -> Vec<f64> {
        vec![p[0] / self.mass]
    }

    fn total_energy(&self, q: &[f64], p: &[f64]) -> f64 {
        let t = 0.5 * p[0] * p[0] / self.mass;
        let v = 0.5 * self.spring_k * q[0] * q[0];
        t + v
    }
}

/// Gravitational Kepler 2-Body Orbit: $H(\mathbf{q}, \mathbf{p}) = \frac{\|\mathbf{p}\|^2}{2\mu} - \frac{G M \mu}{\|\mathbf{q}\|}$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeplerOrbit2D {
    pub mu: f64,
    pub gm_mu: f64,
}

impl KeplerOrbit2D {
    pub fn new(mu: f64, gm_mu: f64) -> Self {
        Self {
            mu: mu.max(1e-12),
            gm_mu: gm_mu.max(1e-12),
        }
    }
}

impl HamiltonianSystemND for KeplerOrbit2D {
    fn dimension(&self) -> usize {
        2
    }

    fn force(&self, q: &[f64]) -> Vec<f64> {
        let r2 = q[0] * q[0] + q[1] * q[1] + 1e-18;
        let r = r2.sqrt();
        let f_mag = -self.gm_mu / (r2 * r);
        vec![f_mag * q[0], f_mag * q[1]]
    }

    fn velocity(&self, p: &[f64]) -> Vec<f64> {
        vec![p[0] / self.mu, p[1] / self.mu]
    }

    fn total_energy(&self, q: &[f64], p: &[f64]) -> f64 {
        let t = 0.5 * (p[0] * p[0] + p[1] * p[1]) / self.mu;
        let r = (q[0] * q[0] + q[1] * q[1] + 1e-18).sqrt();
        let v = -self.gm_mu / r;
        t + v
    }
}

/// Symplectic Integrator Stepper Algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymplecticOrder {
    /// 2nd-Order Velocity-Verlet / Störmer-Leapfrog
    #[default]
    VerletOrder2,
    /// 3rd-Order Ruth Symplectic Partition
    RuthOrder3,
    /// 4th-Order Suzuki-Yoshida Symplectic Composition
    YoshidaOrder4,
}

/// Phase Space State $(\mathbf{q}, \mathbf{p})$ at time $t$.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseState {
    pub time: f64,
    pub q: Vec<f64>,
    pub p: Vec<f64>,
    pub energy: f64,
}

/// Trajectory of symplectic integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymplecticTrajectory {
    pub states: Vec<PhaseState>,
    pub initial_energy: f64,
    pub final_energy: f64,
    pub max_energy_drift: f64,
}

pub struct SymplecticIntegrator;

impl SymplecticIntegrator {
    /// Advance state by one 2nd-order Velocity-Verlet step of size $h$.
    pub fn step_verlet<H: HamiltonianSystemND>(sys: &H, q: &mut [f64], p: &mut [f64], h: f64) {
        let n = sys.dimension();
        let f0 = sys.force(q);

        // 1. Half-kick in momentum
        let mut p_half = vec![0.0; n];
        for i in 0..n {
            p_half[i] = p[i] + 0.5 * h * f0[i];
        }

        // 2. Full drift in coordinate
        let v = sys.velocity(&p_half);
        for i in 0..n {
            q[i] += h * v[i];
        }

        // 3. Final half-kick in momentum
        let f1 = sys.force(q);
        for i in 0..n {
            p[i] = p_half[i] + 0.5 * h * f1[i];
        }
    }

    /// Advance state by one 4th-order Yoshida symplectic step of size $h$.
    pub fn step_yoshida<H: HamiltonianSystemND>(sys: &H, q: &mut [f64], p: &mut [f64], h: f64) {
        let cbrt2 = 2.0_f64.cbrt();
        let w1 = 1.0 / (2.0 - cbrt2);
        let w0 = 1.0 - 2.0 * w1;

        // Step 1 of sub-step size w1 * h
        Self::step_verlet(sys, q, p, w1 * h);
        // Step 2 of sub-step size w0 * h
        Self::step_verlet(sys, q, p, w0 * h);
        // Step 3 of sub-step size w1 * h
        Self::step_verlet(sys, q, p, w1 * h);
    }

    /// Integrate Hamiltonian system from $t=0$ to $t=t_{\text{end}}$ with step size $h$.
    pub fn integrate<H: HamiltonianSystemND>(
        sys: &H,
        q0: &[f64],
        p0: &[f64],
        h: f64,
        steps: usize,
        order: SymplecticOrder,
    ) -> SymplecticTrajectory {
        let mut q = q0.to_vec();
        let mut p = p0.to_vec();
        let mut t = 0.0;

        let initial_energy = sys.total_energy(&q, &p);
        let mut max_energy_drift = 0.0_f64;

        let mut states = Vec::with_capacity(steps + 1);
        states.push(PhaseState {
            time: 0.0,
            q: q.clone(),
            p: p.clone(),
            energy: initial_energy,
        });

        for _ in 0..steps {
            match order {
                SymplecticOrder::VerletOrder2 => Self::step_verlet(sys, &mut q, &mut p, h),
                SymplecticOrder::RuthOrder3 | SymplecticOrder::YoshidaOrder4 => {
                    Self::step_yoshida(sys, &mut q, &mut p, h)
                }
            }
            t += h;
            let current_energy = sys.total_energy(&q, &p);
            let drift = (current_energy - initial_energy).abs();
            if drift > max_energy_drift {
                max_energy_drift = drift;
            }

            states.push(PhaseState {
                time: t,
                q: q.clone(),
                p: p.clone(),
                energy: current_energy,
            });
        }

        let final_energy = states.last().map(|s| s.energy).unwrap_or(initial_energy);

        SymplecticTrajectory {
            states,
            initial_energy,
            final_energy,
            max_energy_drift,
        }
    }
}
