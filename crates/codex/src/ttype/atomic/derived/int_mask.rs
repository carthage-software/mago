use std::cmp::Reverse;
use std::collections::BinaryHeap;

use mago_word::Word;
use mago_word::word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::union::TUnion;

// Limit to prevent combinatorial explosion (max 20 values = ~1M combinations)
const MAX_MASK_VALUES: usize = 20;

/// Represents `int-mask<1, 2, 4>` utility type.
///
/// This type expands to a union of all possible bitmask combinations.
/// For example, `int-mask<1, 2, 4>` expands to `0|1|2|3|4|5|6|7`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TIntMask {
    values: Vec<TUnion>,
}

impl TIntMask {
    #[must_use]
    pub fn new(values: Vec<TUnion>) -> Self {
        Self { values }
    }

    #[inline]
    #[must_use]
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
    /// For each value, append each existing combination OR-ed with that value.
    #[must_use]
    pub fn calculate_mask_combinations(values: &[i64]) -> Vec<i64> {
        let n = values.len();
        if n == 0 {
            return vec![0];
        }

        let n = n.min(MAX_MASK_VALUES);
        let values = &values[..n];

        let mut result = Vec::with_capacity(1 << n);
        result.push(0);
        for &value in values {
            let count = result.len();
            for index in 0..count {
                result.push(result[index] | value);
            }
        }

        result.sort_unstable();
        result.dedup();
        result
    }

    #[must_use]
    pub fn calculate_smallest_mask_combinations(values: &[i64], limit: usize) -> Vec<i64> {
        let values = &values[..values.len().min(MAX_MASK_VALUES)];
        if (1usize << values.len()) <= limit || values.iter().any(|value| *value < 0) {
            let mut combinations = Self::calculate_mask_combinations(values);
            combinations.truncate(limit);
            return combinations;
        }

        let mut combinations = Vec::with_capacity(limit);
        let mut candidates = BinaryHeap::from([Reverse(0)]);
        while combinations.len() < limit
            && let Some(Reverse(combination)) = candidates.pop()
        {
            if combinations.last() == Some(&combination) {
                continue;
            }

            combinations.push(combination);
            for value in values {
                if combination | value != combination {
                    candidates.push(Reverse(combination | value));
                }
            }
        }

        combinations
    }
}

impl TType for TIntMask {
    fn append_child_nodes<'types>(&'types self, children: &mut Vec<TypeRef<'types>>) {
        children.extend(self.values.iter().map(TypeRef::Union));
    }

    fn needs_population(&self) -> bool {
        self.values.iter().any(crate::ttype::TType::needs_population)
    }

    fn is_expandable(&self) -> bool {
        true
    }

    fn get_id(&self) -> Word {
        let mut id: Vec<u8> = b"int-mask<".to_vec();
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                id.extend_from_slice(b", ");
            }
            id.extend_from_slice(value.get_id().as_bytes());
        }
        id.push(b'>');
        word(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::TIntMask;

    fn subset_combinations(values: &[i64]) -> Vec<i64> {
        let values = &values[..values.len().min(20)];
        let mut result = Vec::with_capacity(1 << values.len());
        for mask in 0..(1u64 << values.len()) {
            let mut combination = 0;
            for (index, &value) in values.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    combination |= value;
                }
            }

            result.push(combination);
        }

        result.sort_unstable();
        result.dedup();
        result
    }

    #[test]
    fn mask_combinations_match_subsets() {
        let alphabet = [0, 1, 2, 3, -1, i64::MIN, i64::MAX];
        for length in 0..=4 {
            for mut value in 0..alphabet.len().pow(length) {
                let mut values = vec![0; length as usize];
                for entry in &mut values {
                    *entry = alphabet[value % alphabet.len()];
                    value /= alphabet.len();
                }

                assert_eq!(TIntMask::calculate_mask_combinations(&values), subset_combinations(&values), "{values:?}");
            }
        }
    }

    #[test]
    fn smallest_mask_combinations_are_a_prefix_of_all_combinations() {
        let bits: Vec<i64> = (0..=20).map(|bit| 1 << bit).collect();
        let cases: [&[i64]; 7] = [
            &[],
            &[0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
            &[3, 5, 6, 12, 24, 48, 96, 192, 384, 768],
            &[1024, 1, 512, 2, 256, 4, 128, 8, 64, 16, 32, 32, 0],
            &bits,
            &[-1, 1, 2, 4, 8, 16, 32, 64, 128, 256],
            &[i64::MAX, 7, 9, 11, 1 << 40, 1 << 41, 1 << 42, 1 << 43, 1 << 44, 3],
        ];

        for values in cases {
            let all = TIntMask::calculate_mask_combinations(values);
            for limit in [0, 1, 2, 7, 128, 129, 130, all.len(), all.len() + 1] {
                assert_eq!(
                    TIntMask::calculate_smallest_mask_combinations(values, limit),
                    all[..limit.min(all.len())],
                    "{values:?} {limit}"
                );
            }
        }
    }

    #[test]
    fn mask_combinations_use_only_the_first_twenty_values() {
        let mut values = vec![0, 1, 1, 2, 2, 3, i64::MIN, i64::MIN];
        values.resize(20, 0);
        let expected = subset_combinations(&values);
        assert_eq!(TIntMask::calculate_mask_combinations(&values), expected);

        values.push(4);
        assert_eq!(TIntMask::calculate_mask_combinations(&values), expected);
    }
}
