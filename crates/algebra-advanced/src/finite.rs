//! # `algebra-finite`
//!
//! Finite Fields (Galois Fields GF(p^n)) and Modular Arithmetic (ℤ/nℤ).

use num_bigint::BigInt;
use num_traits::{One, Zero};

/// Modulo arithmetic integer element in ℤ/nℤ.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuloInt {
    /// Remainder value in range [0, modulus - 1].
    pub value: BigInt,
    /// Modulus n.
    pub modulus: BigInt,
}

impl ModuloInt {
    /// Create a new modulo integer carrying modulus safely.
    pub fn new(value: BigInt, modulus: BigInt) -> Self {
        let mut rem = value % &modulus;
        if rem < BigInt::zero() {
            rem += &modulus;
        }
        Self {
            value: rem,
            modulus,
        }
    }

    /// Add two modulo integers in same ℤ/nℤ.
    pub fn add(&self, other: &ModuloInt) -> Option<Self> {
        if self.modulus != other.modulus {
            return None;
        }
        Some(Self::new(&self.value + &other.value, self.modulus.clone()))
    }

    /// Multiply two modulo integers in same ℤ/nℤ.
    pub fn mul(&self, other: &ModuloInt) -> Option<Self> {
        if self.modulus != other.modulus {
            return None;
        }
        Some(Self::new(&self.value * &other.value, self.modulus.clone()))
    }

    /// Modular exponentiation (base^exp mod n) using fast binary exponentiation.
    pub fn pow(&self, exp: &BigInt) -> Self {
        let mut base = self.value.clone();
        let mut e = exp.clone();
        let mut result = BigInt::one();

        base %= &self.modulus;
        while e > BigInt::zero() {
            if &e % 2 == BigInt::one() {
                result = (result * &base) % &self.modulus;
            }
            e >>= 1;
            base = (&base * &base) % &self.modulus;
        }
        Self {
            value: result,
            modulus: self.modulus.clone(),
        }
    }
}

/// Galois Field GF(p^n) representation defined by irreducible modulus polynomial coefficients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaloisField {
    /// Characteristic prime p.
    pub p: u32,
    /// Field extension degree n.
    pub degree: u32,
    /// Irreducible polynomial coefficients (lowest to highest degree), e.g. x^2 + 1 -> [1, 0, 1].
    pub modulus_poly: Option<Vec<i64>>,
}

impl GaloisField {
    /// Create a new finite Galois field GF(p^n) without explicit polynomial modulus.
    pub fn new(p: u32, degree: u32) -> Self {
        Self {
            p,
            degree,
            modulus_poly: None,
        }
    }

    /// Create a new finite Galois field GF(p^n) with explicit irreducible polynomial modulus.
    pub fn with_modulus(p: u32, degree: u32, modulus_poly: Vec<i64>) -> Self {
        Self {
            p,
            degree,
            modulus_poly: Some(modulus_poly),
        }
    }

    /// Construct GF(p^2) from an inert Gaussian prime $p \equiv 3 \pmod 4$ with modulus $x^2 + 1$.
    pub fn from_gaussian_inert_prime(p: u32) -> Option<Self> {
        if p % 4 == 3 && algebra_engine::numbertheory::is_prime(p as u64) {
            Some(Self::with_modulus(p, 2, vec![1, 0, 1]))
        } else {
            None
        }
    }

    /// Construct GF(p^2) from an inert Eisenstein prime $p \equiv 2 \pmod 3$ with modulus $x^2 - x + 1$.
    pub fn from_eisenstein_inert_prime(p: u32) -> Option<Self> {
        if p % 3 == 2 && algebra_engine::numbertheory::is_prime(p as u64) {
            Some(Self::with_modulus(p, 2, vec![1, -1, 1]))
        } else {
            None
        }
    }

    /// Total order (cardinality) of the field p^n.
    pub fn cardinality(&self) -> u64 {
        (self.p as u64).pow(self.degree)
    }
}
