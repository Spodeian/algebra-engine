use algebra_advanced::gpu::{GpuComputeKernel, SimdPolyVector};
use algebra_core::ExprGraph;
use algebra_engine::matrix::SymbolicMatrix;

#[test]
fn test_simd_poly_vector() {
    let a = SimdPolyVector::new(vec![1, 2, 3]);
    let b = SimdPolyVector::new(vec![4, 5, 6]);

    let sum = a.add_simd(&b);
    assert_eq!(sum.coeffs, vec![5, 7, 9]);

    let prod = a.multiply_simd(&b);
    assert_eq!(prod.coeffs, vec![4, 13, 28, 27, 18]);
}

#[test]
fn test_gpu_matrix_multiply_blocks() {
    let graph = ExprGraph::new();
    let kernel = GpuComputeKernel::new();

    let a11 = graph.integer(1);
    let a12 = graph.integer(2);
    let a21 = graph.integer(3);
    let a22 = graph.integer(4);

    let m1 = SymbolicMatrix::new(2, 2, vec![a11, a12, a21, a22]).unwrap();
    let result = kernel.multiply_matrix_blocks(&graph, &m1, &m1).unwrap();

    assert_eq!(result.rows, 2);
    assert_eq!(result.cols, 2);
}
