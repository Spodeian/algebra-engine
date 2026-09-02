use algebra_engine::sets::{ClassType, InfiniteSetKind, IntSetIterator, MathSetRepresentation};

#[test]
fn test_set_union_intersection() {
    let s1 = MathSetRepresentation::Infinite(InfiniteSetKind::Integers);
    let s2 = MathSetRepresentation::Empty;

    assert_eq!(s1.clone().union(s2.clone()), s1);
    assert_eq!(s1.intersection(s2), MathSetRepresentation::Empty);
}

#[test]
fn test_class_type() {
    let r = MathSetRepresentation::Infinite(InfiniteSetKind::Reals);
    let surr = MathSetRepresentation::Infinite(InfiniteSetKind::Surreals);

    assert_eq!(r.class_type(), ClassType::Set);
    assert_eq!(surr.class_type(), ClassType::ProperClass);
}

#[test]
fn test_int_iterator() {
    let mut iter = IntSetIterator::new(1, Some(3));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
}
