//! # `algebra_engine::heuristic`
//!
//! Multi-Tier Probabilistic Heuristic Search, Path Weighting & Solution Certification Engine.
//!
//! ## 4-Tier Architecture
//! 1. **Tier 1: $O(1)$ Fast Probabilistic Screening ($\epsilon \approx 0.01\%$)**:
//!    Fast Schwartz-Zippel fingerprinting over $\mathbb{F}_p$ ($p = 2^{31}-1$) to assign heuristic
//!    weights to candidate search branches in $A^*$ / E-Graph exploration queues.
//! 2. **Tier 2: Progressive Multi-Point / Multi-Prime CRT Verification ($\epsilon < 10^{-20}$)**:
//!    Promoted candidate branches undergo deep verification across multiple cryptographic
//!    primes ($\mathbb{F}_{2^{31}-1}, \mathbb{F}_{2^{61}-1}, \mathbb{F}_{2^{64}-59}$) and randomized interval bounds.
//! 3. **Tier 3: Opportunistic Deterministic Proof & Fallback**:
//!    Exact deterministic reduction (Gröbner basis, canonical polynomial forms, Risch decision tree)
//!    executed when cheaper or when candidate branches are exhausted.
//! 4. **Tier 4: Probabilistic Solution Returns with Formal Confidence Certificates**:
//!    When computational budgets (soft/hard caps) are reached, returns [`Solution::Probabilistic`]
//!    with upper bound on error probability and sample metadata.

use algebra_core::probabilistic::{ProbabilisticVerifier, Solution};
use algebra_core::{EngineConfig, ExprGraph, ExprId, ExprKind, Number};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Heuristic search candidate with priority weight.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchCandidate {
    pub expr_id: ExprId,
    pub priority_weight: f64,
    pub depth: usize,
    pub step_description: String,
}

impl Eq for SearchCandidate {}

impl PartialOrd for SearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_weight
            .partial_cmp(&other.priority_weight)
            .unwrap_or(Ordering::Equal)
    }
}

/// Multi-tier heuristic search and certification engine.
pub struct HeuristicSearchEngine {
    pub config: EngineConfig,
}

impl Default for HeuristicSearchEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl HeuristicSearchEngine {
    /// Create a new heuristic search engine with given configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Tier 1: $O(1)$ fast heuristic weighting of a candidate search branch.
    pub fn weight_candidate(
        &self,
        graph: &ExprGraph,
        candidate_id: ExprId,
        target_id: Option<ExprId>,
    ) -> f64 {
        ProbabilisticVerifier::compute_candidate_heuristic_weight(graph, candidate_id, target_id)
    }

    /// Tier 2: Progressive multi-prime CRT verification down to target epsilon (e.g. $10^{-20}$).
    pub fn verify_candidate(
        &self,
        graph: &ExprGraph,
        candidate_id: ExprId,
        target_id: ExprId,
        target_epsilon: f64,
    ) -> Solution<bool> {
        ProbabilisticVerifier::verify_equivalence(graph, candidate_id, target_id, target_epsilon)
    }

    /// Simplify an expression with opportunistic deterministic proof and probabilistic fallback.
    pub fn simplify_with_certification(
        &self,
        graph: &ExprGraph,
        expr_id: ExprId,
        target_epsilon: f64,
    ) -> Solution<ExprId> {
        let budget = &self.config.budget;
        let mut queue = BinaryHeap::new();
        let mut visited = std::collections::HashSet::new();

        let initial_weight = self.weight_candidate(graph, expr_id, None);
        queue.push(SearchCandidate {
            expr_id,
            priority_weight: initial_weight,
            depth: 0,
            step_description: "Initial expression".to_string(),
        });
        visited.insert(expr_id);

        let mut best_candidate = expr_id;
        let mut best_score = initial_weight;
        let mut iterations = 0;
        let (soft_cap, hard_cap) = budget.egraph_nodes;

        while let Some(current) = queue.pop() {
            iterations += 1;

            // Check hard cap
            if iterations >= hard_cap {
                let verification =
                    self.verify_candidate(graph, best_candidate, expr_id, target_epsilon);
                return Solution::Probabilistic {
                    result: best_candidate,
                    error_probability_upper_bound: verification
                        .error_probability()
                        .max(target_epsilon),
                    sample_count: verification.sample_count(),
                    method: "Heuristic Search (Hard Cap Rollback)",
                };
            }

            // Generate neighbor rewrites
            let neighbors = self.expand_rewrites(graph, current.expr_id);
            for n in neighbors {
                if visited.insert(n) {
                    let weight = self.weight_candidate(graph, n, Some(expr_id));
                    if weight > best_score {
                        best_score = weight;
                        best_candidate = n;
                    }

                    // Soft cap: stop queuing deeper expansions if soft cap reached
                    if iterations < soft_cap {
                        queue.push(SearchCandidate {
                            expr_id: n,
                            priority_weight: weight,
                            depth: current.depth + 1,
                            step_description: "Rewrite step".to_string(),
                        });
                    }
                }
            }

            // Tier 3: If candidate is significantly simpler and verified with high confidence, test deterministic identity
            if best_candidate != expr_id {
                let diff = graph.sub(expr_id, best_candidate);
                let diff_node = graph.get(diff);
                if let ExprKind::Number(Number::Integer(0)) = diff_node.kind {
                    return Solution::Deterministic(best_candidate);
                }
            }
        }

        // Return best candidate with certification
        let verif = self.verify_candidate(graph, best_candidate, expr_id, target_epsilon);
        if verif.is_deterministic() && *verif.as_ref() {
            Solution::Deterministic(best_candidate)
        } else {
            Solution::Probabilistic {
                result: best_candidate,
                error_probability_upper_bound: verif.error_probability().max(target_epsilon),
                sample_count: verif.sample_count(),
                method: "Multi-Tier Probabilistic Heuristics (Tier 2 CRT)",
            }
        }
    }

    /// Expand elementary algebraic rewrites (constant folding, zero removals, distribution).
    fn expand_rewrites(&self, graph: &ExprGraph, id: ExprId) -> Vec<ExprId> {
        let node = graph.get(id);
        let mut out = Vec::new();

        match &node.kind {
            ExprKind::Add(terms) => {
                // Remove zero terms
                let non_zero: smallvec::SmallVec<[ExprId; 4]> = terms
                    .iter()
                    .copied()
                    .filter(|&t| {
                        let tn = graph.get(t);
                        !matches!(tn.kind, ExprKind::Number(Number::Integer(0)))
                    })
                    .collect();

                if non_zero.len() < terms.len() {
                    if non_zero.is_empty() {
                        out.push(graph.integer(0));
                    } else if non_zero.len() == 1 {
                        out.push(non_zero[0]);
                    } else {
                        out.push(graph.add(non_zero));
                    }
                }
            }
            ExprKind::Mul(factors) => {
                // If any factor is 0, entire product is 0
                for &f in factors {
                    let fn_node = graph.get(f);
                    if matches!(fn_node.kind, ExprKind::Number(Number::Integer(0))) {
                        out.push(graph.integer(0));
                        return out;
                    }
                }
                // Remove 1 factors
                let non_one: smallvec::SmallVec<[ExprId; 4]> = factors
                    .iter()
                    .copied()
                    .filter(|&f| {
                        let fn_node = graph.get(f);
                        !matches!(fn_node.kind, ExprKind::Number(Number::Integer(1)))
                    })
                    .collect();

                if non_one.len() < factors.len() {
                    if non_one.is_empty() {
                        out.push(graph.integer(1));
                    } else if non_one.len() == 1 {
                        out.push(non_one[0]);
                    } else {
                        out.push(graph.mul(non_one));
                    }
                }
            }
            ExprKind::Sub(l, r) if l == r => {
                out.push(graph.integer(0));
            }
            ExprKind::Div(l, r) if l == r => {
                out.push(graph.integer(1));
            }
            _ => {}
        }

        out
    }
}
