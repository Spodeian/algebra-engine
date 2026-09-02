use algebra_engine::branch::{
    BranchCutFunctionKind, BranchCutGeometry, BranchCutManager, BranchSelection, MultiValuedResult,
};
use std::f64::consts::PI;

#[test]
fn test_log_principal_and_higher_sheets() {
    // 1. ln(1) on principal sheet (k=0) -> 0.0 + 0.0i
    let res_principal = BranchCutManager::eval(
        BranchCutFunctionKind::Log,
        1.0,
        0.0,
        &BranchSelection::Principal,
    )
    .unwrap();
    let (re, im) = res_principal.primary_value();
    assert!(re.abs() < 1e-10);
    assert!(im.abs() < 1e-10);

    // 2. ln(1) on sheet k=1 -> 0.0 + 2*pi*i
    let res_sheet1 = BranchCutManager::eval(
        BranchCutFunctionKind::Log,
        1.0,
        0.0,
        &BranchSelection::Sheet(1),
    )
    .unwrap();
    let (re1, im1) = res_sheet1.primary_value();
    assert!(re1.abs() < 1e-10);
    assert!((im1 - 2.0 * PI).abs() < 1e-10);

    // 3. ln(-1) on principal sheet -> 0.0 + pi*i
    let res_neg = BranchCutManager::eval(
        BranchCutFunctionKind::Log,
        -1.0,
        0.0,
        &BranchSelection::Principal,
    )
    .unwrap();
    let (re_neg, im_neg) = res_neg.primary_value();
    assert!(re_neg.abs() < 1e-10);
    assert!((im_neg - PI).abs() < 1e-10);

    // 4. Discontinuity jump across negative real axis: Delta ln(-1) = 2*pi*i
    let (jump_re, jump_im) =
        BranchCutManager::eval_jump(BranchCutFunctionKind::Log, -1.0, 0.0).unwrap();
    assert!(jump_re.abs() < 1e-5);
    assert!((jump_im - 2.0 * PI).abs() < 1e-5);
}

#[test]
fn test_sqrt_all_sheets_and_multi_valued() {
    // sqrt(4) across all sheets -> { +2, -2 }
    let res_mv = BranchCutManager::eval(
        BranchCutFunctionKind::Sqrt,
        4.0,
        0.0,
        &BranchSelection::AllSheets { max_branches: 2 },
    )
    .unwrap();

    if let MultiValuedResult::Set(branches) = res_mv {
        assert_eq!(branches.len(), 2);
        assert!((branches[0].0 - 2.0).abs() < 1e-10);
        assert!((branches[1].0 - (-2.0)).abs() < 1e-10);
    } else {
        panic!("Expected multi-valued set");
    }

    // sqrt(-1) on principal sheet -> 0 + 1i
    let res_i = BranchCutManager::eval(
        BranchCutFunctionKind::Sqrt,
        -1.0,
        0.0,
        &BranchSelection::Principal,
    )
    .unwrap();
    let (re_i, im_i) = res_i.primary_value();
    assert!(re_i.abs() < 1e-10);
    assert!((im_i - 1.0).abs() < 1e-10);
}

#[test]
fn test_nth_root_three_branches() {
    // Cube roots of 8 -> 2, 2*e^(i*2pi/3), 2*e^(i*4pi/3)
    let res_cube = BranchCutManager::eval(
        BranchCutFunctionKind::Root(3),
        8.0,
        0.0,
        &BranchSelection::AllSheets { max_branches: 3 },
    )
    .unwrap();

    let vals = res_cube.all_values();
    assert_eq!(vals.len(), 3);

    // Principal root = 2
    assert!((vals[0].0 - 2.0).abs() < 1e-10);
    assert!(vals[0].1.abs() < 1e-10);

    // Second root = 2 * (-0.5 + i*sqrt(3)/2) = -1 + i*sqrt(3)
    assert!((vals[1].0 - (-1.0)).abs() < 1e-5);
    assert!((vals[1].1 - 3.0f64.sqrt()).abs() < 1e-5);
}

#[test]
fn test_custom_branch_cut() {
    // Custom cut along ray at angle -pi/2
    let custom_cut = BranchCutGeometry::Ray {
        origin: (0.0, 0.0),
        angle_rad: -PI / 2.0,
    };
    let res = BranchCutManager::eval(
        BranchCutFunctionKind::Log,
        -1.0,
        0.0,
        &BranchSelection::CustomCut {
            geometry: custom_cut,
            sheet: 0,
        },
    )
    .unwrap();

    let (re, im) = res.primary_value();
    assert!(re.abs() < 1e-10);
    assert!(im.is_finite());
}

#[test]
fn test_lambert_w_principal_branch() {
    // W_0(e) = 1.0 (since 1 * e^1 = e)
    let e = std::f64::consts::E;
    let res = BranchCutManager::eval(
        BranchCutFunctionKind::LambertW,
        e,
        0.0,
        &BranchSelection::Principal,
    )
    .unwrap();

    let (re, im) = res.primary_value();
    assert!((re - 1.0).abs() < 1e-5);
    assert!(im.abs() < 1e-5);
}

#[test]
fn test_analytic_continuation_circle_path() {
    // Path circling the origin once counter-clockwise: z(t) = cos(t) + i*sin(t) for t in [0, 2*pi]
    let n_steps = 50;
    let mut path = Vec::with_capacity(n_steps + 1);
    for i in 0..=n_steps {
        let t = 2.0 * PI * (i as f64) / (n_steps as f64);
        path.push((t.cos(), t.sin()));
    }

    let (val_re, val_im, final_sheet) =
        BranchCutManager::analytic_continuation(BranchCutFunctionKind::Log, &path, 0).unwrap();

    assert_eq!(final_sheet, 1);
    assert!(val_re.abs() < 1e-5);
    assert!((val_im - 2.0 * PI).abs() < 1e-5);
}
