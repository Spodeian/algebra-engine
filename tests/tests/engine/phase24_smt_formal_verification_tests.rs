//! Integration Tests for Phase 24: SMT-LIB2 Automated Theorem Prover (ATP) Bridge & Formal Verification.

use algebra_engine::smt::{ProofCertificate, SmtExpr, SmtLogic, SmtProblem, SmtSort};
use algebra_engine::{proof_certificate, smt_assert, smt_problem};

#[test]
fn test_smt_qf_nra_nonlinear_real_serialization() {
    let mut problem = SmtProblem::with_logic(SmtLogic::QfNra);
    problem.add_var("x", SmtSort::Real);
    problem.add_var("y", SmtSort::Real);

    // x^2 + y^2 <= 1.0
    let circle_bound = SmtExpr::Le(
        Box::new(SmtExpr::Add(vec![
            SmtExpr::Pow(
                Box::new(SmtExpr::Var("x".to_string())),
                Box::new(SmtExpr::Real(2.0)),
            ),
            SmtExpr::Pow(
                Box::new(SmtExpr::Var("y".to_string())),
                Box::new(SmtExpr::Real(2.0)),
            ),
        ])),
        Box::new(SmtExpr::Real(1.0)),
    );
    problem.assert(circle_bound);

    // x + y >= 1.2
    let line_bound = SmtExpr::Ge(
        Box::new(SmtExpr::Add(vec![
            SmtExpr::Var("x".to_string()),
            SmtExpr::Var("y".to_string()),
        ])),
        Box::new(SmtExpr::Real(1.2)),
    );
    problem.assert(line_bound);

    let smt_script = problem.to_smtlib2_string();
    assert!(smt_script.contains("(set-logic QF_NRA)"));
    assert!(smt_script.contains("(declare-const x Real)"));
    assert!(smt_script.contains("(declare-const y Real)"));
    assert!(smt_script.contains("(assert (<= (+ (^ x 2.0) (^ y 2.0)) 1.0))"));
    assert!(smt_script.contains("(assert (>= (+ x y) 1.2))"));
    assert!(smt_script.contains("(check-sat)"));
    assert!(smt_script.contains("(get-model)"));
}

#[test]
fn test_smt_qf_bv_bitvector_serialization() {
    let mut problem = SmtProblem::with_logic(SmtLogic::QfBv);
    problem.add_var("reg_a", SmtSort::BitVec(32));
    problem.add_var("reg_b", SmtSort::BitVec(32));

    // (bvand reg_a reg_b) == 0
    let disj = SmtExpr::Eq(
        Box::new(SmtExpr::BvAnd(
            Box::new(SmtExpr::Var("reg_a".to_string())),
            Box::new(SmtExpr::Var("reg_b".to_string())),
        )),
        Box::new(SmtExpr::BitVec(0, 32)),
    );
    problem.assert(disj);

    let script = problem.to_smtlib2_string();
    assert!(smt_script_contains(&script, "(set-logic QF_BV)"));
    assert!(smt_script_contains(
        &script,
        "(declare-const reg_a (_ BitVec 32))"
    ));
    assert!(smt_script_contains(
        &script,
        "(assert (= (bvand reg_a reg_b) (_ bv0 32)))"
    ));
}

fn smt_script_contains(script: &str, target: &str) -> bool {
    script.contains(target)
}

#[test]
fn test_smt_expr_ast_operators() {
    let expr = SmtExpr::Ite(
        Box::new(SmtExpr::Lt(
            Box::new(SmtExpr::Var("x".to_string())),
            Box::new(SmtExpr::Real(0.0)),
        )),
        Box::new(SmtExpr::Neg(Box::new(SmtExpr::Var("x".to_string())))),
        Box::new(SmtExpr::Var("x".to_string())),
    );
    assert_eq!(expr.to_smtlib2(), "(ite (< x 0.0) (- x) x)");

    let bool_expr = SmtExpr::Implies(
        Box::new(SmtExpr::Var("P".to_string())),
        Box::new(SmtExpr::Or(vec![
            SmtExpr::Var("Q".to_string()),
            SmtExpr::Not(Box::new(SmtExpr::Var("R".to_string()))),
        ])),
    );
    assert_eq!(bool_expr.to_smtlib2(), "(=> P (or Q (not R)))");
}

#[test]
fn test_lean4_proof_certificate_generation() {
    let cert = ProofCertificate::new(
        "real_nonneg_sum",
        vec![
            ("a".to_string(), "ℝ".to_string()),
            ("b".to_string(), "ℝ".to_string()),
            ("ha".to_string(), "0 ≤ a".to_string()),
            ("hb".to_string(), "0 ≤ b".to_string()),
        ],
        "0 ≤ a + b",
        "linarith",
    );

    let lean4_code = cert.to_lean4_theorem();
    assert!(lean4_code.contains(
        "theorem real_nonneg_sum (a : ℝ) (b : ℝ) (ha : 0 ≤ a) (hb : 0 ≤ b) : 0 ≤ a + b := by"
    ));
    assert!(lean4_code.contains("linarith"));
}

#[test]
fn test_coq_proof_certificate_generation() {
    let cert = ProofCertificate::new(
        "algebraic_diff_squares",
        vec![
            ("x".to_string(), "R".to_string()),
            ("y".to_string(), "R".to_string()),
        ],
        "(x + y) * (x - y) = x * x - y * y",
        "ring",
    );

    let coq_code = cert.to_coq_lemma();
    assert!(coq_code.contains(
        "Lemma algebraic_diff_squares (x : R) (y : R) : (x + y) * (x - y) = x * x - y * y."
    ));
    assert!(coq_code.contains("Proof."));
    assert!(coq_code.contains("ring."));
    assert!(coq_code.contains("Qed."));
}

#[test]
fn test_phase24_dsl_macros() {
    let mut p = smt_problem!(SmtLogic::QfLra);
    p.add_var("z", SmtSort::Real);
    smt_assert!(
        p,
        SmtExpr::Gt(
            Box::new(SmtExpr::Var("z".to_string())),
            Box::new(SmtExpr::Real(0.0))
        )
    );

    let script = p.to_smtlib2_string();
    assert!(script.contains("(set-logic QF_LRA)"));

    let cert = proof_certificate!(
        "pythagorean_identity",
        [("x", "Real")],
        "sin(x)^2 + cos(x)^2 = 1",
        "ring"
    );
    let lean_out = cert.to_lean4_theorem();
    assert!(lean_out
        .contains("theorem pythagorean_identity (x : Real) : sin(x)^2 + cos(x)^2 = 1 := by"));
}
