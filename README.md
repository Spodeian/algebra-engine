# Universal Rust Algebra Engine (URAE)

[![Rust Workspace](https://img.shields.io/badge/Rust-1.85%2B%20%7C%202021%20Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Proprietary-blue.svg)](LICENSE)
[![Tests Passing](https://img.shields.io/badge/Tests-345%20Passing%20%7C%200%20Failures-brightgreen.svg)](tests/)
[![Clippy](https://img.shields.io/badge/Clippy-0%20Warnings-brightgreen.svg)](Cargo.toml)
[![WASM Ready](https://img.shields.io/badge/WebAssembly-Cloudflare%20Pages%20%7C%20PWA-9cf?logo=webassembly)](crates/urae-wasm)

A high-performance, modular formal mathematics computer algebra system (CAS), multi-physics engineering analysis platform, and reactive computational notepad application built in pure Rust and WebAssembly.

Designed for dual deployment as an ultra-responsive **Native Desktop Application (Windows, macOS, Linux with hardware acceleration)** and a zero-cold-start **Serverless Web Application (WASM / Cloudflare Pages / PWA)**.

---

## 🏛️ Layered Architecture & Workspace Crates

The workspace is organized into **11 cohesive, modular crates** with a centralized integration test suite under `tests/`:

```mermaid
graph TD
    subgraph Layer 1: Foundation
        Core["crates/algebra-core<br/>• ExprGraph Arena DAG Storage<br/>• Symbol Table & Interning<br/>• Certified Interval Arithmetic<br/>• Multi-Notation Parser & 2D Formatters"]
    end

    subgraph Layer 2: Core Computational CAS Engine
        Engine["crates/algebra-engine<br/>• E-Graph Simplification (egg)<br/>• Symbolic Solvers & Calculus<br/>• Polynomials, Linear Algebra & Tensors<br/>• Combinatorics & Topology<br/>• Number Theory & Control Systems<br/>• Knuth Up-Arrows, Tetration & Tropical Semirings"]
    end

    subgraph Layer 3: Advanced Mathematics, Physics & Proofs
        Advanced["crates/algebra-advanced<br/>• Formal Proof Export (Lean 4 / Coq / SMT-LIB2)<br/>• Category Theory, Monads & Isomorphisms<br/>• Quantum Bra-Kets & Spinor Helicity<br/>• Stochastic Itô Calculus & Galois Fields<br/>• Statistical Mechanics & Thermofluids<br/>• Parametric CAD Machinery & B-Spline IGA<br/>• Exceptional Lie Groups & Octonions<br/>• Holonomic D-Modules & Zeilberger Proofs<br/>• AI Mathematical Copilot"]
    end

    subgraph Layer 4: Master Facade & Prelude
        Urae["crates/urae<br/>Master Facade & Unified Prelude (`use urae::prelude::*;`)"]
    end

    subgraph Layer 5: Applications, Protocols & Interfaces
        Notebook["crates/urae-notebook<br/>Reactive Continuous Notepad (egui/eframe)"]
        CLI["crates/urae-cli<br/>Interactive REPL & JSON Streaming CLI"]
        WASM["crates/urae-wasm<br/>Cloudflare Pages / Edge WASM Engine"]
        LSP["crates/urae-lsp<br/>Language Server Protocol Engine"]
        Kernel["crates/urae-kernel<br/>Jupyter Protocol Backend"]
        Py["crates/urae-py<br/>PyO3 Python Bindings"]
        FFI["crates/urae-ffi<br/>C ABI Foreign Function Interface"]
    end

    subgraph Layer 6: Workspace Test Suite
        Tests["tests/<br/>Centralized Workspace Test Suite (345+ Tests)"]
    end

    Core --> Engine
    Core --> Advanced
    Engine --> Advanced
    Core --> Urae
    Engine --> Urae
    Advanced --> Urae
    Urae --> Notebook
    Urae --> CLI
    Urae --> WASM
    Urae --> LSP
    Urae --> Kernel
    Urae --> Py
    Urae --> FFI
    Notebook --> Tests
    Engine --> Tests
    Advanced --> Tests
    Urae --> Tests
    FFI --> Tests
```

---

## 📦 Crate Breakdown

| Crate | Path | Description |
| :--- | :--- | :--- |
| **`algebra-core`** | [`crates/algebra-core`](crates/algebra-core) | Memory-safe arena DAG storage (`ExprGraph`), symbol table interning, certified interval arithmetic ($[a, b]$ with directed rounding), multi-notation parser (`nom`), and 2D Unicode / LaTeX formatters. |
| **`algebra-engine`** | [`crates/algebra-engine`](crates/algebra-engine) | Core symbolic CAS: non-greedy E-Graph equality saturation (`egg`), ODE/PDE solvers, calculus, dense/CSR sparse linear algebra, covariant/contravariant tensors, topological Betti numbers, Knuth up-arrows, and tropical semirings. |
| **`algebra-advanced`** | [`crates/algebra-advanced`](crates/algebra-advanced) | Advanced formal mathematical structures: formal proof certificates (Lean 4, Coq, SMT-LIB2), Category Theory, Quantum Spinors & Helicity, Parametric CAD & Isogeometric Analysis (IGA), Exceptional Lie Groups ($E_8, G_2$) & Octonions, Holonomic $D$-Modules, and AI copilot integration. |
| **`urae`** | [`crates/urae`](crates/urae) | Primary umbrella facade providing the unified prelude (`use urae::prelude::*;`) and complete CAS API. |
| **`urae-notebook`** | [`crates/urae-notebook`](crates/urae-notebook) | Reactive continuous mathematical notepad application built with `egui` and `eframe`. Features in-situ click-to-edit cards, 60 FPS slider updates, dynamic 2D plotting, 3D simulation viewport, matrix/ODE/units builder palettes, multi-level undo/redo, and in-app example gallery. |
| **`urae-cli`** | [`crates/urae-cli`](crates/urae-cli) | High-speed command-line REPL and JSON stream processing interface. |
| **`urae-wasm`** | [`crates/urae-wasm`](crates/urae-wasm) | WebAssembly edge runtime for Cloudflare Workers & Pages, enabling web notebook execution and sub-millisecond edge API endpoints. |
| **`urae-lsp`** | [`crates/urae-lsp`](crates/urae-lsp) | Language Server Protocol (LSP) backend for mathematical script editing, diagnostics, completions, and hover documentation in VS Code, Helix, and Neovim. |
| **`urae-kernel`** | [`crates/urae-kernel`](crates/urae-kernel) | Native Jupyter kernel implementation supporting `.ipynb` execution and MIME-type mathematical rendering. |
| **`urae-py`** | [`crates/urae-py`](crates/urae-py) | High-performance Python extension module powered by PyO3. |
| **`urae-ffi`** | [`crates/urae-ffi`](crates/urae-ffi) | Standard C ABI foreign function interface for cross-language integration (C, C++, Julia, Fortran). |

---

## ✨ Core Mathematical & Physics Capabilities

### 1. Symbolic CAS & Equality Saturation
- **E-Graph Equality Saturation (`egg`)**: Non-greedy pattern matching discovers the simplest canonical algebraic forms without getting trapped in local algorithmic minima.
- **Symbolic Calculus & Differential Algebra**: Symbolic limits, derivatives, Taylor/Laurent series, and automated Risch indefinite integration.
- **Polynomial Algebra & Gröbner Bases**: Multivariate polynomial arithmetic, Buchberger algorithm, and polynomial GCDs over finite fields and rationals.

### 2. Parametric CAD, Machinery & Isogeometric Analysis (IGA)
- **Parametric Machinery Generators**: Involute spur and helical gears with custom pressure angles and teeth count, ISO standard metric screws with thread profiles, and NACA 4-digit symmetric and cambered aerodynamic airfoils.
- **B-Spline & NURBS Geometry**: Knot vectors, basis function recurrence, B-spline curves, surfaces, and control net evaluation.
- **Isogeometric Analysis (IGA)**: Exact CAD geometry analysis integration, numerical quadrature, and stiffness matrix assembly for elasticity and thermal conduction.

### 3. Advanced Theoretical Physics & Abstract Algebra
- **Quantum Field Theory & Spinor Helicity**: Dirac gamma matrices, 4D Clifford algebra traces, Weyl 2-component spinor angle/square brackets $\langle p q \rangle [p q]$, and Parke-Taylor tree-level gluon scattering amplitudes.
- **Exceptional Lie Groups & Hypercomplex Algebras**: 8D non-associative Octonions $\mathbb{O}$, Albert Jordan algebras $H_3(\mathbb{O})$, and $E_8$ root systems with Cartan-Killing forms.
- **Holonomic $D$-Modules & Zeilberger's Algorithm**: Differential operators in Weyl algebra $A_n(\mathbb{C})$ ($[d, x] = 1$) with automated creative telescoping proofs for hypergeometric summation identities.
- **Certified Arithmetic & Complex Analysis**: Directed-rounding interval arithmetic $[a, b]$, and complex branch cut Riemann surface topological sheet visualizers ($\sqrt{z}, \ln(z), \arcsin(z)$).

### 4. Formal Proof Verification & Cross-System Export
- **Formal Proof Export**: Automatic translation of algebraic transformation traces to formal verification scripts in **Lean 4**, **Coq**, and **SMT-LIB2**.
- **Cross-Format Interoperability**: Dual `.urae` (compressed BSON/JSON) and `.ipynb` (Jupyter notebook format) import/export.

---

## 🖥️ URAE Continuous Reactive Notepad

The **URAE Notebook** (`urae-notebook`) is an interactive desktop and browser GUI environment built on `egui` and `eframe`:

### Key UI Features
- **4 Unified View Modes**:
  - **Fluid Notepad**: Document canvas with live mathematical result cards and click-to-edit in-situ line modifications.
  - **Split View**: Side-by-side raw script editor and instantaneous live reactive output.
  - **Live Preview**: Clean presentation mode displaying rendered mathematical cards.
  - **Text Editor**: Direct multi-line script editor.
- **60 FPS Reactive Parameter Scrubbing**: Dragging sliders instantly updates downstream equations, plots, and domains with fine-grained delta invalidation.
- **Interactive 3D Viewport (`Viewport3D`)**: Hardware-accelerated / software-projected 3D wireframe and shaded mesh visualization with multi-mode shading (*Stress*, *Temperature*, *Pressure*, *Normals*, *Monochrome*) and orbital camera navigation.
- **Transactional Undo / Redo Subsystem**:
  - Immutable `HistorySnapshot` records document text, parameter values, symbol roles, and card modes.
  - Global physical keyboard shortcuts (`Ctrl+Z`, `Cmd+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`, `Cmd+Shift+Z`).
  - Top navigation bar toolbar buttons `⮌ Undo` / `⮎ Redo` with action name preview tooltips.
  - Command Palette triggers (`undo`, `redo`).
- **Interactive Builder Palettes & Wizards**:
  - **Parametric CAD Machinery**: Generate gears, screws, and airfoils with live parameter tuning.
  - **Matrix Preset Generator**: Hilbert, Toeplitz, Vandermonde, Pauli, and Dirac matrices.
  - **ODE / PDE Boundary Condition Wizard**: Cauchy, Dirichlet, Neumann, and Robin conditions.
  - **Physical Units & Dimensions Palette**: SI unit selection and automatic dimensional consistency verification.
  - **Branch Cut & Interval Pickers**: Complex branch cut topology and domain bounds.
- **In-App Example Gallery & Interactive Tutorials**:
  - 12 production-grade mathematical example notebooks across 7 domains:
    - *Symbolic Calculus & ODEs*
    - *Quantum Field Theory & Spinor Helicity*
    - *Affine Kac-Moody & Virasoro CFT*
    - *Parametric CAD & Isogeometric Analysis*
    - *Non-Euclidean & Hyperbolic Tessellations*
    - *Topological Data Analysis & Discrete Hodge*
    - *Exceptional Lie Groups & Octonions*
    - *$p$-Adic Hodge Theory & Fontaine Period Rings*
    - *Holonomic $D$-Modules & Zeilberger Proofs*
    - *SMT-LIB2 Verification & Formal Certificates*
    - *Multiphysics FEM & Neural ODEs*
    - *Geometric Deep Learning & $SE(3)$ Equivariance*
  - One-click load directly into the active editor with automatic history tracking.
- **Fuzzy Command Palette (`Ctrl+K` / `Cmd+K`)**: Search and execute commands, templates, and example notebooks instantaneously.

---

## 🚀 Quickstart

### 1. Launch Native Desktop Notepad GUI
```bash
cargo run --release -p urae-notebook
```

### 2. Launch Interactive Command-Line REPL
```bash
cargo run --release -p urae-cli
```

### 3. Deploy WebAssembly & Cloudflare Pages
```bash
# Automated local build and deployment script
./deploy.sh
```

### 4. Use as a Rust Library
Add URAE to your `Cargo.toml`:
```toml
[dependencies]
urae = { path = "crates/urae" }
```

Use the unified prelude in your code:
```rust
use urae::prelude::*;

fn main() {
    let graph = ExprGraph::new();
    let x_sym = graph.symbols.get_or_intern("x");
    let parser = ExprParser::new(&graph);
    
    // Parse symbolic expression
    let expr = parser.parse("x^2 + 0").unwrap();

    // Differentiate & simplify via E-Graph equality saturation
    let diff = graph.diff(expr, x_sym);
    let simplified = Simplifier::simplify(&graph, expr).unwrap();

    // Format to LaTeX
    let latex = LatexFormatter.format(&graph, simplified).unwrap();
    println!("LaTeX: {}", latex); // {x}^{2}

    // Esoteric Knuth Up-Arrow tetration
    let tet = tetration(2, 3); // 2 ^^ 3 = 16
    println!("2 ^^ 3 = {}", tet);
}
```

---

## 🧪 Testing & Verification

All workspace integration and unit tests are centralized in the `tests/` directory:

```bash
# Run all 345+ workspace unit, integration, and doc tests
cargo test --workspace

# Run specific integration suites
cargo test --test engine_suite      # Core symbolic engine & physics (281 tests)
cargo test --test notebook_suite    # GUI notepad, canvas, UI, undo/redo & examples (41 tests)
cargo test --test facade_suite      # Umbrella facade & macro integration (11 tests)
cargo test --test ffi_suite         # C ABI & foreign function interface (2 tests)

# Verify zero compiler warnings & zero clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Verify WebAssembly target compilation
cargo check --target wasm32-unknown-unknown -p urae-wasm
```

---

## ⚙️ Compilation & Optimization Profiles

Optimized for maximum runtime efficiency and minimal binary footprint:

```toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
overflow-checks = false
```

---

## 📄 License

Proprietary. All rights reserved.
