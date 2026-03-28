use super::Uint;
use num::{BigUint, ToPrimitive, traits::Pow};

impl<E: ToPrimitive> Pow<E> for Uint {
    type Output = Self;
    fn pow(self, exp: E) -> Self {
        let e = exp.to_u32().expect("Exponent too large for BigUint::pow");
        match self {
            Uint::Machine(a) => {
                if let Some(result) = a.checked_pow(e) {
                    Uint::Machine(result)
                } else {
                    Uint::from(BigUint::from(a).pow(e))
                }
            }
            Uint::Promoted(a) => Uint::Promoted(a.pow(e)),
        }
    }
}
