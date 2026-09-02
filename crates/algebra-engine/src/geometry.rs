//! # `algebra-geometry`
//!
//! Algebraic Geometry: Arbitrarily Defined Curvilinear Coordinate Systems,
//! Differential Scale Factors, Vector Calculus Operators (Gradient, Divergence, Laplacian),
//! Sylvester Resultant matrices, Discriminants, Projective Varieties, 2D/3D CAD Primitives,
//! and Constructive Solid Geometry (CSG).

use crate::calculus::SymbolicCalculus;
use crate::matrix::SymbolicMatrix;
use algebra_core::{AlgebraResult, ExprGraph, ExprId, SymbolId};

/// 2D Point primitive $P = (x, y)$.
#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// 2D Circle primitive with center $C = (x, y)$ and radius $r$.
#[derive(Debug, Clone, PartialEq)]
pub struct Circle2D {
    pub center: Point2D,
    pub radius: f64,
}

impl Circle2D {
    pub fn new(x: f64, y: f64, radius: f64) -> Self {
        Self {
            center: Point2D { x, y },
            radius,
        }
    }

    /// Check if point is inside or on boundary of circle.
    pub fn contains(&self, p: &Point2D) -> bool {
        let dx = p.x - self.center.x;
        let dy = p.y - self.center.y;
        (dx * dx + dy * dy) <= self.radius * self.radius
    }
}

/// Constructive Solid Geometry (CSG) Boolean Node.
#[derive(Debug, Clone, PartialEq)]
pub enum CsgNode {
    Circle(Circle2D),
    Union(Box<CsgNode>, Box<CsgNode>),
    Intersection(Box<CsgNode>, Box<CsgNode>),
    Difference(Box<CsgNode>, Box<CsgNode>),
}

impl CsgNode {
    pub fn contains(&self, p: &Point2D) -> bool {
        match self {
            CsgNode::Circle(c) => c.contains(p),
            CsgNode::Union(a, b) => a.contains(p) || b.contains(p),
            CsgNode::Intersection(a, b) => a.contains(p) && b.contains(p),
            CsgNode::Difference(a, b) => a.contains(p) && !b.contains(p),
        }
    }
}

/// Specification for an arbitrarily defined curvilinear coordinate system in $N$ dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArbitraryCurvilinearSystem {
    pub name: String,
    pub coord_names: Vec<String>,
    pub scale_factors: Vec<ExprId>,
    pub forward_transforms: Vec<ExprId>,
}

impl ArbitraryCurvilinearSystem {
    pub fn new(
        name: impl Into<String>,
        coord_names: Vec<String>,
        scale_factors: Vec<ExprId>,
        forward_transforms: Vec<ExprId>,
    ) -> Self {
        Self {
            name: name.into(),
            coord_names,
            scale_factors,
            forward_transforms,
        }
    }

    /// Create standard 3D Spherical Coordinate System.
    pub fn spherical_3d(graph: &ExprGraph) -> Self {
        let r = graph.symbol("r");
        let theta = graph.symbol("theta");
        let phi = graph.symbol("phi");

        let h1 = graph.integer(1);
        let h2 = r;
        let sin_theta = graph.function("sin", [theta]);
        let h3 = graph.mul([r, sin_theta]);

        let (x, y, z) = CoordinateTransform::spherical_to_cartesian(graph, r, theta, phi);

        Self::new(
            "Spherical3D",
            vec!["r".to_string(), "theta".to_string(), "phi".to_string()],
            vec![h1, h2, h3],
            vec![x, y, z],
        )
    }

    /// Create standard 3D Cylindrical Coordinate System.
    pub fn cylindrical_3d(graph: &ExprGraph) -> Self {
        let r = graph.symbol("r");
        let theta = graph.symbol("theta");
        let z = graph.symbol("z");

        let h1 = graph.integer(1);
        let h2 = r;
        let h3 = graph.integer(1);

        let (x, y, z_out) = CoordinateTransform::cartesian_to_cylindrical(graph, r, theta, z);

        Self::new(
            "Cylindrical3D",
            vec!["r".to_string(), "theta".to_string(), "z".to_string()],
            vec![h1, h2, h3],
            vec![x, y, z_out],
        )
    }
}

/// Supported coordinate systems for 2D and 3D spatial domains.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoordinateSystem {
    Cartesian,
    Polar,
    Cylindrical,
    Spherical,
    Arbitrary(ArbitraryCurvilinearSystem),
}

/// Coordinate System Transformations, Scale Factors, and Differential Operators.
pub struct CoordinateTransform;

impl CoordinateTransform {
    pub fn cartesian_to_polar(graph: &ExprGraph, x: ExprId, y: ExprId) -> (ExprId, ExprId) {
        let two = graph.integer(2);
        let x_sq = graph.pow(x, two);
        let y_sq = graph.pow(y, two);
        let sum_sq = graph.add([x_sq, y_sq]);
        let half = graph.rational(1, 2);
        let r = graph.pow(sum_sq, half);
        let theta = graph.function("atan2", [y, x]);
        (r, theta)
    }

    pub fn polar_to_cartesian(graph: &ExprGraph, r: ExprId, theta: ExprId) -> (ExprId, ExprId) {
        let cos_t = graph.function("cos", [theta]);
        let sin_t = graph.function("sin", [theta]);
        let x = graph.mul([r, cos_t]);
        let y = graph.mul([r, sin_t]);
        (x, y)
    }

    pub fn cartesian_to_spherical(
        graph: &ExprGraph,
        x: ExprId,
        y: ExprId,
        z: ExprId,
    ) -> (ExprId, ExprId, ExprId) {
        let two = graph.integer(2);
        let x_sq = graph.pow(x, two);
        let y_sq = graph.pow(y, two);
        let z_sq = graph.pow(z, two);
        let sum_sq = graph.add([x_sq, y_sq, z_sq]);
        let half = graph.rational(1, 2);
        let r = graph.pow(sum_sq, half);

        let theta = graph.function("atan2", [y, x]);
        let z_r = graph.div(z, r);
        let phi = graph.function("acos", [z_r]);
        (r, theta, phi)
    }

    pub fn spherical_to_cartesian(
        graph: &ExprGraph,
        r: ExprId,
        theta: ExprId,
        phi: ExprId,
    ) -> (ExprId, ExprId, ExprId) {
        let sin_p = graph.function("sin", [phi]);
        let cos_p = graph.function("cos", [phi]);
        let cos_t = graph.function("cos", [theta]);
        let sin_t = graph.function("sin", [theta]);

        let x = graph.mul([r, sin_p, cos_t]);
        let y = graph.mul([r, sin_p, sin_t]);
        let z = graph.mul([r, cos_p]);
        (x, y, z)
    }

    pub fn cartesian_to_cylindrical(
        graph: &ExprGraph,
        x: ExprId,
        y: ExprId,
        z: ExprId,
    ) -> (ExprId, ExprId, ExprId) {
        let (r, theta) = Self::cartesian_to_polar(graph, x, y);
        (r, theta, z)
    }

    pub fn gradient(
        graph: &ExprGraph,
        sys: &ArbitraryCurvilinearSystem,
        scalar_field: ExprId,
        coords: &[SymbolId],
    ) -> Vec<ExprId> {
        let mut components = Vec::with_capacity(coords.len());

        for (i, &q_i) in coords.iter().enumerate() {
            let df_dq = graph.diff(scalar_field, q_i);
            let h_i = if i < sys.scale_factors.len() {
                sys.scale_factors[i]
            } else {
                graph.integer(1)
            };
            components.push(graph.div(df_dq, h_i));
        }

        components
    }

    pub fn divergence(
        graph: &ExprGraph,
        sys: &ArbitraryCurvilinearSystem,
        vector_field: &[ExprId],
        coords: &[SymbolId],
    ) -> ExprId {
        let j_jacobian = graph.mul(sys.scale_factors.clone());
        let mut div_terms = Vec::with_capacity(coords.len());

        for (i, &q_i) in coords.iter().enumerate() {
            if i < vector_field.len() && i < sys.scale_factors.len() {
                let a_i = vector_field[i];
                let h_i = sys.scale_factors[i];
                let j_hi = graph.div(j_jacobian, h_i);
                let term = graph.mul([a_i, j_hi]);
                let d_term = graph.diff(term, q_i);
                div_terms.push(d_term);
            }
        }

        let sum_d = graph.add(div_terms);
        graph.div(sum_d, j_jacobian)
    }

    pub fn laplacian(
        graph: &ExprGraph,
        sys: &ArbitraryCurvilinearSystem,
        scalar_field: ExprId,
        coords: &[SymbolId],
    ) -> ExprId {
        let grad = Self::gradient(graph, sys, scalar_field, coords);
        Self::divergence(graph, sys, &grad, coords)
    }
}

/// Polynomial elimination and resultant utilities.
pub struct ResultantBuilder;

impl ResultantBuilder {
    pub fn sylvester_matrix_2_1(
        graph: &ExprGraph,
        p_coeffs: [ExprId; 3],
        q_coeffs: [ExprId; 2],
    ) -> AlgebraResult<SymbolicMatrix> {
        let [a, b, c] = p_coeffs;
        let [d, e] = q_coeffs;
        let zero = graph.integer(0);

        SymbolicMatrix::new(3, 3, vec![a, b, c, d, e, zero, zero, d, e])
    }

    pub fn quadratic_discriminant(graph: &ExprGraph, a: ExprId, b: ExprId, c: ExprId) -> ExprId {
        let two = graph.integer(2);
        let b_sq = graph.pow(b, two);
        let four = graph.integer(4);
        let ac4 = graph.mul([four, a, c]);
        graph.sub(b_sq, ac4)
    }
}
