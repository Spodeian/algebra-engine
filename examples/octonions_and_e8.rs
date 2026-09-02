//! URAE Example: Exceptional Lie Groups, Octonions & E8 Root Lattice
//! Run via: `cargo run --example octonions_and_e8`

use urae::prelude::*;

fn main() {
    println!("=== URAE Exceptional Lie Groups & Octonions ===");

    // 1. Octonion Non-associativity & Alternativity
    let x = octonion!(1, 0, 1, 0, 0, 0, 0, 0);
    let y = octonion!(0, 1, 0, 1, 0, 0, 0, 0);
    let z = octonion!(0, 0, 1, 0, 1, 0, 0, 0);

    let assoc = octonion_associator!(x, y, z);
    println!("Associator [x, y, z] = {:?}", assoc.coords);
    println!("Associator Norm = {}", assoc.norm());

    let alt = octonion_associator!(x, x, y);
    println!("Alternativity [x, x, y] Norm = {}", alt.norm());

    // 2. Albert Exceptional Jordan Algebra H_3(O)
    let albert = AlbertAlgebra::new([1.0, 1.0, 1.0], [x, Octonion::zero(), Octonion::zero()]);
    println!("Albert Element Trace = {}", albert.trace());
    println!("Albert Freudenthal Determinant = {}", albert.determinant());

    // 3. E8 Root Lattice
    let roots = E8Lattice::all_roots();
    println!("E8 Root System contains {} roots.", roots.len());
    println!("=== Execution Complete ===");
}
