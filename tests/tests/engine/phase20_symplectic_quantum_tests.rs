//! Integration Tests for Phase 20: Symplectic Geometric Mechanics & Quantum Circuits.

use algebra_engine::quantum_circuit::{QuantumCircuit, QuantumGate};
use algebra_engine::symplectic::{
    HarmonicOscillator, KeplerOrbit2D, SymplecticIntegrator, SymplecticOrder,
};
use algebra_engine::{quantum_bell_state, quantum_ghz_state, symplectic_integrate};

#[test]
fn test_harmonic_oscillator_symplectic_yoshida_energy_conservation() {
    let osc = HarmonicOscillator::new(1.0, 10.0);
    let q0 = [2.0];
    let p0 = [0.0];
    let h = 0.01;
    let steps = 5000;

    let traj =
        SymplecticIntegrator::integrate(&osc, &q0, &p0, h, steps, SymplecticOrder::YoshidaOrder4);
    assert_eq!(traj.states.len(), steps + 1);

    // Initial energy: 0.5 * 10 * 2^2 = 20.0
    assert!((traj.initial_energy - 20.0).abs() < 1e-10);
    assert!((traj.final_energy - 20.0).abs() < 1e-4);
    assert!(traj.max_energy_drift < 1e-3);
}

#[test]
fn test_kepler_orbit_symplectic_energy_and_periodicity() {
    let kepler = KeplerOrbit2D::new(1.0, 100.0);
    // Circular orbit at radius r=10.0: v = sqrt(GM/r) = sqrt(100/10) = sqrt(10)
    let v_circ = 10.0_f64.sqrt();
    let q0 = [10.0, 0.0];
    let p0 = [0.0, v_circ];
    let h = 0.005;
    let steps = 2000;

    let traj = SymplecticIntegrator::integrate(
        &kepler,
        &q0,
        &p0,
        h,
        steps,
        SymplecticOrder::YoshidaOrder4,
    );
    assert_eq!(traj.states.len(), steps + 1);

    // Energy: T + V = 0.5 * 10 - 100/10 = 5 - 10 = -5.0
    assert!((traj.initial_energy - (-5.0)).abs() < 1e-10);
    assert!((traj.final_energy - (-5.0)).abs() < 1e-3);
}

#[test]
fn test_quantum_bell_state_and_entanglement() {
    let bell = QuantumCircuit::bell_pair();
    assert_eq!(bell.num_qubits, 2);

    let probs = bell.probabilities();
    assert_eq!(probs.len(), 4);

    // |00> has probability 0.5
    assert!((probs[0] - 0.5).abs() < 1e-10);
    // |01> has probability 0.0
    assert!(probs[1].abs() < 1e-10);
    // |10> has probability 0.0
    assert!(probs[2].abs() < 1e-10);
    // |11> has probability 0.5
    assert!((probs[3] - 0.5).abs() < 1e-10);
}

#[test]
fn test_quantum_ghz_state_n_qubits() {
    let ghz = QuantumCircuit::ghz_state(3);
    assert_eq!(ghz.num_qubits, 3);

    let probs = ghz.probabilities();
    assert_eq!(probs.len(), 8);

    // |000> = index 0 (0.5)
    assert!((probs[0] - 0.5).abs() < 1e-10);
    // |111> = index 7 (0.5)
    assert!((probs[7] - 0.5).abs() < 1e-10);
    for p in probs.iter().take(7).skip(1) {
        assert!(p.abs() < 1e-10);
    }
}

#[test]
fn test_quantum_circuit_openqasm_export() {
    let mut qc = QuantumCircuit::new(3);
    qc.apply_gate(QuantumGate::H { target: 0 });
    qc.apply_gate(QuantumGate::CNOT {
        control: 0,
        target: 1,
    });
    qc.apply_gate(QuantumGate::Toffoli {
        control1: 0,
        control2: 1,
        target: 2,
    });
    qc.apply_gate(QuantumGate::Rz {
        target: 2,
        theta: std::f64::consts::FRAC_PI_2,
    });

    let qasm = qc.to_openqasm();
    assert!(qasm.contains("OPENQASM 2.0;"));
    assert!(qasm.contains("h q[0];"));
    assert!(qasm.contains("cx q[0], q[1];"));
    assert!(qasm.contains("ccx q[0], q[1], q[2];"));
    assert!(qasm.contains("rz(") && qasm.contains("q[2];"));
}

#[test]
fn test_phase20_dsl_macros() {
    let osc = HarmonicOscillator::new(1.0, 5.0);
    let q0 = [1.0];
    let p0 = [0.0];
    let traj = symplectic_integrate!(&osc, q0 = &q0, p0 = &p0, h = 0.01, steps = 100);
    assert_eq!(traj.states.len(), 101);

    let bell = quantum_bell_state!();
    assert_eq!(bell.num_qubits, 2);

    let ghz4 = quantum_ghz_state!(4);
    assert_eq!(ghz4.num_qubits, 4);
    let probs = ghz4.probabilities();
    assert!((probs[0] - 0.5).abs() < 1e-10);
    assert!((probs[15] - 0.5).abs() < 1e-10);
}
