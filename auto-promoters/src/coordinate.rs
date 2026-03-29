use super::Uint;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Coordinate {
    row: Uint,
    col: Uint,
}

impl Coordinate {
    fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    fn magnitude_l1(&self)
}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        x + y
    }
}
