//! Unit and Integration Tests for Const Evaluation Environments across URAE.

use algebra_advanced::clifford::{BladeMask, CliffordSignature};
use algebra_advanced::quantum::QuantumOperator;
use algebra_core::config::{BudgetStatus, ResourceBudget};
use algebra_core::domain::{Domain, DomainCategory};
use algebra_core::id::{DomainId, ExprId, SymbolId};
use algebra_core::interval::RealInterval;
use algebra_core::number::{Constant, Number};
use algebra_core::numbers::{DualNumber, HyperrealNumber};

// Compile-time static / const definitions
const CONST_EXPR_ID: ExprId = ExprId::new(42);
const CONST_DOMAIN_ID: DomainId = DomainId::new(7);
const CONST_SYMBOL_ID: SymbolId = SymbolId::new(101);

const CONST_NUM_INT: Number = Number::integer(42);
const CONST_NUM_RAT: Number = Number::rational(3, 4);
const CONST_NUM_FLOAT: Number = Number::float(std::f64::consts::PI);
const CONST_NUM_PI: Number = Number::PI;
const CONST_NUM_E: Number = Number::E;
const CONST_NUM_ZERO: Number = Number::ZERO;
const CONST_NUM_ONE: Number = Number::ONE;
const CONST_NUM_INF: Number = Number::INFINITY;

const CONST_INTERVAL_UNIT: RealInterval = RealInterval::UNIT;
const CONST_INTERVAL_CUSTOM: RealInterval = RealInterval::new(-5.0, 5.0);
const CONST_INTERVAL_POINT: RealInterval = RealInterval::point(3.0);
const CONST_INTERVAL_ADD: RealInterval = CONST_INTERVAL_CUSTOM.add(&CONST_INTERVAL_POINT);
const CONST_INTERVAL_SUB: RealInterval = CONST_INTERVAL_CUSTOM.sub(&CONST_INTERVAL_POINT);
const CONST_INTERVAL_RECIP: Option<RealInterval> = RealInterval::new(2.0, 4.0).recip();

const CONST_DUAL_ZERO: DualNumber = DualNumber::ZERO;
const CONST_DUAL_ONE: DualNumber = DualNumber::ONE;
const CONST_DUAL_EPS: DualNumber = DualNumber::EPSILON;
const CONST_DUAL_VAR: DualNumber = DualNumber::variable(5.0);
const CONST_DUAL_CONST: DualNumber = DualNumber::constant(10.0);
const CONST_DUAL_ADD: DualNumber = CONST_DUAL_VAR.add(&CONST_DUAL_CONST);
const CONST_DUAL_MUL: DualNumber = CONST_DUAL_VAR.mul(&CONST_DUAL_CONST);

const CONST_HYPERREAL: HyperrealNumber = HyperrealNumber::new(3.0, 1.0, 0.0);

const CONST_BUDGET_DEFAULT: ResourceBudget = ResourceBudget::DEFAULT;
const CONST_BUDGET_HPC: ResourceBudget = ResourceBudget::hpc();
const CONST_BUDGET_EMBEDDED: ResourceBudget = ResourceBudget::embedded();
const CONST_BUDGET_STATUS: BudgetStatus = CONST_BUDGET_DEFAULT.check_egraph_nodes(10_000);

const CONST_SIG_PGA: CliffordSignature = CliffordSignature::PGA3D;
const CONST_SIG_STA: CliffordSignature = CliffordSignature::spacetime();
const CONST_SIG_CGA: CliffordSignature = CliffordSignature::conformal();
const CONST_SIG_DIM: usize = CONST_SIG_STA.dim();
const CONST_METRIC_0: i64 = CONST_SIG_STA.metric_diag(0);
const CONST_METRIC_1: i64 = CONST_SIG_STA.metric_diag(1);

const CONST_BLADE_E1: BladeMask = BladeMask::E1;
const CONST_BLADE_E2: BladeMask = BladeMask::E2;
const CONST_BLADE_MUL: (BladeMask, i64) = CONST_BLADE_E1.multiply(&CONST_BLADE_E2, &CONST_SIG_PGA);

const CONST_Q_OP_ANNIHILATE: QuantumOperator = QuantumOperator::annihilation(CONST_SYMBOL_ID);
const CONST_Q_OP_CREATE: QuantumOperator = QuantumOperator::creation(CONST_SYMBOL_ID);

const CONST_DOMAIN_REAL: Domain = Domain::Reals;
const CONST_DOMAIN_CAT: DomainCategory = CONST_DOMAIN_REAL.category();
const CONST_IS_FIELD: bool = CONST_DOMAIN_CAT.is_field();
const CONST_IS_RING: bool = CONST_DOMAIN_CAT.is_ring();
const CONST_IS_SCALAR: bool = CONST_DOMAIN_REAL.is_scalar();
const CONST_IS_COMMUTATIVE: bool = CONST_DOMAIN_REAL.is_commutative_mul();

#[test]
fn test_const_identifiers() {
    assert_eq!(CONST_EXPR_ID.index(), 42);
    assert_eq!(CONST_EXPR_ID.as_usize(), 42);
    assert_eq!(CONST_DOMAIN_ID.index(), 7);
    assert_eq!(CONST_SYMBOL_ID.index(), 101);
}

#[test]
fn test_const_numbers() {
    assert_eq!(CONST_NUM_INT, Number::Integer(42));
    assert_eq!(CONST_NUM_RAT, Number::Rational(3, 4));
    assert_eq!(CONST_NUM_FLOAT.as_f64(), Some(std::f64::consts::PI));
    assert!(CONST_NUM_ZERO.is_zero());
    assert!(!CONST_NUM_ZERO.is_one());
    assert!(CONST_NUM_ONE.is_one());
    assert!(!CONST_NUM_ONE.is_zero());
    assert_eq!(CONST_NUM_PI.as_f64(), Some(std::f64::consts::PI));
    assert_eq!(CONST_NUM_E.as_f64(), Some(std::f64::consts::E));
    assert_eq!(CONST_NUM_INF.as_f64(), Some(f64::INFINITY));
    assert_eq!(Constant::EulerGamma.approx_f64(), Some(0.5772156649015329));
}

#[test]
fn test_const_intervals() {
    assert_eq!(CONST_INTERVAL_UNIT.inf, 0.0);
    assert_eq!(CONST_INTERVAL_UNIT.sup, 1.0);
    assert_eq!(CONST_INTERVAL_CUSTOM.inf, -5.0);
    assert_eq!(CONST_INTERVAL_CUSTOM.sup, 5.0);
    assert_eq!(CONST_INTERVAL_CUSTOM.mid(), 0.0);
    assert_eq!(CONST_INTERVAL_CUSTOM.diam(), 10.0);
    assert_eq!(CONST_INTERVAL_CUSTOM.rad(), 5.0);
    assert!(CONST_INTERVAL_CUSTOM.contains(0.0));
    assert!(CONST_INTERVAL_CUSTOM.contains(5.0));
    assert!(!CONST_INTERVAL_CUSTOM.contains(6.0));
    assert_eq!(CONST_INTERVAL_ADD, RealInterval::new(-2.0, 8.0));
    assert_eq!(CONST_INTERVAL_SUB, RealInterval::new(-8.0, 2.0));
    assert_eq!(CONST_INTERVAL_RECIP, Some(RealInterval::new(0.25, 0.5)));
}

#[test]
fn test_const_dual_and_hyperreal() {
    assert_eq!(CONST_DUAL_ZERO.real, 0.0);
    assert_eq!(CONST_DUAL_ZERO.dual, 0.0);
    assert_eq!(CONST_DUAL_ONE.real, 1.0);
    assert_eq!(CONST_DUAL_ONE.dual, 0.0);
    assert_eq!(CONST_DUAL_EPS.real, 0.0);
    assert_eq!(CONST_DUAL_EPS.dual, 1.0);
    assert_eq!(CONST_DUAL_VAR.real, 5.0);
    assert_eq!(CONST_DUAL_VAR.dual, 1.0);
    assert_eq!(CONST_DUAL_CONST.real, 10.0);
    assert_eq!(CONST_DUAL_CONST.dual, 0.0);
    assert_eq!(CONST_DUAL_ADD.real, 15.0);
    assert_eq!(CONST_DUAL_ADD.dual, 1.0);
    assert_eq!(CONST_DUAL_MUL.real, 50.0);
    assert_eq!(CONST_DUAL_MUL.dual, 10.0);

    assert_eq!(CONST_HYPERREAL.standard_part(), 3.0);
    assert_eq!(CONST_HYPERREAL.infinitesimal, 1.0);
}

#[test]
fn test_const_budget_and_config() {
    assert_eq!(CONST_BUDGET_DEFAULT.egraph_nodes, (8_000, 25_000));
    assert_eq!(CONST_BUDGET_HPC.max_memory_mb, 32_768);
    assert_eq!(CONST_BUDGET_EMBEDDED.max_memory_mb, 64);
    assert!(matches!(
        CONST_BUDGET_STATUS,
        BudgetStatus::SoftCapExceeded { .. }
    ));
}

#[test]
fn test_const_clifford_and_quantum() {
    assert_eq!(CONST_SIG_DIM, 4);
    assert_eq!(CONST_SIG_CGA.dim(), 5);
    assert_eq!(CONST_METRIC_0, 1);
    assert_eq!(CONST_METRIC_1, -1);
    assert_eq!(CONST_BLADE_E1.grade(), 1);
    assert_eq!(CONST_BLADE_E2.grade(), 1);
    assert_eq!(CONST_BLADE_MUL.0, BladeMask(3)); // e1 ^ e2 = bitmask 0b11 = 3
    assert_eq!(CONST_BLADE_MUL.1, 1); // positive orientation

    assert_eq!(CONST_Q_OP_ANNIHILATE.mode, CONST_SYMBOL_ID);
    assert_eq!(CONST_Q_OP_CREATE.mode, CONST_SYMBOL_ID);
}

#[test]
fn test_const_domains() {
    assert_eq!(CONST_DOMAIN_CAT, DomainCategory::Field);
    const {
        assert!(CONST_IS_FIELD);
        assert!(CONST_IS_RING);
        assert!(CONST_IS_SCALAR);
        assert!(CONST_IS_COMMUTATIVE);
    }
}
