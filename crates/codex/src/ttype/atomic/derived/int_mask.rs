use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

/// Represents `int-mask<1, 2, 4>` utility type.
///
/// This type expands to a union of all possible bitmask combinations.
/// For example, `int-mask<1, 2, 4>` expands to `0|1|2|3|4|5|6|7`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
pub struct TIntMask {
    values: Vec<TUnion>,
}

impl TIntMask {
    pub fn new(values: Vec<TUnion>) -> Self {
        Self { values }
    }

    #[inline]
    pub fn get_values(&self) -> &[TUnion] {
        &self.values
    }

    #[inline]
    pub fn get_values_mut(&mut self) -> &mut [TUnion] {
        &mut self.values
    }

    /// Calculate all bitmask combinations from integer values.
    ///
    /// Given values `[1, 2, 4]`, returns `[0, 1, 2, 3, 4, 5, 6, 7]`.
    ///
    /// The algorithm works by iterating through all possible subsets (using a bitmask
    /// from 0 to 2^n - 1) and OR-ing together the values in each subset.
    pub fn calculate_mask_combinations(values: &[i64]) -> Vec<i64> {
        let n = values.len();
        if n == 0 {
            return vec![0];
        }

        // Limit to prevent combinatorial explosion (max 20 values = ~1M combinations)
        let max_values = 20;
        let n = n.min(max_values);
        let values = &values[..n];

        let mut result = Vec::with_capacity(1 << n);
        for mask in 0..(1u64 << n) {
            let mut combination = 0i64;
            for (i, &value) in values.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    combination |= value;
                }
            }
            result.push(combination);
        }

        result.sort_unstable();
        result.dedup();
        result
    }
}

impl TType for TIntMask {
    fn get_child_nodes(&self) -> Vec<TypeRef<'_>> {
        self.values.iter().map(TypeRef::Union).collect()
    }

    fn needs_population(&self) -> bool {
        self.values.iter().any(crate::ttype::TType::needs_population)
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        let mut id = String::from("int-mask<");
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                id.push_str(", ");
            }
            id.push_str(value.get_id().as_str());
        }
        id.push('>');
        atom(&id)
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
