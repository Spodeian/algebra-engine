use algebra_advanced::StructureTensorAlgebra;

#[test]
fn test_nested_structure_tensor_complex_dual() {
    // 1. Construct Complex Dual Numbers C (x) D = {1, i, eps, i*eps}
    let cd = StructureTensorAlgebra::complex_dual();
    assert_eq!(cd.dim, 4);
    assert_eq!(cd.basis_names, vec!["1", "eps", "i", "i*eps"]);

    // Represent i = [0, 0, 1, 0] and eps = [0, 1, 0, 0]
    let i_elem = vec![0.0, 0.0, 1.0, 0.0];
    let eps_elem = vec![0.0, 1.0, 0.0, 0.0];

    // Product i * eps = [0, 0, 0, 1]
    let i_eps = cd.multiply(&i_elem, &eps_elem).unwrap();
    assert_eq!(i_eps, vec![0.0, 0.0, 0.0, 1.0]);

    // Product eps * i = [0, 0, 0, 1] (Commutative across tensor factors!)
    let eps_i = cd.multiply(&eps_elem, &i_elem).unwrap();
    assert_eq!(eps_i, vec![0.0, 0.0, 0.0, 1.0]);

    // Commutator [i, eps] = 0
    let comm = cd.commutator(&i_elem, &eps_elem).unwrap();
    assert_eq!(comm, vec![0.0, 0.0, 0.0, 0.0]);

    // (i * eps)^2 = 0
    let i_eps_sq = cd.multiply(&i_eps, &i_eps).unwrap();
    assert_eq!(i_eps_sq, vec![0.0, 0.0, 0.0, 0.0]);

    // Regular Matrix representation isomorphism: L(a * b) = L(a) * L(b)
    let l_i = cd.regular_matrix_representation(&i_elem).unwrap();
    let l_eps = cd.regular_matrix_representation(&eps_elem).unwrap();
    let l_i_eps = cd.regular_matrix_representation(&i_eps).unwrap();

    // Matrix multiply L(i) * L(eps)
    let mut prod_mat = vec![vec![0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            for k in 0..4 {
                prod_mat[r][c] += l_i[r][k] * l_eps[k][c];
            }
        }
    }
    assert_eq!(prod_mat, l_i_eps);
}

#[test]
fn test_nested_structure_tensor_dual_quaternions() {
    // Dual Quaternions H (x) D (8-dimensional SE(3) screw algebra)
    let dq = StructureTensorAlgebra::dual_quaternion();
    assert_eq!(dq.dim, 8);

    // Basis: [1, eps, i, i*eps, j, j*eps, k, k*eps]
    let i_elem = vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let j_elem = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0];
    let eps_elem = vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    // 1. eps commutes with i: [i, eps] = 0
    let comm_i_eps = dq.commutator(&i_elem, &eps_elem).unwrap();
    assert_eq!(comm_i_eps, vec![0.0; 8]);

    // 2. But i and j do not commute: [i, j] = 2k
    let comm_i_j = dq.commutator(&i_elem, &j_elem).unwrap();
    // 2 * k = [0, 0, 0, 0, 0, 0, 2, 0]
    assert_eq!(comm_i_j[6], 2.0);
}
