//! # `algebra_engine::gpu`
//!
//! GPU Compute Shader Generation & Matrix-Free Kernel Abstraction.
//!
//! Provides WebGPU (WGSL) compute shader emission for:
//! - High-throughput multi-point expression evaluation ($10^6+$ points in parallel).
//! - Matrix-free Finite Element stiffness-vector contraction $\mathbf{y} = \mathbf{K} \mathbf{x}$.
//! - Complex fractal iteration kernels (Mandelbrot & Julia sets).

/// GPU Compute Pipeline & Shader Descriptor.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuKernelDescriptor {
    pub kernel_name: String,
    pub workgroup_size: [u32; 3],
    pub wgsl_source: String,
    pub input_buffer_count: usize,
    pub output_buffer_count: usize,
}

/// WebGPU (WGSL) Compute Shader Generator.
pub struct WgslShaderGenerator;

impl WgslShaderGenerator {
    /// Generate a WGSL compute shader for multi-point polynomial/function evaluation over a 1D/2D/3D grid.
    pub fn generate_grid_evaluation_shader(
        expr_wgsl: &str,
        workgroup_size: u32,
    ) -> GpuKernelDescriptor {
        let source = format!(
            r#"// Auto-generated URAE WGSL Grid Evaluation Compute Shader
@group(0) @binding(0) var<storage, read> input_coords: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_values: array<f32>;

struct UniformParams {{
    total_elements: u32,
    scale_factor: f32,
}};
@group(0) @binding(2) var<uniform> params: UniformParams;

@compute @workgroup_size({workgroup_size}, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let idx = global_id.x;
    if (idx >= params.total_elements) {{
        return;
    }}
    let x = input_coords[idx];
    let val = {expr_wgsl};
    output_values[idx] = val;
}}
"#
        );

        GpuKernelDescriptor {
            kernel_name: "grid_evaluation".into(),
            workgroup_size: [workgroup_size, 1, 1],
            wgsl_source: source,
            input_buffer_count: 1,
            output_buffer_count: 1,
        }
    }

    /// Generate a WGSL compute shader for Matrix-Free Finite Element Stiffness-Vector Multiplication:
    /// $$\mathbf{y} = \sum_e \mathbf{P}_e^T \mathbf{K}_e \mathbf{P}_e \mathbf{x}$$
    pub fn generate_matrix_free_fem_shader(
        dofs_per_elem: u32,
        workgroup_size: u32,
    ) -> GpuKernelDescriptor {
        let source = format!(
            r#"// Auto-generated URAE WGSL Matrix-Free FEM Stiffness-Vector Contraction Shader
@group(0) @binding(0) var<storage, read> elem_connectivity: array<u32>;
@group(0) @binding(1) var<storage, read> elem_stiffness: array<f32>;
@group(0) @binding(2) var<storage, read> global_vector_x: array<f32>;
@group(0) @binding(3) var<storage, read_write> global_vector_y: array<atomic<f32>>;

struct FemParams {{
    num_elements: u32,
    e_mod: f32,
}};
@group(0) @binding(4) var<uniform> params: FemParams;

@compute @workgroup_size({workgroup_size}, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let elem_idx = global_id.x;
    if (elem_idx >= params.num_elements) {{
        return;
    }}

    let dof = {dofs_per_elem}u;
    var u_local: array<f32, {dofs_per_elem}>;

    // Gather local DOFs
    for (var i: u32 = 0u; i < dof; i = i + 1u) {{
        let global_node = elem_connectivity[elem_idx * dof + i];
        u_local[i] = global_vector_x[global_node];
    }}

    // Local matrix-vector multiply & scatter atomic accumulation
    for (var i: u32 = 0u; i < dof; i = i + 1u) {{
        var sum: f32 = 0.0;
        for (var j: u32 = 0u; j < dof; j = j + 1u) {{
            let k_val = elem_stiffness[elem_idx * dof * dof + i * dof + j];
            sum = sum + k_val * u_local[j];
        }}
        let target_node = elem_connectivity[elem_idx * dof + i];
        // Accumulate into global output vector
        atomicAdd(&global_vector_y[target_node], sum);
    }}
}}
"#
        );

        GpuKernelDescriptor {
            kernel_name: "matrix_free_fem".into(),
            workgroup_size: [workgroup_size, 1, 1],
            wgsl_source: source,
            input_buffer_count: 3,
            output_buffer_count: 1,
        }
    }

    /// Generate a WGSL compute shader for Mandelbrot / Julia Complex Fractal generation.
    pub fn generate_complex_fractal_shader(
        max_iters: u32,
        workgroup_size_xy: u32,
    ) -> GpuKernelDescriptor {
        let source = format!(
            r#"// Auto-generated URAE WGSL Complex Fractal Compute Shader
@group(0) @binding(0) var<storage, read_write> pixel_buffer: array<u32>;

struct FractalParams {{
    width: u32,
    height: u32,
    center_re: f32,
    center_im: f32,
    zoom: f32,
    max_iterations: u32,
}};
@group(0) @binding(1) var<uniform> params: FractalParams;

@compute @workgroup_size({workgroup_size_xy}, {workgroup_size_xy}, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let px = global_id.x;
    let py = global_id.y;

    if (px >= params.width || py >= params.height) {{
        return;
    }}

    let aspect = f32(params.width) / f32(params.height);
    let scale_x = 3.0 / (params.zoom * f32(params.width));
    let scale_y = 3.0 / (params.zoom * f32(params.height));

    let c_re = params.center_re + (f32(px) - f32(params.width) * 0.5) * scale_x * aspect;
    let c_im = params.center_im + (f32(py) - f32(params.height) * 0.5) * scale_y;

    var z_re: f32 = 0.0;
    var z_im: f32 = 0.0;
    var iter: u32 = 0u;

    while (z_re * z_re + z_im * z_im <= 4.0 && iter < {max_iters}u) {{
        let next_re = z_re * z_re - z_im * z_im + c_re;
        let next_im = 2.0 * z_re * z_im + c_im;
        z_re = next_re;
        z_im = next_im;
        iter = iter + 1u;
    }}

    let idx = py * params.width + px;
    pixel_buffer[idx] = iter;
}}
"#
        );

        GpuKernelDescriptor {
            kernel_name: "complex_fractal".into(),
            workgroup_size: [workgroup_size_xy, workgroup_size_xy, 1],
            wgsl_source: source,
            input_buffer_count: 0,
            output_buffer_count: 1,
        }
    }
}
