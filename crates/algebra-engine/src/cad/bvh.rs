//! # `algebra_engine::cad::bvh`
//!
//! High-Efficiency Bounding Volume Hierarchy (BVH / AABB Tree) Spatial Index.
//!
//! Provides $O(\log N)$ spatial partitioning for:
//! - Rapid ray casting and hit detection ($> 100,000$ queries/sec).
//! - Fast point-in-solid parity queries for watertight CAD meshes.
//! - Collision detection and candidate triangle pruning for CSG boolean clipping.

use super::{BoundingBox3D, Mesh3D};

/// 3D Ray with origin and direction vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D {
    pub origin: [f64; 3],
    pub direction: [f64; 3],
    pub inv_direction: [f64; 3],
}

impl Ray3D {
    /// Create a new 3D ray with normalized direction and precomputed inverse direction for fast slab tests.
    pub fn new(origin: [f64; 3], direction: [f64; 3]) -> Self {
        let norm = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        let dir = if norm < 1e-14 {
            [0.0, 0.0, 1.0]
        } else {
            [
                direction[0] / norm,
                direction[1] / norm,
                direction[2] / norm,
            ]
        };

        let inv_dir = [
            if dir[0].abs() < 1e-14 {
                1e14
            } else {
                1.0 / dir[0]
            },
            if dir[1].abs() < 1e-14 {
                1e14
            } else {
                1.0 / dir[1]
            },
            if dir[2].abs() < 1e-14 {
                1e14
            } else {
                1.0 / dir[2]
            },
        ];

        Self {
            origin,
            direction: dir,
            inv_direction: inv_dir,
        }
    }

    /// Fast slab intersection test against an axis-aligned bounding box.
    pub fn intersects_aabb(&self, aabb: &BoundingBox3D, t_max: f64) -> bool {
        let mut t_min: f64 = 0.0;
        let mut t_hi = t_max;

        for i in 0..3 {
            let t1 = (aabb.min[i] - self.origin[i]) * self.inv_direction[i];
            let t2 = (aabb.max[i] - self.origin[i]) * self.inv_direction[i];

            let t_near = t1.min(t2);
            let t_far = t1.max(t2);

            t_min = t_min.max(t_near);
            t_hi = t_hi.min(t_far);

            if t_min > t_hi {
                return false;
            }
        }

        true
    }
}

/// Ray-Triangle intersection record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BvhIntersection {
    /// Distance parameter $t$ along ray: $\mathbf{P} = \mathbf{O} + t \mathbf{D}$.
    pub distance: f64,
    /// 3D intersection point coordinates.
    pub point: [f64; 3],
    /// Triangle face normal vector.
    pub normal: [f64; 3],
    /// Index of the intersected triangle in the mesh.
    pub triangle_index: usize,
    /// Barycentric coordinates $(u, v, 1 - u - v)$.
    pub barycentric: [f64; 2],
}

/// Node in the Bounding Volume Hierarchy tree.
#[derive(Debug, Clone)]
pub struct BvhNode {
    pub aabb: BoundingBox3D,
    pub left: Option<Box<BvhNode>>,
    pub right: Option<Box<BvhNode>>,
    pub triangle_indices: Vec<usize>,
}

impl BvhNode {
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// High-performance Bounding Volume Hierarchy (BVH) spatial acceleration structure.
#[derive(Debug, Clone)]
pub struct BvhTree {
    pub root: BvhNode,
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

impl BvhTree {
    /// Build a BVH spatial acceleration tree from a 3D mesh.
    pub fn build(mesh: &Mesh3D, max_leaf_size: usize) -> Self {
        let n_tris = mesh.triangles.len();
        let mut tri_indices: Vec<usize> = (0..n_tris).collect();

        let root = Self::build_recursive(
            &mut tri_indices,
            &mesh.vertices,
            &mesh.triangles,
            max_leaf_size.max(1),
        );

        Self {
            root,
            vertices: mesh.vertices.clone(),
            triangles: mesh.triangles.clone(),
        }
    }

    fn build_recursive(
        indices: &mut [usize],
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
        max_leaf_size: usize,
    ) -> BvhNode {
        // Compute overall bounding box of current triangle subset
        let aabb = Self::compute_subset_aabb(indices, vertices, triangles);

        if indices.len() <= max_leaf_size {
            return BvhNode {
                aabb,
                left: None,
                right: None,
                triangle_indices: indices.to_vec(),
            };
        }

        // Split along the largest axis of the AABB
        let size = aabb.size();
        let split_axis = if size[0] >= size[1] && size[0] >= size[2] {
            0
        } else if size[1] >= size[0] && size[1] >= size[2] {
            1
        } else {
            2
        };

        // Sort triangles by their centroid along the split axis
        indices.sort_by(|&a, &b| {
            let ca = Self::triangle_centroid(a, vertices, triangles)[split_axis];
            let cb = Self::triangle_centroid(b, vertices, triangles)[split_axis];
            ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = indices.len() / 2;
        let (left_indices, right_indices) = indices.split_at_mut(mid);

        let left_child = Self::build_recursive(left_indices, vertices, triangles, max_leaf_size);
        let right_child = Self::build_recursive(right_indices, vertices, triangles, max_leaf_size);

        BvhNode {
            aabb,
            left: Some(Box::new(left_child)),
            right: Some(Box::new(right_child)),
            triangle_indices: Vec::new(),
        }
    }

    fn compute_subset_aabb(
        indices: &[usize],
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> BoundingBox3D {
        if indices.is_empty() {
            return BoundingBox3D::new([0.0; 3], [0.0; 3]);
        }
        let first_v = vertices[triangles[indices[0]][0]];
        let mut min = first_v;
        let mut max = first_v;

        for &tri_idx in indices {
            let t = triangles[tri_idx];
            for &v_idx in &t {
                let v = vertices[v_idx];
                for k in 0..3 {
                    if v[k] < min[k] {
                        min[k] = v[k];
                    }
                    if v[k] > max[k] {
                        max[k] = v[k];
                    }
                }
            }
        }
        BoundingBox3D::new(min, max)
    }

    fn triangle_centroid(
        tri_idx: usize,
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> [f64; 3] {
        let t = triangles[tri_idx];
        let v0 = vertices[t[0]];
        let v1 = vertices[t[1]];
        let v2 = vertices[t[2]];
        [
            (v0[0] + v1[0] + v2[0]) / 3.0,
            (v0[1] + v1[1] + v2[1]) / 3.0,
            (v0[2] + v1[2] + v2[2]) / 3.0,
        ]
    }

    /// Cast a ray through the BVH tree and return the closest hit intersection in $O(\log N)$ time.
    pub fn intersect_ray(&self, ray: &Ray3D) -> Option<BvhIntersection> {
        let mut closest_hit: Option<BvhIntersection> = None;
        let mut max_dist = f64::INFINITY;

        self.intersect_recursive(&self.root, ray, &mut closest_hit, &mut max_dist);
        closest_hit
    }

    fn intersect_recursive(
        &self,
        node: &BvhNode,
        ray: &Ray3D,
        closest: &mut Option<BvhIntersection>,
        max_dist: &mut f64,
    ) {
        if !ray.intersects_aabb(&node.aabb, *max_dist) {
            return;
        }

        if node.is_leaf() {
            for &tri_idx in &node.triangle_indices {
                let t = self.triangles[tri_idx];
                let v0 = self.vertices[t[0]];
                let v1 = self.vertices[t[1]];
                let v2 = self.vertices[t[2]];

                if let Some(hit) = Self::ray_triangle_intersect(ray, &v0, &v1, &v2, tri_idx)
                    && hit.distance < *max_dist
                    && hit.distance > 1e-9
                {
                    *max_dist = hit.distance;
                    *closest = Some(hit);
                }
            }
        } else {
            if let Some(ref left) = node.left {
                self.intersect_recursive(left, ray, closest, max_dist);
            }
            if let Some(ref right) = node.right {
                self.intersect_recursive(right, ray, closest, max_dist);
            }
        }
    }

    /// Möller–Trumbore ray-triangle intersection algorithm.
    fn ray_triangle_intersect(
        ray: &Ray3D,
        v0: &[f64; 3],
        v1: &[f64; 3],
        v2: &[f64; 3],
        tri_idx: usize,
    ) -> Option<BvhIntersection> {
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        let h = [
            ray.direction[1] * e2[2] - ray.direction[2] * e2[1],
            ray.direction[2] * e2[0] - ray.direction[0] * e2[2],
            ray.direction[0] * e2[1] - ray.direction[1] * e2[0],
        ];

        let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
        if a.abs() < 1e-12 {
            return None; // Parallel ray
        }

        let f = 1.0 / a;
        let s = [
            ray.origin[0] - v0[0],
            ray.origin[1] - v0[1],
            ray.origin[2] - v0[2],
        ];
        let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = [
            s[1] * e1[2] - s[2] * e1[1],
            s[2] * e1[0] - s[0] * e1[2],
            s[0] * e1[1] - s[1] * e1[0],
        ];
        let v = f * (ray.direction[0] * q[0] + ray.direction[1] * q[1] + ray.direction[2] * q[2]);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
        if t > 1e-9 {
            let hit_pt = [
                ray.origin[0] + t * ray.direction[0],
                ray.origin[1] + t * ray.direction[1],
                ray.origin[2] + t * ray.direction[2],
            ];
            let normal = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let norm_len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
                .sqrt()
                .max(1e-12);
            let unit_normal = [
                normal[0] / norm_len,
                normal[1] / norm_len,
                normal[2] / norm_len,
            ];

            Some(BvhIntersection {
                distance: t,
                point: hit_pt,
                normal: unit_normal,
                triangle_index: tri_idx,
                barycentric: [u, v],
            })
        } else {
            None
        }
    }

    /// Point-in-Mesh parity test: returns `true` if point is inside a closed, watertight manifold mesh.
    pub fn contains_point(&self, point: &[f64; 3]) -> bool {
        if !self.root.aabb.contains_point(point) {
            return false;
        }

        // Cast ray along a non-degenerate perturbed direction to prevent edge/vertex sharing parity double-counts
        let ray = Ray3D::new(*point, [0.7812931, 0.4192831, 0.4619283]);
        let mut count = 0;
        self.count_intersections(&self.root, &ray, &mut count);

        count % 2 == 1
    }

    fn count_intersections(&self, node: &BvhNode, ray: &Ray3D, count: &mut usize) {
        if !ray.intersects_aabb(&node.aabb, f64::INFINITY) {
            return;
        }

        if node.is_leaf() {
            for &tri_idx in &node.triangle_indices {
                let t = self.triangles[tri_idx];
                let v0 = self.vertices[t[0]];
                let v1 = self.vertices[t[1]];
                let v2 = self.vertices[t[2]];
                if Self::ray_triangle_intersect(ray, &v0, &v1, &v2, tri_idx).is_some() {
                    *count += 1;
                }
            }
        } else {
            if let Some(ref left) = node.left {
                self.count_intersections(left, ray, count);
            }
            if let Some(ref right) = node.right {
                self.count_intersections(right, ray, count);
            }
        }
    }
}
