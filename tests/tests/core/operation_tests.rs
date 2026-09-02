use algebra_core::operation::*;
use algebra_core::parser::parse_operation;

#[test]
fn test_parse_operations_calculus_and_solvers() {
    let op = parse_operation("diff x^3, x");
    assert!(
        matches!(op, MathOperation::Differentiate { ref expression, ref variable, .. } if expression == "x^3" && variable.as_deref() == Some("x"))
    );

    let op2 = parse_operation("integrate cos(x), x");
    assert!(
        matches!(op2, MathOperation::Integrate { ref expression, ref variable, .. } if expression == "cos(x)" && variable.as_deref() == Some("x"))
    );

    let op3 = parse_operation("solve x^2 - 4 = 0, x");
    assert!(
        matches!(op3, MathOperation::Solve { ref equation, ref variable } if equation == "x^2 - 4 = 0" && variable.as_deref() == Some("x"))
    );
}

#[test]
fn test_parse_operations_transformations() {
    let op_exp = parse_operation("expand (x + 1)^3");
    assert!(
        matches!(op_exp, MathOperation::Simplify { ref expression, strategy: SimplifyStrategy::Expand } if expression == "(x + 1)^3")
    );

    let op_fac = parse_operation("factor x^2 - 1");
    assert!(
        matches!(op_fac, MathOperation::Simplify { ref expression, strategy: SimplifyStrategy::Factor } if expression == "x^2 - 1")
    );

    let op_tog = parse_operation("together 1/x + 1/y");
    assert!(
        matches!(op_tog, MathOperation::Simplify { ref expression, strategy: SimplifyStrategy::Together } if expression == "1/x + 1/y")
    );

    let op_can = parse_operation("cancel (x^2 - 1)/(x - 1)");
    assert!(
        matches!(op_can, MathOperation::Simplify { ref expression, strategy: SimplifyStrategy::Cancel } if expression == "(x^2 - 1)/(x - 1)")
    );

    let op_horn = parse_operation("horner 2*x^2 + 3*x + 1, x");
    assert!(
        matches!(op_horn, MathOperation::Simplify { ref expression, strategy: SimplifyStrategy::Horner { var: Some(ref v) } } if expression == "2*x^2 + 3*x + 1" && v == "x")
    );
}

#[test]
fn test_parse_operations_esoteric_hyperop() {
    let op_tet = parse_operation("tetration 2 3");
    assert!(matches!(
        op_tet,
        MathOperation::HyperOp(HyperOpKind::Tetration { a: 2, b: 3 })
    ));

    let op_knuth = parse_operation("knuth 2 2 3");
    assert!(matches!(
        op_knuth,
        MathOperation::HyperOp(HyperOpKind::KnuthUpArrow {
            a: 2,
            arrows: 2,
            b: 3
        })
    ));

    let op_ack = parse_operation("ackermann 2 2");
    assert!(matches!(
        op_ack,
        MathOperation::HyperOp(HyperOpKind::Ackermann { m: 2, n: 2 })
    ));

    let op_slog = parse_operation("slog 2 16");
    assert!(
        matches!(op_slog, MathOperation::HyperOp(HyperOpKind::SuperLog { base, val }) if base == 2.0 && val == 16.0)
    );
}

#[test]
fn test_parse_operations_tropical_and_number_theory() {
    let op_mp = parse_operation("maxplus_add 3 5");
    assert!(
        matches!(op_mp, MathOperation::Tropical(TropicalOpKind::MaxPlusAdd { a, b }) if a == 3.0 && b == 5.0)
    );

    let op_cf = parse_operation("cf 1.41421356, 5");
    assert!(
        matches!(op_cf, MathOperation::NumberTheory(NumberTheoryOpKind::ContinuedFraction { ref val_str, max_terms: 5 }) if val_str == "1.41421356")
    );

    let op_prime = parse_operation("is_prime 101");
    assert!(matches!(
        op_prime,
        MathOperation::NumberTheory(NumberTheoryOpKind::IsPrime { n: 101 })
    ));

    let op_gcd = parse_operation("gcd 30, 20");
    assert!(matches!(
        op_gcd,
        MathOperation::NumberTheory(NumberTheoryOpKind::ExtendedGcd { a: 30, b: 20 })
    ));
}

#[test]
fn test_parse_operations_matrices_and_combinatorics() {
    let op_det = parse_operation("det [[1, 2], [3, 4]]");
    assert!(
        matches!(op_det, MathOperation::Matrix(MatrixOpKind::Determinant { ref matrix_str }) if matrix_str == "[[1, 2], [3, 4]]")
    );

    let op_fact = parse_operation("factorial 10");
    assert!(matches!(
        op_fact,
        MathOperation::Combinatorics(CombinatoricsOpKind::Factorial { n: 10 })
    ));

    let op_comb = parse_operation("combinations 10, 3");
    assert!(matches!(
        op_comb,
        MathOperation::Combinatorics(CombinatoricsOpKind::Combinations { n: 10, k: 3 })
    ));

    let op_cat = parse_operation("catalan 5");
    assert!(matches!(
        op_cat,
        MathOperation::Combinatorics(CombinatoricsOpKind::Catalan { n: 5 })
    ));
}

#[test]
fn test_parse_operations_transforms_and_physics() {
    let op_lap = parse_operation("laplace sin(t), t, s");
    assert!(
        matches!(op_lap, MathOperation::Transform(TransformOpKind::Laplace { ref expression, ref time_var, ref freq_var }) if expression == "sin(t)" && time_var == "t" && freq_var == "s")
    );

    let op_conv = parse_operation("convert 100 [km/h] to [m/s]");
    assert!(
        matches!(op_conv, MathOperation::Physics(PhysicsOpKind::Convert { val, ref from_unit, ref to_unit }) if (val - 100.0).abs() < 1e-6 && from_unit == "km/h" && to_unit == "m/s")
    );
}

#[test]
fn test_parse_operations_declarations_and_system() {
    let op_decl = parse_operation("a: Parameter = 2.00 [m]");
    assert!(
        matches!(op_decl, MathOperation::Declaration { ref name, role: SymbolRoleKind::Parameter, value: Some(v), unit: Some(ref u) } if name == "a" && (v - 2.0).abs() < 1e-6 && u == "m")
    );

    let op_sb = parse_operation("{ x in Reals | -5 <= x <= 5 }");
    assert!(
        matches!(op_sb, MathOperation::SetBuilder { ref var_name, ref domain_type, ref condition } if var_name == "x" && domain_type == "Reals" && condition == "-5 <= x <= 5")
    );

    let op_help = parse_operation("help");
    assert!(matches!(
        op_help,
        MathOperation::System(SystemCommandKind::Help)
    ));

    let op_vars = parse_operation("vars");
    assert!(matches!(
        op_vars,
        MathOperation::System(SystemCommandKind::Vars)
    ));
}
