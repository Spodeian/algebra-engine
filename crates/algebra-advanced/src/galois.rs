//! # `algebra_advanced::galois`
//!
//! Galois Theory & Algebraic Field Extensions:
//! - Field extensions $K / F$
//! - Minimal Polynomials
//! - Galois Group $\text{Gal}(K / \mathbb{Q})$ classification
//! - Solvability by Radicals (Abel-Ruffini theorem checks)

use serde::{Deserialize, Serialize};

/// Classification of Galois group structure for low-degree polynomials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GaloisGroupType {
    /// Trivial identity group
    Identity,
    /// Cyclic group $C_n$
    Cyclic(usize),
    /// Dihedral group $D_n$
    Dihedral(usize),
    /// Alternating group $A_n$
    Alternating(usize),
    /// Symmetric group $S_n$
    Symmetric(usize),
    /// Klein four-group $V_4 \cong C_2 \times C_2$
    KleinFour,
}

impl GaloisGroupType {
    /// Check if the Galois group is solvable (by radicals).
    /// All $S_n$ and $A_n$ for $n \ge 5$ are non-solvable.
    pub fn is_solvable(&self) -> bool {
        match self {
            GaloisGroupType::Identity => true,
            GaloisGroupType::Cyclic(_) => true,
            GaloisGroupType::Dihedral(_) => true,
            GaloisGroupType::KleinFour => true,
            GaloisGroupType::Alternating(n) => *n < 5,
            GaloisGroupType::Symmetric(n) => *n < 5,
        }
    }
}

/// Simple field extension $F(\alpha) / F$.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldExtension {
    pub base_field: String,
    pub generator_symbol: String,
    pub degree: usize,
    pub is_galois: bool,
    pub galois_group: Option<GaloisGroupType>,
}

impl FieldExtension {
    /// Create quadratic extension $\mathbb{Q}(\sqrt{d})$.
    pub fn quadratic(d: i64) -> Self {
        Self {
            base_field: "Q".to_string(),
            generator_symbol: format!("sqrt({})", d),
            degree: 2,
            is_galois: true,
            galois_group: Some(GaloisGroupType::Cyclic(2)),
        }
    }

    /// Check if roots of this polynomial extension are expressible in radicals.
    pub fn is_solvable_by_radicals(&self) -> bool {
        self.galois_group
            .as_ref()
            .map(|g| g.is_solvable())
            .unwrap_or(self.degree < 5)
    }
}
