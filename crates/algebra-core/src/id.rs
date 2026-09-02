//! Type-safe identifier handles for Arena indexing and Symbol interning.

use std::fmt;

/// Lightweight 32-bit handle representing an immutable expression node in the [`ExprGraph`](crate::graph::ExprGraph).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct ExprId(pub u32);

impl ExprId {
    /// Create a new `ExprId` from a raw index.
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the raw `u32` index value.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Convert to `usize` for array indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for ExprId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// Lightweight 32-bit handle representing a mathematical domain specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct DomainId(pub u32);

impl DomainId {
    /// Create a new `DomainId` from a raw index.
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the raw `u32` index value.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Convert to `usize` for array indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for DomainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Domain#{}", self.0)
    }
}

/// Lightweight 32-bit handle representing an interned symbol string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct SymbolId(pub u32);

impl SymbolId {
    /// Create a new `SymbolId` from a raw index.
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Get the raw `u32` index value.
    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Convert to `usize` for array indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sym#{}", self.0)
    }
}
