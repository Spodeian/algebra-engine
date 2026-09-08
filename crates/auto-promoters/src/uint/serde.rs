use super::Uint;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use num::BigUint;

impl Serialize for Uint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Serialize as a standard u64 if it fits, for cross-language compatibility
            Uint::Machine(v) => serializer.serialize_u64(*v as u64),
            // BigUint usually serializes as a sequence of bytes or a large integer
            Uint::Promoted(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Uint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Try to deserialize into a BigUint first, as it can catch both
        // small integers and large ones.
        let big = BigUint::deserialize(deserializer)?;
        // Use our From<BigUint> logic to automatically demote to Machine if possible.
        Ok(Uint::from(big))
    }
}
