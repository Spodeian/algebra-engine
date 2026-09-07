use super::Uint;
use num::integer::Roots;

impl Roots for Uint {
    fn sqrt(&self) -> Self {
        match self {
            // Use the primitive usize implementation of Roots::sqrt
            Uint::Machine(n) => Uint::Machine(n.sqrt()),
            // Use the BigUint implementation for Promoted values
            Uint::Promoted(n) => Uint::from(n.sqrt()),
        }
    }

    fn nth_root(&self, n: u32) -> Self {
        if n == 0 {
            panic!("nth_root: n must be non-zero");
        }
        if n == 1 {
            return self.clone();
        }

        match self {
            // Use the primitive usize implementation of Roots::nth_root
            Uint::Machine(v) => Uint::Machine(v.nth_root(n)),
            Uint::Promoted(v) => Uint::from(v.nth_root(n)),
        }
    }
}
