//! Callable sources by expression span, without retaining full type unions.

use std::rc::Rc;

use foldhash::HashMap;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::{TAtomic, callable::TCallable};
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::union::TUnion;

#[derive(Debug, Default)]
pub struct CallableTypes {
    entries: Box<[Entry]>,
}

#[derive(Debug)]
struct Entry {
    span: (u32, u32),
    targets: Box<[FunctionLikeIdentifier]>,
    uncertain: bool,
}

impl CallableTypes {
    pub(super) fn collect(expressions: &HashMap<(u32, u32), Rc<TUnion>>, codebase: &CodebaseMetadata) -> Self {
        let mut entries = Vec::new();
        for (&span, union) in expressions {
            let mut targets = Vec::new();
            let mut uncertain = false;
            for atomic in union.types.iter() {
                if let Some(target) = callable_target(atomic, codebase) {
                    targets.push(target);
                } else {
                    uncertain = true;
                }
            }

            if !targets.is_empty() {
                targets.sort_unstable();
                targets.dedup();
                entries.push(Entry { span, targets: targets.into_boxed_slice(), uncertain });
            }
        }

        entries.sort_unstable_by_key(|entry| (entry.span.1, entry.span.0));
        Self { entries: entries.into_boxed_slice() }
    }

    /// The complete callee expression immediately before an argument list.
    #[must_use]
    pub fn at_end(&self, end: u32) -> Option<(&[FunctionLikeIdentifier], bool)> {
        let entry = self.entries.get(self.entries.partition_point(|entry| entry.span.1 < end))?;
        (entry.span.1 == end).then_some((&entry.targets, entry.uncertain))
    }
}

fn callable_target(atomic: &TAtomic, codebase: &CodebaseMetadata) -> Option<FunctionLikeIdentifier> {
    let callable = cast_atomic_to_callable(atomic, codebase, None)?;
    let target = match callable.as_ref() {
        TCallable::Alias(target) => *target,
        TCallable::Signature(signature) => signature.source?,
    };

    let target = match target {
        FunctionLikeIdentifier::Method(class, method) => {
            codebase.get_declaring_method_identifier(&MethodIdentifier::new(class, method)).into()
        }
        target => target,
    };

    (!codebase.get_function_like(&target)?.flags.is_magic_method()).then_some(target)
}

#[cfg(test)]
mod tests {
    use mago_codex::metadata::flags::MetadataFlags;
    use mago_codex::metadata::function_like::{FunctionLikeKind, FunctionLikeMetadata};
    use mago_codex::ttype::get_literal_string;
    use mago_span::Span;
    use mago_word::word;

    use super::*;

    #[test]
    fn complete_expression_keeps_unknown_union_branches() {
        let mut codebase = CodebaseMetadata::default();
        codebase.function_likes.insert(
            (word(""), word("read")),
            Box::new(FunctionLikeMetadata::new(
                FunctionLikeKind::Function,
                word("read"),
                word("read"),
                Span::dummy(0, 0),
                MetadataFlags::empty(),
            )),
        );

        let key = FunctionLikeIdentifier::Function(word("read"));
        let alias = TAtomic::Callable(TCallable::Alias(key));
        let expressions = HashMap::from_iter([
            ((0, 10), Rc::new(TUnion::from_vec(vec![alias.clone(), TAtomic::Null]))),
            ((6, 10), Rc::new(TUnion::from_atomic(alias))),
        ]);

        let types = CallableTypes::collect(&expressions, &codebase);
        assert_eq!(types.at_end(10), Some(([key].as_slice(), true)));
        assert!(types.at_end(9).is_none());
        assert!(types.at_end(11).is_none());
    }

    #[test]
    fn ordinary_string_literals_do_not_keep_callable_entries() {
        let expressions = HashMap::from_iter([((0, 4), Rc::new(get_literal_string(word("text"))))]);
        let types = CallableTypes::collect(&expressions, &CodebaseMetadata::default());
        assert!(types.entries.is_empty());
    }
}
