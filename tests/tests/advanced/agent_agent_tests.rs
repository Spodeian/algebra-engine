use algebra_advanced::agent::{AlgebraAgentInterface, ExplanationGenerator};
use algebra_core::ExprGraph;
use serde_json::json;

#[test]
fn test_agent_get_tool_definitions() {
    let agent = AlgebraAgentInterface::new();
    let defs = agent.get_tool_definitions();
    assert!(!defs.is_empty());

    let names: Vec<String> = defs.into_iter().map(|d| d.name).collect();
    assert!(names.contains(&"differentiate".to_string()));
    assert!(names.contains(&"integrate".to_string()));
    assert!(names.contains(&"simplify".to_string()));
    assert!(names.contains(&"export_proof".to_string()));
}

#[test]
fn test_agent_execute_differentiate_tool() {
    let graph = ExprGraph::new();
    let agent = AlgebraAgentInterface::new();

    let args = json!({
        "expression": "x^2 + 1",
        "variable": "x"
    });

    let res = agent
        .execute_tool_call(&graph, "differentiate", &args)
        .unwrap();
    assert_eq!(res["success"], true);
    assert!(
        res["result_latex"].as_str().unwrap().contains("x")
            || res["result_latex"].as_str().unwrap().contains("2")
    );
    assert!(res["explanation"].as_str().unwrap().contains("x"));
}

#[test]
fn test_agent_explanation_generator() {
    let explanation = ExplanationGenerator::explain_differentiation("x^2", "x", "2 x");
    assert!(explanation.contains("Power Rule"));
    assert!(explanation.contains("2 x"));
}

#[test]
fn test_ai_config_off_by_default() {
    use algebra_advanced::agent::{AiConfig, AiProvider};

    let config = AiConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.provider, AiProvider::GoogleAi);
    assert!(!config.is_configured());
    assert!(config.status_summary().contains("Disabled"));
}

#[test]
fn test_ai_provider_configurations() {
    use algebra_advanced::agent::{AiConfig, AiProvider};

    // Google AI with key
    let mut g_config = AiConfig::new(AiProvider::GoogleAi);
    g_config.enabled = true;
    assert!(!g_config.is_configured());
    g_config.api_key = Some("test_key".to_string());
    assert!(g_config.is_configured());

    // Anthropic with key
    let mut a_config = AiConfig::new(AiProvider::Anthropic);
    a_config.enabled = true;
    a_config.api_key = Some("sk-ant-test".to_string());
    assert!(a_config.is_configured());

    // LM Studio local server
    let mut lm_config = AiConfig::new(AiProvider::LmStudio);
    lm_config.enabled = true;
    assert!(lm_config.is_configured());

    // OpenCode local/remote server
    let mut oc_config = AiConfig::new(AiProvider::OpenCode);
    oc_config.enabled = true;
    assert!(oc_config.is_configured());

    // OpenRouter gateway
    let mut or_config = AiConfig::new(AiProvider::OpenRouter);
    or_config.enabled = true;
    or_config.api_key = Some("or-test-key".to_string());
    assert!(or_config.is_configured());
}

#[test]
fn test_agent_query_when_disabled() {
    use algebra_advanced::agent::{AiConfig, AlgebraAgentInterface};

    let graph = ExprGraph::new();
    let agent = AlgebraAgentInterface::new();
    let config = AiConfig::default(); // Off by default

    let res = agent.query(&config, "Explain derivatives", &graph);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("off by default"));
}
