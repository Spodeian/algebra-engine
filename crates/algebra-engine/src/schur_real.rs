//! Real Schur decomposition `A = Q T Q^T` (Q orthogonal, T quasi upper triangular) in floating point: Householder reduction
//! to Hessenberg form, then the Francis double-shift QR iteration with deflation. Complex conjugate eigenvalues appear as 2x2
//! blocks on the diagonal of T; real eigenvalues are triangularised exactly by a plane rotation.

type M = Vec<Vec<f64>>;

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Householder vector `u` with `(I - 2 u u^T / u^T u) x = alpha e1`; `None` when there is nothing to annihilate.
fn householder(x: &[f64]) -> Option<(Vec<f64>, f64)> {
    let norm = dot(x, x).sqrt();
    if norm == 0.0 {
        return None;
    }
    let alpha = if x[0] >= 0.0 { -norm } else { norm };
    let mut u = x.to_vec();
    u[0] -= alpha;
    if dot(&u, &u) == 0.0 {
        return None;
    }
    Some((u, alpha))
}

/// Apply the reflector from the right to columns `c0..c0 + u.len()`, rows `0..rows`.
fn reflect_cols(m: &mut M, c0: usize, u: &[f64], rows: usize) {
    let uu = dot(u, u);
    for row in m.iter_mut().take(rows) {
        let d: f64 = u.iter().enumerate().map(|(j, uj)| row[c0 + j] * uj).sum();
        let f = 2.0 * d / uu;
        for (j, uj) in u.iter().enumerate() {
            row[c0 + j] -= f * uj;
        }
    }
}

/// Apply the reflector from the left to rows `r0..r0 + u.len()`, columns `c0..cols`.
fn reflect_rows(m: &mut M, r0: usize, u: &[f64], c0: usize, cols: usize) {
    let uu = dot(u, u);
    for c in c0..cols {
        let d: f64 = u.iter().enumerate().map(|(i, ui)| m[r0 + i][c] * ui).sum();
        let f = 2.0 * d / uu;
        for (i, ui) in u.iter().enumerate() {
            m[r0 + i][c] -= f * ui;
        }
    }
}

fn hessenberg(h: &mut M, q: &mut M) {
    let n = h.len();
    for k in 0..n.saturating_sub(2) {
        let x: Vec<f64> = (k + 1..n).map(|i| h[i][k]).collect();
        let Some((u, alpha)) = householder(&x) else {
            continue;
        };
        let r0 = k + 1;
        reflect_rows(h, r0, &u, k, n);
        reflect_cols(h, r0, &u, n);
        reflect_cols(q, r0, &u, n);
        h[r0][k] = alpha;
        for row in h.iter_mut().skip(r0 + 1) {
            row[k] = 0.0;
        }
    }
}

/// One Francis double-shift sweep on the active window `l..=hi` (at least three rows).
fn francis_step(h: &mut M, q: &mut M, l: usize, hi: usize, exceptional: bool) {
    let n = h.len();
    let (s, t) = if exceptional {
        let s = 1.5 * (h[hi][hi - 1].abs() + h[hi - 1][hi - 2].abs());
        (s, s * s)
    } else {
        let s = h[hi - 1][hi - 1] + h[hi][hi];
        let t = h[hi - 1][hi - 1] * h[hi][hi] - h[hi - 1][hi] * h[hi][hi - 1];
        (s, t)
    };
    let mut x: Vec<f64> = vec![
        h[l][l] * h[l][l] + h[l][l + 1] * h[l + 1][l] - s * h[l][l] + t,
        h[l + 1][l] * (h[l][l] + h[l + 1][l + 1] - s),
        h[l + 1][l] * h[l + 2][l + 1],
    ];
    for k in l..hi {
        if k > l {
            let len = (hi - k + 1).min(3);
            x = (0..len).map(|i| h[k + i][k - 1]).collect();
        }
        let Some((u, alpha)) = householder(&x) else {
            continue;
        };
        let c0 = if k > l { k - 1 } else { l };
        reflect_rows(h, k, &u, c0, n);
        let rows = (k + 3).min(hi) + 1;
        reflect_cols(h, k, &u, rows);
        reflect_cols(q, k, &u, n);
        if k > l {
            h[k][k - 1] = alpha;
            for row in h.iter_mut().take(k + u.len()).skip(k + 1) {
                row[k - 1] = 0.0;
            }
        }
    }
}

/// A 2x2 diagonal block with real eigenvalues is rotated to upper triangular form; complex blocks are left alone.
fn standardise_2x2(h: &mut M, q: &mut M, p: usize) {
    let n = h.len();
    let (a, b, c, d) = (h[p][p], h[p][p + 1], h[p + 1][p], h[p + 1][p + 1]);
    let half = (a - d) / 2.0;
    let disc = half * half + b * c;
    if disc < 0.0 {
        return;
    }
    let lambda = (a + d) / 2.0 + disc.sqrt();
    let (v0, v1) = if (lambda - a).abs() + b.abs() >= (lambda - d).abs() + c.abs() {
        (b, lambda - a)
    } else {
        (lambda - d, c)
    };
    let r = v0.hypot(v1);
    if r == 0.0 {
        return;
    }
    let (cs, sn) = (v0 / r, v1 / r);
    // Right multiplication by G = [[cs, -sn], [sn, cs]] on columns p and p+1.
    for row in h.iter_mut() {
        let (x, y) = (row[p], row[p + 1]);
        row[p] = cs * x + sn * y;
        row[p + 1] = -sn * x + cs * y;
    }
    for row in q.iter_mut() {
        let (x, y) = (row[p], row[p + 1]);
        row[p] = cs * x + sn * y;
        row[p + 1] = -sn * x + cs * y;
    }
    // Left multiplication by G transposed on rows p and p+1.
    for col in 0..n {
        let (x, y) = (h[p][col], h[p + 1][col]);
        h[p][col] = cs * x + sn * y;
        h[p + 1][col] = -sn * x + cs * y;
    }
    h[p + 1][p] = 0.0;
}

/// The real Schur form of a square matrix: `(T, Q)` with `A = Q T Q^T`.
pub fn decompose(a: &[Vec<f64>]) -> Result<(M, M), String> {
    let n = a.len();
    if n == 0 || a.iter().any(|r| r.len() != n) {
        return Err("schur needs a non-empty square matrix".into());
    }
    let mut h = a.to_vec();
    let mut q: M = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    hessenberg(&mut h, &mut q);
    let norm = h
        .iter()
        .flatten()
        .map(|v| v.abs())
        .fold(0.0, f64::max)
        .max(f64::MIN_POSITIVE);
    let mut hi = n as isize - 1;
    let (mut iter, mut total) = (0usize, 0usize);
    while hi >= 0 {
        let top = hi as usize;
        let mut l = top;
        while l > 0 {
            let scale = h[l - 1][l - 1].abs() + h[l][l].abs();
            let scale = if scale == 0.0 { norm } else { scale };
            if h[l][l - 1].abs() <= f64::EPSILON * scale {
                h[l][l - 1] = 0.0;
                break;
            }
            l -= 1;
        }
        if l == top {
            hi -= 1;
            iter = 0;
            continue;
        }
        if l + 1 == top {
            standardise_2x2(&mut h, &mut q, l);
            hi -= 2;
            iter = 0;
            continue;
        }
        iter += 1;
        total += 1;
        if total > 100_000 {
            return Err("the QR iteration did not converge".into());
        }
        francis_step(&mut h, &mut q, l, top, iter % 10 == 0);
    }
    for (i, row) in h.iter_mut().enumerate() {
        for v in row.iter_mut().take(i.saturating_sub(1)) {
            *v = 0.0;
        }
    }
    Ok((h, q))
}
