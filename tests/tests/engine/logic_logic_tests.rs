use algebra_core::SymbolId;
use algebra_engine::logic::{AssumptionsContext, BoolExpr, Predicate};

#[test]
fn test_assumptions_deduction() {
    let mut ctx = AssumptionsContext::new();
    let sym = SymbolId::new(1);

    ctx.assume(sym, Predicate::Positive);
    assert!(ctx.is(sym, Predicate::Positive));
    assert!(ctx.is(sym, Predicate::NonNegative));
    assert!(ctx.is(sym, Predicate::NonZero));
    assert!(ctx.is(sym, Predicate::Real));
    assert!(ctx.is_consistent());
}

#[test]
fn test_assumptions_contradiction() {
    let mut ctx = AssumptionsContext::new();
    let sym = SymbolId::new(1);

    ctx.assume(sym, Predicate::Positive);
    ctx.assume(sym, Predicate::Negative);
    assert!(!ctx.is_consistent());
}

#[test]
fn test_bool_expr_simplification() {
    let expr = BoolExpr::And(vec![
        BoolExpr::True,
        BoolExpr::Not(Box::new(BoolExpr::False)),
        BoolExpr::Var(1),
    ]);
    assert_eq!(expr.simplify(), BoolExpr::Var(1));
}

#[test]
fn test_interval_bounds() {
    use algebra_engine::logic::IntervalBound;

    let bounds = IntervalBound::closed(0.0, 10.0);
    assert!(bounds.contains(5.0));
    assert!(bounds.contains(0.0));
    assert!(bounds.contains(10.0));
    assert!(!bounds.contains(11.0));

    let mut ctx = AssumptionsContext::new();
    let sym = SymbolId::new(1);
    ctx.set_bounds(sym, bounds);
    assert!(ctx.get_bounds(sym).unwrap().contains(5.0));
}

#[test]
fn test_three_valued_logic() {
    use algebra_engine::logic::{ThreeValuedLogic, ThreeValuedSystem, ThreeValuedValue};

    let t = ThreeValuedValue::True;
    let u = ThreeValuedValue::Unknown;

    // Kleene K3
    assert_eq!(
        ThreeValuedLogic::or(t, u, ThreeValuedSystem::KleeneK3),
        ThreeValuedValue::True
    );
    assert_eq!(
        ThreeValuedLogic::and(t, u, ThreeValuedSystem::KleeneK3),
        ThreeValuedValue::Unknown
    );

    // Łukasiewicz Ł3 (u -> u = True)
    assert_eq!(
        ThreeValuedLogic::implies(u, u, ThreeValuedSystem::LukasiewiczL3),
        ThreeValuedValue::True
    );

    // Bochvar nonsense propagation
    assert_eq!(
        ThreeValuedLogic::and(t, u, ThreeValuedSystem::Bochvar),
        ThreeValuedValue::Unknown
    );
}

#[test]
fn test_generalized_n_valued_logic() {
    use algebra_engine::logic::GeneralizedValuedLogic;

    // 4-valued logic with designated values {2, 3}
    let logic4 = GeneralizedValuedLogic::<4>::new(&[2, 3]);
    assert!(logic4.is_designated(3));
    assert!(!logic4.is_designated(1));
    assert_eq!(logic4.eval_not(0), 3);
    assert_eq!(logic4.eval_and(2, 3), 2);
}
