use foldhash::fast::RandomState;
use indexmap::IndexSet;

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::ReferenceOrigin;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::ClassLikeMemberSelector;
use mago_word::Word;

use super::AnalysisArtifacts;
use super::CheckpointAction;
use crate::resolver::method::MethodResolutionResult;
use crate::resolver::property::DeclaredProperty;
use crate::resolver::property::PropertyResolutionResult;
use crate::resolver::property::find_property_in_mixins;
use crate::resolver::property::resolve_declared_property;

mod callable_types;
mod variable_types;

pub use callable_types::{BoundParameter, BoundSignature, CallableTarget, CallableTypes};
pub use variable_types::VariableTypes;

/// Owned editor data from one completed file analysis.
#[derive(Debug, Default)]
pub struct EditorFileData {
    /// One row per possible target, sorted by selector span.
    pub methods: Box<[MethodReference]>,
    /// Property, class constant, and enum case targets, sorted by selector span.
    pub members: Box<[MemberReference]>,
    /// Named object types, sorted by expression span. Scalar types are omitted.
    pub expression_classes: Box<[ExpressionClasses]>,
    /// Full types for direct variables, without analyzer `Rc` ownership.
    pub variable_types: VariableTypes,
    /// Known callable sources, sorted by the callee expression's end offset.
    pub callable_types: CallableTypes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemberKind {
    Property,
    ClassConstant,
    EnumCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberReference {
    pub selector_span: (u32, u32),
    pub receiver_span: (u32, u32),
    pub origin: ReferenceOrigin,
    pub kind: MemberKind,
    pub lookup_class: Option<Word>,
    /// Case-sensitive name; property names include `$`.
    pub name: Word,
    /// Together with `kind`, identifies the target across inherited accesses.
    pub declaration: Option<Span>,
    pub static_access: bool,
    pub dynamic: bool,
    pub write: bool,
    pub magic: bool,
    pub uncertain: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MethodReference {
    pub selector_span: (u32, u32),
    pub expression_span: (u32, u32),
    pub receiver_span: (u32, u32),
    pub origin: ReferenceOrigin,
    pub receiver_class: Word,
    /// The declaring class and method, with ASCII case folded.
    pub method: MethodIdentifier,
    pub declaration: Option<Span>,
    pub static_access: bool,
    /// The selector is computed, so its source text cannot be renamed as an identifier.
    pub dynamic: bool,
    /// The target is a magic handler or a method declared in a docblock.
    pub magic: bool,
    /// At least one receiver or selector could not be resolved safely.
    pub uncertain: bool,
}

#[derive(Debug, Clone)]
pub struct ExpressionClasses {
    pub span: (u32, u32),
    pub classes: Box<[Word]>,
    pub uncertain: bool,
}

impl ExpressionClasses {
    /// The named object classes of `union`, or `None` when it names none.
    #[must_use]
    pub fn from_union(span: (u32, u32), union: &TUnion) -> Option<Self> {
        let mut classes = Vec::new();
        let mut uncertain = false;
        for atomic in union.types.iter() {
            uncertain |= !collect_classes(atomic, &mut classes);
        }

        if classes.is_empty() {
            return None;
        }

        classes.sort_unstable();
        classes.dedup();
        Some(Self { span, classes: classes.into_boxed_slice(), uncertain })
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EditorArtifacts {
    pub(crate) methods: IndexSet<MethodReference, RandomState>,
    pub(crate) members: IndexSet<MemberReference, RandomState>,
    checkpoints: Vec<(usize, usize)>,
}

impl EditorArtifacts {
    pub(crate) fn begin_checkpoint(&mut self) {
        self.checkpoints.push((self.methods.len(), self.members.len()));
    }

    pub(crate) fn finish_checkpoint(&mut self, action: CheckpointAction) {
        let Some((methods, members)) = self.checkpoints.pop() else {
            unreachable!("editor checkpoint must exist");
        };

        if matches!(action, CheckpointAction::Rollback) {
            self.methods.truncate(methods);
            self.members.truncate(members);
        }
    }

    pub(crate) fn record_methods(
        &mut self,
        codebase: &CodebaseMetadata,
        origin: ReferenceOrigin,
        receiver: Span,
        selector: &ClassLikeMemberSelector<'_>,
        expression: Span,
        static_access: bool,
        result: &MethodResolutionResult,
    ) {
        let uncertain = result.has_ambiguous_target
            || result.has_invalid_target
            || result.encountered_mixed
            || !result.unresolved_methods.is_empty();
        let targets = result
            .resolved_methods
            .iter()
            .map(|target| (target, false))
            .chain(result.undocumented_methods.iter().map(|target| (&target.magic_method, true)));
        for (target, magic) in targets {
            let metadata = codebase.get_method_by_id(&target.method_identifier);
            self.methods.insert(MethodReference {
                selector_span: (selector.start_offset(), selector.end_offset()),
                expression_span: (expression.start.offset, expression.end.offset),
                receiver_span: (receiver.start.offset, receiver.end.offset),
                origin,
                receiver_class: target.classname,
                method: MethodIdentifier::new(
                    target.method_identifier.get_class_name().to_ascii_lowercase(),
                    target.method_identifier.get_method_name().to_ascii_lowercase(),
                ),
                declaration: metadata.and_then(|metadata| metadata.name_span),
                static_access,
                dynamic: !matches!(selector, ClassLikeMemberSelector::Identifier(_)) || result.has_dynamic_selector,
                magic: magic || metadata.is_some_and(|metadata| metadata.flags.is_magic_method()),
                uncertain: uncertain || target.mixin_without_magic_method.is_some(),
            });
        }
    }

    pub(crate) fn record_properties(&mut self, reference: MemberReference, result: &PropertyResolutionResult) {
        let uncertain = result.has_ambiguous_path
            || result.has_error_path
            || result.has_invalid_path
            || result.encountered_mixed
            || result.has_possibly_defined_property;
        for property in &result.properties {
            self.members.insert(MemberReference {
                lookup_class: property.lookup_class,
                name: property.property_name,
                declaration: property.property_span,
                magic: property.is_magic,
                uncertain: reference.uncertain || uncertain || property.property_span.is_none(),
                ..reference
            });
        }
    }

    pub(crate) fn record_saved_properties(
        &mut self,
        codebase: &CodebaseMetadata,
        scope: Option<Word>,
        mut reference: MemberReference,
        receiver_type: Option<&TUnion>,
    ) {
        if reference.lookup_class.is_some() {
            self.record_saved_property(codebase, scope, reference, receiver_type);
            return;
        }

        let mut classes = Vec::new();
        if let Some(receiver_type) = receiver_type {
            for atomic in receiver_type.types.iter() {
                let before = classes.len();
                collect_classes(atomic, &mut classes);
                if reference.static_access {
                    match atomic {
                        TAtomic::Scalar(TScalar::ClassLikeString(
                            TClassLikeString::OfType { constraint, .. } | TClassLikeString::Generic { constraint, .. },
                        )) => {
                            collect_classes(constraint, &mut classes);
                        }
                        _ => {
                            if let Some(class) = atomic
                                .get_class_string_value()
                                .or_else(|| atomic.get_literal_string_value().map(mago_word::word))
                            {
                                classes.push(class);
                            }
                        }
                    }
                }

                reference.uncertain |= classes.len() == before && !atomic.is_null();
            }
        }

        if classes.is_empty() {
            self.record_saved_property(codebase, scope, reference, receiver_type);
        } else {
            reference.uncertain |= classes.iter().any(|class| {
                resolve_saved_property(
                    codebase,
                    scope,
                    MemberReference { lookup_class: Some(*class), ..reference },
                    receiver_type,
                )
                .is_none()
            });

            for class in classes {
                reference.lookup_class = Some(class);
                self.record_saved_property(codebase, scope, reference, receiver_type);
            }
        }
    }

    fn record_saved_property(
        &mut self,
        codebase: &CodebaseMetadata,
        scope: Option<Word>,
        mut reference: MemberReference,
        receiver_type: Option<&TUnion>,
    ) {
        reference = resolve_saved_property(codebase, scope, reference, receiver_type).unwrap_or_else(|| {
            reference.magic = !reference.static_access
                && reference.lookup_class.is_some_and(|class| codebase.method_exists(class.as_bytes(), b"__get"));
            reference.uncertain = true;
            reference
        });

        self.members.insert(reference);
    }
}

fn resolve_saved_property(
    codebase: &CodebaseMetadata,
    scope: Option<Word>,
    reference: MemberReference,
    receiver_type: Option<&TUnion>,
) -> Option<MemberReference> {
    let class = reference.lookup_class?;
    let metadata = codebase.get_class_like(class.as_bytes())?;
    let with_declaration = |property: DeclaredProperty<'_>, magic, uncertain| MemberReference {
        declaration: property.property.name_span.or(property.property.span),
        magic,
        uncertain: reference.uncertain || uncertain || property.property.name_span.is_none(),
        ..reference
    };

    if let Some(property) =
        resolve_declared_property(codebase, metadata, reference.name, !reference.static_access, scope)
    {
        return Some(with_declaration(property, property.is_magic(), false));
    }

    for required in metadata.require_extends.iter().chain(metadata.require_implements.iter()) {
        if let Some(property) = codebase.get_class_like(required.as_bytes()).and_then(|metadata| {
            resolve_declared_property(codebase, metadata, reference.name, !reference.static_access, scope)
        }) && (!reference.static_access || property.property.flags.is_static())
        {
            return Some(with_declaration(property, false, false));
        }
    }

    if reference.static_access {
        return None;
    }

    let default_object = TObject::new_named(class);
    let object = receiver_type
        .and_then(|union| union.types.iter().find_map(|atomic| find_receiver_object(atomic, class)))
        .unwrap_or(&default_object);
    let property = find_property_in_mixins(codebase, metadata, object, &metadata.mixins, reference.name)?;
    let uncertain =
        !property.is_magic() && metadata.flags.is_final() && !codebase.method_exists(class.as_bytes(), b"__get");
    Some(with_declaration(property, property.is_magic(), uncertain))
}

fn find_receiver_object(atomic: &TAtomic, class: Word) -> Option<&TObject> {
    match atomic {
        TAtomic::Object(object)
            if object.get_name().is_some_and(|name| name.as_bytes().eq_ignore_ascii_case(class.as_bytes())) =>
        {
            Some(object)
        }
        TAtomic::GenericParameter(parameter) => {
            parameter.constraint.types.iter().find_map(|atomic| find_receiver_object(atomic, class))
        }
        _ => atomic.get_intersection_types()?.iter().find_map(|atomic| find_receiver_object(atomic, class)),
    }
}

impl MemberReference {
    pub(crate) fn property(origin: ReferenceOrigin, receiver: Span, selector: Span, name: Word) -> Self {
        Self {
            selector_span: (selector.start.offset, selector.end.offset),
            receiver_span: (receiver.start.offset, receiver.end.offset),
            origin,
            kind: MemberKind::Property,
            lookup_class: None,
            name,
            declaration: None,
            static_access: false,
            dynamic: false,
            write: false,
            magic: false,
            uncertain: false,
        }
    }
}

impl AnalysisArtifacts {
    pub(crate) fn with_editor_artifacts(mut self, enabled: bool) -> Self {
        self.editor = enabled.then(EditorArtifacts::default);
        self
    }

    /// Copies editor data without retaining analyzer Rc values or arenas.
    #[must_use]
    pub fn editor_data(&self, source: &[u8], codebase: &CodebaseMetadata) -> Option<EditorFileData> {
        let editor = self.editor.as_ref()?;
        let mut methods: Vec<_> = editor.methods.iter().copied().collect();
        methods.sort_unstable();
        let mut members: Vec<_> = editor.members.iter().copied().collect();
        members.sort_unstable();
        let mut expression_classes: Vec<_> = self
            .expression_types
            .iter()
            .filter_map(|(&span, union)| ExpressionClasses::from_union(span, union))
            .collect();
        expression_classes.sort_unstable_by_key(|types| types.span);
        Some(EditorFileData {
            methods: methods.into_boxed_slice(),
            members: members.into_boxed_slice(),
            expression_classes: expression_classes.into_boxed_slice(),
            variable_types: VariableTypes::collect(source, &self.expression_types),
            callable_types: CallableTypes::collect(&self.expression_types, codebase),
        })
    }
}

fn collect_classes(atomic: &TAtomic, classes: &mut Vec<Word>) -> bool {
    let mut complete = match atomic {
        TAtomic::Object(TObject::Named(object)) => {
            classes.push(object.get_name());
            true
        }
        TAtomic::Object(TObject::Enum(object)) => {
            classes.push(object.get_name());
            true
        }
        TAtomic::GenericParameter(parameter) => {
            let mut complete = true;
            for atomic in parameter.constraint.types.iter() {
                complete &= collect_classes(atomic, classes);
            }

            complete
        }
        _ => atomic.is_null(),
    };

    for atomic in atomic.get_intersection_types().unwrap_or_default() {
        complete &= collect_classes(atomic, classes);
    }

    complete
}

#[cfg(test)]
mod tests {
    use mago_word::word;

    use super::*;

    #[test]
    fn editor_records_follow_nested_checkpoints_and_deduplicate() {
        fn assert_send_sync<T>()
        where
            T: Send + Sync,
        {
        }

        assert_send_sync::<EditorFileData>();
        let codebase = CodebaseMetadata::default();
        let mut artifacts = AnalysisArtifacts::new().with_editor_artifacts(true);
        let reference = MethodReference {
            selector_span: (10, 13),
            expression_span: (5, 15),
            receiver_span: (5, 8),
            origin: ReferenceOrigin::File(word("test.php")),
            receiver_class: word("Child"),
            method: MethodIdentifier::new(word("base"), word("run")),
            declaration: None,
            static_access: false,
            dynamic: false,
            magic: false,
            uncertain: false,
        };

        artifacts.begin_expression_type_checkpoint();
        if let Some(editor) = &mut artifacts.editor {
            assert!(editor.methods.insert(reference));
            assert!(!editor.methods.insert(reference));
            let member =
                MemberReference::property(reference.origin, Span::dummy(5, 8), Span::dummy(10, 13), word("$id"));
            assert!(editor.members.insert(member));
            assert!(!editor.members.insert(member));
        }

        artifacts.begin_expression_type_checkpoint();
        artifacts.finish_expression_type_checkpoint(CheckpointAction::Keep);
        assert_eq!(artifacts.editor_data(b"", &codebase).map(|data| data.methods.len()), Some(1));
        assert_eq!(artifacts.editor_data(b"", &codebase).map(|data| data.members.len()), Some(1));
        artifacts.finish_expression_type_checkpoint(CheckpointAction::Rollback);
        assert_eq!(artifacts.editor_data(b"", &codebase).map(|data| data.methods.len()), Some(0));
        assert_eq!(artifacts.editor_data(b"", &codebase).map(|data| data.members.len()), Some(0));
        assert!(AnalysisArtifacts::new().editor_data(b"", &codebase).is_none());
    }
}
