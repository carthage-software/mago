//! Callable sources by expression span, without retaining full type unions.

use std::rc::Rc;
use std::sync::Arc;

use foldhash::{HashMap, HashSet};
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::{
    TAtomic,
    callable::{TCallable, TCallableSignature},
};
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CallableTarget {
    Full(FunctionLikeIdentifier),
    Partial(Arc<PartialSignature>),
}

impl CallableTarget {
    #[must_use]
    pub fn key(&self) -> FunctionLikeIdentifier {
        match self {
            Self::Full(key) => *key,
            Self::Partial(signature) => signature.key,
        }
    }
}

/// The remaining signature after binding arguments, with rendered types instead of type unions.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartialSignature {
    pub key: FunctionLikeIdentifier,
    pub parameters: Box<[PartialParameter]>,
    pub return_type: Option<Word>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartialParameter {
    /// Includes `$`. Missing names cannot bind named arguments to a declaration.
    pub name: Option<Word>,
    pub type_name: Option<Word>,
    pub by_reference: bool,
    pub variadic: bool,
    pub has_default: bool,
}

impl PartialSignature {
    fn new(key: FunctionLikeIdentifier, signature: &TCallableSignature) -> Self {
        Self {
            key,
            parameters: signature
                .get_parameters()
                .iter()
                .map(|parameter| PartialParameter {
                    name: parameter.get_name().map(|name| name.0),
                    type_name: parameter.get_type_signature().map(TType::get_id),
                    by_reference: parameter.is_by_reference(),
                    variadic: parameter.is_variadic(),
                    has_default: parameter.has_default(),
                })
                .collect(),
            return_type: signature.get_return_type().map(TType::get_id),
        }
    }
}

#[derive(Debug, Default)]
pub struct CallableTypes {
    entries: Box<[Entry]>,
}

#[derive(Debug)]
struct Entry {
    span: (u32, u32),
    targets: Box<[CallableTarget]>,
    uncertain: bool,
}

impl CallableTypes {
    pub(super) fn collect(expressions: &HashMap<(u32, u32), Rc<TUnion>>, codebase: &CodebaseMetadata) -> Self {
        let mut entries = Vec::new();
        let mut partials = HashSet::default();
        for (&span, union) in expressions {
            let mut targets = Vec::new();
            let mut uncertain = false;
            for atomic in union.types.iter() {
                if let Some(target) = callable_target(atomic, codebase, &mut partials) {
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

    /// Targets and their remaining signatures at the complete callee's end offset.
    #[must_use]
    pub fn at_end(&self, end: u32) -> Option<(&[CallableTarget], bool)> {
        let entry = self.entry_at_end(end)?;
        Some((&entry.targets, entry.uncertain))
    }

    /// Targets invoked by the complete callee, including partial applications.
    #[must_use]
    pub fn invocation_targets_at_end(&self, end: u32) -> Option<(&[CallableTarget], bool)> {
        self.at_end(end)
    }

    fn entry_at_end(&self, end: u32) -> Option<&Entry> {
        let entry = self.entries.get(self.entries.partition_point(|entry| entry.span.1 < end))?;
        (entry.span.1 == end).then_some(entry)
    }
}

fn callable_target(
    atomic: &TAtomic,
    codebase: &CodebaseMetadata,
    partials: &mut HashSet<Arc<PartialSignature>>,
) -> Option<CallableTarget> {
    let callable = cast_atomic_to_callable(atomic, codebase, None)?;
    let (target, partial) = match callable.as_ref() {
        TCallable::Alias(target) => (*target, false),
        TCallable::Signature(signature) => (signature.invocation_target()?, signature.is_partial),
    };

    let target = match target {
        FunctionLikeIdentifier::Method(class, method) => {
            codebase.get_declaring_method_identifier(&MethodIdentifier::new(class, method)).into()
        }
        target => target,
    };
    if codebase.get_function_like(&target)?.flags.is_magic_method() {
        return None;
    }

    if partial && let TCallable::Signature(signature) = callable.as_ref() {
        let summary = PartialSignature::new(target, signature);
        let shared = if let Some(shared) = partials.get(&summary) {
            Arc::clone(shared)
        } else {
            let shared = Arc::new(summary);
            partials.insert(Arc::clone(&shared));
            shared
        };

        Some(CallableTarget::Partial(shared))
    } else {
        Some(CallableTarget::Full(target))
    }
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
        assert_eq!(types.at_end(10), Some(([CallableTarget::Full(key)].as_slice(), true)));
        assert!(types.at_end(9).is_none());
        assert!(types.at_end(11).is_none());
    }

    #[test]
    fn ordinary_string_literals_do_not_keep_callable_entries() {
        let expressions = HashMap::from_iter([((0, 4), Rc::new(get_literal_string(word("text"))))]);
        let types = CallableTypes::collect(&expressions, &CodebaseMetadata::default());
        assert!(types.entries.is_empty());
    }

    #[test]
    fn partial_summaries_share_reduced_parameters_and_keep_unknown_branches() {
        use mago_codex::misc::VariableIdentifier;
        use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
        use mago_codex::ttype::get_int;

        let mut codebase = CodebaseMetadata::default();
        let key = FunctionLikeIdentifier::Function(word("read"));
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
        let parameter_type = Arc::new(get_int());
        let parameter_type_owner = Arc::downgrade(&parameter_type);
        let signature = TCallableSignature::new(false, true)
            .with_partial_source(Some(key))
            .with_parameters(vec![
                TCallableParameter::new(Some(parameter_type), true, false, false)
                    .with_name(Some(VariableIdentifier(word("$value")))),
                TCallableParameter::new(None, false, true, true).with_name(Some(VariableIdentifier(word("$rest")))),
            ])
            .with_return_type(Some(Arc::new(get_int())));
        let partial = TAtomic::Callable(TCallable::Signature(signature.clone()));
        let mut unnamed = signature.clone();
        unnamed.parameters[0] = unnamed.parameters[0].clone().with_name(None);
        let expressions = HashMap::from_iter([
            ((0, 10), Rc::new(TUnion::from_atomic(partial.clone()))),
            ((20, 30), Rc::new(TUnion::from_vec(vec![partial, TAtomic::Null]))),
            (
                (40, 50),
                Rc::new(TUnion::from_vec(vec![
                    TAtomic::Callable(TCallable::Signature(signature)),
                    TAtomic::Callable(TCallable::Signature(unnamed)),
                ])),
            ),
        ]);

        let types = CallableTypes::collect(&expressions, &codebase);
        drop(expressions);
        assert!(parameter_type_owner.upgrade().is_none(), "summaries must not retain type unions");
        let (first, uncertain) = types.at_end(10).unwrap();
        assert!(!uncertain);
        assert_eq!(first[0].key(), key);
        let CallableTarget::Partial(first) = &first[0] else { panic!("expected a reduced signature") };
        assert_eq!(
            first.parameters[0],
            PartialParameter {
                name: Some(word("$value")),
                type_name: Some(word("int")),
                by_reference: true,
                variadic: false,
                has_default: false,
            }
        );
        assert_eq!(
            first.parameters[1],
            PartialParameter {
                name: Some(word("$rest")),
                type_name: None,
                by_reference: false,
                variadic: true,
                has_default: true,
            }
        );
        assert_eq!(first.return_type, Some(word("int")));
        let (nullable, uncertain) = types.at_end(30).unwrap();
        assert!(uncertain);
        let CallableTarget::Partial(nullable) = &nullable[0] else { panic!("expected a reduced signature") };
        assert!(Arc::ptr_eq(first, nullable));
        let (distinct, uncertain) = types.at_end(50).unwrap();
        assert!(!uncertain, "unknown names do not erase the invoked declaration");
        assert_eq!(distinct.len(), 2, "the same origin can have different remaining names");
        assert!(distinct.iter().any(
            |target| matches!(target, CallableTarget::Partial(signature) if signature.parameters[0].name.is_none())
        ));
        assert_eq!(types.invocation_targets_at_end(10), types.at_end(10));
    }

    #[test]
    fn callable_summary_layout_keeps_full_targets_compact() {
        #[allow(dead_code)]
        struct PreviousEntry {
            span: (u32, u32),
            targets: Box<[FunctionLikeIdentifier]>,
            uncertain: bool,
            partial: bool,
        }
        assert!(size_of::<CallableTarget>() <= size_of::<FunctionLikeIdentifier>() + size_of::<usize>());
        assert_eq!(size_of::<Entry>(), size_of::<PreviousEntry>());
    }
}
