//! # `urae_notebook::notebook::cache`
//!
//! Line Evaluation Cache and Text Hashing Subsystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Cached incremental evaluation state for a single document line.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineCacheEntry {
    pub text_hash: u64,
    pub evaluated_val: Option<f64>,
    pub output_unicode: String,
    pub output_latex: String,
}

/// In-memory cache for incremental evaluation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LineCache {
    pub entries: HashMap<usize, LineCacheEntry>,
}

/// Key for memoizing plot vertex evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlotCacheKey {
    pub expr_str: String,
    pub slider_hash: u64,
    pub min_bits: u64,
    pub max_bits: u64,
    pub samples: usize,
}

/// Fast thread-safe cache for rendered plot points.
#[derive(Debug, Default)]
pub struct PlotCache {
    entries: std::sync::RwLock<HashMap<PlotCacheKey, Vec<[f64; 2]>>>,
}

impl Clone for PlotCache {
    fn clone(&self) -> Self {
        let map = self.entries.read().map(|r| r.clone()).unwrap_or_default();
        Self {
            entries: std::sync::RwLock::new(map),
        }
    }
}

impl PlotCache {
    pub fn new() -> Self {
        Self {
            entries: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Retrieve memoized plot points if parameter values and range have not changed.
    pub fn get(
        &self,
        expr_str: &str,
        slider_hash: u64,
        min_r: f64,
        max_r: f64,
        samples: usize,
    ) -> Option<Vec<[f64; 2]>> {
        let key = PlotCacheKey {
            expr_str: expr_str.to_string(),
            slider_hash,
            min_bits: min_r.to_bits(),
            max_bits: max_r.to_bits(),
            samples,
        };
        self.entries.read().ok()?.get(&key).cloned()
    }

    /// Store newly evaluated plot points.
    pub fn insert(
        &self,
        expr_str: &str,
        slider_hash: u64,
        min_r: f64,
        max_r: f64,
        samples: usize,
        points: Vec<[f64; 2]>,
    ) {
        let key = PlotCacheKey {
            expr_str: expr_str.to_string(),
            slider_hash,
            min_bits: min_r.to_bits(),
            max_bits: max_r.to_bits(),
            samples,
        };
        if let Ok(mut entries) = self.entries.write() {
            entries.insert(key, points);
        }
    }

    /// Clear all cached plot points (e.g. on full re-eval or document reset).
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }
}

/// Compute a fast, deterministic 64-bit hash of slider values.
pub fn compute_slider_hash(slider_values: &HashMap<String, f64>) -> u64 {
    let mut s = std::collections::hash_map::DefaultHasher::new();
    let mut items: Vec<(&String, &f64)> = slider_values.iter().collect();
    items.sort_by_key(|(k, _)| *k);
    for (k, v) in items {
        k.hash(&mut s);
        v.to_bits().hash(&mut s);
    }
    s.finish()
}
