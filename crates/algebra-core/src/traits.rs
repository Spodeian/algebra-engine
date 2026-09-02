//! # `algebra_core::traits`
//!
//! Formal Abstract Algebra trait hierarchy mapping axiomatic algebraic structures to Rust traits.
//!
//! ## Mathematical Foundations
//! An algebraic structure is defined by an underlying set $S$, operations $\oplus, \otimes, \dots$,
//! and equational axioms. This trait hierarchy provides the axiomatic basis for rule generation
//! and category-theoretic morphisms across URAE.
//!
//! - **Magma**: Set with closed binary operation $(S, \oplus)$.
//! - **Semigroup**: Associative magma $(a \oplus b) \oplus c = a \oplus (b \oplus c)$.
//! - **Monoid**: Semigroup with identity $a \oplus e = e \oplus a = a$.
//! - **Group**: Monoid with inverse $a \oplus a^{-1} = e$.
//! - **Abelian Group**: Group with commutative operation $a \oplus b = b \oplus a$.
//! - **Ring**: $(R, +)$ is an Abelian Group, $(R, \cdot)$ is a Monoid, and $\cdot$ distributes over $+$.
//! - **Commutative Ring**: Ring where multiplication is commutative $a \cdot b = b \cdot a$.
//! - **Field**: Commutative ring where every non-zero element has a multiplicative inverse $a^{-1}$.
//! - **Differential Ring**: Ring equipped with a derivation $D: R \to R$ satisfying the Leibniz rule $D(ab) = D(a)b + aD(b)$.
//! - **Lie Algebra**: Vector space equipped with an alternating bilinear Lie bracket $[A, B]$ satisfying the Jacobi identity.
//! - **Tropical Semiring**: Idempotent semiring $(S, \oplus, \otimes)$ where $a \oplus a = a$.
//! - **Heyting Algebra**: Bounded lattice $(H, \wedge, \lor, \to, \bot, \top)$ for intuitionistic logic.

use crate::error::AlgebraResult;

/// Base trait for mathematical set concepts with membership and identities.
pub trait MathSet {
    /// Associated element type of the set.
    type Element;

    /// Returns `true` if element `x` is contained in this mathematical set.
    fn contains(&self, x: &Self::Element) -> bool;

    /// Additive identity zero element $0$.
    fn zero(&self) -> Self::Element;

    /// Multiplicative identity element $1$.
    fn one(&self) -> Self::Element;
}

/// Algebraic structure equipped with a closed binary operation `op(a, b)`.
pub trait Magma: MathSet {
    /// Apply binary operation: $a \oplus b$.
    fn op(&self, a: &Self::Element, b: &Self::Element) -> AlgebraResult<Self::Element>;
}

/// Magma whose binary operation is associative: $(a \oplus b) \oplus c = a \oplus (b \oplus c)$.
pub trait Semigroup: Magma {}

/// Semigroup with an identity element: $a \oplus e = e \oplus a = a$.
pub trait Monoid: Semigroup {
    /// Identity element of the monoid structure.
    fn identity(&self) -> Self::Element;
}

/// Monoid with inverses for every element: $a \oplus a^{-1} = e$.
pub trait Group: Monoid {
    /// Calculate the inverse element $a^{-1}$.
    fn inverse(&self, element: &Self::Element) -> AlgebraResult<Self::Element>;
}

/// Group whose operation is commutative: $a \oplus b = b \oplus a$.
pub trait AbelianGroup: Group {}

/// Ring structure with addition (Abelian Group) and multiplication (Monoid), distributive.
pub trait Ring: AbelianGroup {
    /// Additive addition: $a + b$.
    fn add(&self, a: &Self::Element, b: &Self::Element) -> AlgebraResult<Self::Element> {
        self.op(a, b)
    }

    /// Multiplicative product: $a \cdot b$.
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> AlgebraResult<Self::Element>;

    /// Additive inverse (negation): $-a$.
    fn neg(&self, a: &Self::Element) -> AlgebraResult<Self::Element> {
        self.inverse(a)
    }
}

/// Ring where multiplication is commutative: $a \cdot b = b \cdot a$.
pub trait CommutativeRing: Ring {}

/// Commutative Ring where every non-zero element has a multiplicative inverse.
pub trait Field: CommutativeRing {
    /// Multiplicative inverse $a^{-1} = 1/a$.
    fn recip(&self, a: &Self::Element) -> AlgebraResult<Self::Element>;

    /// Division $a / b = a \cdot b^{-1}$.
    fn div(&self, a: &Self::Element, b: &Self::Element) -> AlgebraResult<Self::Element> {
        let inv_b = self.recip(b)?;
        self.mul(a, &inv_b)
    }
}

/// Differential Ring equipped with a derivation operator $D: R \to R$ satisfying:
/// 1. Linearity: $D(a + b) = D(a) + D(b)$
/// 2. Leibniz Product Rule: $D(a \cdot b) = D(a) \cdot b + a \cdot D(b)$
pub trait DifferentialRing: Ring {
    /// Apply derivative operator $D(a)$.
    fn derivative(&self, a: &Self::Element) -> AlgebraResult<Self::Element>;
}

/// Vector space over a base field $F$.
pub trait VectorSpace<F: Field>: MathSet {
    /// Associated vector type.
    type Vector;

    /// Vector addition: $\mathbf{v}_1 + \mathbf{v}_2$.
    fn vector_add(&self, v1: &Self::Vector, v2: &Self::Vector) -> AlgebraResult<Self::Vector>;

    /// Scalar multiplication: $c \cdot \mathbf{v}$.
    fn scalar_mul(&self, scalar: &F::Element, v: &Self::Vector) -> AlgebraResult<Self::Vector>;
}

/// Lie Algebra equipped with alternating bilinear bracket $[\cdot, \cdot]$ satisfying the Jacobi identity:
/// $$[[A, B], C] + [[B, C], A] + [[C, A], B] = 0$$
pub trait LieAlgebra<F: Field>: VectorSpace<F> {
    /// Compute the Lie bracket $[A, B]$.
    fn lie_bracket(&self, a: &Self::Vector, b: &Self::Vector) -> AlgebraResult<Self::Vector>;
}

/// Tropical / Idempotent Semiring where addition is idempotent: $a \oplus a = a$.
pub trait TropicalSemiring: MathSet {
    /// Tropical addition (e.g. $\max(a, b)$ or $\min(a, b)$).
    fn tropical_add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Tropical multiplication (e.g. standard addition $a + b$).
    fn tropical_mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;
}

/// Heyting Algebra $(H, \wedge, \lor, \to, \bot, \top)$ governing intuitionistic logic and pseudocomplements.
pub trait HeytingAlgebra: MathSet {
    /// Meet (Conjunction) $a \wedge b$.
    fn meet(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Join (Disjunction) $a \lor b$.
    fn join(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Relative Pseudocomplement (Implication) $a \to b$.
    fn implies(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Pseudocomplement (Negation) $\neg a = a \to \bot$.
    fn not(&self, a: &Self::Element) -> Self::Element {
        self.implies(a, &self.zero())
    }
}

/// Tensor algebra structure over a field.
pub trait TensorAlgebra<F: Field>: VectorSpace<F> {
    /// Tensor contraction / product.
    fn tensor_product(&self, t1: &Self::Vector, t2: &Self::Vector) -> AlgebraResult<Self::Vector>;
}

/// Semiring / Rig structure: $(S, \oplus, \otimes, 0, 1)$ where:
/// - $(S, \oplus, 0)$ is a Commutative Monoid
/// - $(S, \otimes, 1)$ is a Monoid
/// - Multiplication distributes over addition: $a \otimes (b \oplus c) = (a \otimes b) \oplus (a \otimes c)$
/// - Annihilation by zero: $0 \otimes a = a \otimes 0 = 0$
pub trait Semiring: MathSet {
    /// Semiring addition: $a \oplus b$.
    fn add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Semiring multiplication: $a \otimes b$.
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;
}

/// Near-Ring structure: $(N, +, \cdot)$ where:
/// - $(N, +)$ is a Group (not necessarily abelian)
/// - $(N, \cdot)$ is a Semigroup
/// - Only one distributive law is satisfied (e.g. right-distributive: $(a + b) \cdot c = a \cdot c + b \cdot c$).
pub trait NearRing: MathSet {
    /// Near-ring addition: $a + b$.
    fn add(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;

    /// Near-ring additive inverse: $-a$.
    fn neg(&self, a: &Self::Element) -> Self::Element;

    /// Near-ring multiplication: $a \cdot b$.
    fn mul(&self, a: &Self::Element, b: &Self::Element) -> Self::Element;
}

/// Jordan Algebra: Commutative, non-associative algebra over field $F$ satisfying:
/// 1. Commutativity: $x \circ y = y \circ x$
/// 2. Jordan Identity: $(x \circ y) \circ x^2 = x \circ (y \circ x^2)$
pub trait JordanAlgebra<F: Field>: VectorSpace<F> {
    /// Jordan product: $x \circ y$.
    fn jordan_product(&self, x: &Self::Vector, y: &Self::Vector) -> AlgebraResult<Self::Vector>;
}

/// Alternative Algebra: Non-associative algebra satisfying Artin's Alternativity:
/// 1. Left alternative: $(x \cdot x) \cdot y = x \cdot (x \cdot y)$
/// 2. Right alternative: $(y \cdot x) \cdot x = y \cdot (x \cdot x)$
pub trait AlternativeAlgebra<F: Field>: VectorSpace<F> {
    /// Alternative product: $x \cdot y$.
    fn alt_mul(&self, x: &Self::Vector, y: &Self::Vector) -> AlgebraResult<Self::Vector>;
}

/// Hopf Algebra: Bialgebra $(H, \nabla, \eta, \Delta, \epsilon)$ equipped with an Antipode map $S: H \to H$
/// satisfying the antipode identity:
/// $$\nabla \circ (S \otimes \operatorname{id}) \circ \Delta = \eta \circ \epsilon$$
pub trait HopfAlgebra<F: Field>: VectorSpace<F> {
    /// Multiplication (Product) $\nabla: H \otimes H \to H$.
    fn product(&self, a: &Self::Vector, b: &Self::Vector) -> AlgebraResult<Self::Vector>;

    /// Coproduct $\Delta: H \to H \otimes H$ expressed as Sweedler decomposition terms.
    fn coproduct(&self, a: &Self::Vector) -> AlgebraResult<Vec<(Self::Vector, Self::Vector)>>;

    /// Counit $\epsilon: H \to F$.
    fn counit(&self, a: &Self::Vector) -> AlgebraResult<F::Element>;

    /// Antipode map $S: H \to H$.
    fn antipode(&self, a: &Self::Vector) -> AlgebraResult<Self::Vector>;
}
