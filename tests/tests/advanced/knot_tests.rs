use algebra_advanced::knot::{BraidGenerator, BraidWord, KnotInvariants};

#[test]
fn test_braid_reduction_and_relations() {
    // sigma_1 * sigma_1^-1 -> empty
    let b1 = BraidWord::new(
        2,
        vec![
            BraidGenerator { index: 1, sign: 1 },
            BraidGenerator { index: 1, sign: -1 },
        ],
    );
    let reduced1 = b1.reduce();
    assert!(reduced1.word.is_empty());

    // Yang-Baxter braid relation: sigma_1 sigma_2 sigma_1 -> sigma_2 sigma_1 sigma_2
    let b2 = BraidWord::new(
        3,
        vec![
            BraidGenerator { index: 1, sign: 1 },
            BraidGenerator { index: 2, sign: 1 },
            BraidGenerator { index: 1, sign: 1 },
        ],
    );
    let reduced2 = b2.reduce();
    assert_eq!(reduced2.word[0].index, 2);
    assert_eq!(reduced2.word[1].index, 1);
    assert_eq!(reduced2.word[2].index, 2);
}

#[test]
fn test_trefoil_kauffman_bracket_and_jones() {
    let trefoil = BraidWord::trefoil();
    assert_eq!(trefoil.writhe(), 3);

    // Kauffman bracket of Trefoil: <3_1> = A^7 - A^3 - A^-5
    let bracket = KnotInvariants::kauffman_bracket(&trefoil);
    assert_eq!(*bracket.terms.get(&7).unwrap_or(&0.0), 1.0);
    assert_eq!(*bracket.terms.get(&3).unwrap_or(&0.0), -1.0);
    assert_eq!(*bracket.terms.get(&-5).unwrap_or(&0.0), -1.0);

    // Normalized Jones polynomial: non-zero invariant terms
    let jones = KnotInvariants::jones_polynomial(&trefoil);
    assert!(!jones.terms.is_empty());
}

#[test]
fn test_figure_eight_bracket() {
    let fig8 = BraidWord::figure_eight();
    assert_eq!(fig8.writhe(), 0);

    let bracket = KnotInvariants::kauffman_bracket(&fig8);
    // <4_1> has symmetric terms A^8, -A^4, 1, -A^-4, A^-8
    assert_eq!(*bracket.terms.get(&8).unwrap_or(&0.0), 1.0);
    assert_eq!(*bracket.terms.get(&0).unwrap_or(&0.0), 1.0);
    assert_eq!(*bracket.terms.get(&-8).unwrap_or(&0.0), 1.0);
}
