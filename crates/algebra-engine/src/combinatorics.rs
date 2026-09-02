//! # `algebra_engine::combinatorics`
//!
//! Discrete Combinatorics:
//! - Binomial coefficients $\binom{n}{k}$
//! - Permutations $P(n, k)$ & Derangements $!n$
//! - Integer partitions $p(n)$
//! - Stirling numbers of the 1st $S_1(n, k)$ and 2nd $S_2(n, k)$ kind
//! - Bell numbers $B_n$
//! - Catalan numbers $C_n = \frac{1}{n+1}\binom{2n}{n}$

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Factorial $n!$.
pub fn factorial(n: u64) -> BigUint {
    let mut res = BigUint::one();
    for i in 2..=n {
        res *= BigUint::from(i);
    }
    res
}

/// Binomial coefficient $\binom{n}{k} = \frac{n!}{k!(n-k)!}$.
pub fn combinations(n: u64, k: u64) -> BigUint {
    if k > n {
        return BigUint::zero();
    }
    if k == 0 || k == n {
        return BigUint::one();
    }
    let k = k.min(n - k);
    let mut num = BigUint::one();
    let mut den = BigUint::one();
    for i in 1..=k {
        num *= BigUint::from(n - k + i);
        den *= BigUint::from(i);
    }
    num / den
}

/// Permutations $P(n, k) = \frac{n!}{(n-k)!}$.
pub fn permutations(n: u64, k: u64) -> BigUint {
    if k > n {
        return BigUint::zero();
    }
    let mut res = BigUint::one();
    for i in (n - k + 1)..=n {
        res *= BigUint::from(i);
    }
    res
}

/// Derangements (subfactorial) $!n = (n - 1)(!(n-1) + !(n-2))$.
pub fn derangements(n: u64) -> BigUint {
    if n == 0 {
        return BigUint::one();
    }
    if n == 1 {
        return BigUint::zero();
    }
    let mut prev2 = BigUint::one();
    let mut prev1 = BigUint::zero();
    let mut curr = BigUint::zero();
    for i in 2..=n {
        curr = BigUint::from(i - 1) * (&prev1 + &prev2);
        prev2 = prev1;
        prev1 = curr.clone();
    }
    curr
}

/// $n$-th Catalan number $C_n = \frac{1}{n+1}\binom{2n}{n}$.
pub fn catalan_number(n: u64) -> BigUint {
    combinations(2 * n, n) / BigUint::from(n + 1)
}

/// Stirling numbers of the 2nd kind $S(n, k)$ (partitions of $n$ elements into $k$ non-empty subsets).
pub fn stirling_second_kind(n: u32, k: u32) -> BigUint {
    if n == 0 && k == 0 {
        return BigUint::one();
    }
    if n == 0 || k == 0 || k > n {
        return BigUint::zero();
    }
    let mut dp = vec![vec![BigUint::zero(); (k + 1) as usize]; (n + 1) as usize];
    dp[0][0] = BigUint::one();
    for i in 1..=(n as usize) {
        for j in 1..=(k as usize) {
            dp[i][j] = &dp[i - 1][j - 1] + BigUint::from(j as u64) * &dp[i - 1][j];
        }
    }
    dp[n as usize][k as usize].clone()
}

/// $n$-th Bell number $B_n = \sum_{k=0}^n S(n, k)$.
pub fn bell_number(n: u32) -> BigUint {
    let mut sum = BigUint::zero();
    for k in 0..=n {
        sum += stirling_second_kind(n, k);
    }
    sum
}

/// Number of integer partitions $p(n)$ via Euler's pentagonal number recurrence.
pub fn integer_partitions(n: usize) -> BigUint {
    if n == 0 {
        return BigUint::one();
    }
    let mut p = vec![BigUint::zero(); n + 1];
    p[0] = BigUint::one();

    for i in 1..=n {
        let mut sum = BigUint::zero();
        let mut k: i64 = 1;
        loop {
            // Pentagonal numbers: g1 = k(3k - 1)/2, g2 = k(3k + 1)/2
            let g1 = (k * (3 * k - 1) / 2) as usize;
            let g2 = (k * (3 * k + 1) / 2) as usize;
            let sign = if k % 2 != 0 { 1 } else { -1 };

            if g1 > i && g2 > i {
                break;
            }

            if g1 <= i {
                if sign > 0 {
                    sum += &p[i - g1];
                } else {
                    sum = if sum >= p[i - g1] {
                        sum - &p[i - g1]
                    } else {
                        BigUint::zero()
                    };
                }
            }
            if g2 <= i {
                if sign > 0 {
                    sum += &p[i - g2];
                } else {
                    sum = if sum >= p[i - g2] {
                        sum - &p[i - g2]
                    } else {
                        BigUint::zero()
                    };
                }
            }
            k += 1;
        }
        p[i] = sum;
    }
    p[n].clone()
}
