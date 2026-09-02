//! # `urae_notebook::examples`
//!
//! In-App Interactive Example Notebook & Tutorial Registry.
//!
//! Provides 12+ pre-built, production-ready mathematical notebooks spanning all
//! domains of the Universal Rust Algebra Engine (URAE).

use serde::{Deserialize, Serialize};

/// High-level mathematical category for example notebooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExampleCategory {
    CalculusAndOde,
    QuantumAndPhysics,
    CadAndSimulation,
    GeometryAndTopology,
    ExceptionalAndAbstract,
    DModulesAndFormal,
    MultiphysicsAndAi,
}

impl ExampleCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CalculusAndOde => "Calculus & Differential Equations",
            Self::QuantumAndPhysics => "Quantum Field Theory & Physics",
            Self::CadAndSimulation => "Parametric CAD & IGA Simulation",
            Self::GeometryAndTopology => "Non-Euclidean & Topology",
            Self::ExceptionalAndAbstract => "Exceptional Lie & Octonions",
            Self::DModulesAndFormal => "Holonomic D-Modules & Proofs",
            Self::MultiphysicsAndAi => "Multiphysics FEM & Neural ODEs",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::CalculusAndOde => "∫",
            Self::QuantumAndPhysics => "Ψ",
            Self::CadAndSimulation => "⚙",
            Self::GeometryAndTopology => "🌐",
            Self::ExceptionalAndAbstract => "𝔼₈",
            Self::DModulesAndFormal => "📜",
            Self::MultiphysicsAndAi => "🧠",
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::CalculusAndOde,
            Self::QuantumAndPhysics,
            Self::CadAndSimulation,
            Self::GeometryAndTopology,
            Self::ExceptionalAndAbstract,
            Self::DModulesAndFormal,
            Self::MultiphysicsAndAi,
        ]
    }
}

/// Metadata and source content for an embedded example notebook.
#[derive(Debug, Clone)]
pub struct ExampleNotebook {
    pub id: &'static str,
    pub title: &'static str,
    pub category: ExampleCategory,
    pub description: &'static str,
    pub latex_formula: &'static str,
    pub content: &'static str,
    pub tags: &'static [&'static str],
}

/// Master Registry of all embedded URAE example notebooks.
pub struct ExampleRegistry;

impl ExampleRegistry {
    /// Return all curated example notebooks.
    pub fn all_examples() -> &'static [ExampleNotebook] {
        &EXAMPLES
    }

    /// Find an example notebook by its unique identifier.
    pub fn find_by_id(id: &str) -> Option<&'static ExampleNotebook> {
        EXAMPLES.iter().find(|ex| ex.id == id)
    }

    /// Filter example notebooks by category.
    pub fn filter_by_category(cat: ExampleCategory) -> Vec<&'static ExampleNotebook> {
        EXAMPLES.iter().filter(|ex| ex.category == cat).collect()
    }

    /// Search example notebooks by text query across title, description, and tags.
    pub fn search(query: &str) -> Vec<&'static ExampleNotebook> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return EXAMPLES.iter().collect();
        }
        EXAMPLES
            .iter()
            .filter(|ex| {
                ex.title.to_lowercase().contains(&q)
                    || ex.description.to_lowercase().contains(&q)
                    || ex.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }
}

const EXAMPLES: [ExampleNotebook; 12] = [
    // 01: Calculus & ODEs
    ExampleNotebook {
        id: "01_calculus_and_differential_equations",
        title: "Advanced Symbolic Calculus & ODEs",
        category: ExampleCategory::CalculusAndOde,
        description: "Symbolic differentiation, complete transcendental Risch integration, L'Hôpital limits, and damped harmonic oscillator ODEs.",
        latex_formula: "\\mathcal{L}[y](t) = y''(t) + 2\\beta y'(t) + \\omega_0^2 y(t) = 0",
        content: "# URAE Mathematics Notepad: Advanced Calculus, Special Functions & ODEs\n\na: Parameter = 2.50 [m]\nb: Parameter = 1.20 [s^-1]\nomega: Parameter = 3.14159 [rad/s]\nx: Variable\nt: Variable\n\n{ x in Reals | -10.0 <= x <= 10.0 }\n{ t in Reals | 0.0 <= t <= 100.0 }\n\nf(x) = a * exp(-b * x) * cos(omega * x)\ndiff(f(x), x)\ndiff(diff(f(x), x), x)\nintegrate(x^3 * exp(x), x)\n\nseries(sin(x) / x, x, 0, 8)\nlimit((1 - cos(x)) / x^2, x, 0)\n",
        tags: &["calculus", "risch", "integral", "derivative", "ode", "limits", "series"],
    },

    // 02: QFT & Physics
    ExampleNotebook {
        id: "02_quantum_field_theory_and_clifford",
        title: "Quantum Field Theory & Spinor Helicity",
        category: ExampleCategory::QuantumAndPhysics,
        description: "Dirac gamma traces, space-time Clifford algebra Cl(1, 3), massless Weyl spinors, and Parke-Taylor MHV tree amplitudes.",
        latex_formula: "\\mathcal{A}_n^{\\mathrm{MHV}}(1^-, 2^-, 3^+, \\dots, n^+) = \\frac{\\langle 1 2 \\rangle^4}{\\prod_{k=1}^n \\langle k, k+1 \\rangle}",
        content: "# URAE Mathematics Notepad: Quantum Field Theory & Space-Time Clifford Algebra\n\ndirac_trace([0, 1, 0, 1])\ndirac_trace([0, 1, 2, 3])\n\nspinor_bracket_angle(1, 2)\nspinor_bracket_square(2, 1)\n\nparke_taylor_mhv(4, [1, 2])\npassarino_veltman_b0(100.0, 0.0, 0.0)\npassarino_veltman_b1(100.0, 0.0, 0.0)\n",
        tags: &["qft", "dirac", "gamma", "spinor", "helicity", "parke-taylor", "amplitudes"],
    },

    // 03: Parametric CAD & IGA
    ExampleNotebook {
        id: "03_parametric_cad_and_isogeometric_analysis",
        title: "Parametric CAD & Isogeometric Analysis",
        category: ExampleCategory::CadAndSimulation,
        description: "Precision involute spur gears, ISO metric threaded bolts, NACA airfoils, Cox-de Boor B-splines, and NURBS IGA stiffness quadrature.",
        latex_formula: "K_{ab} = \\int_\\Omega \\nabla R_a \\cdot \\nabla R_b \\, d\\Omega",
        content: "# URAE Mathematics Notepad: Parametric CAD & Isogeometric Analysis (IGA)\n\ngear!(module = 2.0, teeth = 18, width = 10.0)\nscrew!(dia = 10.0, pitch = 1.5, length = 25.0)\nairfoil!(code = \"2412\", chord = 10.0, span = 25.0)\n\ncox_de_boor_basis!(0.5, 0, 2, [0.0, 0.0, 0.0, 1.0, 1.0, 1.0])\n",
        tags: &["cad", "gear", "screw", "airfoil", "nurbs", "iga", "b-spline"],
    },

    // 04: Non-Euclidean Geometry
    ExampleNotebook {
        id: "04_non_euclidean_geometry_and_tessellations",
        title: "Non-Euclidean & Hyperbolic Tessellations",
        category: ExampleCategory::GeometryAndTopology,
        description: "Poincaré Disk model, Upper Half-Plane, Cayley conformal transform, Gauss-Bonnet area defects, and Schläfli {7, 3} tilings.",
        latex_formula: "ds^2 = \\frac{4(dx^2 + dy^2)}{(1 - x^2 - y^2)^2}, \\quad \\operatorname{Area}(\\Delta) = \\pi - (\\alpha + \\beta + \\gamma)",
        content: "# URAE Mathematics Notepad: Non-Euclidean Geometry & Hyperbolic Tessellations\n\np1 = poincare_point(0.2, 0.3)\np2 = poincare_point(-0.4, 0.5)\n\nhyperbolic_dist(p1, p2)\nhyperbolic_triangle_area(0.5, 0.6, 0.7)\nspherical_triangle_area(1.2, 1.3, 1.4)\n",
        tags: &["poincare", "hyperbolic", "geometry", "tessellation", "schlafli", "gauss-bonnet"],
    },

    // 05: Exceptional Lie & Octonions
    ExampleNotebook {
        id: "05_exceptional_lie_groups_and_octonions",
        title: "Exceptional Lie Groups & Octonions",
        category: ExampleCategory::ExceptionalAndAbstract,
        description: "8D non-associative Octonions O, Fano plane multiplication, Albert exceptional Jordan algebra H_3(O), and E8 root lattice.",
        latex_formula: "[x, y, z] = (xy)z - x(yz), \\quad \\det(X) = \\xi_1 \\xi_2 \\xi_3 - \\sum \\xi_i N(x_i) + 2 \\operatorname{Re}(x_1 x_2 x_3)",
        content: "# URAE Mathematics Notepad: Exceptional Lie Groups & Octonions\n\nx = octonion!(1, 0, 1, 0, 0, 0, 0, 0)\ny = octonion!(0, 1, 0, 1, 0, 0, 0, 0)\nz = octonion!(0, 0, 1, 0, 1, 0, 0, 0)\n\noctonion_associator!(x, y, z)\noctonion_associator!(x, x, y)\n",
        tags: &["octonion", "albert", "jordan", "e8", "exceptional", "lie", "fano"],
    },

    // 06: Holonomic D-Modules
    ExampleNotebook {
        id: "06_holonomic_dmodules_and_zeilberger",
        title: "Holonomic D-Modules & Zeilberger Proofs",
        category: ExampleCategory::DModulesAndFormal,
        description: "Non-commutative Weyl algebra A_n, Leibniz normal form [d, x] = 1, and automated Zeilberger creative telescoping proofs for binomial sums.",
        latex_formula: "L(n, S_n) F(n, k) = (S_k - 1) G(n, k), \\quad [\\partial_i, x_j] = \\delta_{ij}\\mathbf{I}",
        content: "# URAE Mathematics Notepad: Holonomic D-Modules & Zeilberger Creative Telescoping\n\nx = weyl_x!(1, 0, 1)\nd = weyl_d!(1, 0, 1)\n\nweyl_commute!(d, x)\nzeilberger_binomial_proof!()\nalmkvist_zeilberger_gaussian!()\n",
        tags: &["weyl", "d-modules", "zeilberger", "creative-telescoping", "binomial", "hypergeometric"],
    },

    // 07: Multiphysics FEM & Neural ODEs
    ExampleNotebook {
        id: "07_multiphysics_fem_and_neural_odes",
        title: "Multiphysics FEM & Continuous Neural ODEs",
        category: ExampleCategory::MultiphysicsAndAi,
        description: "2D Poisson heat conduction finite elements, Symplectic 4th-order Yoshida integrators, and continuous Neural ODEs with Adjoint Sensitivity.",
        latex_formula: "-\\nabla \\cdot (k \\nabla T) = Q, \\quad \\frac{d\\mathbf{z}(t)}{dt} = f_\\theta(\\mathbf{z}(t), t)",
        content: "# URAE Mathematics Notepad: Multiphysics Finite Elements & Neural ODEs\n\nmesh_2d(grid_x = 10, grid_y = 10, element_type = Tri3)\npoisson_solver(conductivity = 45.0, heat_source = 1000.0)\n\nsymplectic_yoshida(q0 = 1.0, p0 = 0.0, dt = 0.01, steps = 1000)\nneural_ode_forward(t_span = [0.0, 1.0], state0 = [1.0, 0.5])\n",
        tags: &["fem", "pde", "heat", "poisson", "symplectic", "yoshida", "neural-ode", "adjoint"],
    },

    // 08: Topological Data Analysis & Discrete Hodge
    ExampleNotebook {
        id: "08_topological_data_analysis_and_discrete_hodge",
        title: "Topological Data Analysis & Discrete Hodge",
        category: ExampleCategory::GeometryAndTopology,
        description: "Vietoris-Rips simplicial filtration over point clouds, GF(2) boundary matrix reduction, persistence barcodes, and discrete Hodge Laplacian.",
        latex_formula: "\\Delta_0 = d_0^* d_0, \\quad \\beta_k(\\epsilon) = \\operatorname{rank}(H_k(K_\\epsilon))",
        content: "# URAE Mathematics Notepad: Topological Data Analysis & Discrete Hodge\n\nvietoris_rips!(max_dim = 2, threshold = 1.5)\npersistence_diagram!()\ndiscrete_laplacian_0!()\n",
        tags: &["tda", "vietoris-rips", "homology", "barcodes", "betti", "hodge", "laplacian"],
    },

    // 09: SMT-LIB2 Automated Verification & Proofs
    ExampleNotebook {
        id: "09_smt_lib2_formal_verification",
        title: "SMT-LIB2 Verification & Formal Proof Certificates",
        category: ExampleCategory::DModulesAndFormal,
        description: "Automated theorem proving format serializer for Z3/CVC5 across QF_NRA / QF_LRA and constructive proof certificate generator for Lean 4 & Coq.",
        latex_formula: "\\vdash \\forall x \\in \\mathbb{R}, \\; x^2 \\ge 0 \\quad [\\text{Lean 4: } \\mathtt{positivity}]",
        content: "# URAE Mathematics Notepad: SMT-LIB2 Automated Verification & Proof Certificates\n\nsmt_problem!(logic = \"QF_NRA\")\nsmt_assert!(\"x^2 + y^2 >= 0\")\nproof_certificate!(target = \"Lean4\", theorem = \"x_sq_nonneg\")\n",
        tags: &["smt", "z3", "formal-methods", "lean4", "coq", "proof-assistant", "verification"],
    },

    // 10: Geometric Deep Learning & SE(3) Equivariance
    ExampleNotebook {
        id: "10_geometric_deep_learning_se3",
        title: "Geometric Deep Learning & SE(3) Equivariance",
        category: ExampleCategory::MultiphysicsAndAi,
        description: "Real/complex spherical harmonics Y_lm, Clebsch-Gordan tensor product series, SE(3) steerable convolutions, and Clifford Cl(3, 0) neural layers.",
        latex_formula: "\\mathcal{D}^{(\\ell_1)} \\otimes \\mathcal{D}^{(\\ell_2)} = \\bigoplus_{|\\ell_1 - \\ell_2|}^{\\ell_1 + \\ell_2} \\mathcal{D}^{(\\ell)}",
        content: "# URAE Mathematics Notepad: Geometric Deep Learning & SE(3) Equivariance\n\nspherical_harmonic!(l = 2, m = 1, theta = 0.5, phi = 1.2)\nclebsch_gordan!(l1 = 1, m1 = 1, l2 = 1, m2 = -1, l = 0, m = 0)\nclifford_layer_forward!(in_dim = 8, out_dim = 8)\n",
        tags: &["geometric-dl", "spherical-harmonics", "clebsch-gordan", "se3", "equivariance", "clifford-neural"],
    },

    // 11: Affine Kac-Moody & Virasoro CFT
    ExampleNotebook {
        id: "11_affine_kac_moody_and_virasoro_cft",
        title: "Affine Kac-Moody & Virasoro Conformal Field Theory",
        category: ExampleCategory::QuantumAndPhysics,
        description: "Virasoro Lie brackets, central charge c, minimal model Kac tables h_{r, s}, affine loop current commutators, and OPE Laurent poles.",
        latex_formula: "[L_m, L_n] = (m - n) L_{m+n} + \\frac{c}{12}(m^3 - m) \\delta_{m+n, 0}",
        content: "# URAE Mathematics Notepad: Affine Kac-Moody & Virasoro CFT\n\nvirasoro_bracket!(2, -2, 0.5)\nkac_moody_bracket!(1, 2, 2, -2, 1.0)\nsugawara_central_charge!(dim_g = 3, k = 2.0, dual_coxeter = 2.0)\nkac_conformal_weight!(1, 2, 3)\n",
        tags: &["cft", "virasoro", "kac-moody", "sugawara", "central-charge", "ope", "minimal-models"],
    },

    // 12: p-Adic Hodge Theory & Fontaine Period Rings
    ExampleNotebook {
        id: "12_padic_hodge_theory_and_fontaine",
        title: "p-Adic Hodge Theory & Fontaine Period Rings",
        category: ExampleCategory::ExceptionalAndAbstract,
        description: "Fontaine filtered (Phi, N)-modules with N*Phi = p*Phi*N, local Galois representation admissibility (crystalline, semistable), and Tate twists.",
        latex_formula: "N \\Phi = p \\Phi N, \\quad D_{\\mathrm{cris}}(V) = (B_{\\mathrm{cris}} \\otimes_{\\mathbb{Q}_p} V)^{G_K}",
        content: "# URAE Mathematics Notepad: p-Adic Hodge Theory & Fontaine Period Rings\n\nfontaine_module!(dim = 2, p = 7)\ntate_twist_weights!([0, 2], 2)\ntate_twist_frob!([7.0, 49.0], 7, 1)\n",
        tags: &["padic", "hodge", "fontaine", "frobenius", "galois", "tate-twist", "period-rings"],
    },
];
