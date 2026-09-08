//! # `urae-cli` (URAE Command Line Interface)
//!
//! Interactive REPL binary for Universal Rust Algebra Engine (URAE).
#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--json" || arg == "-j") {
        use std::io::{self, Read};
        let mut input_str = String::new();
        if io::stdin().read_to_string(&mut input_str).is_ok() {
            let graph = algebra_core::ExprGraph::new();
            let json_resp = urae_cli::process_json_request(&graph, &input_str);
            println!("{}", json_resp);
        }
    } else {
        urae_cli::run_repl();
    }
}
