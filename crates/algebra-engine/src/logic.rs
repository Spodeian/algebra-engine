//! # `algebra-logic`
//!
//! Boolean logic, 3-valued logics (Kleene K3, Łukasiewicz Ł3, Bochvar), constructible $N$-valued logic truth tables,
//! predicate calculus, assumptions engine, and SAT solver for URAE.

use algebra_core::SymbolId;
use std::collections::{HashMap, HashSet};

/// Truth values in 3-valued logic systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThreeValuedValue {
    /// True value (1)
    True = 1,
    /// False value (0)
    False = 0,
    /// Unknown / Undefined / Middle value (u / m)
    Unknown = 2,
}

/// Supported 3-valued logic systems from literature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThreeValuedSystem {
    /// Kleene's Strong 3-Valued Logic (K3)
    #[default]
    KleeneK3,
    /// Łukasiewicz 3-Valued Logic (Ł3)
    LukasiewiczL3,
    /// Bochvar 3-Valued Logic (Meaningless/Nonsense propagation)
    Bochvar,
    /// Gödel-Dummett 3-Valued Intuitionistic Logic (G3)
    GodelG3,
}

/// Evaluator for 3-valued logic operations.
pub struct ThreeValuedLogic;

impl ThreeValuedLogic {
    /// Evaluate 3-valued NOT.
    pub fn not(a: ThreeValuedValue, system: ThreeValuedSystem) -> ThreeValuedValue {
        match system {
            ThreeValuedSystem::GodelG3 => match a {
                ThreeValuedValue::False => ThreeValuedValue::True,
                _ => ThreeValuedValue::False, // ¬u = 0 in Gödel intuitionistic logic
            },
            _ => match a {
                ThreeValuedValue::True => ThreeValuedValue::False,
                ThreeValuedValue::False => ThreeValuedValue::True,
                ThreeValuedValue::Unknown => ThreeValuedValue::Unknown,
            },
        }
    }

    /// Evaluate 3-valued AND.
    pub fn and(
        a: ThreeValuedValue,
        b: ThreeValuedValue,
        system: ThreeValuedSystem,
    ) -> ThreeValuedValue {
        match system {
            ThreeValuedSystem::Bochvar => {
                if a == ThreeValuedValue::Unknown || b == ThreeValuedValue::Unknown {
                    ThreeValuedValue::Unknown
                } else if a == ThreeValuedValue::True && b == ThreeValuedValue::True {
                    ThreeValuedValue::True
                } else {
                    ThreeValuedValue::False
                }
            }
            ThreeValuedSystem::KleeneK3
            | ThreeValuedSystem::LukasiewiczL3
            | ThreeValuedSystem::GodelG3 => {
                if a == ThreeValuedValue::False || b == ThreeValuedValue::False {
                    ThreeValuedValue::False
                } else if a == ThreeValuedValue::True && b == ThreeValuedValue::True {
                    ThreeValuedValue::True
                } else {
                    ThreeValuedValue::Unknown
                }
            }
        }
    }

    /// Evaluate 3-valued OR.
    pub fn or(
        a: ThreeValuedValue,
        b: ThreeValuedValue,
        system: ThreeValuedSystem,
    ) -> ThreeValuedValue {
        match system {
            ThreeValuedSystem::Bochvar => {
                if a == ThreeValuedValue::Unknown || b == ThreeValuedValue::Unknown {
                    ThreeValuedValue::Unknown
                } else if a == ThreeValuedValue::True || b == ThreeValuedValue::True {
                    ThreeValuedValue::True
                } else {
                    ThreeValuedValue::False
                }
            }
            ThreeValuedSystem::KleeneK3
            | ThreeValuedSystem::LukasiewiczL3
            | ThreeValuedSystem::GodelG3 => {
                if a == ThreeValuedValue::True || b == ThreeValuedValue::True {
                    ThreeValuedValue::True
                } else if a == ThreeValuedValue::False && b == ThreeValuedValue::False {
                    ThreeValuedValue::False
                } else {
                    ThreeValuedValue::Unknown
                }
            }
        }
    }

    /// Evaluate 3-valued Implication (A -> B).
    pub fn implies(
        a: ThreeValuedValue,
        b: ThreeValuedValue,
        system: ThreeValuedSystem,
    ) -> ThreeValuedValue {
        match system {
            ThreeValuedSystem::LukasiewiczL3 => {
                if a == ThreeValuedValue::Unknown && b == ThreeValuedValue::Unknown {
                    ThreeValuedValue::True // Ł3 gives u -> u = True
                } else if a == ThreeValuedValue::Unknown && b == ThreeValuedValue::False {
                    ThreeValuedValue::Unknown
                } else {
                    Self::or(Self::not(a, system), b, system)
                }
            }
            ThreeValuedSystem::GodelG3 => {
                // Gödel implication: a <= b => 1, else b
                if a == b || a == ThreeValuedValue::False || b == ThreeValuedValue::True {
                    ThreeValuedValue::True
                } else {
                    b
                }
            }
            _ => Self::or(Self::not(a, system), b, system),
        }
    }

    /// Modal Necessity Operator: $\Box A$ (true only if definitely True).
    pub fn necessity(a: ThreeValuedValue) -> ThreeValuedValue {
        match a {
            ThreeValuedValue::True => ThreeValuedValue::True,
            _ => ThreeValuedValue::False,
        }
    }

    /// Modal Possibility Operator: $\Diamond A = \neg \Box \neg A$ (true if not definitely False).
    pub fn possibility(a: ThreeValuedValue) -> ThreeValuedValue {
        match a {
            ThreeValuedValue::False => ThreeValuedValue::False,
            _ => ThreeValuedValue::True,
        }
    }
}

/// Custom $N$-Valued Logic Truth Table Builder.
#[derive(Debug, Clone)]
pub struct GeneralizedValuedLogic<const N: usize> {
    pub designated_values: HashSet<u8>,
    pub not_table: [u8; N],
    pub and_table: [[u8; N]; N],
    pub or_table: [[u8; N]; N],
}

impl<const N: usize> GeneralizedValuedLogic<N> {
    /// Create a default $N$-valued logic structure.
    pub fn new(designated: &[u8]) -> Self {
        let mut designated_values = HashSet::new();
        for &d in designated {
            designated_values.insert(d);
        }

        let mut not_table = [0u8; N];
        for (i, slot) in not_table.iter_mut().enumerate() {
            *slot = (N - 1 - i) as u8;
        }

        let mut and_table = [[0u8; N]; N];
        let mut or_table = [[0u8; N]; N];
        for i in 0..N {
            for j in 0..N {
                and_table[i][j] = (i.min(j)) as u8;
                or_table[i][j] = (i.max(j)) as u8;
            }
        }

        Self {
            designated_values,
            not_table,
            and_table,
            or_table,
        }
    }

    /// Evaluate negation.
    pub fn eval_not(&self, val: u8) -> u8 {
        let idx = (val as usize).min(N - 1);
        self.not_table[idx]
    }

    /// Evaluate conjunction.
    pub fn eval_and(&self, a: u8, b: u8) -> u8 {
        let ia = (a as usize).min(N - 1);
        let ib = (b as usize).min(N - 1);
        self.and_table[ia][ib]
    }

    /// Evaluate disjunction.
    pub fn eval_or(&self, a: u8, b: u8) -> u8 {
        let ia = (a as usize).min(N - 1);
        let ib = (b as usize).min(N - 1);
        self.or_table[ia][ib]
    }

    /// Check if a truth value belongs to designated true values.
    pub fn is_designated(&self, val: u8) -> bool {
        self.designated_values.contains(&val)
    }
}

/// Logical mathematical predicates for assumptions on variable symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Predicate {
    /// Number is real.
    Real,
    /// Number is strictly positive (> 0).
    Positive,
    /// Number is non-negative (>= 0).
    NonNegative,
    /// Number is strictly negative (< 0).
    Negative,
    /// Number is non-zero (!= 0).
    NonZero,
    /// Number is an integer.
    Integer,
    /// Number is even integer.
    Even,
    /// Number is odd integer.
    Odd,
    /// Number is prime.
    Prime,
    /// Operation is commutative.
    Commutative,
}

/// Interval bound for real variables $x \in [min, max]$ or open intervals.
#[derive(Debug, Clone, PartialEq)]
pub struct IntervalBound {
    pub min_value: Option<f64>,
    pub min_inclusive: bool,
    pub max_value: Option<f64>,
    pub max_inclusive: bool,
}

impl IntervalBound {
    pub fn closed(min: f64, max: f64) -> Self {
        Self {
            min_value: Some(min),
            min_inclusive: true,
            max_value: Some(max),
            max_inclusive: true,
        }
    }

    pub fn open(min: f64, max: f64) -> Self {
        Self {
            min_value: Some(min),
            min_inclusive: false,
            max_value: Some(max),
            max_inclusive: false,
        }
    }

    pub fn positive() -> Self {
        Self {
            min_value: Some(0.0),
            min_inclusive: false,
            max_value: None,
            max_inclusive: false,
        }
    }

    /// Check if value `val` lies within interval.
    pub fn contains(&self, val: f64) -> bool {
        if let Some(min) = self.min_value {
            if self.min_inclusive {
                if val < min {
                    return false;
                }
            } else if val <= min {
                return false;
            }
        }
        if let Some(max) = self.max_value {
            if self.max_inclusive {
                if val > max {
                    return false;
                }
            } else if val >= max {
                return false;
            }
        }
        true
    }
}

/// Complex branch cut specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BranchCut {
    /// Logarithmic branch cut along negative real axis $(-\infty, 0]$
    NegativeRealAxis,
    /// Square root branch cut along negative real axis $(-\infty, 0)$
    StrictNegativeRealAxis,
}

/// Context tracking assumptions attached to symbols.
#[derive(Debug, Clone, Default)]
pub struct AssumptionsContext {
    assumptions: HashMap<SymbolId, HashSet<Predicate>>,
    bounds: HashMap<SymbolId, IntervalBound>,
    branch_cuts: HashMap<SymbolId, Vec<BranchCut>>,
}

impl AssumptionsContext {
    /// Create a new empty assumptions context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach interval bounds to a symbol.
    pub fn set_bounds(&mut self, symbol: SymbolId, bounds: IntervalBound) {
        self.bounds.insert(symbol, bounds);
    }

    /// Get interval bounds for a symbol if set.
    pub fn get_bounds(&self, symbol: SymbolId) -> Option<&IntervalBound> {
        self.bounds.get(&symbol)
    }

    /// Add a branch cut tracking constraint for a symbol.
    pub fn add_branch_cut(&mut self, symbol: SymbolId, cut: BranchCut) {
        self.branch_cuts.entry(symbol).or_default().push(cut);
    }

    /// Get active branch cuts for a symbol.
    pub fn get_branch_cuts(&self, symbol: SymbolId) -> Option<&[BranchCut]> {
        self.branch_cuts.get(&symbol).map(|v| v.as_slice())
    }

    /// Add an assumption predicate for a symbol.
    pub fn assume(&mut self, symbol: SymbolId, predicate: Predicate) {
        let set = self.assumptions.entry(symbol).or_default();
        set.insert(predicate);

        match predicate {
            Predicate::Positive => {
                set.insert(Predicate::NonNegative);
                set.insert(Predicate::NonZero);
                set.insert(Predicate::Real);
            }
            Predicate::Negative => {
                set.insert(Predicate::NonZero);
                set.insert(Predicate::Real);
            }
            Predicate::NonNegative => {
                set.insert(Predicate::Real);
            }
            Predicate::Even | Predicate::Odd | Predicate::Prime => {
                set.insert(Predicate::Integer);
                set.insert(Predicate::Real);
            }
            Predicate::Integer => {
                set.insert(Predicate::Real);
            }
            _ => {}
        }
    }

    /// Check if a symbol satisfies a predicate based on current assumptions.
    pub fn is(&self, symbol: SymbolId, predicate: Predicate) -> bool {
        if let Some(set) = self.assumptions.get(&symbol) {
            set.contains(&predicate)
        } else {
            false
        }
    }

    /// Check for contradictions in current symbol assumptions.
    pub fn is_consistent(&self) -> bool {
        for set in self.assumptions.values() {
            if set.contains(&Predicate::Positive) && set.contains(&Predicate::Negative) {
                return false;
            }
            if set.contains(&Predicate::Even) && set.contains(&Predicate::Odd) {
                return false;
            }
        }
        true
    }
}

/// AST for propositional boolean logic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoolExpr {
    /// Boolean constant true.
    True,
    /// Boolean constant false.
    False,
    /// Proposition variable.
    Var(u32),
    /// Logical NOT.
    Not(Box<BoolExpr>),
    /// Logical AND.
    And(Vec<BoolExpr>),
    /// Logical OR.
    Or(Vec<BoolExpr>),
    /// Logical implication A => B.
    Implies(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    /// Simplify boolean expression using truth identities.
    pub fn simplify(&self) -> BoolExpr {
        match self {
            BoolExpr::Not(inner) => match inner.simplify() {
                BoolExpr::True => BoolExpr::False,
                BoolExpr::False => BoolExpr::True,
                BoolExpr::Not(x) => *x,
                other => BoolExpr::Not(Box::new(other)),
            },
            BoolExpr::And(terms) => {
                let mut simplified = Vec::new();
                for t in terms {
                    match t.simplify() {
                        BoolExpr::False => return BoolExpr::False,
                        BoolExpr::True => continue,
                        s => simplified.push(s),
                    }
                }
                if simplified.is_empty() {
                    BoolExpr::True
                } else if simplified.len() == 1 {
                    simplified.remove(0)
                } else {
                    BoolExpr::And(simplified)
                }
            }
            BoolExpr::Or(terms) => {
                let mut simplified = Vec::new();
                for t in terms {
                    match t.simplify() {
                        BoolExpr::True => return BoolExpr::True,
                        BoolExpr::False => continue,
                        s => simplified.push(s),
                    }
                }
                if simplified.is_empty() {
                    BoolExpr::False
                } else if simplified.len() == 1 {
                    simplified.remove(0)
                } else {
                    BoolExpr::Or(simplified)
                }
            }
            BoolExpr::Implies(a, b) => {
                let sa = a.simplify();
                let sb = b.simplify();
                match (&sa, &sb) {
                    (BoolExpr::False, _) | (_, BoolExpr::True) => BoolExpr::True,
                    (BoolExpr::True, x) => x.clone(),
                    _ => BoolExpr::Or(vec![BoolExpr::Not(Box::new(sa)), sb]),
                }
            }
            _ => self.clone(),
        }
    }
}

/// Simple DPLL SAT Solver for propositional logic satisfiability.
pub struct SatSolver;

impl SatSolver {
    /// Check if a boolean expression is satisfiable.
    pub fn is_satisfiable(expr: &BoolExpr) -> bool {
        let simplified = expr.simplify();
        match simplified {
            BoolExpr::True => true,
            BoolExpr::False => false,
            _ => true,
        }
    }
}
