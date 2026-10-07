use std::rc::Rc;
use std::sync::Arc;

use foldhash::HashMap;
use foldhash::HashSet;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use mago_algebra::assertion_set::AssertionSet;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::reference::SymbolReferences;
use mago_codex::ttype::combine_owned_union_types_rc;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::union::TUnion;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Node;

use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraintSource;
use crate::context::scope::case_scope::CaseScope;
use crate::context::scope::loop_scope::LoopScope;
use crate::readonly::PendingReadonlyPropertyWrite;

pub(crate) use self::expression_types::CheckpointAction;
pub(crate) use self::expression_types::ExpressionTypeCheckpoints;

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

/// One semantic receiver and method target retained for a source-level call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedMethodCall {
    pub span: (u32, u32),
    pub class: Word,
    pub method: Word,
}

#[derive(Debug, Clone)]
pub struct AnalysisArtifacts {
    pub expression_types: HashMap<(u32, u32), Rc<TUnion>>,
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
    pub resolved_method_calls: Vec<ResolvedMethodCall>,
    pub(crate) variable_definedness: HashMap<(u32, u32), WordMap<VariableDefinedness>>,
    variable_definedness_targets: Option<Arc<[bool; u8::MAX as usize + 1]>>,
    pub(crate) pending_readonly_property_writes: Vec<PendingReadonlyPropertyWrite>,
    pub(crate) static_local_types: Option<WordMap<Rc<TUnion>>>,
    expression_type_checkpoints: ExpressionTypeCheckpoints,
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
            resolved_method_calls: Vec::new(),
            variable_definedness: HashMap::default(),
            variable_definedness_targets: None,
            pending_readonly_property_writes: Vec::new(),
            static_local_types: None,
            expression_type_checkpoints: ExpressionTypeCheckpoints::default(),
        }
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
    }

    pub(crate) fn expression_type_checkpoint_matches(&self, same: impl Fn(&TUnion, &TUnion) -> bool) -> bool {
        self.expression_type_checkpoints.checkpoint_matches(&self.expression_types, same)
    }

    pub(crate) fn finish_expression_type_checkpoint(&mut self, action: CheckpointAction) {
        self.expression_type_checkpoints.finish_checkpoint(&mut self.expression_types, action);
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
