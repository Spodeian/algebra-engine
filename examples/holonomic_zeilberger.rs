//! URAE Example: Holonomic D-Modules & Zeilberger Creative Telescoping
//! Run via: `cargo run --example holonomic_zeilberger`

use urae::prelude::*;

fn main() {
    println!("=== URAE Holonomic D-Modules & Zeilberger Creative Telescoping ===");

    // 1. Non-commutative Weyl Algebra A_1
    let x = weyl_x!(1, 0, 1);
    let d = weyl_d!(1, 0, 1);
    let comm = weyl_commute!(d, x);
    println!("Canonical Weyl Commutator [∂, x] = {:?}", comm);

    // 2. Automated Zeilberger Binomial Proof
    let proof = zeilberger_binomial_proof!();
    println!("Proved sum_k binom(n, k) = 2^n: Recurrence = {}, Verified = {}", proof.recurrence_operator, proof.is_valid);

    // 3. Almkvist-Zeilberger Gaussian Integral ODE
    let ode_proof = almkvist_zeilberger_gaussian!();
    println!("Differential annihilating operator for gaussian integral: Verified = {}", ode_proof.is_valid);
    println!("=== Execution Complete ===");
}
