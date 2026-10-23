use std::rc::Rc;
use std::sync::Arc;

use foldhash::HashMap;
use foldhash::HashSet;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::ascii_lowercase_word;

use mago_algebra::assertion_set::AssertionSet;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::combine_owned_union_types_rc;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Node;

use crate::artifacts::editor::EditorArtifacts;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraintSource;
use crate::context::scope::case_scope::CaseScope;
use crate::context::scope::loop_scope::LoopScope;
use crate::readonly::PendingReadonlyPropertyWrite;
use crate::utils::expression::is_derived_access_path;

pub(crate) use self::expression_types::CheckpointAction;
pub(crate) use self::expression_types::ExpressionTypeCheckpoints;

pub mod editor;
mod expression_types;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum VariableDefinedness {
    Defined = 1,
    PossiblyDefined = 2,
}

/// Represents scope information extracted from a `Closure::bind()` or `Closure::bindTo()` call.
/// This is used to pass the bound class scope to closure/arrow function analysis.
#[derive(Debug, Clone)]
pub struct ClosureBindScope {
    /// The class name for the bound scope (from the newScope argument).
    pub class_name: Option<Word>,
    /// Whether the closure has `$this` bound (newThis argument is non-null object).
    pub has_this: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CallableInvocationKind {
    Immediately,
}

/// Exceptions a callable may throw after its local `try`/`catch` handling.
#[derive(Debug, Clone, Default)]
pub(crate) struct CallableEffectSummary {
    /// Exception class names mapped to the source spans that may throw them.
    pub possibly_thrown_exceptions: WordMap<HashSet<Span>>,
}

fn merge_callable_effect_summary(target: &mut CallableEffectSummary, source: CallableEffectSummary) {
    for (exception, spans) in source.possibly_thrown_exceptions {
        target.possibly_thrown_exceptions.entry(exception).or_default().extend(spans);
    }
}

/// The resolved callsite where a callable is supplied as a callback argument.
#[derive(Debug, Clone)]
pub(crate) struct CallbackCallsiteContext {
    /// The resolved function or method being called, when statically known.
    pub(crate) target: Option<FunctionLikeIdentifier>,
    /// The mapped parameter position, or the source argument position if the target parameter is unresolved.
    pub(crate) parameter_offset: usize,
    /// The invocation contract from the parameter metadata, when declared.
    pub(crate) invocation_kind: Option<CallableInvocationKind>,
}

#[derive(Debug, Clone)]
pub(crate) struct DeferredCallableCallsiteContext {
    pub(crate) target: Option<FunctionLikeIdentifier>,
    pub(crate) parameter_offset: usize,
    pub(crate) invocation_kind: Option<CallableInvocationKind>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CallableUsage {
    directly_invoked: bool,
    stored: bool,
    unlocated_stored: bool,
    escapes_scope: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CallableStorageLocation {
    pub scope: Option<FunctionLikeIdentifier>,
    pub variable: Word,
}

/// One semantic receiver and method target retained for a source-level call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedMethodCall {
    pub span: (u32, u32),
    pub class: Word,
    pub method: Word,
}

#[derive(Debug, Clone)]
pub(crate) struct ShortCircuitAssignmentTypes {
    pub when_true: WordMap<Rc<TUnion>>,
    pub when_false: WordMap<Rc<TUnion>>,
}

#[derive(Debug, Clone)]
pub struct AnalysisArtifacts {
    pub expression_types: HashMap<(u32, u32), Rc<TUnion>>,
    pub(crate) short_circuit_assignment_types: HashMap<(u32, u32), ShortCircuitAssignmentTypes>,
    pub if_true_assertions: HashMap<(u32, u32), WordMap<AssertionSet>>,
    pub if_false_assertions: HashMap<(u32, u32), WordMap<AssertionSet>>,
    pub true_branch_only_assertions: HashMap<(u32, u32), WordMap<AssertionSet>>,
    pub inferred_return_types: Vec<Rc<TUnion>>,
    pub inferred_yield_key_types: Vec<TUnion>,
    pub inferred_yield_value_types: Vec<TUnion>,
    pub symbol_references: SymbolReferences,
    pub loop_scope: Option<LoopScope>,
    pub case_scopes: Vec<CaseScope>,
    pub fully_matched_switch_offsets: HashSet<u32>,
    pub inferred_parameter_types: Option<HashMap<usize, TUnion>>,
    pub method_initialized_properties: HashMap<(Word, Word), WordSet>,
    pub method_calls_this_methods: HashMap<(Word, Word), HashSet<Word>>,
    pub method_calls_parent_constructor: HashMap<(Word, Word), bool>,
    pub method_calls_parent_initializer: HashMap<(Word, Word), Word>,
    pub closure_bind_scope: Option<ClosureBindScope>,
    pub(crate) callable_effects: HashMap<(u32, u32), CallableEffectSummary>,
    pub(crate) callable_effects_by_identifier: HashMap<FunctionLikeIdentifier, (Span, CallableEffectSummary)>,
    pub(crate) callback_callsites: HashMap<(u32, u32), CallbackCallsiteContext>,
    pub(crate) callable_callback_callsites: HashMap<FunctionLikeIdentifier, Vec<DeferredCallableCallsiteContext>>,
    pub(crate) callable_usage: HashMap<FunctionLikeIdentifier, CallableUsage>,
    pub(crate) callable_storage_locations: HashMap<FunctionLikeIdentifier, HashSet<CallableStorageLocation>>,
    pub(crate) callable_storage_owners_by_location: HashMap<CallableStorageLocation, HashSet<FunctionLikeIdentifier>>,
    pub(crate) direct_callable_invocation_spans:
        HashMap<FunctionLikeIdentifier, HashSet<(Span, Option<FunctionLikeIdentifier>)>>,
    pub(crate) directly_invoked_callable_storage_locations:
        HashMap<FunctionLikeIdentifier, HashSet<CallableStorageLocation>>,
    pub(crate) callable_callback_storage_locations: HashMap<FunctionLikeIdentifier, HashSet<CallableStorageLocation>>,
    pub(crate) reported_callable_exceptions: HashMap<FunctionLikeIdentifier, WordSet>,
    pub(crate) callback_parameter_invocations: HashMap<(u32, u32), usize>,
    pub(crate) callback_parameter_escapes: HashSet<usize>,
    pub(crate) callback_parameter_effects: HashMap<FunctionLikeIdentifier, HashSet<usize>>,
    pub resolved_method_calls: Vec<ResolvedMethodCall>,
    pub(crate) editor: Option<EditorArtifacts>,
    pub(crate) variable_definedness: HashMap<(u32, u32), WordMap<VariableDefinedness>>,
    variable_definedness_targets: Option<Arc<[bool; u8::MAX as usize + 1]>>,
    pub(crate) pending_readonly_property_writes: Vec<PendingReadonlyPropertyWrite>,
    pub(crate) static_local_types: Option<WordMap<Rc<TUnion>>>,
    expression_type_checkpoints: ExpressionTypeCheckpoints,
    retain_completed_expression_types: bool,
}

impl Default for AnalysisArtifacts {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisArtifacts {
    #[must_use]
    pub fn new() -> Self {
        Self {
            expression_types: HashMap::default(),
            short_circuit_assignment_types: HashMap::default(),
            inferred_return_types: Vec::new(),
            inferred_yield_key_types: Vec::new(),
            inferred_yield_value_types: Vec::new(),
            if_true_assertions: HashMap::default(),
            if_false_assertions: HashMap::default(),
            true_branch_only_assertions: HashMap::default(),
            symbol_references: SymbolReferences::new(),
            case_scopes: Vec::new(),
            loop_scope: None,
            fully_matched_switch_offsets: HashSet::default(),
            inferred_parameter_types: None,
            method_initialized_properties: HashMap::default(),
            method_calls_this_methods: HashMap::default(),
            method_calls_parent_constructor: HashMap::default(),
            method_calls_parent_initializer: HashMap::default(),
            closure_bind_scope: None,
            callable_effects: HashMap::default(),
            callable_effects_by_identifier: HashMap::default(),
            callback_callsites: HashMap::default(),
            callable_callback_callsites: HashMap::default(),
            callable_usage: HashMap::default(),
            callable_storage_locations: HashMap::default(),
            callable_storage_owners_by_location: HashMap::default(),
            direct_callable_invocation_spans: HashMap::default(),
            directly_invoked_callable_storage_locations: HashMap::default(),
            callable_callback_storage_locations: HashMap::default(),
            reported_callable_exceptions: HashMap::default(),
            callback_parameter_invocations: HashMap::default(),
            callback_parameter_escapes: HashSet::default(),
            callback_parameter_effects: HashMap::default(),
            resolved_method_calls: Vec::new(),
            editor: None,
            variable_definedness: HashMap::default(),
            variable_definedness_targets: None,
            pending_readonly_property_writes: Vec::new(),
            static_local_types: None,
            expression_type_checkpoints: ExpressionTypeCheckpoints::default(),
            retain_completed_expression_types: true,
        }
    }

    /// Records the post-catch effects of a closure or arrow function.
    pub(crate) fn record_callable_effects(&mut self, span: Span, effects: CallableEffectSummary) {
        let entry = self.callable_effects.entry((span.start.offset, span.end.offset)).or_default();
        merge_callable_effect_summary(entry, effects);
    }

    pub(crate) fn record_callable_effects_for_identifier(
        &mut self,
        identifier: FunctionLikeIdentifier,
        span: Span,
        effects: CallableEffectSummary,
    ) {
        self.record_callable_effects(span, effects.clone());

        let entry = self
            .callable_effects_by_identifier
            .entry(identifier)
            .or_insert_with(|| (span, CallableEffectSummary::default()));
        for (exception, spans) in effects.possibly_thrown_exceptions {
            entry.1.possibly_thrown_exceptions.entry(exception).or_default().extend(spans);
        }
    }

    /// Returns the post-catch effects recorded for a callable at a source span.
    #[must_use]
    pub(crate) fn get_callable_effects(&self, span: Span) -> Option<&CallableEffectSummary> {
        self.callable_effects.get(&(span.start.offset, span.end.offset))
    }

    pub(crate) fn record_callback_callsite(&mut self, span: Span, callsite: CallbackCallsiteContext) {
        self.callback_callsites.insert((span.start.offset, span.end.offset), callsite);
    }

    pub(crate) fn record_callable_callback_callsite(
        &mut self,
        identifier: FunctionLikeIdentifier,
        callsite: DeferredCallableCallsiteContext,
        storage_location: Option<CallableStorageLocation>,
    ) {
        self.callable_callback_callsites.entry(identifier).or_default().push(callsite);

        if let Some(storage_location) = storage_location {
            self.callable_callback_storage_locations.entry(identifier).or_default().insert(storage_location);
            self.callable_storage_owners_by_location.entry(storage_location).or_default().insert(identifier);
        }
    }

    pub(crate) fn record_direct_callable_invocation(
        &mut self,
        identifier: FunctionLikeIdentifier,
        storage_location: Option<CallableStorageLocation>,
        invocation_span: Span,
        invocation_scope: Option<FunctionLikeIdentifier>,
    ) {
        let canonical_identifier = match identifier {
            FunctionLikeIdentifier::Function(name) => {
                FunctionLikeIdentifier::Function(ascii_lowercase_word(name.as_bytes()))
            }
            FunctionLikeIdentifier::Method(class, method) => FunctionLikeIdentifier::Method(
                ascii_lowercase_word(class.as_bytes()),
                ascii_lowercase_word(method.as_bytes()),
            ),
            FunctionLikeIdentifier::Closure(name) => FunctionLikeIdentifier::Closure(name),
        };
        self.direct_callable_invocation_spans
            .entry(identifier)
            .or_default()
            .insert((invocation_span, invocation_scope));
        self.direct_callable_invocation_spans
            .entry(canonical_identifier)
            .or_default()
            .insert((invocation_span, invocation_scope));

        if identifier.is_closure() {
            self.callable_usage.entry(identifier).or_default().directly_invoked = true;
        }

        if let Some(storage_location) = storage_location {
            let ancestor_storage_locations = self
                .callable_storage_owners_by_location
                .iter()
                .filter_map(|(stored_location, owners)| {
                    (stored_location.scope == storage_location.scope
                        && owners.contains(&identifier)
                        && (stored_location.variable == storage_location.variable
                            || is_derived_access_path(storage_location.variable, stored_location.variable)))
                    .then_some(*stored_location)
                })
                .collect::<Vec<_>>();

            let directly_invoked_locations =
                self.directly_invoked_callable_storage_locations.entry(identifier).or_default();
            directly_invoked_locations.insert(storage_location);
            directly_invoked_locations.extend(ancestor_storage_locations);
            self.callable_storage_owners_by_location.entry(storage_location).or_default().insert(identifier);
        }
    }

    pub(crate) fn record_callable_storage(
        &mut self,
        identifier: FunctionLikeIdentifier,
        escapes_scope: bool,
        storage_location: Option<CallableStorageLocation>,
    ) {
        let usage = self.callable_usage.entry(identifier).or_default();
        usage.stored = true;
        usage.escapes_scope |= escapes_scope;

        if let Some(storage_location) = storage_location {
            self.callable_storage_locations.entry(identifier).or_default().insert(storage_location);
            self.callable_storage_owners_by_location.entry(storage_location).or_default().insert(identifier);
        }

        if storage_location.is_none() || escapes_scope {
            usage.unlocated_stored = true;
        }
    }

    pub(crate) fn replace_callable_storage_location(
        &mut self,
        storage_location: CallableStorageLocation,
        identifiers: HashSet<FunctionLikeIdentifier>,
        escapes_scope: bool,
    ) {
        let Some(previous_identifiers) = self.callable_storage_owners_by_location.remove(&storage_location) else {
            return;
        };

        for identifier in previous_identifiers.iter().copied() {
            if !identifiers.contains(&identifier) {
                self.remove_callable_storage_location_for_identifier(identifier, storage_location);
            }
        }

        for identifier in identifiers {
            if previous_identifiers.contains(&identifier) {
                self.callable_storage_owners_by_location.entry(storage_location).or_default().insert(identifier);

                let usage = self.callable_usage.entry(identifier).or_default();
                usage.stored = true;
                usage.escapes_scope |= escapes_scope;
                usage.unlocated_stored |= escapes_scope;
            } else {
                self.record_callable_storage(identifier, escapes_scope, Some(storage_location));
            }
        }
    }

    pub(crate) fn clear_callable_storage_location(&mut self, storage_location: CallableStorageLocation) {
        let locations_to_clear = self
            .callable_storage_owners_by_location
            .keys()
            .copied()
            .filter(|location| {
                location.scope == storage_location.scope
                    && (location.variable == storage_location.variable
                        || is_derived_access_path(location.variable, storage_location.variable))
            })
            .collect::<Vec<_>>();

        for location in locations_to_clear {
            self.clear_callable_storage_location_exact(location);
        }
    }

    fn clear_callable_storage_location_exact(&mut self, storage_location: CallableStorageLocation) {
        let Some(identifiers) = self.callable_storage_owners_by_location.remove(&storage_location) else {
            return;
        };

        for identifier in identifiers {
            self.remove_callable_storage_location_for_identifier(identifier, storage_location);
        }
    }

    fn remove_callable_storage_location_for_identifier(
        &mut self,
        identifier: FunctionLikeIdentifier,
        storage_location: CallableStorageLocation,
    ) {
        let remove_storage_locations =
            if let Some(storage_locations) = self.callable_storage_locations.get_mut(&identifier) {
                storage_locations.remove(&storage_location);
                storage_locations.is_empty()
            } else {
                false
            };

        if remove_storage_locations {
            self.callable_storage_locations.remove(&identifier);
        }

        for locations in [
            self.directly_invoked_callable_storage_locations.get_mut(&identifier),
            self.callable_callback_storage_locations.get_mut(&identifier),
        ]
        .into_iter()
        .flatten()
        {
            locations.remove(&storage_location);
        }

        let has_locations = self.callable_storage_locations.contains_key(&identifier);
        if let Some(usage) = self.callable_usage.get_mut(&identifier) {
            usage.stored = usage.unlocated_stored || has_locations;
        }
    }

    #[must_use]
    pub(crate) fn callable_escapes_scope(&self, identifier: FunctionLikeIdentifier) -> bool {
        self.callable_usage.get(&identifier).is_some_and(|usage| usage.escapes_scope)
    }

    #[must_use]
    pub(crate) fn has_uninvoked_callable_storage_location(&self, identifier: FunctionLikeIdentifier) -> bool {
        let Some(storage_locations) = self.callable_storage_locations.get(&identifier) else {
            return false;
        };

        let directly_invoked_locations = self.directly_invoked_callable_storage_locations.get(&identifier);
        let callback_locations = self.callable_callback_storage_locations.get(&identifier);

        storage_locations.iter().any(|location| {
            !directly_invoked_locations.is_some_and(|locations| locations.contains(location))
                && !callback_locations.is_some_and(|locations| locations.contains(location))
        })
    }

    pub(crate) fn record_reported_callable_exception(&mut self, identifier: FunctionLikeIdentifier, exception: Word) {
        self.reported_callable_exceptions.entry(identifier).or_default().insert(exception);
    }

    #[must_use]
    pub(crate) fn get_reported_callable_exceptions(&self, identifier: FunctionLikeIdentifier) -> Option<&WordSet> {
        self.reported_callable_exceptions.get(&identifier)
    }

    #[must_use]
    pub(crate) fn get_callable_effects_by_identifier(
        &self,
        identifier: FunctionLikeIdentifier,
    ) -> Option<(Span, &CallableEffectSummary)> {
        self.callable_effects_by_identifier.get(&identifier).map(|(span, effects)| (*span, effects))
    }

    pub(crate) fn iter_callable_effects_by_identifier(
        &self,
    ) -> impl Iterator<Item = (FunctionLikeIdentifier, Span, &CallableEffectSummary)> {
        self.callable_effects_by_identifier.iter().map(|(identifier, (span, effects))| (*identifier, *span, effects))
    }

    #[must_use]
    pub(crate) fn should_report_stored_callable(&self, identifier: FunctionLikeIdentifier) -> bool {
        let Some(usage) = self.callable_usage.get(&identifier) else {
            return false;
        };

        let has_storage_locations =
            self.callable_storage_locations.get(&identifier).is_some_and(|locations| !locations.is_empty());
        let unhandled_storage_location = if has_storage_locations {
            self.has_uninvoked_callable_storage_location(identifier)
        } else if identifier.is_closure() {
            !usage.directly_invoked
        } else {
            return usage.stored;
        };

        usage.stored && (unhandled_storage_location || usage.escapes_scope)
    }

    /// Returns the resolved callsite where a closure was supplied as a callback argument.
    #[must_use]
    pub(crate) fn get_callback_callsite(&self, span: Span) -> Option<&CallbackCallsiteContext> {
        self.callback_callsites.get(&(span.start.offset, span.end.offset))
    }

    pub(crate) fn record_callback_parameter_invocation(&mut self, span: Span, parameter_offset: usize) {
        self.callback_parameter_invocations.insert((span.start.offset, span.end.offset), parameter_offset);
    }

    pub(crate) fn record_callback_parameter_escape(&mut self, parameter_offset: usize) {
        self.callback_parameter_escapes.insert(parameter_offset);
    }

    pub(crate) fn record_callback_parameter_effects(
        &mut self,
        identifier: FunctionLikeIdentifier,
        effects: HashSet<usize>,
    ) {
        self.callback_parameter_effects.entry(identifier).or_default().extend(effects);
    }

    #[must_use]
    pub(crate) fn has_callback_parameter_invocation(
        &self,
        identifier: &FunctionLikeIdentifier,
        parameter_offset: usize,
    ) -> bool {
        self.callback_parameter_effects.get(identifier).is_some_and(|parameters| parameters.contains(&parameter_offset))
    }

    #[must_use]
    pub(crate) fn callback_is_invoked_immediately(
        &self,
        target: Option<FunctionLikeIdentifier>,
        parameter_offset: usize,
        invocation_kind: Option<CallableInvocationKind>,
    ) -> bool {
        invocation_kind == Some(CallableInvocationKind::Immediately)
            || target.is_some_and(|target| self.has_callback_parameter_invocation(&target, parameter_offset))
    }

    pub(crate) fn absorb_deferred_callable_tracking(&mut self, other: &mut Self) {
        self.callback_callsites.extend(std::mem::take(&mut other.callback_callsites));
        for (identifier, callsites) in std::mem::take(&mut other.callable_callback_callsites) {
            self.callable_callback_callsites.entry(identifier).or_default().extend(callsites);
        }
        for (identifier, other_usage) in std::mem::take(&mut other.callable_usage) {
            let usage = self.callable_usage.entry(identifier).or_default();
            usage.directly_invoked |= other_usage.directly_invoked;
            usage.stored |= other_usage.stored;
            usage.unlocated_stored |= other_usage.unlocated_stored;
            usage.escapes_scope |= other_usage.escapes_scope;
        }
        for (identifier, storage_locations) in std::mem::take(&mut other.callable_storage_locations) {
            self.callable_storage_locations.entry(identifier).or_default().extend(storage_locations);
        }
        for (storage_location, identifiers) in std::mem::take(&mut other.callable_storage_owners_by_location) {
            self.callable_storage_owners_by_location.entry(storage_location).or_default().extend(identifiers);
        }
        for (identifier, spans) in std::mem::take(&mut other.direct_callable_invocation_spans) {
            self.direct_callable_invocation_spans.entry(identifier).or_default().extend(spans);
        }
        for (identifier, invoked_locations) in std::mem::take(&mut other.directly_invoked_callable_storage_locations) {
            self.directly_invoked_callable_storage_locations.entry(identifier).or_default().extend(invoked_locations);
        }
        for (identifier, callback_locations) in std::mem::take(&mut other.callable_callback_storage_locations) {
            self.callable_callback_storage_locations.entry(identifier).or_default().extend(callback_locations);
        }
    }

    pub(crate) fn absorb_callable_effects(&mut self, other: &mut Self) {
        for (span, effects) in std::mem::take(&mut other.callable_effects) {
            let entry = self.callable_effects.entry(span).or_default();
            merge_callable_effect_summary(entry, effects);
        }
        for (identifier, (span, effects)) in std::mem::take(&mut other.callable_effects_by_identifier) {
            let entry = self
                .callable_effects_by_identifier
                .entry(identifier)
                .or_insert_with(|| (span, CallableEffectSummary::default()));
            for (exception, spans) in effects.possibly_thrown_exceptions {
                entry.1.possibly_thrown_exceptions.entry(exception).or_default().extend(spans);
            }
        }

        for (identifier, exceptions) in std::mem::take(&mut other.reported_callable_exceptions) {
            self.reported_callable_exceptions.entry(identifier).or_default().extend(exceptions);
        }
        for (identifier, effects) in std::mem::take(&mut other.callback_parameter_effects) {
            self.callback_parameter_effects.entry(identifier).or_default().extend(effects);
        }
    }

    pub(crate) fn with_retained_expression_types(mut self, retain: bool) -> Self {
        self.retain_completed_expression_types = retain;
        self
    }

    pub(crate) fn should_retain_completed_expression_types(&self) -> bool {
        self.retain_completed_expression_types || !self.expression_type_checkpoints.is_empty()
    }

    pub(crate) fn retains_completed_expression_types(&self) -> bool {
        self.retain_completed_expression_types
    }

    pub(crate) fn record_static_local_types(
        &mut self,
        block_context: &BlockContext<'_>,
        codebase: &CodebaseMetadata,
        options: CombinerOptions,
    ) {
        let Some(static_local_types) = self.static_local_types.as_mut() else {
            return;
        };

        for variable in &block_context.static_locals {
            if block_context
                .by_reference_constraints
                .get(variable)
                .is_some_and(|constraint| constraint.source == ReferenceConstraintSource::Static)
            {
                continue;
            }

            let Some(variable_type) = block_context.locals.get(variable) else {
                continue;
            };

            if let Some(previous_type) = static_local_types.get_mut(variable) {
                let previous = std::mem::replace(previous_type, Rc::clone(variable_type));
                *previous_type = combine_owned_union_types_rc(previous, variable_type, codebase, options);
            } else {
                static_local_types.insert(*variable, Rc::clone(variable_type));
            }
        }
    }

    pub(crate) fn with_variable_definedness_targets(
        mut self,
        targets: Option<Arc<[bool; u8::MAX as usize + 1]>>,
    ) -> Self {
        self.variable_definedness_targets = targets;
        self
    }

    pub(crate) fn variable_definedness_targets(&self) -> Option<Arc<[bool; u8::MAX as usize + 1]>> {
        self.variable_definedness_targets.clone()
    }

    #[inline]
    pub(crate) fn record_variable_definedness(&mut self, node: Node<'_, '_>, block_context: &BlockContext<'_>) {
        let Some(targets) = self.variable_definedness_targets.as_deref() else {
            return;
        };

        let span = node.span();
        if !node_or_same_span_descendant_is_targeted(node, span, targets) {
            return;
        }

        let mut variables = WordMap::default();
        for (variable, variable_type) in &block_context.locals {
            if !is_plain_variable(*variable) {
                continue;
            }

            let definedness = if variable_type.possibly_undefined_from_try()
                || variable_type.possibly_undefined()
                    && block_context.possibly_undefined_variable_ids.contains(variable)
            {
                VariableDefinedness::PossiblyDefined
            } else {
                VariableDefinedness::Defined
            };
            variables.insert(*variable, definedness);
        }

        for variable in &block_context.variables_possibly_in_scope {
            if is_plain_variable(*variable) {
                variables.entry(*variable).or_insert(VariableDefinedness::PossiblyDefined);
            }
        }

        self.variable_definedness.insert((span.start.offset, span.end.offset), variables);
    }

    pub(crate) fn set_loop_scope(&mut self, loop_scope: LoopScope) {
        let previous_scope = self.loop_scope.take().map(Box::new);
        self.loop_scope = Some(loop_scope.with_parent_loop(previous_scope));
    }

    /// SAFETY: the caller must ensure that `self.loop_scope` is not `None`.
    pub(crate) unsafe fn take_loop_scope_unchecked(&mut self) -> LoopScope {
        let mut loop_scope = unsafe {
            // SAFETY: the caller must ensure that `self.loop_scope` is not `None`.
            self.loop_scope.take().unwrap_unchecked()
        };

        match loop_scope.parent_loop.take() {
            Some(parent_loop) => {
                self.loop_scope = Some(*parent_loop);
            }
            None => {
                self.loop_scope = None;
            }
        }

        loop_scope
    }

    pub(crate) fn get_loop_scope_mut(&mut self) -> Option<&mut LoopScope> {
        self.loop_scope.as_mut()
    }

    pub(crate) fn record_loop_assignment_target(&mut self, target: Word) {
        let mut loop_scope = self.loop_scope.as_mut();
        while let Some(scope) = loop_scope {
            if scope.tracks_assignment_targets {
                scope.assignment_targets.insert(target);
            }

            loop_scope = scope.parent_loop.as_deref_mut();
        }
    }

    /// Set the type of expression `expression` to `t`.
    #[inline]
    pub fn set_expression_type<T>(&mut self, expression: &T, t: TUnion)
    where
        T: HasSpan,
    {
        self.set_rc_expression_type(expression, Rc::new(t));
    }

    /// Get the type of expression `expression`.
    #[inline]
    pub fn get_expression_type<T>(&self, expression: &T) -> Option<&TUnion>
    where
        T: HasSpan,
    {
        let t = self.expression_types.get(&get_expression_range(expression))?;

        Some(&**t)
    }

    /// Set the type of expression `expression` to `t`.
    #[inline]
    pub fn set_rc_expression_type<T>(&mut self, expression: &T, t: Rc<TUnion>)
    where
        T: HasSpan,
    {
        self.expression_type_checkpoints.insert(&mut self.expression_types, get_expression_range(expression), t);
    }

    /// Get the type of expression `expression`.
    #[inline]
    pub fn get_rc_expression_type<T>(&self, expression: &T) -> Option<&Rc<TUnion>>
    where
        T: HasSpan,
    {
        self.expression_types.get(&get_expression_range(expression))
    }

    pub(crate) fn begin_expression_type_checkpoint(&mut self) {
        self.expression_type_checkpoints.begin_checkpoint();
        if let Some(editor) = &mut self.editor {
            editor.begin_checkpoint();
        }
    }

    pub(crate) fn begin_expression_type_recording(&mut self) {
        self.expression_type_checkpoints.begin_checkpoint();
    }

    pub(crate) fn finish_expression_type_recording(&mut self) -> Vec<((u32, u32), Rc<TUnion>)> {
        self.expression_type_checkpoints.finish_recording(&self.expression_types)
    }

    pub(crate) fn expression_type_checkpoint_matches(&self, same: impl Fn(&TUnion, &TUnion) -> bool) -> bool {
        self.expression_type_checkpoints.checkpoint_matches(&self.expression_types, same)
    }

    pub(crate) fn finish_expression_type_checkpoint(&mut self, action: CheckpointAction) {
        self.expression_type_checkpoints.finish_checkpoint(&mut self.expression_types, action);
        if let Some(editor) = &mut self.editor {
            editor.finish_checkpoint(action);
        }
    }

    pub(crate) fn remove_expression_type(&mut self, range: &(u32, u32)) {
        self.expression_type_checkpoints.remove(&mut self.expression_types, range);
    }

    pub(crate) fn extend_expression_types(&mut self, types: impl IntoIterator<Item = ((u32, u32), Rc<TUnion>)>) {
        if self.expression_type_checkpoints.is_empty() {
            self.expression_types.extend(types);
        } else {
            for (range, ty) in types {
                self.expression_type_checkpoints.insert(&mut self.expression_types, range, ty);
            }
        }
    }

    pub(crate) fn detach_expression_type_checkpoints(&mut self) -> Option<ExpressionTypeCheckpoints> {
        if self.expression_type_checkpoints.is_empty() {
            return None;
        }

        Some(self.expression_type_checkpoints.detach(&self.expression_types))
    }

    pub(crate) fn restore_expression_type_checkpoints(&mut self, checkpoints: ExpressionTypeCheckpoints) {
        self.expression_type_checkpoints = checkpoints;
    }
}

fn node_or_same_span_descendant_is_targeted(
    node: Node<'_, '_>,
    span: Span,
    targets: &[bool; u8::MAX as usize + 1],
) -> bool {
    if targets[node.kind() as usize] {
        return true;
    }

    let mut targeted = false;
    node.visit_children(|child| {
        if !targeted && child.span() == span {
            targeted = node_or_same_span_descendant_is_targeted(child, span, targets);
        }
    });

    targeted
}

fn is_plain_variable(variable: Word) -> bool {
    let bytes = variable.as_bytes();

    bytes.starts_with(b"$")
        && !bytes.contains(&b'[')
        && !bytes.windows(2).any(|window| window == b"->" || window == b"::")
}

#[inline]
pub fn get_expression_range<T>(expression: &T) -> (u32, u32)
where
    T: HasSpan,
{
    let span = expression.span();

    (span.start.offset, span.end.offset)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::rc::Rc;

    use mago_codex::context::ScopeContext;
    use mago_codex::metadata::CodebaseMetadata;
    use mago_codex::reference::ReferenceOrigin;
    use mago_codex::ttype::combine_union_types;
    use mago_codex::ttype::combiner::CombinerOptions;
    use mago_codex::ttype::flags::UnionFlags;
    use mago_codex::ttype::get_int;
    use mago_codex::ttype::get_list;
    use mago_codex::ttype::get_string;
    use mago_codex::ttype::union::TUnion;
    use mago_span::Span;
    use mago_word::WordMap;
    use mago_word::word;

    use super::AnalysisArtifacts;
    use crate::context::block::BlockContext;
    use crate::context::block::ReferenceConstraint;
    use crate::context::block::ReferenceConstraintSource;

    fn static_context(variable_type: Rc<TUnion>) -> BlockContext<'static> {
        let mut context = BlockContext::new(ScopeContext::new(ReferenceOrigin::File(word("static.php"))), false);
        context.static_locals.insert(word("$state"));
        context.locals.insert(word("$state"), variable_type);
        context
    }

    #[test]
    fn static_local_records_share_unchanged_types_and_keep_independent_snapshots() {
        let variable = word("$state");
        let original = Rc::new(get_list(get_int()));
        let mut context = static_context(Rc::clone(&original));
        let mut artifacts = AnalysisArtifacts::new();
        artifacts.static_local_types = Some(WordMap::default());
        let codebase = CodebaseMetadata::new();
        let options = CombinerOptions::default();

        artifacts.record_static_local_types(&context, &codebase, options);
        assert!(Rc::ptr_eq(&artifacts.static_local_types.as_ref().unwrap()[&variable], &original));
        let saved = artifacts.clone();

        artifacts.record_static_local_types(&context, &codebase, options);
        assert!(Rc::ptr_eq(&artifacts.static_local_types.as_ref().unwrap()[&variable], &original));
        assert!(Rc::ptr_eq(&saved.static_local_types.as_ref().unwrap()[&variable], &original));

        *Rc::make_mut(context.locals.get_mut(&variable).unwrap()) = get_string();
        artifacts.record_static_local_types(&context, &codebase, options);
        let expected = combine_union_types(&original, &get_string(), &codebase, options);
        let actual = &artifacts.static_local_types.as_ref().unwrap()[&variable];
        assert_eq!(actual.types, expected.types);
        assert_eq!(actual.flags, expected.flags);
        assert!(Rc::ptr_eq(&saved.static_local_types.as_ref().unwrap()[&variable], &original));
        assert_eq!(original.as_ref(), &get_list(get_int()));
    }

    #[test]
    fn equal_static_local_records_keep_the_first_types_flags() {
        let variable = word("$state");
        let mut first = get_list(get_int());
        first.flags.insert(UnionFlags::POPULATED);
        let first = Rc::new(first);
        let mut next = get_list(get_int());
        next.flags.insert(UnionFlags::NULLSAFE_NULL);
        let next = Rc::new(next);
        assert_eq!(first, next);

        let context = static_context(next);
        let mut artifacts = AnalysisArtifacts::new();
        artifacts.static_local_types = Some(WordMap::from_iter([(variable, Rc::clone(&first))]));
        artifacts.record_static_local_types(&context, &CodebaseMetadata::new(), CombinerOptions::default());

        let actual = &artifacts.static_local_types.as_ref().unwrap()[&variable];
        assert!(Rc::ptr_eq(actual, &first));
        assert_eq!(actual.flags, UnionFlags::POPULATED);
    }

    #[test]
    fn explicit_static_constraints_do_not_enter_the_inferred_type_map() {
        let variable = word("$state");
        let mut context = static_context(Rc::new(get_int()));
        context.by_reference_constraints.insert(
            variable,
            ReferenceConstraint::new(Span::dummy(0, 0), ReferenceConstraintSource::Static, Some(Rc::new(get_int()))),
        );
        let mut artifacts = AnalysisArtifacts::new();
        artifacts.static_local_types = Some(WordMap::default());
        artifacts.record_static_local_types(&context, &CodebaseMetadata::new(), CombinerOptions::default());
        assert!(artifacts.static_local_types.as_ref().unwrap().is_empty());
    }
}
