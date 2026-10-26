use foldhash::HashSet;
use mago_allocator::Arena;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::parameter::CallableInvocationKind;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::cast::cast_atomic_to_callable;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Access;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Variable;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::CallableStorageLocation;
use crate::artifacts::DeferredCallableCallsiteContext;
use crate::code::IssueCode;
use crate::context::Context;
use crate::invocation::InvocationArgumentsSource;
use crate::settings::normalize_allowed_checked_exception_callable;
use crate::statement::function_like::is_exception_unchecked;
use crate::utils::misc::unwrap_expression;

pub(crate) fn is_local_callable_storage_expression(expression: &Expression<'_>) -> bool {
    match unwrap_expression(expression) {
        Expression::Variable(Variable::Direct(variable)) => !is_superglobal(variable.name),
        Expression::ArrayAccess(array_access) => is_local_callable_storage_expression(array_access.array),
        _ => false,
    }
}

pub(crate) fn callable_storage_target_escapes_scope(expression: &Expression<'_>) -> bool {
    match unwrap_expression(expression) {
        Expression::Access(Access::Property(_) | Access::NullSafeProperty(_) | Access::StaticProperty(_)) => true,
        Expression::Variable(Variable::Direct(variable)) => is_superglobal(variable.name),
        Expression::ArrayAccess(array_access) => callable_storage_target_escapes_scope(array_access.array),
        _ => false,
    }
}

pub(crate) fn callable_storage_target_is_static_local(expression: &Expression<'_>, static_locals: &WordSet) -> bool {
    match unwrap_expression(expression) {
        Expression::Variable(Variable::Direct(variable)) => static_locals.contains(&word(variable.name)),
        Expression::ArrayAccess(array_access) => {
            callable_storage_target_is_static_local(array_access.array, static_locals)
        }
        _ => false,
    }
}

pub(crate) fn static_local_expression_may_be_nullish(
    expression: &Expression<'_>,
    expression_type: Option<&TUnion>,
    artifacts: &AnalysisArtifacts,
) -> bool {
    let Some(static_local_types) = artifacts.static_local_types.as_ref() else {
        return false;
    };

    match unwrap_expression(expression) {
        Expression::Variable(Variable::Direct(variable)) => {
            static_local_types.get(&word(variable.name)).is_some_and(|variable_type| type_may_be_nullish(variable_type))
        }
        Expression::ArrayAccess(array_access) => static_local_root(array_access.array).is_some_and(|variable| {
            static_local_types.contains_key(&variable) && expression_type.is_none_or(type_may_be_nullish)
        }),
        _ => false,
    }
}

fn static_local_root(expression: &Expression<'_>) -> Option<mago_word::Word> {
    match unwrap_expression(expression) {
        Expression::Variable(Variable::Direct(variable)) => Some(word(variable.name)),
        Expression::ArrayAccess(array_access) => static_local_root(array_access.array),
        _ => None,
    }
}

fn type_may_be_nullish(union: &TUnion) -> bool {
    union.has_nullish() || union.possibly_undefined() || union.possibly_undefined_from_try()
}

fn is_superglobal(name: &[u8]) -> bool {
    matches!(
        name,
        b"$GLOBALS"
            | b"$_SERVER"
            | b"$_GET"
            | b"$_POST"
            | b"$_FILES"
            | b"$_COOKIE"
            | b"$_SESSION"
            | b"$_REQUEST"
            | b"$_ENV"
    )
}

pub(crate) fn record_stored_callables(
    artifacts: &mut AnalysisArtifacts,
    value_type: &TUnion,
    codebase: &CodebaseMetadata,
    escapes_scope: bool,
    storage_location: Option<CallableStorageLocation>,
) {
    let mut identifiers = HashSet::default();

    for atomic in value_type.types.iter() {
        collect_callable_identifiers(atomic, codebase, &mut identifiers);
    }

    for identifier in identifiers {
        artifacts.record_callable_storage(identifier, escapes_scope, storage_location);
    }
}

pub(crate) fn record_stored_array_callables(
    artifacts: &mut AnalysisArtifacts,
    value_type: &TUnion,
    codebase: &CodebaseMetadata,
    escapes_scope: bool,
    storage_location: Option<CallableStorageLocation>,
) {
    for identifier in stored_array_callable_identifiers(value_type, codebase) {
        artifacts.record_callable_storage(identifier, escapes_scope, storage_location);
    }
}

pub(crate) fn refresh_stored_array_callables(
    artifacts: &mut AnalysisArtifacts,
    value_type: &TUnion,
    codebase: &CodebaseMetadata,
    escapes_scope: bool,
    storage_location: CallableStorageLocation,
) {
    artifacts.replace_callable_storage_location(
        storage_location,
        stored_array_callable_identifiers(value_type, codebase),
        escapes_scope,
    );
}

fn stored_array_callable_identifiers(
    value_type: &TUnion,
    codebase: &CodebaseMetadata,
) -> HashSet<FunctionLikeIdentifier> {
    let mut identifiers = HashSet::default();

    for atomic in value_type.types.iter() {
        if matches!(atomic, TAtomic::Array(_)) {
            collect_callable_identifiers(atomic, codebase, &mut identifiers);
        }
    }

    identifiers
}

pub(crate) fn record_array_plus_rhs_callables(
    artifacts: &mut AnalysisArtifacts,
    left_type: &TUnion,
    right_type: &TUnion,
    codebase: &CodebaseMetadata,
    escapes_scope: bool,
    storage_location: Option<CallableStorageLocation>,
) {
    let left_arrays = left_type
        .types
        .iter()
        .filter_map(|atomic| match atomic {
            TAtomic::Array(array) => Some(array),
            _ => None,
        })
        .collect::<Vec<_>>();
    let left_may_be_mixed = left_type.types.iter().any(TAtomic::is_mixed);

    if left_arrays.is_empty() && !left_may_be_mixed {
        return;
    }

    for atomic in right_type.types.iter() {
        let TAtomic::Array(right_array) = atomic else {
            continue;
        };

        let right_items = match right_array {
            TArray::Keyed(array) => array
                .known_items
                .as_deref()
                .map(|items| items.iter().map(|(key, (_, value_type))| (*key, value_type)).collect::<Vec<_>>()),
            TArray::List(array) => array.known_elements.as_deref().map(|items| {
                items
                    .iter()
                    .filter_map(|(index, (_, value_type))| {
                        i64::try_from(*index).ok().map(|index| (ArrayKey::Integer(index), value_type))
                    })
                    .collect::<Vec<_>>()
            }),
        };

        let Some(right_items) = right_items else {
            continue;
        };

        for (array_key, value_type) in right_items {
            if !left_may_be_mixed
                && !left_arrays.is_empty()
                && left_arrays.iter().all(|array| array_guarantees_array_key(array, &array_key))
            {
                continue;
            }

            let mut identifiers = HashSet::default();
            for value_atomic in value_type.types.iter() {
                collect_callable_identifiers(value_atomic, codebase, &mut identifiers);
            }

            for identifier in identifiers {
                artifacts.record_callable_storage(identifier, escapes_scope, storage_location);
            }
        }
    }
}

fn array_guarantees_array_key(array: &TArray, key: &ArrayKey) -> bool {
    match array {
        TArray::Keyed(array) => {
            array.known_items.as_deref().and_then(|items| items.get(key)).is_some_and(|(optional, _)| !optional)
        }
        TArray::List(array) => {
            let ArrayKey::Integer(index) = key else {
                return false;
            };

            let Ok(index) = usize::try_from(*index) else {
                return false;
            };

            (array.non_empty && index == 0)
                || array
                    .known_elements
                    .as_deref()
                    .and_then(|items| items.get(&index))
                    .is_some_and(|(optional, _)| !optional)
                || array.known_count.is_some_and(|count| index < count)
        }
    }
}

pub(crate) fn record_unresolved_callable_closure_arguments<A>(
    context: &Context<'_, '_, A>,
    artifacts: &mut AnalysisArtifacts,
    arguments: &InvocationArgumentsSource<'_, '_>,
    invocation_span: Span,
    scope: Option<FunctionLikeIdentifier>,
) where
    A: Arena,
{
    if !context.settings.check_throws || !context.settings.forbid_checked_exceptions_in_callables {
        return;
    }

    for (source_argument_offset, argument) in arguments.iter_arguments().enumerate() {
        let Some(expression) = argument.value().map(unwrap_expression) else {
            continue;
        };

        if !matches!(expression, Expression::Closure(_) | Expression::ArrowFunction(_)) {
            continue;
        }

        let identifier = FunctionLikeIdentifier::for_closure(context.source_file, expression.span());
        artifacts.record_callable_callback_callsite(
            identifier,
            DeferredCallableCallsiteContext {
                invocation_span,
                scope,
                target: None,
                parameter_offset: source_argument_offset,
                invocation_kind: None,
            },
            None,
        );
    }
}

pub(crate) fn record_unresolved_callable_arguments<A>(
    context: &Context<'_, '_, A>,
    artifacts: &mut AnalysisArtifacts,
    arguments: &InvocationArgumentsSource<'_, '_>,
    invocation_span: Span,
    scope: Option<FunctionLikeIdentifier>,
) where
    A: Arena,
{
    if !context.settings.check_throws || !context.settings.forbid_checked_exceptions_in_callables {
        return;
    }

    for (source_argument_offset, argument) in arguments.iter_arguments().enumerate() {
        let Some(expression) = argument.value() else {
            continue;
        };

        let mut identifiers = HashSet::default();
        if let Some(expression_type) = artifacts.get_expression_type(expression) {
            for atomic in expression_type.types.iter() {
                collect_callable_identifiers(atomic, context.codebase, &mut identifiers);
            }
        }

        let expression = unwrap_expression(expression);
        if matches!(expression, Expression::Closure(_) | Expression::ArrowFunction(_)) {
            continue;
        }

        let callsite = DeferredCallableCallsiteContext {
            invocation_span,
            scope,
            target: None,
            parameter_offset: source_argument_offset,
            invocation_kind: None,
        };
        for identifier in identifiers {
            artifacts.record_callable_callback_callsite(identifier, callsite.clone(), None);
        }
    }
}

fn collect_callable_identifiers(
    atomic: &TAtomic,
    codebase: &CodebaseMetadata,
    identifiers: &mut HashSet<FunctionLikeIdentifier>,
) {
    if let Some(callable) = cast_atomic_to_callable(atomic, codebase, None) {
        let identifier = match callable.as_ref() {
            TCallable::Alias(identifier) => Some(*identifier),
            TCallable::Signature(signature) => signature.get_source(),
        };

        if let Some(identifier) = identifier {
            identifiers.insert(identifier);
        }

        return;
    }

    let TAtomic::Array(array) = atomic else {
        return;
    };

    let known_elements = match array {
        TArray::Keyed(array) => array.known_items.as_deref().map(|items| items.values().collect::<Vec<_>>()),
        TArray::List(array) => array.known_elements.as_deref().map(|items| items.values().collect::<Vec<_>>()),
    };

    if let Some(known_elements) = known_elements {
        for (_, value_type) in known_elements {
            for value_atomic in value_type.types.iter() {
                collect_callable_identifiers(value_atomic, codebase, identifiers);
            }
        }
    }
}

pub(crate) fn is_allowed_checked_exception_callable<A>(
    context: &Context<'_, '_, A>,
    callback_parameter: Option<(FunctionLikeIdentifier, usize)>,
) -> bool
where
    A: Arena,
{
    let Some((identifier, parameter_offset)) = callback_parameter else {
        return false;
    };

    let resolved_name = identifier.as_string();
    let allowlist_key = normalize_allowed_checked_exception_callable(&format!(
        "{}:{parameter_offset}",
        resolved_name.trim_start_matches('\\')
    ));
    context.settings.allowed_checked_exception_callables.contains(&allowlist_key)
}

pub(crate) fn has_unallowed_deferred_callable_callsite<A>(
    context: &Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
    identifier: FunctionLikeIdentifier,
) -> bool
where
    A: Arena,
{
    artifacts.callable_callback_callsites.get(&identifier).is_some_and(|callsites| {
        callsites.iter().any(|callsite| {
            !is_allowed_checked_exception_callable(
                context,
                callsite.target.map(|target| (target, callsite.parameter_offset)),
            )
        })
    })
}

/// Reports checked exceptions that can escape a callable which may run later.
pub(crate) fn report_deferred_callable_exceptions<A>(
    context: &mut Context<'_, '_, A>,
    callable_span: Span,
    possibly_thrown_exceptions: &WordMap<HashSet<Span>>,
    callback_parameter: Option<(FunctionLikeIdentifier, usize)>,
    already_reported_exceptions: Option<&WordSet>,
) -> WordSet
where
    A: Arena,
{
    if !context.settings.check_throws || !context.settings.forbid_checked_exceptions_in_callables {
        return WordSet::default();
    }

    if is_allowed_checked_exception_callable(context, callback_parameter) {
        return WordSet::default();
    }

    let mut reported_exceptions = WordSet::default();
    let target_description = callback_parameter
        .map(|(identifier, _)| format!("callback passed to `{}`", identifier.as_string()))
        .unwrap_or_else(|| "callable that may be invoked later".to_string());

    for (exception, spans) in possibly_thrown_exceptions {
        if is_exception_unchecked(context, *exception)
            || already_reported_exceptions.is_some_and(|reported| reported.contains(exception))
        {
            continue;
        }

        let mut issue = Issue::error(format!("Potentially unhandled exception `{exception}` in {target_description}."));
        for span in spans {
            issue = issue.with_annotation(Annotation::primary(*span).with_message("Exception may be thrown here"));
        }

        issue = issue
            .with_annotation(
                Annotation::secondary(callable_span).with_message("This callable can escape before it is invoked"),
            )
            .with_note("Declaring `@throws` does not handle an exception in a deferred callable.")
            .with_help("Catch the exception inside the callable or allow this API callback parameter explicitly.");

        context.collector.report_with_code(IssueCode::UnhandledThrownType, issue);
        reported_exceptions.insert(*exception);
    }

    reported_exceptions
}

/// Reports deferred callback exceptions and stored callable exceptions for a file.
pub(crate) fn report_deferred_callable_exceptions_for_file<A>(
    context: &mut Context<'_, '_, A>,
    artifacts: &AnalysisArtifacts,
) where
    A: Arena,
{
    if !context.settings.check_throws || !context.settings.forbid_checked_exceptions_in_callables {
        return;
    }

    let named_callable_identifiers = artifacts
        .callable_callback_callsites
        .keys()
        .chain(artifacts.callable_usage.keys())
        .copied()
        .collect::<HashSet<_>>();

    for identifier in named_callable_identifiers {
        if identifier.is_closure() {
            continue;
        }

        let Some((span, effects)) = artifacts
            .get_callable_effects_by_identifier(identifier)
            .map(|(span, effects)| (span, effects.possibly_thrown_exceptions.clone()))
            .or_else(|| get_declared_callable_effects(context, identifier))
        else {
            continue;
        };

        let has_callback_callsite = artifacts.callable_callback_callsites.contains_key(&identifier);
        let mut already_reported_exceptions =
            artifacts.get_reported_callable_exceptions(identifier).cloned().unwrap_or_default();
        if let Some(callsites) = artifacts.callable_callback_callsites.get(&identifier) {
            for callsite in callsites {
                if callsite.invocation_kind == Some(CallableInvocationKind::Immediately) {
                    continue;
                }

                let reported_exceptions = report_deferred_callable_exceptions(
                    context,
                    span,
                    &effects,
                    callsite.target.map(|target| (target, callsite.parameter_offset)),
                    Some(&already_reported_exceptions),
                );
                already_reported_exceptions.extend(reported_exceptions);
            }
        }

        if artifacts.should_report_stored_callable(identifier)
            && (!has_callback_callsite
                || artifacts.callable_escapes_scope(identifier)
                || artifacts.has_uninvoked_callable_storage_location(identifier))
        {
            report_deferred_callable_exceptions(context, span, &effects, None, Some(&already_reported_exceptions));
        }
    }

    let callable_effects = artifacts.iter_callable_effects_by_identifier().collect::<Vec<_>>();

    for (identifier, span, effects) in callable_effects {
        if !identifier.is_closure() {
            continue;
        }

        let callsites = artifacts.callable_callback_callsites.get(&identifier);
        let has_callback_callsite = callsites.is_some();
        let report_stored_callable = artifacts.should_report_stored_callable(identifier)
            && (!has_callback_callsite
                || artifacts.callable_escapes_scope(identifier)
                || artifacts.has_uninvoked_callable_storage_location(identifier));
        let has_deferred_callsite = callsites.is_some_and(|callsites| {
            callsites.iter().any(|callsite| callsite.invocation_kind != Some(CallableInvocationKind::Immediately))
        });

        if !report_stored_callable && !has_deferred_callsite {
            continue;
        }

        let mut already_reported_exceptions =
            artifacts.get_reported_callable_exceptions(identifier).cloned().unwrap_or_default();
        if let Some(callsites) = callsites {
            for callsite in callsites {
                if callsite.invocation_kind == Some(CallableInvocationKind::Immediately) {
                    continue;
                }

                let reported_exceptions = report_deferred_callable_exceptions(
                    context,
                    span,
                    &effects.possibly_thrown_exceptions,
                    callsite.target.map(|target| (target, callsite.parameter_offset)),
                    Some(&already_reported_exceptions),
                );
                already_reported_exceptions.extend(reported_exceptions);
            }
        }

        if report_stored_callable {
            report_deferred_callable_exceptions(
                context,
                span,
                &effects.possibly_thrown_exceptions,
                None,
                Some(&already_reported_exceptions),
            );
        }
    }
}

fn get_declared_callable_effects<A>(
    context: &Context<'_, '_, A>,
    identifier: FunctionLikeIdentifier,
) -> Option<(Span, WordMap<HashSet<Span>>)>
where
    A: Arena,
{
    let metadata = context.codebase.get_function_like(&identifier)?;
    let class_like = identifier
        .as_method_identifier()
        .and_then(|method| context.codebase.get_class_like(method.get_class_name().as_bytes()));
    let mut effects: WordMap<HashSet<Span>> = WordMap::default();

    for thrown_type in context.codebase.get_function_like_thrown_types(class_like, metadata) {
        for atomic_type in thrown_type.type_union.types.iter() {
            for exception in atomic_type.get_all_object_names() {
                effects.entry(exception).or_default().insert(thrown_type.span);
            }
        }
    }

    Some((metadata.span, effects))
}
