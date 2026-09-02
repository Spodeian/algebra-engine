//! Workspace integration tests for set theory and discrete iterators.

use algebra_core::ExprGraph;
use algebra_engine::sets::{ClassType, InfiniteSetKind, IntSetIterator, MathSetRepresentation};

#[test]
fn test_set_operations_and_class_types() {
    let reals = MathSetRepresentation::Infinite(InfiniteSetKind::Reals);
    let integers = MathSetRepresentation::Infinite(InfiniteSetKind::Integers);
    let surreals = MathSetRepresentation::Infinite(InfiniteSetKind::Surreals);

    // Class type checks (NBG set vs proper class)
    assert_eq!(reals.class_type(), ClassType::Set);
    assert_eq!(integers.class_type(), ClassType::Set);
    assert_eq!(surreals.class_type(), ClassType::ProperClass);

    // Union and Intersection identities
    let empty = MathSetRepresentation::Empty;
    let union_res = reals.clone().union(empty.clone());
    assert_eq!(union_res, reals);

    let inter_res = reals.intersection(empty);
    assert_eq!(inter_res, MathSetRepresentation::Empty);
}

#[test]
fn test_discrete_set_iterator() {
    let graph = ExprGraph::new();
    let _one = graph.integer(1);
    let _ten = graph.integer(10);

    let mut iter = IntSetIterator::new(1, Some(5));
    let values: Vec<i64> = iter.by_ref().collect();
    assert_eq!(values, vec![1, 2, 3, 4, 5]);
}
