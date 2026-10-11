//! Full types for direct variable tokens, shared within one file snapshot.

use std::rc::Rc;

use foldhash::HashMap;
use foldhash::fast::RandomState;
use indexmap::IndexSet;
use mago_codex::ttype::union::TUnion;

#[derive(Debug, Default)]
pub struct VariableTypes {
    entries: Box<[Entry]>,
    types: Box<[TUnion]>,
}

#[derive(Debug)]
struct Entry {
    span: (u32, u32),
    type_index: u32,
}

impl VariableTypes {
    pub(super) fn collect(source: &[u8], expressions: &HashMap<(u32, u32), Rc<TUnion>>) -> Self {
        let mut entries = Vec::new();
        let mut types: IndexSet<TUnion, RandomState> = IndexSet::default();
        for (&span, union) in expressions {
            let Some(text) = source.get(span.0 as usize..span.1 as usize) else { continue };
            if !is_direct_variable(text) {
                continue;
            }

            let type_index =
                types.get_index_of(union.as_ref()).unwrap_or_else(|| types.insert_full(union.as_ref().clone()).0);
            entries.push(Entry { span, type_index: type_index as u32 });
        }

        entries.sort_unstable_by_key(|entry| entry.span);
        Self { entries: entries.into_boxed_slice(), types: types.into_iter().collect() }
    }

    /// Look up only bytes inside the variable token, including its `$`.
    #[must_use]
    pub fn at_offset(&self, offset: u32) -> Option<((u32, u32), &TUnion)> {
        let index = self.entries.partition_point(|entry| entry.span.0 <= offset).checked_sub(1)?;
        let entry = &self.entries[index];
        (offset < entry.span.1).then(|| (entry.span, &self.types[entry.type_index as usize]))
    }
}

fn is_direct_variable(source: &[u8]) -> bool {
    let Some((&first, rest)) = source.strip_prefix(b"$").and_then(<[u8]>::split_first) else { return false };
    let start = |byte: u8| byte == b'_' || byte.is_ascii_alphabetic() || byte >= 0x80;
    start(first) && rest.iter().all(|byte| start(*byte) || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::ttype::atomic::TAtomic;
    use mago_codex::ttype::{get_int, get_literal_int, get_nullable_string};

    use super::*;
    use crate::artifacts::AnalysisArtifacts;

    #[test]
    fn variable_types_keep_full_unions_and_exact_token_boundaries() {
        let source = b"$value $other $value->field 42 ${name} $$name";
        let mut expressions = HashMap::default();
        expressions.insert((0, 6), Rc::new(get_nullable_string()));
        expressions.insert((7, 13), Rc::new(get_literal_int(3)));
        for span in [(14, 27), (28, 30), (31, 38), (39, 45), (100, 110)] {
            expressions.insert(span, Rc::new(get_int()));
        }

        let variables = VariableTypes::collect(source, &expressions);
        assert_eq!(variables.entries.len(), 2);
        assert_eq!(variables.at_offset(0).map(|(_, ty)| ty), Some(&get_nullable_string()));
        assert_eq!(variables.at_offset(5).map(|(span, _)| span), Some((0, 6)));
        assert_eq!(variables.at_offset(8).and_then(|(_, ty)| ty.get_single_literal_int_value()), Some(3));
        for offset in [6, 13, 14, source.len() as u32] {
            assert!(variables.at_offset(offset).is_none(), "{offset}");
        }

        assert!(is_direct_variable("$café".as_bytes()));
        assert!(is_direct_variable(b"$this"));
        assert!(is_direct_variable(b"$_GET"));
        assert!(!is_direct_variable(b"$9invalid"));
    }

    #[test]
    fn variable_types_share_equal_types_across_repeated_variables() {
        fn assert_send_sync<T>()
        where
            T: Send + Sync,
        {
        }
        assert_send_sync::<VariableTypes>();
        let source = "$value ".repeat(4096);
        let expressions = (0..4096).map(|i| ((i * 7, i * 7 + 6), Rc::new(get_nullable_string()))).collect();
        let variables = VariableTypes::collect(source.as_bytes(), &expressions);
        assert_eq!(variables.entries.len(), 4096);
        assert_eq!(variables.types.len(), 1);
        assert_eq!(std::mem::size_of::<Entry>(), 12);
    }

    #[test]
    fn variable_types_share_equal_unions_with_different_atomic_order() {
        let source = "$value ".repeat(64);
        let mut expressions = HashMap::default();
        for value in 0..32 {
            let sorted = TUnion::from_vec(vec![TAtomic::Null, get_literal_int(value).get_single().clone()]);
            let mut reversed = sorted.clone();
            reversed.types.to_mut().reverse();
            assert_ne!(sorted.types, reversed.types);
            assert_eq!(sorted, reversed);
            let start = value as u32 * 14;
            expressions.insert((start, start + 6), Rc::new(sorted));
            expressions.insert((start + 7, start + 13), Rc::new(reversed));
        }

        let variables = VariableTypes::collect(source.as_bytes(), &expressions);
        assert_eq!(variables.entries.len(), 64);
        assert_eq!(variables.types.len(), 32);
        for [left, right] in variables.entries.as_chunks::<2>().0 {
            assert_eq!(left.type_index, right.type_index);
        }
    }

    #[test]
    fn variable_types_export_requires_editor_artifacts() {
        let codebase = CodebaseMetadata::default();
        let mut artifacts = AnalysisArtifacts::new();
        artifacts.expression_types.insert((0, 6), Rc::new(get_int()));
        assert!(artifacts.editor_data(b"$value", &codebase).is_none());
        let artifacts = artifacts.with_editor_artifacts(true);
        assert!(artifacts.editor_data(b"", &codebase).is_some_and(|data| data.variable_types.at_offset(0).is_none()));
        assert!(
            artifacts
                .editor_data(b"$value", &codebase)
                .is_some_and(|data| data.variable_types.at_offset(0).is_some_and(|(_, ty)| ty == &get_int()))
        );
    }
}
