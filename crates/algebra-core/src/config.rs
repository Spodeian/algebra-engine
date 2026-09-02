//! # `algebra_core::config`
//!
//! Configurable Resource Budgets, Soft and Hard Caps, Graceful Degradation Policies,
//! and System Hardware-Aware Profile Tuning.
//!
//! ## Resource Budget Architecture
//! To prevent out-of-memory errors or infinite compute loops across diverse targets
//! (from embedded WebAssembly to multi-terabyte HPC nodes), URAE employs a two-tier
//! **Soft vs. Hard Cap Envelope**:
//!
//! - **Soft Cap (Graceful Degradation)**: When exceeded, the engine alters its strategy
//!   (e.g., disables expanding rewrite rules in E-Graphs, truncates Gröbner S-pairs
//!   by sugar degree, or appends an asymptotic remainder $O(x^N)$ to series).
//! - **Hard Cap (Safety Rollback)**: The absolute execution ceiling. Surpassing this
//!   aborts computation cleanly and returns `AlgebraError::EvaluationError("Resource limit exceeded")`.

use std::env;

/// Evaluation status against configured resource envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetStatus {
    /// Resource consumption is within normal operating limits.
    WithinLimits,
    /// Soft cap exceeded: apply graceful degradation / heuristics.
    SoftCapExceeded {
        resource: &'static str,
        current: usize,
        soft_cap: usize,
    },
    /// Hard cap exceeded: execution must abort immediately.
    HardCapExceeded {
        resource: &'static str,
        current: usize,
        hard_cap: usize,
    },
}

/// Resource Budget Envelopes for URAE computation engines.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceBudget {
    /// E-Graph maximum e-nodes capacity (soft cap, hard cap).
    pub egraph_nodes: (usize, usize),
    /// Gröbner Basis S-pair queue limit (soft cap, hard cap).
    pub grobner_spairs: (usize, usize),
    /// Numerical ODE & PDE maximum integration steps (soft cap, hard cap).
    pub ode_steps: (usize, usize),
    /// Non-linear iterative solver maximum iterations (soft cap, hard cap).
    pub solver_iterations: (usize, usize),
    /// Asymptotic Series expansion maximum order $N$ (soft cap, hard cap).
    pub series_order: (usize, usize),
    /// Knot & Braid maximum crossing number (soft cap, hard cap).
    pub knot_crossings: (usize, usize),
    /// Wall-clock timeout limit in seconds.
    pub timeout_secs: f64,
    /// Maximum virtual memory allocation limit in megabytes.
    pub max_memory_mb: usize,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl ResourceBudget {
    /// Standard Desktop Profile.
    pub const DEFAULT: Self = Self {
        egraph_nodes: (8_000, 25_000),
        grobner_spairs: (5_000, 50_000),
        ode_steps: (2_000, 20_000),
        solver_iterations: (30, 100),
        series_order: (12, 50),
        knot_crossings: (16, 32),
        timeout_secs: 5.0,
        max_memory_mb: 512,
    };

    /// High-Performance Cluster (HPC) Profile for large workstation computations.
    pub const HPC: Self = Self {
        egraph_nodes: (250_000, 1_000_000),
        grobner_spairs: (1_000_000, 5_000_000),
        ode_steps: (500_000, 2_000_000),
        solver_iterations: (1_000, 5_000),
        series_order: (250, 1_000),
        knot_crossings: (128, 512),
        timeout_secs: 60.0,
        max_memory_mb: 32_768,
    };

    /// Embedded & WebAssembly Profile for constrained in-browser execution.
    pub const EMBEDDED: Self = Self {
        egraph_nodes: (2_000, 6_000),
        grobner_spairs: (100, 1_000),
        ode_steps: (500, 2_000),
        solver_iterations: (20, 50),
        series_order: (8, 20),
        knot_crossings: (8, 16),
        timeout_secs: 1.5,
        max_memory_mb: 64,
    };

    /// High-Performance Cluster (HPC) Profile for large workstation computations.
    #[inline]
    pub const fn hpc() -> Self {
        Self::HPC
    }

    /// Embedded & WebAssembly Profile for constrained in-browser execution.
    #[inline]
    pub const fn embedded() -> Self {
        Self::EMBEDDED
    }

    /// Check if E-Graph node count is within limits.
    #[inline]
    pub const fn check_egraph_nodes(&self, current: usize) -> BudgetStatus {
        let (soft, hard) = self.egraph_nodes;
        if current >= hard {
            BudgetStatus::HardCapExceeded {
                resource: "E-Graph Nodes",
                current,
                hard_cap: hard,
            }
        } else if current >= soft {
            BudgetStatus::SoftCapExceeded {
                resource: "E-Graph Nodes",
                current,
                soft_cap: soft,
            }
        } else {
            BudgetStatus::WithinLimits
        }
    }

    /// Check if Gröbner S-pairs are within limits.
    #[inline]
    pub const fn check_spairs(&self, current: usize) -> BudgetStatus {
        let (soft, hard) = self.grobner_spairs;
        if current >= hard {
            BudgetStatus::HardCapExceeded {
                resource: "Gröbner S-Pairs",
                current,
                hard_cap: hard,
            }
        } else if current >= soft {
            BudgetStatus::SoftCapExceeded {
                resource: "Gröbner S-Pairs",
                current,
                soft_cap: soft,
            }
        } else {
            BudgetStatus::WithinLimits
        }
    }

    /// Apply environment variable overrides if present.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("URAE_MAX_EGRAPH_NODES") {
            if let Ok(n) = val.parse::<usize>() {
                self.egraph_nodes = (n / 2, n);
            }
        }
        if let Ok(val) = env::var("URAE_MAX_SPAIRS") {
            if let Ok(n) = val.parse::<usize>() {
                self.grobner_spairs = (n / 2, n);
            }
        }
        if let Ok(val) = env::var("URAE_TIMEOUT_SECS") {
            if let Ok(t) = val.parse::<f64>() {
                self.timeout_secs = t;
            }
        }
        if let Ok(val) = env::var("URAE_MAX_MEMORY_MB") {
            if let Ok(m) = val.parse::<usize>() {
                self.max_memory_mb = m;
            }
        }
    }
}

/// Global Engine Configuration with heuristics and execution flags.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineConfig {
    /// Active resource budget envelope.
    pub budget: ResourceBudget,
    /// Enable Schwartz-Zippel probabilistic equivalence pre-filter $O(1)$.
    pub enable_schwartz_zippel: bool,
    /// Enable Rayon multi-core parallel compute routines.
    pub enable_parallelism: bool,
    /// Enable AST arena node recycling across evaluations.
    pub enable_arena_recycling: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let mut budget = ResourceBudget::default();
        budget.apply_env_overrides();
        Self {
            budget,
            enable_schwartz_zippel: true,
            enable_parallelism: true,
            enable_arena_recycling: true,
        }
    }
}

impl EngineConfig {
    /// Auto-detect available system hardware (RAM & logical CPU cores).
    pub fn detect_hardware() -> Self {
        let mut config = Self::default();
        #[cfg(not(target_arch = "wasm32"))]
        {
            let cores = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(4);

            if cores >= 16 {
                config.budget = ResourceBudget::hpc();
            }
        }
        config.budget.apply_env_overrides();
        config
    }

    /// Preset HPC profile.
    pub fn hpc() -> Self {
        let mut config = Self {
            budget: ResourceBudget::hpc(),
            enable_schwartz_zippel: true,
            enable_parallelism: true,
            enable_arena_recycling: true,
        };
        config.budget.apply_env_overrides();
        config
    }

    /// Preset Embedded / WASM profile.
    pub fn embedded() -> Self {
        Self {
            budget: ResourceBudget::embedded(),
            enable_schwartz_zippel: true,
            enable_parallelism: false,
            enable_arena_recycling: true,
        }
    }

    /// Builder method to override max egraph nodes.
    pub fn with_max_egraph_nodes(mut self, soft: usize, hard: usize) -> Self {
        self.budget.egraph_nodes = (soft, hard);
        self
    }

    /// Builder method to override max Gröbner S-pairs.
    pub fn with_max_spairs(mut self, soft: usize, hard: usize) -> Self {
        self.budget.grobner_spairs = (soft, hard);
        self
    }

    /// Builder method to override timeout.
    pub fn with_timeout_secs(mut self, secs: f64) -> Self {
        self.budget.timeout_secs = secs;
        self
    }
}
