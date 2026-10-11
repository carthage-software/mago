//! Callable sources by expression span, without retaining full type unions.

use std::rc::Rc;
use std::sync::Arc;

use foldhash::{HashMap, HashSet};
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::atomic::{
    TAtomic,
    callable::{TCallable, TCallableSignature},
};
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::display::NameDisplay;
use mago_codex::ttype::union::TUnion;
use mago_word::Word;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CallableTarget {
    Full(FunctionLikeIdentifier),
    Signature(Arc<BoundSignature>),
}

impl CallableTarget {
    #[must_use]
    pub fn key(&self) -> FunctionLikeIdentifier {
        match self {
            Self::Full(key) => *key,
            Self::Signature(signature) => signature.key,
        }
    }
}

/// A signature with bound arguments or class types, without retaining type unions.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundSignature {
    pub key: FunctionLikeIdentifier,
    pub parameters: Box<[BoundParameter]>,
    pub return_type: Option<Box<str>>,
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BoundParameter {
    /// Includes `$`. Missing names cannot bind named arguments to a declaration.
    pub name: Option<Word>,
    pub type_name: Option<Box<str>>,
    pub by_reference: bool,
    pub variadic: bool,
    pub has_default: bool,
}

impl BoundSignature {
    fn new(key: FunctionLikeIdentifier, signature: &TCallableSignature, names: &NameDisplay<'_>) -> Self {
        Self {
            key,
            parameters: signature
                .get_parameters()
                .iter()
                .map(|parameter| BoundParameter {
                    name: parameter.get_name().map(|name| name.0),
                    type_name: parameter.get_type_signature().map(|ty| names.union(ty).into_boxed_str()),
                    by_reference: parameter.is_by_reference(),
                    variadic: parameter.is_variadic(),
                    has_default: parameter.has_default(),
                })
                .collect(),
            return_type: signature.get_return_type().map(|ty| names.union(ty).into_boxed_str()),
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
        let mut signatures = HashSet::default();
        for (&span, union) in expressions {
            let mut targets = Vec::new();
            let mut uncertain = false;
            let mut names = None;
            for atomic in union.types.iter() {
                if let Some(target) = callable_target(atomic, codebase, &mut signatures, union, &mut names) {
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

fn callable_target<'code>(
    atomic: &TAtomic,
    codebase: &'code CodebaseMetadata,
    signatures: &mut HashSet<Arc<BoundSignature>>,
    union: &TUnion,
    names: &mut Option<NameDisplay<'code>>,
) -> Option<CallableTarget> {
    let callable = cast_atomic_to_callable(atomic, codebase, None)?;
    let target = match callable.as_ref() {
        TCallable::Alias(target) => *target,
        TCallable::Signature(signature) => signature.invocation_target()?,
    };

    let target = match target {
        FunctionLikeIdentifier::Method(class, method) => {
            codebase.get_declaring_method_identifier(&MethodIdentifier::new(class, method)).into()
        }
        target => target,
    };
    let declaration = codebase.get_function_like(&target)?;
    let target = match target {
        FunctionLikeIdentifier::Function(_) => FunctionLikeIdentifier::Function(declaration.name),
        target => target,
    };

    if declaration.flags.is_magic_method() {
        return None;
    }

    if let TCallable::Signature(signature) = callable.as_ref()
        && (signature.is_partial || target.is_closure() && closure_types_differ(signature, declaration))
    {
        let names = names.get_or_insert_with(|| callable_names(codebase, union));
        let summary = BoundSignature::new(target, signature, names);
        let shared = if let Some(shared) = signatures.get(&summary) {
            Arc::clone(shared)
        } else {
            let shared = Arc::new(summary);
            signatures.insert(Arc::clone(&shared));
            shared
        };

        Some(CallableTarget::Signature(shared))
    } else {
        Some(CallableTarget::Full(target))
    }
}

fn closure_types_differ(signature: &TCallableSignature, declaration: &FunctionLikeMetadata) -> bool {
    declaration
        .return_type_metadata
        .as_ref()
        .is_some_and(|ty| signature.get_return_type().map(|bound| &bound.types) != Some(&ty.type_union.types))
        || signature.get_parameters().iter().zip(&declaration.parameters).any(|(bound, parameter)| {
            parameter
                .type_metadata
                .as_ref()
                .is_some_and(|ty| bound.get_type_signature().map(|bound| &bound.types) != Some(&ty.type_union.types))
        })
}

/// Full targets display declaration types; include them alongside reduced signatures.
fn callable_names<'code>(codebase: &'code CodebaseMetadata, union: &TUnion) -> NameDisplay<'code> {
    let mut types = vec![union];
    for atomic in union.types.iter() {
        let Some(callable) = cast_atomic_to_callable(atomic, codebase, None) else { continue };
        let key = match callable.as_ref() {
            TCallable::Alias(key) => *key,
            TCallable::Signature(signature) => {
                let Some(key) = signature.get_source() else { continue };
                key
            }
        };

        let function = match key {
            FunctionLikeIdentifier::Method(class, method) => {
                codebase.get_declaring_method(class.as_bytes(), method.as_bytes())
            }
            _ => codebase.get_function_like(&key),
        };

        if let Some(function) = function {
            types.extend(
                function
                    .parameters
                    .iter()
                    .filter_map(|parameter| parameter.type_metadata.as_ref().map(|ty| &ty.type_union)),
            );

            types.extend(function.return_type_metadata.as_ref().map(|ty| &ty.type_union));
        }
    }

    NameDisplay::for_types(codebase, types.into_iter())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let CallableTarget::Signature(first) = &first[0] else { panic!("expected a reduced signature") };
        assert_eq!(first.parameters.len(), 2);
        assert_eq!(
            first.parameters[0],
            BoundParameter {
                name: Some(word("$value")),
                type_name: Some("int".into()),
                by_reference: true,
                variadic: false,
                has_default: false,
            }
        );
        assert_eq!(
            first.parameters[1],
            BoundParameter {
                name: Some(word("$rest")),
                type_name: None,
                by_reference: false,
                variadic: true,
                has_default: true,
            }
        );
        assert_eq!(first.return_type.as_deref(), Some("int"));
        let (nullable, uncertain) = types.at_end(30).unwrap();
        assert!(uncertain);
        let CallableTarget::Signature(nullable) = &nullable[0] else { panic!("expected a reduced signature") };
        assert!(Arc::ptr_eq(first, nullable));
        let (distinct, uncertain) = types.at_end(50).unwrap();
        assert!(!uncertain, "unknown names do not erase the invoked declaration");
        assert_eq!(distinct.len(), 2, "the same origin can have different remaining names");
        assert!(distinct.iter().any(
            |target| matches!(target, CallableTarget::Signature(signature) if signature.parameters[0].name.is_none())
        ));
        assert_eq!(types.invocation_targets_at_end(10), types.at_end(10));
        let summary = Arc::downgrade(first);
        drop(types);
        assert!(summary.upgrade().is_none(), "dropping the file must free its shared display text");
    }

    #[test]
    fn partial_type_display_keeps_literals_and_collisions_across_alternatives() {
        use mago_codex::metadata::ttype::TypeMetadata;
        use mago_codex::ttype::TType;
        use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
        use mago_codex::ttype::atomic::object::{TObject, named::TNamedObject};

        let key = FunctionLikeIdentifier::Function(word("read"));
        let mut codebase = CodebaseMetadata::default();
        let mut metadata = FunctionLikeMetadata::new(
            FunctionLikeKind::Function,
            word("read"),
            word("read"),
            Span::dummy(0, 0),
            MetadataFlags::empty(),
        );

        let named = |name| TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(word(name)))));
        metadata.return_type_metadata = Some(TypeMetadata::new(named("Right\\User"), Span::dummy(0, 0)));
        codebase.function_likes.insert((word(""), word("read")), Box::new(metadata));
        let literal = get_literal_string(word("Left\\User'|enum(Right\\User)"));
        let signature = |name| {
            TCallableSignature::new(false, true).with_partial_source(Some(key)).with_parameters(vec![
                TCallableParameter::new(Some(Arc::new(named(name))), false, false, false),
                TCallableParameter::new(Some(Arc::new(literal.clone())), false, false, false),
            ])
        };

        let partial = |name| TAtomic::Callable(TCallable::Signature(signature(name)));
        let expressions = HashMap::from_iter([
            ((0, 10), Rc::new(TUnion::from_vec(vec![partial("Left\\User"), partial("Right\\User")]))),
            (
                (20, 30),
                Rc::new(TUnion::from_vec(vec![partial("Left\\User"), TAtomic::Callable(TCallable::Alias(key))])),
            ),
            (
                (40, 50),
                Rc::new(TUnion::from_vec(vec![
                    partial("Left\\User"),
                    TAtomic::Callable(TCallable::Signature(
                        TCallableSignature::new(false, true).with_source(Some(key)),
                    )),
                ])),
            ),
        ]);

        let collected = CallableTypes::collect(&expressions, &codebase);
        let (alternatives, uncertain) = collected.at_end(10).unwrap();
        assert!(!uncertain);
        let displayed: Vec<_> = alternatives
            .iter()
            .map(|target| {
                let CallableTarget::Signature(signature) = target else { panic!("expected partial") };
                assert_eq!(signature.parameters[1].type_name.as_deref(), Some(literal.get_id().as_str().unwrap()));
                signature.parameters[0].type_name.as_deref().unwrap()
            })
            .collect();
        assert_eq!(displayed.len(), 2, "display spelling must not merge distinct parameter types");
        assert!(displayed.contains(&"\\Left\\User") && displayed.contains(&"\\Right\\User"));
        for end in [30, 50] {
            let (mixed, _) = collected.at_end(end).unwrap();
            let signature = mixed
                .iter()
                .find_map(
                    |target| if let CallableTarget::Signature(signature) = target { Some(signature) } else { None },
                )
                .unwrap();
            assert_eq!(
                signature.parameters[0].type_name.as_deref(),
                Some("\\Left\\User"),
                "full declaration metadata must share the collision context"
            );
        }
    }

    #[test]
    fn bound_template_display_shares_origins_with_full_declarations() {
        use mago_codex::metadata::ttype::TypeMetadata;
        use mago_codex::misc::GenericParent;
        use mago_codex::ttype::atomic::generic::TGenericParameter;
        use mago_codex::ttype::get_mixed;

        let key = FunctionLikeIdentifier::Function(word("read"));
        let template = |owner| {
            TUnion::from_atomic(TAtomic::GenericParameter(TGenericParameter::new(
                word("T"),
                Arc::new(get_mixed()),
                GenericParent::ClassLike(word(owner)),
            )))
        };
        let mut declaration = FunctionLikeMetadata::new(
            FunctionLikeKind::Function,
            word("read"),
            word("read"),
            Span::dummy(0, 0),
            MetadataFlags::empty(),
        );
        declaration.return_type_metadata = Some(TypeMetadata::new(template("Right\\Box"), Span::dummy(0, 0)));
        let mut codebase = CodebaseMetadata::default();
        codebase.function_likes.insert((word(""), word("read")), Box::new(declaration));
        let partial = TCallableSignature::new(false, true)
            .with_partial_source(Some(key))
            .with_return_type(Some(Arc::new(template("Left\\Box"))));
        let expressions = HashMap::from_iter([(
            (0, 10),
            Rc::new(TUnion::from_vec(vec![
                TAtomic::Callable(TCallable::Signature(partial)),
                TAtomic::Callable(TCallable::Alias(key)),
            ])),
        )]);
        let types = CallableTypes::collect(&expressions, &codebase);
        let (targets, uncertain) = types.at_end(10).unwrap();
        assert!(!uncertain);
        assert!(targets.contains(&CallableTarget::Full(key)));
        let bound = targets
            .iter()
            .find_map(|target| match target {
                CallableTarget::Signature(bound) => Some(bound),
                _ => None,
            })
            .unwrap();
        assert_eq!(bound.return_type.as_deref(), Some(r"\Left\Box::T"));
        let full = &codebase.get_function_like(&key).unwrap().return_type_metadata.as_ref().unwrap().type_union;
        assert_eq!(NameDisplay::fully_qualified(&codebase).union(full), r"\Right\Box::T");
    }

    #[test]
    fn full_closures_only_store_signatures_when_declared_types_change() {
        use mago_codex::metadata::ttype::TypeMetadata;
        use mago_codex::ttype::atomic::object::{TObject, named::TNamedObject};
        use mago_codex::ttype::get_int;

        let name = word("{closure:a.php:1:1}");
        let key = FunctionLikeIdentifier::Closure(name);
        let mut declaration =
            FunctionLikeMetadata::new(FunctionLikeKind::Closure, name, name, Span::dummy(0, 0), MetadataFlags::empty());
        let named = |name| TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(word(name)))));
        declaration.return_type_metadata = Some(TypeMetadata::new(named("parent"), Span::dummy(0, 0)));
        let mut codebase = CodebaseMetadata::default();
        codebase.function_likes.insert((word(""), name), Box::new(declaration));
        let signature = |ty| {
            TAtomic::Callable(TCallable::Signature(
                TCallableSignature::new(false, true).with_source(Some(key)).with_return_type(Some(Arc::new(ty))),
            ))
        };
        let mut flagged = named("parent");
        flagged.set_by_reference(true);
        let expressions = HashMap::from_iter([
            ((0, 10), Rc::new(TUnion::from_atomic(signature(named("parent"))))),
            ((20, 30), Rc::new(TUnion::from_atomic(signature(named("Domain\\Base"))))),
            ((40, 50), Rc::new(TUnion::from_atomic(signature(flagged)))),
        ]);

        let collected = CallableTypes::collect(&expressions, &codebase);
        assert_eq!(collected.at_end(10), Some(([CallableTarget::Full(key)].as_slice(), false)));
        assert_eq!(collected.at_end(50), Some(([CallableTarget::Full(key)].as_slice(), false)));
        let (targets, uncertain) = collected.at_end(30).unwrap();
        assert!(!uncertain);
        let CallableTarget::Signature(bound) = &targets[0] else { panic!("expected a bound signature") };
        assert_eq!(bound.key, key);
        assert_eq!(bound.return_type.as_deref(), Some("Base"));

        codebase.function_likes.get_mut(&(word(""), name)).unwrap().return_type_metadata = None;
        let inferred = HashMap::from_iter([((0, 10), Rc::new(TUnion::from_atomic(signature(get_int()))))]);
        let collected = CallableTypes::collect(&inferred, &codebase);
        assert_eq!(collected.at_end(10), Some(([CallableTarget::Full(key)].as_slice(), false)));
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
