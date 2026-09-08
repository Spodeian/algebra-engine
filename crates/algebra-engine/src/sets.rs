//! # `algebra-sets`
//!
//! ZF/ZFC set theory, NBG classes, infinite sets, and set operations for URAE.

use algebra_core::ExprId;

/// Standard well-known infinite mathematical sets / fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfiniteSetKind {
    /// Natural numbers ℕ = {0, 1, 2, ...}
    Naturals,
    /// Integers ℤ
    Integers,
    /// Rational numbers ℚ
    Rationals,
    /// Real numbers ℝ
    Reals,
    /// Complex numbers ℂ
    Complex,
    /// Surreal numbers №
    Surreals,
    /// P-adic numbers ℚ_p
    PAdic(u32),
    /// Ring of Adeles 𝔸
    Adeles,
}

/// NBG Set vs Proper Class distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassType {
    /// Standard small set (element of other sets).
    Set,
    /// Proper class (too large to be a set, e.g., the Class of all Sets V).
    ProperClass,
}

/// Structural representation of sets in ZF/ZFC and NBG set theory.
#[derive(Debug, Clone, PartialEq)]
pub enum MathSetRepresentation {
    /// Empty set ∅
    Empty,

    /// Finite explicit set {x_1, x_2, ..., x_n}
    Finite(Vec<ExprId>),

    /// Continuous interval (a, b), [a, b], [a, b), or (a, b]
    Interval {
        lower: ExprId,
        upper: ExprId,
        inclusive_lower: bool,
        inclusive_upper: bool,
    },

    /// Well-known infinite set or proper class
    Infinite(InfiniteSetKind),

    /// Set comprehension { x ∈ S | P(x) }
    Comprehension {
        var: ExprId,
        domain: Box<MathSetRepresentation>,
        predicate_expr: ExprId,
    },

    /// Set Union S_1 ∪ S_2 ∪ ... ∪ S_n
    Union(Vec<MathSetRepresentation>),

    /// Set Intersection S_1 ∩ S_2 ∩ ... ∩ S_n
    Intersection(Vec<MathSetRepresentation>),

    /// Set Difference S_1 \ S_2
    Difference(Box<MathSetRepresentation>, Box<MathSetRepresentation>),

    /// Power set P(S)
    PowerSet(Box<MathSetRepresentation>),
}

impl MathSetRepresentation {
    /// Returns the class type (Set or Proper Class) of this structure.
    pub fn class_type(&self) -> ClassType {
        match self {
            Self::Infinite(InfiniteSetKind::Surreals) => ClassType::ProperClass,
            _ => ClassType::Set,
        }
    }

    /// Check if set is empty ∅.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Calculate symbolic union of two sets.
    pub fn union(self, other: MathSetRepresentation) -> MathSetRepresentation {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        if self == other {
            return self;
        }
        MathSetRepresentation::Union(vec![self, other])
    }

    /// Calculate symbolic intersection of two sets.
    pub fn intersection(self, other: MathSetRepresentation) -> MathSetRepresentation {
        if self.is_empty() || other.is_empty() {
            return MathSetRepresentation::Empty;
        }
        if self == other {
            return self;
        }
        MathSetRepresentation::Intersection(vec![self, other])
    }
}

/// Lazy iterator over discrete integer set elements.
pub struct IntSetIterator {
    current: i64,
    end: Option<i64>,
}

impl IntSetIterator {
    /// Create a bounded or infinite integer lazy iterator.
    pub fn new(start: i64, end: Option<i64>) -> Self {
        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for IntSetIterator {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(limit) = self.end
            && self.current > limit
        {
            return None;
        }
        let val = self.current;
        self.current += 1;
        Some(val)
    }
}
