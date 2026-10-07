use std::rc::Rc;

use foldhash::HashMap;
use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::clause::Clause;
use mago_allocator::Arena;
use mago_codex::assertion::Assertion;
use mago_codex::context::ScopeContext;
use mago_codex::ttype::TType;
use mago_codex::ttype::TypeRef;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::union::TUnion;
use mago_names::scope::NamespaceScope;
use mago_span::Span;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Node;
use mago_syntax::cst::Statement;
use mago_word::WordMap;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::case_scope::CaseScope;
use crate::context::scope::loop_scope::LoopScope;

#[cfg(test)]
thread_local! {
    static SKIPPED_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn record_skipped_passes(count: usize) {
    SKIPPED_PASSES.with(|skipped| skipped.set(skipped.get() + count));
}

pub(super) fn supports_context<'ctx, A>(
    context: &Context<'ctx, '_, A>,
    input: &BlockContext<'ctx>,
    parent: &BlockContext<'ctx>,
    pre_loop: &BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
) -> bool
where
    A: Arena,
{
    context.plugin_registry.allows_loop_convergence()
        && context.external_analysis_session.is_none()
        && context.type_resolution_context.is_empty()
        && artifacts.static_local_types.is_none()
        && artifacts.pending_readonly_property_writes.is_empty()
        && [input, parent, pre_loop]
            .iter()
            .all(|block| block.finally_scope.is_none() && block.if_body_context.is_none())
}

pub(super) fn supports_nodes(
    statements: &[Statement<'_>],
    conditions: &[&Expression<'_>],
    posts: &[&Expression<'_>],
) -> bool {
    let mut pending = statements.iter().map(Node::Statement).collect::<Vec<_>>();
    pending.extend(conditions.iter().chain(posts).map(|expression| Node::Expression(expression)));
    while let Some(node) = pending.pop() {
        // Class checks read reference sets that earlier loop passes may extend.
        if matches!(
            node,
            Node::Class(_) | Node::AnonymousClass(_) | Node::Enum(_) | Node::Trait(_) | Node::Interface(_)
        ) {
            return false;
        }
        node.visit_children(|child| pending.push(child));
    }
    true
}

pub(super) struct Snapshot<'ctx> {
    input: BlockContext<'ctx>,
    loop_locals: WordMap<Rc<TUnion>>,
    parent: BlockContext<'ctx>,
    pre_loop: BlockContext<'ctx>,
    loop_scope: LoopScope,
    scope: NamespaceScope,
    statement_span: Span,
    artifacts: AnalysisArtifacts,
    outputs: [usize; 5],
    issues_empty: bool,
}

impl<'ctx> Snapshot<'ctx> {
    pub(super) fn capture<A>(
        context: &Context<'ctx, '_, A>,
        input: &BlockContext<'ctx>,
        loop_context: &BlockContext<'ctx>,
        parent: &BlockContext<'ctx>,
        pre_loop: &BlockContext<'ctx>,
        scope: &LoopScope,
        artifacts: &AnalysisArtifacts,
        issues_empty: bool,
    ) -> Option<Box<Self>>
    where
        A: Arena,
    {
        if !supports_context(context, input, parent, pre_loop, artifacts) {
            return None;
        }

        // Keep inputs to later analysis, but not append-only outputs or reference sets.
        let mut saved = AnalysisArtifacts::new();
        saved.if_true_assertions.clone_from(&artifacts.if_true_assertions);
        saved.if_false_assertions.clone_from(&artifacts.if_false_assertions);
        saved.true_branch_only_assertions.clone_from(&artifacts.true_branch_only_assertions);
        saved.loop_scope.clone_from(&artifacts.loop_scope);
        saved.case_scopes.clone_from(&artifacts.case_scopes);
        saved.fully_matched_switch_offsets.clone_from(&artifacts.fully_matched_switch_offsets);
        saved.inferred_parameter_types.clone_from(&artifacts.inferred_parameter_types);
        saved.method_initialized_properties.clone_from(&artifacts.method_initialized_properties);
        saved.method_calls_this_methods.clone_from(&artifacts.method_calls_this_methods);
        saved.method_calls_parent_constructor.clone_from(&artifacts.method_calls_parent_constructor);
        saved.method_calls_parent_initializer.clone_from(&artifacts.method_calls_parent_initializer);
        saved.closure_bind_scope.clone_from(&artifacts.closure_bind_scope);
        saved.variable_definedness.clone_from(&artifacts.variable_definedness);

        Some(Box::new(Self {
            input: input.clone(),
            loop_locals: loop_context.locals.clone(),
            parent: parent.clone(),
            pre_loop: pre_loop.clone(),
            loop_scope: scope.clone(),
            scope: context.scope.clone(),
            statement_span: context.statement_span,
            artifacts: saved,
            outputs: output_lengths(artifacts),
            issues_empty,
        }))
    }

    pub(super) fn outputs_match(&self, artifacts: &AnalysisArtifacts, issues_empty: bool) -> bool {
        self.issues_empty == issues_empty && self.outputs == output_lengths(artifacts)
    }

    pub(super) fn matches<A>(
        &self,
        context: &Context<'ctx, '_, A>,
        input: &BlockContext<'ctx>,
        loop_context: &BlockContext<'ctx>,
        parent: &BlockContext<'ctx>,
        pre_loop: &BlockContext<'ctx>,
        scope: &LoopScope,
        artifacts: &AnalysisArtifacts,
        issues_empty: bool,
    ) -> bool
    where
        A: Arena,
    {
        self.outputs_match(artifacts, issues_empty)
            && context.type_resolution_context.is_empty()
            && self.scope == context.scope
            && self.statement_span == context.statement_span
            && artifacts.static_local_types.is_none()
            && same_block(&self.input, input)
            && same_types(&self.loop_locals, &loop_context.locals)
            && same_block(&self.parent, parent)
            && same_block(&self.pre_loop, pre_loop)
            && same_scope(&self.loop_scope, scope)
            && same_artifact_inputs(&self.artifacts, artifacts)
            && artifacts.expression_type_checkpoint_matches(same_union)
    }
}

fn output_lengths(artifacts: &AnalysisArtifacts) -> [usize; 5] {
    [
        artifacts.inferred_return_types.len(),
        artifacts.inferred_yield_key_types.len(),
        artifacts.inferred_yield_value_types.len(),
        artifacts.resolved_method_calls.len(),
        artifacts.pending_readonly_property_writes.len(),
    ]
}

#[cfg(test)]
fn same_artifacts(left: &AnalysisArtifacts, right: &AnalysisArtifacts) -> bool {
    left.expression_types.len() == right.expression_types.len()
        && left
            .expression_types
            .iter()
            .all(|(span, ty)| right.expression_types.get(span).is_some_and(|other| same_union(ty, other)))
        && same_artifact_inputs(left, right)
}

fn same_artifact_inputs(left: &AnalysisArtifacts, right: &AnalysisArtifacts) -> bool {
    same_assertion_map(&left.if_true_assertions, &right.if_true_assertions)
        && same_assertion_map(&left.if_false_assertions, &right.if_false_assertions)
        && same_assertion_map(&left.true_branch_only_assertions, &right.true_branch_only_assertions)
        && same_optional_scope(left.loop_scope.as_ref(), right.loop_scope.as_ref())
        && left.case_scopes.len() == right.case_scopes.len()
        && left.case_scopes.iter().zip(&right.case_scopes).all(|(left, right)| same_case(left, right))
        && left.fully_matched_switch_offsets == right.fully_matched_switch_offsets
        && match (&left.inferred_parameter_types, &right.inferred_parameter_types) {
            (Some(left), Some(right)) => {
                left.len() == right.len()
                    && left.iter().all(|(key, ty)| right.get(key).is_some_and(|other| same_union(ty, other)))
            }
            (None, None) => true,
            _ => false,
        }
        && left.method_initialized_properties == right.method_initialized_properties
        && left.method_calls_this_methods == right.method_calls_this_methods
        && left.method_calls_parent_constructor == right.method_calls_parent_constructor
        && left.method_calls_parent_initializer == right.method_calls_parent_initializer
        && left.closure_bind_scope.as_ref().map(|scope| (scope.class_name, scope.has_this))
            == right.closure_bind_scope.as_ref().map(|scope| (scope.class_name, scope.has_this))
        && left.variable_definedness == right.variable_definedness
}

fn same_case(left: &CaseScope, right: &CaseScope) -> bool {
    match (&left.break_vars, &right.break_vars) {
        (Some(left), Some(right)) => same_types(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn same_assertion_map(
    left: &HashMap<(u32, u32), WordMap<AssertionSet>>,
    right: &HashMap<(u32, u32), WordMap<AssertionSet>>,
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .all(|(key, assertions)| right.get(key).is_some_and(|other| same_named_assertions(assertions, other)))
}

fn same_named_assertions(left: &WordMap<AssertionSet>, right: &WordMap<AssertionSet>) -> bool {
    left.len() == right.len()
        && left.iter().all(|(key, clauses)| {
            right.get(key).is_some_and(|other| {
                clauses.len() == other.len()
                    && clauses.iter().zip(other).all(|(left, right)| {
                        left.len() == right.len()
                            && left.iter().zip(right).all(|(left, right)| same_assertion(left, right))
                    })
            })
        })
}

fn same_assertion(left: &Assertion, right: &Assertion) -> bool {
    if !left.cmp(right).is_eq() {
        return false;
    }

    match (left, right) {
        (Assertion::InArray(left), Assertion::InArray(right))
        | (Assertion::NotInArray(left), Assertion::NotInArray(right)) => same_union(left, right),
        _ => match (left.get_type(), right.get_type()) {
            (Some(left), Some(right)) => same_atomic_exceptions(left, right) && same_child_exceptions(left, right),
            _ => true,
        },
    }
}

fn same_clauses(left: &[Rc<Clause>], right: &[Rc<Clause>]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            Rc::ptr_eq(left, right)
                || (left.condition_span == right.condition_span
                    && left.span == right.span
                    && left.hash == right.hash
                    && left.wedge == right.wedge
                    && left.reconcilable == right.reconcilable
                    && left.generated == right.generated
                    && left.possibilities.len() == right.possibilities.len()
                    && left.possibilities.iter().zip(&right.possibilities).all(|((key, a), (other_key, b))| {
                        key == other_key
                            && a.len() == b.len()
                            && a.iter()
                                .zip(b)
                                .all(|((hash, a), (other_hash, b))| hash == other_hash && same_assertion(a, b))
                    }))
        })
}

fn same_block(left: &BlockContext<'_>, right: &BlockContext<'_>) -> bool {
    same_types(&left.locals, &right.locals)
        && same_references(left, right)
        && same_flow(left, right)
        && left.has_same_class_type_relations(right)
}

fn same_references(left: &BlockContext<'_>, right: &BlockContext<'_>) -> bool {
    left.references_in_scope == right.references_in_scope
        && left.referenced_counts == right.referenced_counts
        && left.references_to_external_scope == right.references_to_external_scope
        && left.references_possibly_from_confusing_scope == right.references_possibly_from_confusing_scope
        && left.by_reference_constraints.len() == right.by_reference_constraints.len()
        && left.by_reference_constraints.iter().all(|(key, constraint)| {
            right.by_reference_constraints.get(key).is_some_and(|other| {
                constraint.constraint_span == other.constraint_span
                    && constraint.source == other.source
                    && match (&constraint.constraint_type, &other.constraint_type) {
                        (Some(left), Some(right)) => same_union(left, right),
                        (None, None) => true,
                        _ => false,
                    }
            })
        })
}

fn same_types(left: &WordMap<Rc<TUnion>>, right: &WordMap<Rc<TUnion>>) -> bool {
    left.len() == right.len()
        && left.iter().all(|(variable, left)| right.get(variable).is_some_and(|right| same_union(left, right)))
}

fn same_union(left: &TUnion, right: &TUnion) -> bool {
    std::ptr::eq(left, right)
        || (left.flags == right.flags && std::ptr::eq(left.types.as_ref(), right.types.as_ref()))
        || (left.cmp(right).is_eq() && same_child_exceptions(left, right))
}

fn same_atomic_exceptions(left: &TAtomic, right: &TAtomic) -> bool {
    if std::ptr::eq(left, right) {
        return true;
    }
    match (left, right) {
        // Callable ordering omits names and child traversal omits closure-this types.
        (TAtomic::Callable(_), _) => false,
        (
            TAtomic::Scalar(TScalar::Float(TFloat::Literal(left))),
            TAtomic::Scalar(TScalar::Float(TFloat::Literal(right))),
        ) => left.to_bits() == right.to_bits(),
        _ => true,
    }
}

fn same_child_exceptions(left: &impl TType, right: &impl TType) -> bool {
    let mut left_children = left.get_child_nodes();
    let mut right_children = right.get_child_nodes();
    while let Some(left) = left_children.pop() {
        let Some(right) = right_children.pop() else {
            return false;
        };

        let (left, right) = match (left, right) {
            (TypeRef::Union(left), TypeRef::Union(right)) => (left.get_child_nodes(), right.get_child_nodes()),
            (TypeRef::Atomic(left), TypeRef::Atomic(right)) => {
                if !same_atomic_exceptions(left, right) {
                    return false;
                }
                (left.get_child_nodes(), right.get_child_nodes())
            }
            _ => return false,
        };

        left_children.extend(left);
        right_children.extend(right);
    }

    right_children.is_empty()
}

fn same_scope_context(left: &ScopeContext<'_>, right: &ScopeContext<'_>) -> bool {
    fn same_metadata<T>(left: Option<&T>, right: Option<&T>) -> bool {
        match (left, right) {
            (Some(left), Some(right)) => std::ptr::eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }
    left.get_reference_origin() == right.get_reference_origin()
        && left.is_static() == right.is_static()
        && same_metadata(left.get_function_like(), right.get_function_like())
        && same_metadata(left.get_class_like(), right.get_class_like())
        && match (left.get_property_hook(), right.get_property_hook()) {
            (Some((left_name, left)), Some((right_name, right))) => {
                left_name == right_name && std::ptr::eq(left, right)
            }
            (None, None) => true,
            _ => false,
        }
}

fn same_flow(left: &BlockContext<'_>, right: &BlockContext<'_>) -> bool {
    left.flags == right.flags
        && same_scope_context(&left.scope, &right.scope)
        && left.static_locals == right.static_locals
        && left.variables_possibly_in_scope == right.variables_possibly_in_scope
        && left.conditionally_referenced_variable_ids == right.conditionally_referenced_variable_ids
        && left.assigned_variable_ids == right.assigned_variable_ids
        && left.possibly_assigned_variable_ids == right.possibly_assigned_variable_ids
        && left.derived_local_sources == right.derived_local_sources
        && left.possibly_undefined_variable_ids == right.possibly_undefined_variable_ids
        && same_clauses(&left.clauses, &right.clauses)
        && same_clauses(&left.reconciled_expression_clauses, &right.reconciled_expression_clauses)
        && left.known_functions == right.known_functions
        && left.known_constants == right.known_constants
        && left.break_types == right.break_types
        && left.loop_bounds == right.loop_bounds
        && left.parent_conflicting_clause_variables == right.parent_conflicting_clause_variables
        && left.control_actions == right.control_actions
        && left.possibly_thrown_exceptions == right.possibly_thrown_exceptions
        && left.definitely_initialized_properties == right.definitely_initialized_properties
        && left.possibly_initialized_properties == right.possibly_initialized_properties
        && left.definitely_uninitialized_property_ids == right.definitely_uninitialized_property_ids
        && left.definitely_called_methods == right.definitely_called_methods
        && left.called_methods == right.called_methods
        && left.calls_parent_initializer == right.calls_parent_initializer
        && same_named_assertions(&left.active_method_call_assertions, &right.active_method_call_assertions)
        && same_named_assertions(&left.stable_method_call_assertions, &right.stable_method_call_assertions)
        && left.stable_method_calls == right.stable_method_calls
        && left.finally_scope.is_none()
        && right.finally_scope.is_none()
        && left.if_body_context.is_none()
        && right.if_body_context.is_none()
}

fn same_scope(left: &LoopScope, right: &LoopScope) -> bool {
    left.span == right.span
        && (left.iteration_count == 0) == (right.iteration_count == 0)
        && same_types(&left.parent_context_variables, &right.parent_context_variables)
        && same_types(&left.redefined_loop_variables, &right.redefined_loop_variables)
        && same_types(&left.possibly_redefined_loop_variables, &right.possibly_redefined_loop_variables)
        && same_types(&left.possibly_redefined_loop_parent_variables, &right.possibly_redefined_loop_parent_variables)
        && same_types(&left.possibly_defined_loop_parent_variables, &right.possibly_defined_loop_parent_variables)
        && same_types(&left.by_reference_loop_mutations, &right.by_reference_loop_mutations)
        && left.assignment_targets == right.assignment_targets
        && left.tracks_assignment_targets == right.tracks_assignment_targets
        && left.variables_possibly_in_scope == right.variables_possibly_in_scope
        && left.final_actions == right.final_actions
        && left.truthy_pre_conditions == right.truthy_pre_conditions
        && left.condition_always_false == right.condition_always_false
        && same_optional_scope(left.parent_loop.as_deref(), right.parent_loop.as_deref())
}

fn same_optional_scope(left: Option<&LoopScope>, right: Option<&LoopScope>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => same_scope(left, right),
        (None, None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::artifacts::CheckpointAction;
    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_codex::context::ScopeContext;
    use mago_codex::misc::VariableIdentifier;
    use mago_codex::populator::populate_codebase;
    use mago_codex::reference::ReferenceOrigin;
    use mago_codex::reference::SymbolReferences;
    use mago_codex::scanner::scan_program;
    use mago_codex::ttype::atomic::callable::parameter::TCallableParameter;
    use mago_codex::ttype::atomic::callable::{TCallable, TCallableSignature};
    use mago_codex::ttype::flags::UnionFlags;
    use mago_codex::ttype::get_int;
    use mago_codex::ttype::get_list;
    use mago_codex::ttype::get_string;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::word;

    use super::*;
    use crate::Analyzer;
    use crate::analysis_result::AnalysisResult;
    use crate::plugin::PluginRegistry;
    use crate::plugin::ProgramHook;
    use crate::plugin::Provider;
    use crate::plugin::ProviderMeta;
    use crate::settings::Settings;

    struct UntrustedNoop;

    impl Provider for UntrustedNoop {
        fn meta() -> &'static ProviderMeta {
            static META: ProviderMeta =
                ProviderMeta::new("convergence-test", "Convergence test", "Disables convergence");
            &META
        }
    }

    impl ProgramHook for UntrustedNoop {}

    fn assert_analysis_parity(source: &str, depth: u8) -> usize {
        let arena = LocalArena::new();
        let file = File::ephemeral(Cow::Borrowed(b"convergence.php"), Cow::Owned(source.as_bytes().to_vec()));
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "{:?}", program.errors);
        let names = NameResolver::new(&arena).resolve(program);
        let settings = Settings { loop_assignment_depth_threshold: depth, ..Settings::default() };
        let mut codebase = scan_program(&arena, &file, program, &names, settings.version);
        let mut references = SymbolReferences::new();
        populate_codebase(&mut codebase, &mut references, WordSet::default(), HashSet::default());

        let mut original_registry = PluginRegistry::with_library_providers();
        original_registry.register_program_hook(UntrustedNoop);
        assert!(!original_registry.allows_loop_convergence());
        let optimized_registry = PluginRegistry::with_library_providers();
        assert!(optimized_registry.allows_loop_convergence());

        SKIPPED_PASSES.with(|skipped| skipped.set(0));
        let mut original_result = AnalysisResult::new(references.clone());
        let mut original = Analyzer::new(&arena, &file, &names, &codebase, &original_registry, settings.clone())
            .analyze_with_artifacts(program, &mut original_result)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(SKIPPED_PASSES.with(std::cell::Cell::get), 0, "custom hooks must keep the old path");
        assert!(original.detach_expression_type_checkpoints().is_none());
        let mut optimized_result = AnalysisResult::new(references);
        SKIPPED_PASSES.with(|skipped| skipped.set(0));
        let mut optimized = Analyzer::new(&arena, &file, &names, &codebase, &optimized_registry, settings)
            .analyze_with_artifacts(program, &mut optimized_result)
            .unwrap_or_else(|error| panic!("{error}"));

        assert_eq!(
            original_result.issues.iter().collect::<Vec<_>>(),
            optimized_result.issues.iter().collect::<Vec<_>>()
        );
        assert_eq!(original_result.symbol_references, optimized_result.symbol_references);
        assert!(same_artifacts(&original, &optimized), "analysis input artifacts differ");
        assert_eq!(output_lengths(&original), output_lengths(&optimized));
        assert!(
            original.inferred_return_types.iter().zip(&optimized.inferred_return_types).all(|(a, b)| same_union(a, b))
        );
        assert!(
            original
                .inferred_yield_key_types
                .iter()
                .zip(&optimized.inferred_yield_key_types)
                .all(|(a, b)| same_union(a, b))
        );
        assert!(
            original
                .inferred_yield_value_types
                .iter()
                .zip(&optimized.inferred_yield_value_types)
                .all(|(a, b)| same_union(a, b))
        );
        assert_eq!(original.resolved_method_calls, optimized.resolved_method_calls);
        assert_eq!(
            format!("{:?}", original.pending_readonly_property_writes),
            format!("{:?}", optimized.pending_readonly_property_writes)
        );
        assert_eq!(original.static_local_types, optimized.static_local_types);
        assert!(optimized.detach_expression_type_checkpoints().is_none());
        SKIPPED_PASSES.with(std::cell::Cell::get)
    }

    #[test]
    fn stable_nested_loops_preserve_diagnostics_and_artifacts() {
        let skipped = assert_analysis_parity(
            "<?php
            function nested(bool $more): void {
                $a = 0; $b = 0; $c = 0;
                while ($more) {
                    $a = 'a';
                    while ($more) {
                        $b = 'b';
                        while ($more) { $c = 'c'; echo $a, $b, $c; }
                    }
                }
                echo $a, $b, $c;
            }
        ",
            3,
        );
        assert!(skipped > 0, "the fixture must exercise the fixed-point path");
    }

    #[test]
    fn loop_flow_cases_preserve_diagnostics_and_artifacts() {
        for source in [
            "<?php function references(bool $more): void {
                $a = 0; $b = 0; $ref =& $a;
                while ($more) { $ref = 's'; $ref =& $b; unset($a); $a = 0; }
                echo $a, $b, $ref;
            }",
            "<?php function exits(bool $more, int $choice): void {
                $a = 0;
                while ($more) {
                    $a = 's';
                    while ($more) {
                        switch ($choice) {
                            case 1: $a = true; break 2;
                            case 2: $a = null; continue 3;
                            default: $a = 1;
                        }
                    }
                }
                echo $a;
            }",
            "<?php function conditions(bool $more, int $n): void {
                for ($i = 0; $i < $n; $i++) { $a = $i; echo $a; }
                $shift = 0;
                do { if ($shift > 56) { echo 'overflow'; } $shift += 7; } while ($more);
                echo $shift;
            }",
            "<?php function temporary(bool $more): void {
                $a = 0;
                while ($more) { echo $temporary; $temporary = $a; $a = 's'; }
            }",
            "<?php function static_local(bool $more): mixed {
                static $a = 0;
                while ($more) { $a = 's'; if ($more) { return $a; } }
                return $a;
            }
            function generator(bool $more): iterable {
                $a = 0; while ($more) { $a = 's'; yield $a; }
            }",
            "<?php class Nullable { public ?self $next = null; }
            function nullable(bool $more, ?Nullable $a): void {
                while ($more) { $a = $a?->next; echo $a->next; }
            }",
        ] {
            assert_analysis_parity(source, 3);
        }
    }

    #[test]
    fn comparison_includes_flags_ignored_by_type_equality() {
        let ordinary = get_int();
        let mut nullsafe = ordinary.clone();
        nullsafe.flags.insert(UnionFlags::NULLSAFE_NULL);
        assert_eq!(ordinary, nullsafe);
        assert!(!same_union(&ordinary, &nullsafe));
        assert!(!same_union(&get_list(ordinary), &get_list(nullsafe)));
    }

    #[test]
    fn shared_type_storage_still_checks_flags_and_slice_lengths() {
        let left = get_int();
        let mut right = left.clone();
        assert!(!std::ptr::eq(&raw const left, &raw const right));
        assert!(std::ptr::eq(left.types.as_ref(), right.types.as_ref()));
        assert!(same_union(&left, &right));

        right.flags.insert(UnionFlags::POPULATED);
        assert!(!same_union(&left, &right));

        let atoms = mago_codex::ttype::shared::INT_STRING_ATOMIC_SLICE;
        let prefix = TUnion::new(Cow::Borrowed(&atoms[..1]));
        let whole = TUnion::new(Cow::Borrowed(atoms));
        assert!(std::ptr::eq(prefix.types.as_ptr(), whole.types.as_ptr()));
        assert!(!same_union(&prefix, &whole));
    }

    #[test]
    fn comparison_preserves_nested_order_float_bits_and_callable_metadata() {
        let mut forward = get_int();
        forward.types.to_mut().push(get_string().types[0].clone());
        let mut backward = forward.clone();
        backward.types.to_mut().reverse();
        assert_eq!(forward, backward);
        assert!(!same_union(&get_list(forward), &get_list(backward)));

        let positive = TUnion::from_atomic(TAtomic::Scalar(TScalar::Float(TFloat::literal(0.0))));
        let negative = TUnion::from_atomic(TAtomic::Scalar(TScalar::Float(TFloat::literal(-0.0))));
        assert_eq!(positive, negative);
        assert!(!same_union(&positive, &negative));
        assert!(!same_union(&get_list(positive), &get_list(negative)));

        let first =
            TCallableParameter::new(None, false, false, false).with_name(Some(VariableIdentifier(word("$first"))));
        let second =
            TCallableParameter::new(None, false, false, false).with_name(Some(VariableIdentifier(word("$second"))));
        let first =
            TAtomic::Callable(TCallable::Signature(TCallableSignature::new(false, false).with_parameters(vec![first])));
        let second = TAtomic::Callable(TCallable::Signature(
            TCallableSignature::new(false, false).with_parameters(vec![second]),
        ));
        assert_eq!(first, second);
        assert!(!same_assertion(&Assertion::IsType(first.clone()), &Assertion::IsType(second.clone())));
        assert!(!same_union(&TUnion::from_atomic(first), &TUnion::from_atomic(second)));
    }

    #[test]
    fn declarations_that_read_reference_sets_keep_the_old_path() {
        let arena = LocalArena::new();
        for source in [
            "<?php class C { private int $a; }",
            "<?php $a = new class { private int $a; };",
            "<?php enum E { case A; }",
            "<?php trait T {}",
            "<?php interface I {}",
            "<?php $f = function () { return new class {}; };",
        ] {
            let file = File::ephemeral(Cow::Borrowed(b"declaration.php"), Cow::Borrowed(source.as_bytes()));
            let program = parse_file(&arena, &file);
            assert!(!program.has_errors());
            assert!(!supports_nodes(program.statements.as_slice(), &[], &[]));
        }
    }

    #[test]
    fn failed_loop_pass_closes_its_checkpoint() {
        use crate::plugin::PluginError;
        use crate::plugin::context::HookContext;
        use crate::plugin::hook::{ExpressionHook, HookResult};
        use mago_collector::Collector;
        use mago_span::HasSpan;
        use mago_syntax::cst::Literal;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct FailOnSecondVisit(Arc<AtomicUsize>);
        impl Provider for FailOnSecondVisit {
            fn meta() -> &'static ProviderMeta {
                UntrustedNoop::meta()
            }
        }
        impl ExpressionHook for FailOnSecondVisit {
            fn after_expression(&self, expression: &Expression<'_>, _: &mut HookContext<'_, '_>) -> HookResult<()> {
                if matches!(expression, Expression::Literal(Literal::Integer(value)) if value.value == Some(777))
                    && self.0.fetch_add(1, Ordering::SeqCst) == 1
                {
                    return Err(PluginError::Internal { reason: "loop checkpoint test".to_owned() });
                }
                Ok(())
            }
        }
        let arena = LocalArena::new();
        let file = File::ephemeral(
            Cow::Borrowed(b"failed-loop.php"),
            Cow::Borrowed(b"<?php $a = 0; $more = true; while ($more) { $a = 's'; $marker = 777; }"),
        );
        let program = parse_file(&arena, &file);
        let names = NameResolver::new(&arena).resolve(program);
        let settings = Settings { loop_assignment_depth_threshold: 3, ..Settings::default() };
        let codebase = scan_program(&arena, &file, program, &names, settings.version);
        let mut registry = PluginRegistry::with_library_providers();
        let calls = Arc::new(AtomicUsize::new(0));
        registry.register_expression_hook(FailOnSecondVisit(Arc::clone(&calls)));
        // Inject a failure after capture; public hook registrations remain excluded.
        registry.mark_built_in_registrations();
        let collector = Collector::new(&arena, &file, program, crate::COLLECTOR_CATEGORIES);
        let mut context = Context::new(
            &arena,
            &codebase,
            &file,
            &names,
            &settings,
            program.statements[0].span(),
            program.trivia.as_slice(),
            collector,
            &registry,
            None,
            None,
        );
        let mut block = BlockContext::new(ScopeContext::new(ReferenceOrigin::File(word("failed-loop.php"))), false);
        let mut artifacts = AnalysisArtifacts::new();
        artifacts.begin_expression_type_checkpoint();
        assert!(
            crate::analyze_statements(program.statements.as_slice(), &mut context, &mut block, &mut artifacts).is_err()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        artifacts.finish_expression_type_checkpoint(CheckpointAction::Rollback);
        assert!(artifacts.expression_types.is_empty());
        assert!(artifacts.detach_expression_type_checkpoints().is_none());
    }

    #[test]
    fn changed_expression_types_reach_outer_checkpoints() {
        let mut artifacts = AnalysisArtifacts::new();
        let original = Rc::new(get_int());
        artifacts.expression_types.insert((0, 1), Rc::clone(&original));
        artifacts.begin_expression_type_checkpoint();
        artifacts.begin_expression_type_checkpoint();
        artifacts.extend_expression_types([((0, 1), Rc::new(get_string()))]);
        assert!(!artifacts.expression_type_checkpoint_matches(same_union));
        artifacts.finish_expression_type_checkpoint(CheckpointAction::Keep);
        assert!(!artifacts.expression_type_checkpoint_matches(same_union));
        artifacts.finish_expression_type_checkpoint(CheckpointAction::Rollback);
        assert!(Rc::ptr_eq(&artifacts.expression_types[&(0, 1)], &original));
        assert!(artifacts.detach_expression_type_checkpoints().is_none());
    }

    #[test]
    fn unchanged_types_do_not_hide_changed_flow_inputs() {
        let mut before = BlockContext::new(ScopeContext::new(ReferenceOrigin::File(word("test.php"))), false);
        before.locals.insert(word("$a"), Rc::new(get_int()));
        let mut after = before.clone();
        after.possibly_undefined_variable_ids.insert(word("$a"));
        assert!(same_types(&before.locals, &after.locals));
        assert!(!same_flow(&before, &after));
    }
}
