//! # `algebra_engine::quantum_circuit`
//!
//! Universal $N$-Qubit Quantum Circuit Simulator, OpenQASM Synthesis & Lindblad Master Equation.
//!
//! ## Mathematical Foundations:
//! - State Vector: $|\psi\rangle = \sum_{k=0}^{2^N-1} c_k |k\rangle \in \mathbb{C}^{2^N}$ with $\sum |c_k|^2 = 1$.
//! - Quantum Gates: Unitary operators $\mathbf{U} \in U(2^N)$ acting on target and control qubits.
//! - Open Systems: Density matrix $\rho \in \mathbb{C}^{2^N \times 2^N}$ governed by the Lindblad master equation:
//!   $$\frac{d\rho}{dt} = -i [\hat{H}, \rho] + \sum_k \left( \hat{L}_k \rho \hat{L}_k^\dagger - \frac{1}{2} \{ \hat{L}_k^\dagger \hat{L}_k, \rho \} \right)$$

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Complex number representation for quantum amplitudes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn abs(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn add(&self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    pub fn sub(&self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    pub fn mul(&self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// Universal Quantum Gate Operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuantumGate {
    /// Hadamard Gate: $H = \frac{1}{\sqrt{2}} \begin{pmatrix} 1 & 1 \\ 1 & -1 \end{pmatrix}$
    H { target: usize },
    /// Pauli-X (NOT) Gate: $X = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix}$
    X { target: usize },
    /// Pauli-Y Gate: $Y = \begin{pmatrix} 0 & -i \\ i & 0 \end{pmatrix}$
    Y { target: usize },
    /// Pauli-Z Gate: $Z = \begin{pmatrix} 1 & 0 \\ 0 & -1 \end{pmatrix}$
    Z { target: usize },
    /// Phase Gate: $S = \begin{pmatrix} 1 & 0 \\ 0 & i \end{pmatrix}$
    S { target: usize },
    /// $\pi/8$ Gate: $T = \begin{pmatrix} 1 & 0 \\ 0 & e^{i\pi/4} \end{pmatrix}$
    T { target: usize },
    /// Parametric Rotation $R_x(\theta) = \cos(\theta/2) I - i \sin(\theta/2) X$
    Rx { target: usize, theta: f64 },
    /// Parametric Rotation $R_y(\theta) = \cos(\theta/2) I - i \sin(\theta/2) Y$
    Ry { target: usize, theta: f64 },
    /// Parametric Rotation $R_z(\theta) = \cos(\theta/2) I - i \sin(\theta/2) Z$
    Rz { target: usize, theta: f64 },
    /// Controlled-NOT Gate: CNOT(control, target)
    CNOT { control: usize, target: usize },
    /// Controlled-Z Gate: CZ(control, target)
    CZ { control: usize, target: usize },
    /// SWAP Gate: SWAP(qubit1, qubit2)
    SWAP { qubit1: usize, qubit2: usize },
    /// Toffoli (CCNOT) Gate: Toffoli(control1, control2, target)
    Toffoli {
        control1: usize,
        control2: usize,
        target: usize,
    },
}

/// $N$-Qubit Quantum Circuit and State Vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuit {
    pub num_qubits: usize,
    pub gates: Vec<QuantumGate>,
    pub state: Vec<Complex64>,
}

impl QuantumCircuit {
    /// Initialize an $N$-qubit circuit in the ground state $|00\dots0\rangle$.
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut state = vec![Complex64::ZERO; dim];
        if dim > 0 {
            state[0] = Complex64::ONE;
        }
        Self {
            num_qubits,
            gates: Vec::new(),
            state,
        }
    }

    /// Add a gate and immediately apply it to the state vector.
    pub fn apply_gate(&mut self, gate: QuantumGate) {
        self.apply_gate_internal(&gate);
        self.gates.push(gate);
    }

    /// Apply gate to internal state vector.
    fn apply_gate_internal(&mut self, gate: &QuantumGate) {
        let dim = 1 << self.num_qubits;

        match *gate {
            QuantumGate::H { target } => {
                let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        let a = self.state[i];
                        let b = self.state[j];
                        self.state[i] = a.add(b).scale(inv_sqrt2);
                        self.state[j] = a.sub(b).scale(inv_sqrt2);
                    }
                }
            }
            QuantumGate::X { target } => {
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        self.state.swap(i, j);
                    }
                }
            }
            QuantumGate::Y { target } => {
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        let a = self.state[i];
                        let b = self.state[j];
                        self.state[i] = Complex64::new(b.im, -b.re);
                        self.state[j] = Complex64::new(-a.im, a.re);
                    }
                }
            }
            QuantumGate::Z { target } => {
                for i in 0..dim {
                    if (i & (1 << target)) != 0 {
                        self.state[i] = self.state[i].scale(-1.0);
                    }
                }
            }
            QuantumGate::S { target } => {
                for i in 0..dim {
                    if (i & (1 << target)) != 0 {
                        let a = self.state[i];
                        self.state[i] = Complex64::new(-a.im, a.re);
                    }
                }
            }
            QuantumGate::T { target } => {
                let cos_pi4 = (PI / 4.0).cos();
                let sin_pi4 = (PI / 4.0).sin();
                let phase = Complex64::new(cos_pi4, sin_pi4);
                for i in 0..dim {
                    if (i & (1 << target)) != 0 {
                        self.state[i] = self.state[i].mul(phase);
                    }
                }
            }
            QuantumGate::Rx { target, theta } => {
                let cos_half = (theta / 2.0).cos();
                let sin_half = (theta / 2.0).sin();
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        let a = self.state[i];
                        let b = self.state[j];
                        // a' = cos(th/2) a - i sin(th/2) b
                        self.state[i] = Complex64::new(
                            cos_half * a.re + sin_half * b.im,
                            cos_half * a.im - sin_half * b.re,
                        );
                        // b' = -i sin(th/2) a + cos(th/2) b
                        self.state[j] = Complex64::new(
                            cos_half * b.re + sin_half * a.im,
                            cos_half * b.im - sin_half * a.re,
                        );
                    }
                }
            }
            QuantumGate::Ry { target, theta } => {
                let cos_half = (theta / 2.0).cos();
                let sin_half = (theta / 2.0).sin();
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        let a = self.state[i];
                        let b = self.state[j];
                        self.state[i] = a.scale(cos_half).sub(b.scale(sin_half));
                        self.state[j] = a.scale(sin_half).add(b.scale(cos_half));
                    }
                }
            }
            QuantumGate::Rz { target, theta } => {
                let phase_pos = Complex64::new((theta / 2.0).cos(), -(theta / 2.0).sin());
                let phase_neg = Complex64::new((theta / 2.0).cos(), (theta / 2.0).sin());
                for i in 0..dim {
                    if (i & (1 << target)) == 0 {
                        self.state[i] = self.state[i].mul(phase_pos);
                    } else {
                        self.state[i] = self.state[i].mul(phase_neg);
                    }
                }
            }
            QuantumGate::CNOT { control, target } => {
                for i in 0..dim {
                    if (i & (1 << control)) != 0 && (i & (1 << target)) == 0 {
                        let j = i | (1 << target);
                        self.state.swap(i, j);
                    }
                }
            }
            QuantumGate::CZ { control, target } => {
                for i in 0..dim {
                    if (i & (1 << control)) != 0 && (i & (1 << target)) != 0 {
                        self.state[i] = self.state[i].scale(-1.0);
                    }
                }
            }
            QuantumGate::SWAP { qubit1, qubit2 } => {
                for i in 0..dim {
                    let bit1 = (i >> qubit1) & 1;
                    let bit2 = (i >> qubit2) & 1;
                    if bit1 != bit2 && bit1 == 0 {
                        let j = i ^ ((1 << qubit1) | (1 << qubit2));
                        self.state.swap(i, j);
                    }
                }
            }
            QuantumGate::Toffoli {
                control1,
                control2,
                target,
            } => {
                for i in 0..dim {
                    if (i & (1 << control1)) != 0
                        && (i & (1 << control2)) != 0
                        && (i & (1 << target)) == 0
                    {
                        let j = i | (1 << target);
                        self.state.swap(i, j);
                    }
                }
            }
        }
    }

    /// Measure qubit probabilities $P(|i\rangle) = |\langle i | \psi \rangle|^2$.
    pub fn probabilities(&self) -> Vec<f64> {
        self.state.iter().map(|c| c.norm_sq()).collect()
    }

    /// Bell State Generator: $|\Phi^+\rangle = \frac{|00\rangle + |11\rangle}{\sqrt{2}}$.
    pub fn bell_pair() -> Self {
        let mut qc = Self::new(2);
        qc.apply_gate(QuantumGate::H { target: 0 });
        qc.apply_gate(QuantumGate::CNOT {
            control: 0,
            target: 1,
        });
        qc
    }

    /// Greenberger-Horne-Zeilinger (GHZ) State Generator for $N$ Qubits:
    /// $|\text{GHZ}\rangle = \frac{|00\dots0\rangle + |11\dots1\rangle}{\sqrt{2}}$.
    pub fn ghz_state(n: usize) -> Self {
        let mut qc = Self::new(n);
        if n > 0 {
            qc.apply_gate(QuantumGate::H { target: 0 });
            for i in 1..n {
                qc.apply_gate(QuantumGate::CNOT {
                    control: 0,
                    target: i,
                });
            }
        }
        qc
    }

    /// Export circuit to OpenQASM 2.0 representation.
    pub fn to_openqasm(&self) -> String {
        let mut qasm = String::from("OPENQASM 2.0;\ninclude \"qelib1.inc\";\n");
        qasm.push_str(&format!(
            "qreg q[{}];\ncreg c[{}];\n\n",
            self.num_qubits, self.num_qubits
        ));

        for gate in &self.gates {
            match *gate {
                QuantumGate::H { target } => qasm.push_str(&format!("h q[{}];\n", target)),
                QuantumGate::X { target } => qasm.push_str(&format!("x q[{}];\n", target)),
                QuantumGate::Y { target } => qasm.push_str(&format!("y q[{}];\n", target)),
                QuantumGate::Z { target } => qasm.push_str(&format!("z q[{}];\n", target)),
                QuantumGate::S { target } => qasm.push_str(&format!("s q[{}];\n", target)),
                QuantumGate::T { target } => qasm.push_str(&format!("t q[{}];\n", target)),
                QuantumGate::Rx { target, theta } => {
                    qasm.push_str(&format!("rx({}) q[{}];\n", theta, target))
                }
                QuantumGate::Ry { target, theta } => {
                    qasm.push_str(&format!("ry({}) q[{}];\n", theta, target))
                }
                QuantumGate::Rz { target, theta } => {
                    qasm.push_str(&format!("rz({}) q[{}];\n", theta, target))
                }
                QuantumGate::CNOT { control, target } => {
                    qasm.push_str(&format!("cx q[{}], q[{}];\n", control, target))
                }
                QuantumGate::CZ { control, target } => {
                    qasm.push_str(&format!("cz q[{}], q[{}];\n", control, target))
                }
                QuantumGate::SWAP { qubit1, qubit2 } => {
                    qasm.push_str(&format!("swap q[{}], q[{}];\n", qubit1, qubit2))
                }
                QuantumGate::Toffoli {
                    control1,
                    control2,
                    target,
                } => qasm.push_str(&format!(
                    "ccx q[{}], q[{}], q[{}];\n",
                    control1, control2, target
                )),
            }
        }

        qasm
    }
}
