use mago_allocator::Arena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordSet;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::expression::assignment::PropertyWriteKind;
use crate::statement::class_like::initialization::compute_class_initializer_initializations;
use crate::statement::class_like::initialization::compute_transitive_initializations;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalInitializationState {
    Uninitialized,
    EntryDependent,
    PossiblyInitialized,
    Initialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryContext {
    OrdinaryMethod,
    ClassInitializer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvalidWriteReason {
    AlreadyInitialized,
    Mutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingCertainty {
    Invalid(InvalidWriteReason),
    Possible,
    EntryDependent(EntryContext),
}

#[derive(Debug, Clone)]
pub(crate) struct PendingReadonlyPropertyWrite {
    current_class: Word,
    current_method: Word,
    declaring_class: Word,
    property_name: Word,
    access_span: Span,
    member_span: Span,
    definition_span: Option<Span>,
    certainty: PendingCertainty,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn check_property_write<A>(
    context: &Context<'_, '_, A>,
    block_context: &BlockContext<'_>,
    artifacts: &mut AnalysisArtifacts,
    declaring_class: Word,
    property_name: Word,
    property_access_id: Option<Word>,
    access_span: Span,
    member_span: Span,
    write_kind: PropertyWriteKind,
) where
    A: Arena,
{
    let Some(declaring_class_metadata) = context.codebase.get_class_like(declaring_class.as_bytes()) else {
        return;
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(&property_name) else {
        return;
    };

    if !property_metadata.flags.is_readonly() {
        return;
    }

    let Some(current_class) = block_context.scope.get_class_like_name() else {
        return;
    };

    let Some(function_like) = block_context.scope.get_function_like() else {
        return;
    };

    let property_can_be_null =
        property_metadata.type_declaration_metadata.as_ref().is_none_or(|metadata| metadata.type_union.can_be_null());

    let current_value_type = property_access_id.and_then(|id| block_context.locals.get(&id));
    let guard_proves_uninitialized = !property_can_be_null
        && property_access_id.is_some_and(|id| block_context.definitely_uninitialized_property_ids.contains(&id));

    let is_constructor = function_like.method_metadata.as_ref().is_some_and(|method| method.is_constructor);
    let is_clone_reinitialization = function_like.method_metadata.is_some()
        && context.settings.version.is_supported(Feature::ReadonlyPropertyReinitializationInClone)
        && function_like.name.as_bytes().eq_ignore_ascii_case(b"__clone");

    let is_class_initializer = function_like.method_metadata.is_some()
        && context
            .codebase
            .get_class_like(current_class.as_bytes())
            .is_some_and(|metadata| context.settings.is_class_initializer_for(metadata, function_like.name));

    let mut local_state = if guard_proves_uninitialized {
        LocalInitializationState::Uninitialized
    } else if block_context.definitely_initialized_properties.contains(&property_name) {
        LocalInitializationState::Initialized
    } else if block_context.possibly_initialized_properties.contains(&property_name) {
        LocalInitializationState::PossiblyInitialized
    } else if is_constructor {
        if property_metadata.flags.is_promoted_property()
            || declaring_class_metadata.initialized_properties.contains(&property_name)
        {
            LocalInitializationState::Initialized
        } else {
            LocalInitializationState::Uninitialized
        }
    } else if is_clone_reinitialization {
        LocalInitializationState::Uninitialized
    } else {
        LocalInitializationState::EntryDependent
    };

    if write_kind == PropertyWriteKind::Direct
        && local_state == LocalInitializationState::Uninitialized
        && block_context.flags.inside_loop()
    {
        local_state = LocalInitializationState::PossiblyInitialized;
    }

    let current_value_is_non_null = current_value_type.is_some_and(|property_type| {
        !property_type.can_be_null()
            && !property_type.possibly_undefined()
            && !property_type.possibly_undefined_from_try()
    });
    let current_value_is_null = current_value_type.is_some_and(|property_type| {
        property_type.is_null() && !property_type.possibly_undefined() && !property_type.possibly_undefined_from_try()
    });

    let certainty = match write_kind {
        PropertyWriteKind::Mutation => PendingCertainty::Invalid(InvalidWriteReason::Mutation),
        PropertyWriteKind::Coalesce if !property_can_be_null || current_value_is_non_null => return,
        PropertyWriteKind::Coalesce => match local_state {
            LocalInitializationState::Uninitialized => return,
            LocalInitializationState::Initialized if current_value_is_null => {
                PendingCertainty::Invalid(InvalidWriteReason::AlreadyInitialized)
            }
            LocalInitializationState::EntryDependent
            | LocalInitializationState::PossiblyInitialized
            | LocalInitializationState::Initialized => PendingCertainty::Possible,
        },
        PropertyWriteKind::Direct => match local_state {
            LocalInitializationState::Uninitialized => return,
            LocalInitializationState::Initialized => PendingCertainty::Invalid(InvalidWriteReason::AlreadyInitialized),
            LocalInitializationState::PossiblyInitialized => PendingCertainty::Possible,
            LocalInitializationState::EntryDependent => PendingCertainty::EntryDependent(if is_class_initializer {
                EntryContext::ClassInitializer
            } else {
                EntryContext::OrdinaryMethod
            }),
        },
    };

    artifacts.pending_readonly_property_writes.push(PendingReadonlyPropertyWrite {
        current_class,
        current_method: function_like.name,
        declaring_class,
        property_name,
        access_span,
        member_span,
        definition_span: property_metadata.span.or(property_metadata.name_span),
        certainty,
    });
}

pub(crate) fn finalize_class_writes<A>(
    context: &mut Context<'_, '_, A>,
    artifacts: &mut AnalysisArtifacts,
    class_metadata: &ClassLikeMetadata,
) where
    A: Arena,
{
    check_parent_constructor_reinitializations(context, artifacts, class_metadata);

    if artifacts.pending_readonly_property_writes.is_empty() {
        return;
    }

    let constructor_initialized = compute_constructor_initializations(context, artifacts, class_metadata);
    let mut lifecycle_initialized = constructor_initialized.clone();
    lifecycle_initialized.extend(compute_class_initializer_initializations(artifacts, context, class_metadata));

    let initializer_reachable =
        collect_reachable_methods(artifacts, class_metadata, initialization_roots(context, class_metadata));
    let clone_reachable = if context.settings.version.is_supported(Feature::ReadonlyPropertyReinitializationInClone) {
        collect_reachable_methods(artifacts, class_metadata, [word(b"__clone")])
    } else {
        WordSet::default()
    };

    let pending_writes = std::mem::take(&mut artifacts.pending_readonly_property_writes);
    for pending in pending_writes {
        if pending.current_class != class_metadata.name {
            artifacts.pending_readonly_property_writes.push(pending);
            continue;
        }

        let certainty = match pending.certainty {
            PendingCertainty::EntryDependent(EntryContext::ClassInitializer) => {
                if constructor_initialized.contains(&pending.property_name) {
                    Some(PendingCertainty::Invalid(InvalidWriteReason::AlreadyInitialized))
                } else {
                    None
                }
            }
            PendingCertainty::EntryDependent(EntryContext::OrdinaryMethod) => {
                if initializer_reachable.contains(&pending.current_method) {
                    if method_is_exclusive_to_reachable_set(
                        context,
                        artifacts,
                        class_metadata,
                        pending.current_method,
                        &initializer_reachable,
                    ) {
                        None
                    } else {
                        Some(PendingCertainty::Possible)
                    }
                } else if clone_reachable.contains(&pending.current_method) {
                    if method_is_exclusive_to_reachable_set(
                        context,
                        artifacts,
                        class_metadata,
                        pending.current_method,
                        &clone_reachable,
                    ) {
                        None
                    } else {
                        Some(PendingCertainty::Possible)
                    }
                } else if lifecycle_initialized.contains(&pending.property_name) {
                    Some(PendingCertainty::Invalid(InvalidWriteReason::AlreadyInitialized))
                } else {
                    Some(PendingCertainty::Possible)
                }
            }
            certainty => Some(certainty),
        };

        let Some(certainty) = certainty else {
            continue;
        };

        match certainty {
            PendingCertainty::Invalid(reason) => report_invalid_write(context, &pending, reason),
            PendingCertainty::Possible => report_possible_write(context, &pending),
            PendingCertainty::EntryDependent(_) => unreachable!(),
        }
    }
}

/// Report promoted readonly properties that a parent constructor initializes again.
///
/// PHP initializes promoted properties before the constructor body. Therefore, a child
/// constructor cannot promote a readonly property that its parent constructor initializes
/// when it calls `parent::__construct()`.
fn check_parent_constructor_reinitializations<A>(
    context: &mut Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    class_metadata: &ClassLikeMetadata,
) where
    A: Arena,
{
    let constructor_name = word(b"__construct");
    let Some(constructor_id) = class_metadata.declaring_method_ids.get(&constructor_name) else {
        return;
    };

    // An inherited constructor has no child promotion before it runs, and only a constructor
    // declared on this class can contain a direct `parent::__construct()` call.
    if constructor_id.get_class_name() != class_metadata.name
        || artifacts.method_calls_parent_constructor.get(&(class_metadata.name, constructor_name)) != Some(&true)
    {
        return;
    }

    if !class_metadata.properties.iter().any(|(&property_name, property_metadata)| {
        class_metadata.declaring_property_ids.get(&property_name) == Some(&class_metadata.name)
            && property_metadata.flags.is_promoted_property()
            && property_metadata.flags.is_readonly()
    }) {
        return;
    }

    let Some(parent_name) = class_metadata.direct_parent_class else {
        return;
    };
    let Some(parent_metadata) = context.codebase.get_class_like(parent_name.as_bytes()) else {
        return;
    };
    let Some(parent_constructor_id) = parent_metadata.declaring_method_ids.get(&constructor_name) else {
        return;
    };

    let parent_constructor_class = parent_constructor_id.get_class_name();
    let Some(parent_constructor_metadata) = context.codebase.get_class_like(parent_constructor_class.as_bytes()) else {
        return;
    };
    let parent_constructor_writes =
        compute_constructor_property_writes(context, artifacts, parent_constructor_metadata);

    for (&property_name, property_metadata) in &class_metadata.properties {
        if class_metadata.declaring_property_ids.get(&property_name) != Some(&class_metadata.name)
            || !property_metadata.flags.is_promoted_property()
            || !property_metadata.flags.is_readonly()
            || !parent_constructor_writes.contains(&property_name)
        {
            continue;
        }

        let Some(parent_property_class) = parent_constructor_metadata
            .declaring_property_ids
            .get(&property_name)
            .and_then(|class_name| context.codebase.get_class_like(class_name.as_bytes()))
        else {
            continue;
        };
        let Some(parent_property_metadata) = parent_property_class.properties.get(&property_name) else {
            continue;
        };

        // Private properties are distinct slots, so the parent writes its own property rather
        // than the promoted property declared on the child.
        if parent_property_metadata.is_final() {
            continue;
        }

        report_parent_constructor_reinitialization(
            context,
            class_metadata,
            property_name,
            property_metadata.name_span.unwrap_or(class_metadata.span),
            parent_property_class,
            parent_property_metadata.name_span.unwrap_or(parent_property_class.span),
        );
    }
}

/// Collect writes performed while running a constructor, including implicit writes from
/// promoted properties. `method_initialized_properties` records explicit assignments only.
fn compute_constructor_property_writes<A>(
    context: &Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    constructor_class: &ClassLikeMetadata,
) -> WordSet
where
    A: Arena,
{
    let constructor_name = word(b"__construct");
    let mut writes = WordSet::default();
    let mut pending_classes = vec![constructor_class.name];
    let mut visited_classes = WordSet::default();

    while let Some(class_name) = pending_classes.pop() {
        if !visited_classes.insert(class_name) {
            continue;
        }

        let Some(class_metadata) = context.codebase.get_class_like(class_name.as_bytes()) else {
            continue;
        };
        let Some(constructor_id) = class_metadata.declaring_method_ids.get(&constructor_name) else {
            continue;
        };

        let declaring_class = constructor_id.get_class_name();
        if declaring_class != class_name {
            pending_classes.push(declaring_class);
            continue;
        }

        for (&property_name, property_declaring_class) in &class_metadata.declaring_property_ids {
            if *property_declaring_class != class_name {
                continue;
            }

            if class_metadata
                .properties
                .get(&property_name)
                .is_some_and(|property| property.flags.is_promoted_property())
            {
                writes.insert(property_name);
            }
        }

        writes.extend(compute_transitive_initializations(
            artifacts,
            context,
            class_name,
            constructor_name,
            class_metadata.flags.is_final(),
            false,
        ));

        if artifacts.method_calls_parent_constructor.get(&(class_name, constructor_name)) != Some(&true) {
            continue;
        }
        let Some(parent_name) = class_metadata.direct_parent_class else {
            continue;
        };
        let Some(parent_metadata) = context.codebase.get_class_like(parent_name.as_bytes()) else {
            continue;
        };
        let Some(parent_constructor_id) = parent_metadata.declaring_method_ids.get(&constructor_name) else {
            continue;
        };

        pending_classes.push(parent_constructor_id.get_class_name());
    }

    writes
}

fn report_parent_constructor_reinitialization<A>(
    context: &mut Context<'_, '_, A>,
    class_metadata: &ClassLikeMetadata,
    property_name: Word,
    property_span: Span,
    parent_property_class: &ClassLikeMetadata,
    parent_property_span: Span,
) where
    A: Arena,
{
    let class_name = class_metadata.original_name;
    let parent_class_name = parent_property_class.original_name;

    context.collector.report_with_code(
        IssueCode::InvalidPropertyWrite,
        Issue::error(format!(
            "Readonly property `{class_name}::{property_name}` is initialized before the parent constructor runs."
        ))
        .with_annotation(
            Annotation::primary(property_span)
                .with_message("This promoted property is initialized before the constructor body"),
        )
        .with_annotation(
            Annotation::secondary(parent_property_span).with_message(format!(
                "`{parent_class_name}::__construct()` initializes this inherited property again"
            )),
        )
        .with_note(
            "Promoted properties are initialized before the constructor body. A parent constructor that initializes the same property performs a second write, which throws an `Error` at runtime.",
        )
        .with_help(
            "Pass inherited properties to `parent::__construct()` as regular parameters; promote only properties initialized by this class.",
        ),
    );
}

fn compute_constructor_initializations<A>(
    context: &Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    class_metadata: &ClassLikeMetadata,
) -> WordSet
where
    A: Arena,
{
    let mut initialized = class_metadata.initialized_properties.clone();

    for (&property_name, declaring_class) in &class_metadata.declaring_property_ids {
        if *declaring_class != class_metadata.name {
            continue;
        }

        let Some(property_metadata) = class_metadata.properties.get(&property_name) else {
            continue;
        };

        if property_metadata.flags.is_promoted_property() {
            initialized.insert(property_name);
        }
    }

    let Some(constructor_id) = class_metadata.declaring_method_ids.get(&word(b"__construct")) else {
        return initialized;
    };

    let constructor_class = constructor_id.get_class_name();
    initialized.extend(compute_transitive_initializations(
        artifacts,
        context,
        constructor_class,
        word(b"__construct"),
        class_metadata.flags.is_final(),
        false,
    ));

    initialized
}

fn initialization_roots<A>(context: &Context<'_, '_, A>, class_metadata: &ClassLikeMetadata) -> Vec<Word>
where
    A: Arena,
{
    let mut roots = Vec::new();
    if class_metadata.declaring_method_ids.contains_key(&word(b"__construct")) {
        roots.push(word(b"__construct"));
    }
    roots.extend(
        context
            .settings
            .applicable_class_initializers(class_metadata)
            .filter(|method| class_metadata.declaring_method_ids.contains_key(method)),
    );

    roots
}

fn collect_reachable_methods(
    artifacts: &AnalysisArtifacts,
    class_metadata: &ClassLikeMetadata,
    roots: impl IntoIterator<Item = Word>,
) -> WordSet {
    let mut visited = WordSet::default();
    let mut worklist = roots.into_iter().collect::<Vec<_>>();
    while let Some(method) = worklist.pop() {
        if !visited.insert(method) {
            continue;
        }

        if let Some(called_methods) = artifacts.method_calls_this_methods.get(&(class_metadata.name, method)) {
            worklist.extend(called_methods.iter().copied());
        }
    }

    visited
}

fn method_is_exclusive_to_reachable_set<A>(
    context: &Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    class_metadata: &ClassLikeMetadata,
    target_method: Word,
    reachable: &WordSet,
) -> bool
where
    A: Arena,
{
    if !context
        .codebase
        .get_method_visibility(class_metadata.name.as_bytes(), target_method.as_bytes())
        .is_some_and(|visibility| visibility.is_private())
    {
        return false;
    }

    for (&(calling_class, calling_method), called_methods) in &artifacts.method_calls_this_methods {
        if calling_class == class_metadata.name
            && called_methods.contains(&target_method)
            && !reachable.contains(&calling_method)
        {
            return false;
        }
    }

    true
}

fn report_invalid_write<A>(
    context: &mut Context<'_, '_, A>,
    pending: &PendingReadonlyPropertyWrite,
    reason: InvalidWriteReason,
) where
    A: Arena,
{
    let declaring_class = context
        .codebase
        .get_class_like(pending.declaring_class.as_bytes())
        .map_or(pending.declaring_class, |metadata| metadata.original_name);

    let (title, primary, note, help) = match reason {
        InvalidWriteReason::AlreadyInitialized => (
            "Cannot modify a readonly property after initialization.",
            "This readonly property is already initialized",
            "Readonly properties may be initialized only once. Every later assignment throws an `Error` at runtime.",
            "Remove this assignment or move the property's one-time initialization to this location.",
        ),
        InvalidWriteReason::Mutation => (
            "Cannot modify a readonly property.",
            "This operation modifies a readonly property",
            "Compound assignments, increments, references, and writes through an array offset are modifications, not initializing assignments.",
            "Initialize the property with a direct assignment instead of modifying it in place.",
        ),
    };

    let mut issue = Issue::error(title)
        .with_annotation(Annotation::primary(pending.member_span).with_message(primary))
        .with_annotation(
            Annotation::secondary(pending.access_span)
                .with_message(format!("Write to `{declaring_class}::{}` occurs here", pending.property_name)),
        )
        .with_note(note)
        .with_help(help);

    if let Some(definition_span) = pending.definition_span {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span).with_message("Property is defined as `readonly` here"),
        );
    }

    context.collector.report_with_code(IssueCode::InvalidPropertyWrite, issue);
}

fn report_possible_write<A>(context: &mut Context<'_, '_, A>, pending: &PendingReadonlyPropertyWrite)
where
    A: Arena,
{
    let declaring_class = context
        .codebase
        .get_class_like(pending.declaring_class.as_bytes())
        .map_or(pending.declaring_class, |metadata| metadata.original_name);

    let mut issue = Issue::warning(format!(
        "Readonly property `{declaring_class}::{}` may already be initialized.",
        pending.property_name
    ))
    .with_annotation(
        Annotation::primary(pending.member_span)
            .with_message("This assignment succeeds only while the property is uninitialized"),
    )
    .with_annotation(
        Annotation::secondary(pending.access_span)
            .with_message(format!("Potential repeated initialization in `{}`", pending.current_method)),
    )
    .with_note(
        "PHP permits the first initialization from an allowed write scope, but a repeated assignment throws an `Error` at runtime.",
    )
    .with_help(
        "Initialize the property in `__construct()`, guard a non-nullable property with `isset()` or `??=`, or configure a framework lifecycle method as a class initializer.",
    );

    if let Some(definition_span) = pending.definition_span {
        issue = issue.with_annotation(
            Annotation::secondary(definition_span).with_message("Property is defined as `readonly` here"),
        );
    }

    context.collector.report_with_code(IssueCode::PossiblyInvalidPropertyWrite, issue);
}
