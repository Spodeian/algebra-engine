use algebra_core::format::LatexFormatter;
use algebra_core::ExprGraph;
use urae_cli::process_input;

#[test]
fn test_cli_process_input() {
    let graph = ExprGraph::new();
    let formatter = LatexFormatter;

    let res = process_input(&graph, &formatter, "x + 2").unwrap();
    assert!(res.contains("x"));
    assert!(res.contains("2"));
}

#[test]
fn test_cli_differentiation() {
    let graph = ExprGraph::new();
    let formatter = LatexFormatter;

    let res = process_input(&graph, &formatter, "diff x^2, x").unwrap();
    assert!(res.contains("2") || res.contains("x"));
}

#[test]
fn test_json_request_response_api() {
    use algebra_core::ExprGraph;
    use urae_cli::{process_json_request, UraeJsonResponse};

    let graph = ExprGraph::new();

    // 1. Nested LaTeX input request
    let json_req = r#"{
        "input": "\\frac{x^2 - 4}{x - 2}",
        "compute_derivative": true,
        "generate_code": true,
        "export_proof": true
    }"#;

    let json_resp = process_json_request(&graph, json_req);
    let resp: UraeJsonResponse = serde_json::from_str(&json_resp).expect("Deserialization failed");

    assert!(resp.success);
    assert!(resp.output_latex.contains("frac") || resp.output_latex.contains("x"));
    assert!(resp.derivative_latex.is_some());
    assert!(resp.rust_code.is_some());
    assert!(resp.lean4_proof.is_some());

    // 2. Command in JSON request
    let cmd_json = r#"{
        "input": "solve x^2 - 9 = 0, x"
    }"#;

    let cmd_resp = process_json_request(&graph, cmd_json);
    let resp_cmd: UraeJsonResponse =
        serde_json::from_str(&cmd_resp).expect("Deserialization failed");
    assert!(resp_cmd.success);
    assert!(resp_cmd.output_unicode.contains("3"));
}
