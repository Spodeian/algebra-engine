//! Integration Tests for Phase 18: Python Ecosystem (`urae-py`) & Jupyter Kernel (`urae-kernel`).

use urae_kernel::UraeKernel;
use urae_py::{
    PyExprGraph, PyInvoluteGear, PyQuantumCircuit, py_diff, py_integrate, py_simplify, py_solve,
};

#[test]
fn test_py_expr_graph_diff_integrate_simplify() {
    let graph = PyExprGraph::parse("x^3 + 2*x").expect("parse failed");
    assert!(graph.latex.contains("{x}^{3}"));

    let diff = graph.diff("x").expect("diff failed");
    assert!(!diff.latex.is_empty());

    let int = graph.integrate("x").expect("integrate failed");
    assert!(!int.latex.is_empty());

    let unsimp = PyExprGraph::parse("(x + 0) * 1 + 0").expect("parse failed");
    let simp = unsimp.simplify().expect("simplify failed");
    assert_eq!(simp.unicode, "x");
}

#[test]
fn test_py_functional_api_and_solver() {
    let diff_res = py_diff("sin(x) * x", "x").expect("diff failed");
    assert!(!diff_res.is_empty());

    let int_res = py_integrate("exp(x)", "x").expect("integrate failed");
    assert!(int_res.contains("e^{x}") || int_res.contains("exp"));

    let simp_res = py_simplify("x * 1 + 0").expect("simplify failed");
    assert_eq!(simp_res, "x");

    let roots = py_solve("x^2 - 16 = 0", "x").expect("solve failed");
    assert_eq!(roots.len(), 2);
}

#[test]
fn test_py_cad_involute_gear_wrapper() {
    let gear = PyInvoluteGear::spur(2.0, 18, 8.0);
    assert_eq!(gear.teeth, 18);
    assert!(gear.vertex_count > 0);
    assert!(gear.triangle_count > 0);

    let obj = gear.to_obj();
    assert!(obj.contains("v ") && obj.contains("f "));

    let stl = gear.to_stl();
    assert!(stl.contains("solid") && stl.contains("facet normal"));
}

#[test]
fn test_py_quantum_circuit_wrapper() {
    let bell = PyQuantumCircuit::bell_pair();
    assert_eq!(bell.num_qubits, 2);
    assert_eq!(bell.probabilities.len(), 4);
    assert!((bell.probabilities[0] - 0.5).abs() < 1e-10);
    assert!((bell.probabilities[3] - 0.5).abs() < 1e-10);

    let ghz = PyQuantumCircuit::ghz_state(3);
    assert_eq!(ghz.num_qubits, 3);
    assert_eq!(ghz.probabilities.len(), 8);
    assert!((ghz.probabilities[0] - 0.5).abs() < 1e-10);
    assert!((ghz.probabilities[7] - 0.5).abs() < 1e-10);

    let qasm = ghz.to_openqasm();
    assert!(qasm.contains("OPENQASM 2.0;"));
}

#[test]
fn test_jupyter_kernel_protocol_and_rich_mime() {
    let mut kernel = UraeKernel::new();
    let info = kernel.kernel_info();
    assert_eq!(info.get("language_info.name").unwrap(), "urae");

    // Test math expression execution
    let reply1 = kernel.execute("x^2 + 2*x + 1");
    assert_eq!(reply1.status, "ok");
    assert!(reply1.mime_bundle.data.contains_key("text/latex"));
    assert!(reply1.mime_bundle.data.contains_key("text/plain"));

    // Test CAD gear 3D HTML widget execution
    let reply2 = kernel.execute("gear!(teeth=16)");
    assert_eq!(reply2.status, "ok");
    assert!(reply2.mime_bundle.data.contains_key("text/html"));
    let html = reply2.mime_bundle.data.get("text/html").unwrap();
    assert!(html.contains("three.min.js") || html.contains("<!DOCTYPE html>"));

    // Test Quantum execution
    let reply3 = kernel.execute("quantum_bell");
    assert_eq!(reply3.status, "ok");
    assert!(reply3.mime_bundle.data.contains_key("text/plain"));
}
