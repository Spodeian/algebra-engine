//! # `algebra-physics`
//!
//! Physical units, physical constants, dimensional analysis, and automated unit conversions.

use crate::calculus::SymbolicCalculus;
use algebra_core::{AlgebraError, AlgebraResult, ExprGraph, ExprId};

/// SI Base Dimensions with arbitrary real number exponents:
/// [Mass, Length, Time, Current, Temperature, Amount, LuminousIntensity]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dimensions {
    pub mass: f64,
    pub length: f64,
    pub time: f64,
    pub current: f64,
    pub temperature: f64,
    pub amount: f64,
    pub intensity: f64,
}

impl Dimensions {
    /// Dimensionless quantity [0, 0, 0, 0, 0, 0, 0]
    pub fn dimensionless() -> Self {
        Self {
            mass: 0.0,
            length: 0.0,
            time: 0.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        }
    }

    /// Length dimension (meters)
    pub fn length() -> Self {
        Self {
            mass: 0.0,
            length: 1.0,
            time: 0.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        }
    }

    /// Mass dimension (kg)
    pub fn mass() -> Self {
        Self {
            mass: 1.0,
            length: 0.0,
            time: 0.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        }
    }

    /// Time dimension (seconds)
    pub fn time() -> Self {
        Self {
            mass: 0.0,
            length: 0.0,
            time: 1.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        }
    }

    /// Velocity dimension [M^0 L^1 T^-1]
    pub fn velocity() -> Self {
        Self {
            mass: 0.0,
            length: 1.0,
            time: -1.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        }
    }

    /// Check if quantity is dimensionless within floating point tolerance.
    pub fn is_dimensionless(&self) -> bool {
        self.mass.abs() < 1e-9
            && self.length.abs() < 1e-9
            && self.time.abs() < 1e-9
            && self.current.abs() < 1e-9
            && self.temperature.abs() < 1e-9
            && self.amount.abs() < 1e-9
            && self.intensity.abs() < 1e-9
    }

    /// Raise dimensional exponents to a real power $\alpha \in \mathbb{R}$.
    pub fn powf(self, alpha: f64) -> Self {
        Self {
            mass: self.mass * alpha,
            length: self.length * alpha,
            time: self.time * alpha,
            current: self.current * alpha,
            temperature: self.temperature * alpha,
            amount: self.amount * alpha,
            intensity: self.intensity * alpha,
        }
    }
}

impl PartialEq for Dimensions {
    fn eq(&self, other: &Self) -> bool {
        (self.mass - other.mass).abs() < 1e-9
            && (self.length - other.length).abs() < 1e-9
            && (self.time - other.time).abs() < 1e-9
            && (self.current - other.current).abs() < 1e-9
            && (self.temperature - other.temperature).abs() < 1e-9
            && (self.amount - other.amount).abs() < 1e-9
            && (self.intensity - other.intensity).abs() < 1e-9
    }
}

impl Eq for Dimensions {}

impl std::ops::Mul for Dimensions {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            mass: self.mass + other.mass,
            length: self.length + other.length,
            time: self.time + other.time,
            current: self.current + other.current,
            temperature: self.temperature + other.temperature,
            amount: self.amount + other.amount,
            intensity: self.intensity + other.intensity,
        }
    }
}

impl std::ops::Div for Dimensions {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            mass: self.mass - other.mass,
            length: self.length - other.length,
            time: self.time - other.time,
            current: self.current - other.current,
            temperature: self.temperature - other.temperature,
            amount: self.amount - other.amount,
            intensity: self.intensity - other.intensity,
        }
    }
}

/// Physical Quantity associating a symbolic expression with SI physical dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    pub value: ExprId,
    pub dimensions: Dimensions,
}

impl Quantity {
    pub fn new(value: ExprId, dimensions: Dimensions) -> Self {
        Self { value, dimensions }
    }

    /// Add two quantities with dimensional consistency check.
    pub fn add(&self, graph: &ExprGraph, other: &Quantity) -> AlgebraResult<Quantity> {
        if self.dimensions != other.dimensions {
            return Err(AlgebraError::DimensionMismatch(format!(
                "Cannot add physical quantities with differing dimensions {:?} and {:?}",
                self.dimensions, other.dimensions
            )));
        }
        let sum_val = graph.add([self.value, other.value]);
        Ok(Quantity::new(sum_val, self.dimensions))
    }

    /// Multiply two physical quantities (combines dimensions).
    pub fn multiply(&self, graph: &ExprGraph, other: &Quantity) -> Quantity {
        let prod_val = graph.mul([self.value, other.value]);
        let prod_dim = self.dimensions * other.dimensions;
        Quantity::new(prod_val, prod_dim)
    }
}

/// Physical constants constructor helpers with CODATA 2022 / NIST exact values.
pub struct Constants;

impl Constants {
    /// Speed of light in vacuum $c = 299,792,458 \text{ m/s}$ (exact SI definition)
    pub fn speed_of_light(graph: &ExprGraph) -> Quantity {
        let val = graph.float(299_792_458.0);
        Quantity::new(val, Dimensions::velocity())
    }

    /// Planck constant $h = 6.62607015 \times 10^{-34} \text{ J s}$ (exact SI definition)
    pub fn planck_h(graph: &ExprGraph) -> Quantity {
        let val = graph.float(6.62607015e-34);
        let action_dim = Dimensions {
            mass: 1.0,
            length: 2.0,
            time: -1.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        };
        Quantity::new(val, action_dim)
    }

    /// Reduced Planck constant $\hbar = \frac{h}{2\pi} = 1.0545718176461565 \times 10^{-34} \text{ J s}$
    pub fn hbar(graph: &ExprGraph) -> Quantity {
        let val = graph.float(1.0545718176461565e-34);
        let action_dim = Dimensions {
            mass: 1.0,
            length: 2.0,
            time: -1.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        };
        Quantity::new(val, action_dim)
    }

    /// Elementary charge $e = 1.602176634 \times 10^{-19} \text{ C}$ (exact SI definition)
    pub fn elementary_charge(graph: &ExprGraph) -> Quantity {
        let val = graph.float(1.602176634e-19);
        let current_time_dim = Dimensions {
            mass: 0.0,
            length: 0.0,
            time: 1.0,
            current: 1.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        };
        Quantity::new(val, current_time_dim)
    }

    /// Boltzmann constant $k_B = 1.380649 \times 10^{-23} \text{ J/K}$ (exact SI definition)
    pub fn boltzmann(graph: &ExprGraph) -> Quantity {
        let val = graph.float(1.380649e-23);
        let k_dim = Dimensions {
            mass: 1.0,
            length: 2.0,
            time: -2.0,
            current: 0.0,
            temperature: -1.0,
            amount: 0.0,
            intensity: 0.0,
        };
        Quantity::new(val, k_dim)
    }

    /// Avogadro constant $N_A = 6.02214076 \times 10^{23} \text{ mol}^{-1}$ (exact SI definition)
    pub fn avogadro(graph: &ExprGraph) -> Quantity {
        let val = graph.float(6.02214076e23);
        let avo_dim = Dimensions {
            mass: 0.0,
            length: 0.0,
            time: 0.0,
            current: 0.0,
            temperature: 0.0,
            amount: -1.0,
            intensity: 0.0,
        };
        Quantity::new(val, avo_dim)
    }

    /// Newtonian Gravitational Constant $G = 6.67430 \times 10^{-11} \text{ m}^3 \text{kg}^{-1} \text{s}^{-2}$ (CODATA 2022)
    pub fn gravitational_constant(graph: &ExprGraph) -> Quantity {
        let val = graph.float(6.67430e-11);
        let g_dim = Dimensions {
            mass: -1.0,
            length: 3.0,
            time: -2.0,
            current: 0.0,
            temperature: 0.0,
            amount: 0.0,
            intensity: 0.0,
        };
        Quantity::new(val, g_dim)
    }
}

/// Analytical Mechanics: Lagrangian formulation $\mathcal{L}(q, \dot{q}, t) = T - V$.
pub struct LagrangianSystem {
    pub lagrangian: ExprId,
}

impl LagrangianSystem {
    pub fn new(lagrangian: ExprId) -> Self {
        Self { lagrangian }
    }

    /// Compute generalized canonical momentum $p_i = \frac{\partial \mathcal{L}}{\partial \dot{q}_i}$.
    pub fn canonical_momentum(&self, graph: &ExprGraph, q_dot: algebra_core::SymbolId) -> ExprId {
        graph.diff(self.lagrangian, q_dot)
    }

    /// Compute Euler-Lagrange equation of motion: $\frac{d}{dt}\left(\frac{\partial \mathcal{L}}{\partial \dot{q}_i}\right) - \frac{\partial \mathcal{L}}{\partial q_i} = 0$.
    pub fn euler_lagrange(
        &self,
        graph: &ExprGraph,
        q: algebra_core::SymbolId,
        q_dot: algebra_core::SymbolId,
        t: algebra_core::SymbolId,
    ) -> ExprId {
        let p_i = self.canonical_momentum(graph, q_dot);
        let dp_dt = graph.diff(p_i, t);
        let dl_dq = graph.diff(self.lagrangian, q);
        graph.sub(dp_dt, dl_dq)
    }
}

/// Analytical Mechanics: Hamiltonian formulation $\mathcal{H}(q, p, t) = \sum_i p_i \dot{q}_i - \mathcal{L}$.
pub struct HamiltonianSystem {
    pub hamiltonian: ExprId,
}

impl HamiltonianSystem {
    pub fn new(hamiltonian: ExprId) -> Self {
        Self { hamiltonian }
    }

    /// Compute Poisson Bracket $\{F, G\} = \sum_i \left(\frac{\partial F}{\partial q_i}\frac{\partial G}{\partial p_i} - \frac{\partial F}{\partial p_i}\frac{\partial G}{\partial q_i}\right)$.
    pub fn poisson_bracket(
        &self,
        graph: &ExprGraph,
        f: ExprId,
        g: ExprId,
        q: algebra_core::SymbolId,
        p: algebra_core::SymbolId,
    ) -> ExprId {
        let df_dq = graph.diff(f, q);
        let df_dp = graph.diff(f, p);
        let dg_dq = graph.diff(g, q);
        let dg_dp = graph.diff(g, p);

        let term1 = graph.mul([df_dq, dg_dp]);
        let term2 = graph.mul([df_dp, dg_dq]);
        graph.sub(term1, term2)
    }
}

/// Special Relativity: Minkowski 4-Vector $A^\mu = (A^0, A^1, A^2, A^3)$ with metric $\eta_{\mu\nu} = \text{diag}(-1, 1, 1, 1)$.
#[derive(Debug, Clone)]
pub struct FourVector {
    pub components: [ExprId; 4],
}

impl FourVector {
    pub fn new(c0: ExprId, c1: ExprId, c2: ExprId, c3: ExprId) -> Self {
        Self {
            components: [c0, c1, c2, c3],
        }
    }

    /// Compute Minkowski invariant scalar product $A^\mu B_\mu = -A^0 B^0 + A^1 B^1 + A^2 B^2 + A^3 B^3$.
    pub fn minkowski_dot(&self, graph: &ExprGraph, other: &FourVector) -> ExprId {
        let t0 = graph.mul([self.components[0], other.components[0]]);
        let neg_t0 = graph.neg(t0);
        let t1 = graph.mul([self.components[1], other.components[1]]);
        let t2 = graph.mul([self.components[2], other.components[2]]);
        let t3 = graph.mul([self.components[3], other.components[3]]);

        graph.add([neg_t0, t1, t2, t3])
    }

    /// Compute invariant squared norm $A^\mu A_\mu$.
    pub fn norm_squared(&self, graph: &ExprGraph) -> ExprId {
        self.minkowski_dot(graph, self)
    }
}
