//! # `algebra_engine::hyperop`
//!
//! Hyperoperations, Knuth's Up-Arrow notation ($a \uparrow^n b$), Tetration, Pentation,
//! Ackermann function, and Super-logarithms.

use num_bigint::BigUint;
use num_traits::One;

/// Compute general hyperoperation $H_n(a, b)$:
/// - $H_0(a, b) = b + 1$ (Successor)
/// - $H_1(a, b) = a + b$ (Addition)
/// - $H_2(a, b) = a \times b$ (Multiplication)
/// - $H_3(a, b) = a^b$ (Exponentiation / $a \uparrow b$)
/// - $H_4(a, b) = a \uparrow\uparrow b$ (Tetration)
/// - $H_5(a, b) = a \uparrow\uparrow\uparrow b$ (Pentation)
pub fn hyperoperation(n: u32, a: u64, b: u64) -> BigUint {
    match n {
        0 => BigUint::from(b + 1),
        1 => BigUint::from(a + b),
        2 => BigUint::from(a) * BigUint::from(b),
        3 => {
            let base = BigUint::from(a);
            base.pow(b as u32)
        }
        4 => tetration(a, b),
        5 => pentation(a, b),
        _ => knuth_up_arrow(a, n - 2, b),
    }
}

/// Knuth's up-arrow notation $a \uparrow^k b$.
/// `arrows` = 1 corresponds to $a^b$.
/// `arrows` = 2 corresponds to $a \uparrow\uparrow b$ (Tetration).
/// `arrows` = 3 corresponds to $a \uparrow\uparrow\uparrow b$ (Pentation).
pub fn knuth_up_arrow(a: u64, arrows: u32, b: u64) -> BigUint {
    if arrows == 0 {
        return BigUint::from(a) * BigUint::from(b);
    }
    if arrows == 1 {
        return BigUint::from(a).pow(b as u32);
    }
    if b == 0 {
        return BigUint::one();
    }
    if b == 1 {
        return BigUint::from(a);
    }

    // a \uparrow^k b = a \uparrow^{k-1} (a \uparrow^k (b - 1))
    let prev_b = knuth_up_arrow(a, arrows, b - 1);
    let prev_b_u64 = prev_b.to_u64_digits().first().copied().unwrap_or(0);
    knuth_up_arrow(a, arrows - 1, prev_b_u64)
}

/// Tetration $a \uparrow\uparrow b = a^{a^{\dots^a}}$ ($b$ copies of $a$).
pub fn tetration(a: u64, b: u64) -> BigUint {
    if b == 0 {
        return BigUint::one();
    }
    if b == 1 {
        return BigUint::from(a);
    }
    let mut res = BigUint::from(a);
    for _ in 1..b {
        let exp = res.to_u64_digits().first().copied().unwrap_or(0);
        res = BigUint::from(a).pow(exp as u32);
    }
    res
}

/// Pentation $a \uparrow\uparrow\uparrow b$.
pub fn pentation(a: u64, b: u64) -> BigUint {
    if b == 0 {
        return BigUint::one();
    }
    let mut res = BigUint::from(a);
    for _ in 1..b {
        let exp = res.to_u64_digits().first().copied().unwrap_or(0);
        res = tetration(a, exp);
    }
    res
}

/// Ackermann function $A(m, n)$:
/// - $A(0, n) = n + 1$
/// - $A(m, 0) = A(m - 1, 1)$
/// - $A(m, n) = A(m - 1, A(m, n - 1))$
pub fn ackermann(m: u64, n: u64) -> u64 {
    match (m, n) {
        (0, n) => n + 1,
        (m, 0) => ackermann(m - 1, 1),
        (m, n) => {
            let inner = ackermann(m, n - 1);
            ackermann(m - 1, inner)
        }
    }
}

/// Continuous super-logarithm approximation $\text{slog}_a(x)$ such that $\text{slog}_a(a \uparrow\uparrow y) = y$.
pub fn super_log(base: f64, mut x: f64) -> f64 {
    if x <= 0.0 {
        return -1.0;
    }
    if (x - 1.0).abs() < 1e-12 {
        return 0.0;
    }
    let mut count = 0.0;
    while x > 1.0 {
        x = x.log(base);
        count += 1.0;
    }
    count + x - 1.0
}
