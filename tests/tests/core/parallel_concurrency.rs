//! Workspace integration tests for thread-safe lock-free hash-consing ExprGraph.

use algebra_core::ExprGraph;
use rayon::prelude::*;
use std::sync::Arc;

#[test]
fn test_concurrent_hash_consing_deduplication() {
    let graph = Arc::new(ExprGraph::new());

    // Concurrently add identical expressions from 100 threads
    let ids: Vec<_> = (0..1000)
        .into_par_iter()
        .map(|i| {
            let x = graph.symbol("x");
            let val = graph.integer((i % 10) as i64);
            graph.add([x, val])
        })
        .collect();

    // Verify lock-free hash-consing deduplicated repeated identical expressions
    assert!(graph.len() < 100);
    assert_eq!(ids.len(), 1000);
}

#[test]
fn test_batch_interning() {
    let graph = ExprGraph::new();
    let x = graph.symbol("x");
    let y = graph.symbol("y");

    let node1 = graph.get(x);
    let node2 = graph.get(y);

    let ids = graph.intern_batch(vec![node1, node2]);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids[0], x);
    assert_eq!(ids[1], y);
}
