//! URAE Example: Quantum Field Theory Dirac Traces and Parke-Taylor Amplitudes
//! Run via: `cargo run --example quantum_amplitudes`

use urae::prelude::*;

fn main() {
    println!("=== URAE High-Energy Quantum Field Theory Example ===");

    // 1. Dirac Gamma Matrix Traces
    let trace_4 = dirac_trace!([0, 1, 0, 1]);
    println!("Tr[γ^0 γ^1 γ^0 γ^1] = {}", trace_4);

    // 2. Weyl Spinors & Mandelstam Invariant
    let s1 = WeylSpinor::from_null_momentum(&[100.0, 0.0, 0.0, 100.0]);
    let s2 = WeylSpinor::from_null_momentum(&[100.0, 0.0, 0.0, -100.0]);

    let ang_12 = spinor_bracket_angle!(s1, s2);
    let sq_21 = spinor_bracket_square!(s2, s1);
    let s_12 = ang_12 * sq_21;
    println!("Mandelstam s_12 = <1 2> [2 1] = {}", s_12);

    // 3. Parke-Taylor MHV Amplitude
    let amp = parke_taylor_mhv!(4, [0, 1]);
    println!("Parke-Taylor 4-gluon MHV factor = {}", amp);
    println!("=== Execution Complete ===");
}
