use std::rc::Rc;

use mago_allocator::Arena;
use mago_codex::context::ScopeContext;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::union::TUnion;
use mago_reporting::IssueCollection;
use mago_span::HasSpan;
use mago_syntax::cst::AnonymousClass;
use mago_syntax::cst::Break;
use mago_syntax::cst::Class;
use mago_syntax::cst::Construct;
use mago_syntax::cst::Continue;
use mago_syntax::cst::DirectVariable;
use mago_syntax::cst::DoWhile;
use mago_syntax::cst::Enum;
use mago_syntax::cst::Expression;
use mago_syntax::cst::For;
use mago_syntax::cst::Foreach;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Goto;
use mago_syntax::cst::IndirectVariable;
use mago_syntax::cst::Interface;
use mago_syntax::cst::Literal;
use mago_syntax::cst::NestedVariable;
use mago_syntax::cst::Statement;
use mago_syntax::cst::Switch;
use mago_syntax::cst::Trait;
use mago_syntax::cst::While;
use mago_syntax::walker::MutWalker;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::r#loop::convergence;

const DYNAMIC_SCOPE_FUNCTIONS: [&[u8]; 6] =
    [b"compact", b"extract", b"get_defined_vars", b"parse_str", b"func_get_args", b"func_get_arg"];

#[derive(Debug, Clone, Default)]
pub(crate) struct LoopMemo {
    footprint: Option<Option<Rc<WordSet>>>,
    last: Option<Rc<MemoEntry>>,
}

#[derive(Debug)]
struct MemoEntry {
    key: BlockContext<'static>,
    retains_expression_types: bool,
    output: BlockContext<'static>,
    issues: IssueCollection,
    expression_types: Vec<((u32, u32), Rc<TUnion>)>,
    assignment_targets: WordSet,
}

pub(crate) fn analyze_loop<'ctx, 'arena, A>(
    statement: &Statement<'arena>,
    context: &mut Context<'ctx, 'arena, A>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    analyze: impl FnOnce(
        &mut Context<'ctx, 'arena, A>,
        &mut BlockContext<'ctx>,
        &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>,
) -> Result<(), AnalysisError>
where
    A: Arena,
{
    if !supports(context, block_context, artifacts) {
        return analyze(context, block_context, artifacts);
    }

    let span = statement.span();
    let key = (span.start.offset, span.end.offset);
    let retains_expression_types = artifacts.should_retain_completed_expression_types();
    let memo = artifacts.loop_memos.entry(key).or_default();
    let Some(footprint) = memo.footprint.get_or_insert_with(|| get_footprint(statement)).clone() else {
        return analyze(context, block_context, artifacts);
    };

    let scope = ScopeContext::new(block_context.scope.get_reference_origin());
    let projected = project(block_context, &footprint).with_scope(scope);
    if let Some(entry) = memo.last.clone()
        && entry.retains_expression_types == retains_expression_types
        && convergence::same_block(&entry.key, &projected)
    {
        for issue in entry.issues.iter() {
            context.collector.report(issue.clone());
        }

        artifacts.extend_expression_types(entry.expression_types.iter().cloned());
        for target in &entry.assignment_targets {
            artifacts.record_loop_assignment_target(*target);
        }

        let unkept = Unkept::split(block_context, &footprint);
        let scope = block_context.scope;
        *block_context = entry.output.clone().with_scope(scope);
        unkept.restore(block_context, &footprint);

        return Ok(());
    }

    let outputs = convergence::output_lengths(artifacts);
    let has_bare_return = artifacts.has_bare_return;
    let unkept = Unkept::split(block_context, &footprint);
    let previous_targets = artifacts.assignment_target_log.replace(WordSet::default());
    artifacts.begin_silent_expression_type_recording();

    let (result, issues) = context.record(|context| analyze(context, block_context, artifacts));

    let expression_types = artifacts.finish_silent_expression_type_recording();
    let assignment_targets =
        std::mem::replace(&mut artifacts.assignment_target_log, previous_targets).unwrap_or_default();
    if let Some(log) = &mut artifacts.assignment_target_log {
        log.extend(assignment_targets.iter().copied());
    }

    let storable = result.is_ok()
        && outputs == convergence::output_lengths(artifacts)
        && has_bare_return == artifacts.has_bare_return
        && supports(context, block_context, artifacts)
        && !block_context.flags.has_returned()
        && unkept.matches(&Unkept::split(block_context, &footprint));

    if !storable {
        for issue in issues {
            context.collector.report(issue);
        }

        if let Some(memo) = artifacts.loop_memos.get_mut(&key) {
            memo.last = None;
        }

        return result;
    }

    for issue in issues.iter() {
        context.collector.report(issue.clone());
    }

    let entry = Rc::new(MemoEntry {
        key: projected,
        retains_expression_types,
        output: block_context.clone().with_scope(ScopeContext::new(block_context.scope.get_reference_origin())),
        issues,
        expression_types,
        assignment_targets,
    });

    if let Some(memo) = artifacts.loop_memos.get_mut(&key) {
        memo.last = Some(entry);
    }

    result
}

fn supports<A>(context: &Context<'_, '_, A>, block_context: &BlockContext<'_>, artifacts: &AnalysisArtifacts) -> bool
where
    A: Arena,
{
    artifacts.loop_scope.is_some()
        && context.plugin_registry.allows_loop_convergence()
        && context.external_analysis_session.is_none()
        && context.type_resolution_context.is_empty()
        && artifacts.static_local_types.is_none()
        && artifacts.pending_readonly_property_writes.is_empty()
        && artifacts.editor.is_none()
        && artifacts.variable_definedness_targets().is_none()
        && block_context.finally_scope.is_none()
        && block_context.if_body_context.is_none()
        && block_context.logical_and_true_context.is_none()
        && block_context.logical_or_false_context.is_none()
}

fn is_kept(id: Word, footprint: &WordSet, locals: &WordMap<Rc<TUnion>>) -> bool {
    let bytes = id.as_bytes();

    bytes.iter().any(|byte| matches!(byte, b'[' | b'-' | b':'))
        || footprint.contains(&id)
        || bytes.starts_with(b"$_")
        || matches!(bytes, b"$GLOBALS" | b"$argc" | b"$argv" | b"$http_response_header" | b"$php_errormsg")
        || locals.get(&id).is_some_and(|ty| ty.types.iter().any(|atomic| matches!(atomic, TAtomic::Resource(_))))
}

fn project<'ctx>(block_context: &BlockContext<'ctx>, footprint: &WordSet) -> BlockContext<'ctx> {
    let mut projected = block_context.clone();
    let locals = &block_context.locals;
    let kept = |id: &Word| is_kept(*id, footprint, locals);

    projected.locals.retain(|id, _| kept(id));
    projected.static_locals.retain(kept);
    projected.variables_possibly_in_scope.retain(kept);
    projected.conditionally_referenced_variable_ids.retain(kept);
    projected.assigned_variable_ids.retain(|id, _| kept(id));
    projected.possibly_assigned_variable_ids.retain(kept);
    projected.derived_local_sources.retain(|id, source| kept(id) || kept(source));
    projected.foreach_variables.retain(kept);
    projected.class_string_sources.retain(|id, source| kept(id) || kept(source));
    projected.possibly_undefined_variable_ids.retain(kept);
    projected.referenced_counts.retain(|id, _| kept(id));
    projected.parent_conflicting_clause_variables.retain(kept);
    projected.by_reference_constraints.retain(|id, _| kept(id));
    projected
}

struct Unkept {
    locals: Vec<(Word, Rc<TUnion>)>,
    static_locals: Vec<Word>,
    variables_possibly_in_scope: Vec<Word>,
    conditionally_referenced_variable_ids: Vec<Word>,
    assigned_variable_ids: Vec<(Word, u32)>,
    possibly_assigned_variable_ids: Vec<Word>,
    derived_local_sources: Vec<(Word, Word)>,
    foreach_variables: Vec<Word>,
    class_string_sources: Vec<(Word, Word)>,
    possibly_undefined_variable_ids: Vec<Word>,
    referenced_counts: Vec<(Word, u32)>,
    parent_conflicting_clause_variables: Vec<Word>,
}

impl Unkept {
    fn split(block_context: &BlockContext<'_>, footprint: &WordSet) -> Self {
        let locals = &block_context.locals;
        let unkept = |id: &Word| !is_kept(*id, footprint, locals);
        let set = |set: &WordSet| {
            let mut words: Vec<Word> = set.iter().copied().filter(unkept).collect();
            words.sort_unstable();
            words
        };

        let mut unkept_locals: Vec<(Word, Rc<TUnion>)> =
            locals.iter().filter(|(id, _)| unkept(id)).map(|(id, ty)| (*id, Rc::clone(ty))).collect();
        unkept_locals.sort_unstable_by_key(|(id, _)| *id);
        let mut assigned_variable_ids: Vec<(Word, u32)> = block_context
            .assigned_variable_ids
            .iter()
            .filter(|(id, _)| unkept(id))
            .map(|(id, at)| (*id, *at))
            .collect();
        assigned_variable_ids.sort_unstable();
        let mut derived_local_sources: Vec<(Word, Word)> = block_context
            .derived_local_sources
            .iter()
            .filter(|(id, source)| unkept(id) && unkept(source))
            .map(|(id, source)| (*id, *source))
            .collect();
        derived_local_sources.sort_unstable();
        let mut class_string_sources: Vec<(Word, Word)> = block_context
            .class_string_sources
            .iter()
            .filter(|(id, source)| unkept(id) && unkept(source))
            .map(|(id, source)| (*id, *source))
            .collect();
        class_string_sources.sort_unstable();
        let mut referenced_counts: Vec<(Word, u32)> = block_context
            .referenced_counts
            .iter()
            .filter(|(id, _)| unkept(id))
            .map(|(id, count)| (*id, *count))
            .collect();
        referenced_counts.sort_unstable();

        Self {
            locals: unkept_locals,
            static_locals: set(&block_context.static_locals),
            variables_possibly_in_scope: set(&block_context.variables_possibly_in_scope),
            conditionally_referenced_variable_ids: set(&block_context.conditionally_referenced_variable_ids),
            assigned_variable_ids,
            possibly_assigned_variable_ids: set(&block_context.possibly_assigned_variable_ids),
            derived_local_sources,
            foreach_variables: set(&block_context.foreach_variables),
            class_string_sources,
            possibly_undefined_variable_ids: set(&block_context.possibly_undefined_variable_ids),
            referenced_counts,
            parent_conflicting_clause_variables: set(&block_context.parent_conflicting_clause_variables),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        self.locals.len() == other.locals.len()
            && self
                .locals
                .iter()
                .zip(&other.locals)
                .all(|((left_id, left), (right_id, right))| left_id == right_id && convergence::same_union(left, right))
            && self.static_locals == other.static_locals
            && self.scoped_variables() == other.variables_possibly_in_scope
            && self.conditionally_referenced_variable_ids == other.conditionally_referenced_variable_ids
            && self.assigned_variable_ids == other.assigned_variable_ids
            && self.possibly_assigned_variable_ids == other.possibly_assigned_variable_ids
            && self.derived_local_sources == other.derived_local_sources
            && self.foreach_variables == other.foreach_variables
            && self.class_string_sources == other.class_string_sources
            && self.possibly_undefined_variable_ids == other.possibly_undefined_variable_ids
            && self.referenced_counts == other.referenced_counts
            && self.parent_conflicting_clause_variables == other.parent_conflicting_clause_variables
    }

    fn scoped_variables(&self) -> Vec<Word> {
        let mut scoped: Vec<Word> =
            self.variables_possibly_in_scope.iter().copied().chain(self.locals.iter().map(|(id, _)| *id)).collect();
        scoped.sort_unstable();
        scoped.dedup();
        scoped
    }

    fn restore(self, block_context: &mut BlockContext<'_>, footprint: &WordSet) {
        let stored = Self::split(block_context, footprint);
        for (id, _) in stored.locals {
            block_context.locals.remove(&id);
        }
        for id in stored.static_locals {
            block_context.static_locals.remove(&id);
        }
        for id in stored.variables_possibly_in_scope {
            block_context.variables_possibly_in_scope.remove(&id);
        }
        for id in stored.conditionally_referenced_variable_ids {
            block_context.conditionally_referenced_variable_ids.remove(&id);
        }
        for (id, _) in stored.assigned_variable_ids {
            block_context.assigned_variable_ids.remove(&id);
        }
        for id in stored.possibly_assigned_variable_ids {
            block_context.possibly_assigned_variable_ids.remove(&id);
        }
        for (id, _) in stored.derived_local_sources {
            block_context.derived_local_sources.remove(&id);
        }
        for id in stored.foreach_variables {
            block_context.foreach_variables.remove(&id);
        }
        for (id, _) in stored.class_string_sources {
            block_context.class_string_sources.remove(&id);
        }
        for id in stored.possibly_undefined_variable_ids {
            block_context.possibly_undefined_variable_ids.remove(&id);
        }
        for (id, _) in stored.referenced_counts {
            block_context.referenced_counts.remove(&id);
        }
        for id in stored.parent_conflicting_clause_variables {
            block_context.parent_conflicting_clause_variables.remove(&id);
        }

        let scoped_variables = self.scoped_variables();
        block_context.locals.extend(self.locals);
        block_context.static_locals.extend(self.static_locals);
        block_context.variables_possibly_in_scope.extend(scoped_variables);
        block_context.conditionally_referenced_variable_ids.extend(self.conditionally_referenced_variable_ids);
        block_context.assigned_variable_ids.extend(self.assigned_variable_ids);
        block_context.possibly_assigned_variable_ids.extend(self.possibly_assigned_variable_ids);
        block_context.derived_local_sources.extend(self.derived_local_sources);
        block_context.foreach_variables.extend(self.foreach_variables);
        block_context.class_string_sources.extend(self.class_string_sources);
        block_context.possibly_undefined_variable_ids.extend(self.possibly_undefined_variable_ids);
        block_context.referenced_counts.extend(self.referenced_counts);
        block_context.parent_conflicting_clause_variables.extend(self.parent_conflicting_clause_variables);
    }
}

fn get_footprint(statement: &Statement<'_>) -> Option<Rc<WordSet>> {
    let mut footprint = Footprint { variables: WordSet::default(), memoizable: true, depth: 0 };
    footprint.walk_statement(statement, &mut ());

    footprint.memoizable.then(|| Rc::new(footprint.variables))
}

struct Footprint {
    variables: WordSet,
    memoizable: bool,
    depth: usize,
}

impl Footprint {
    fn check_level(&mut self, level: Option<&Expression<'_>>) {
        let level = match level {
            None => 1,
            Some(Expression::Literal(Literal::Integer(integer))) => integer.value.unwrap_or(u64::MAX),
            Some(_) => u64::MAX,
        };

        if level > self.depth as u64 {
            self.memoizable = false;
        }
    }
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for Footprint {
    fn walk_in_direct_variable(&mut self, variable: &'ast DirectVariable<'arena>, _context: &mut ()) {
        self.variables.insert(word(variable.name));
    }

    fn walk_in_indirect_variable(&mut self, _variable: &'ast IndirectVariable<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_nested_variable(&mut self, _variable: &'ast NestedVariable<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_function_call(&mut self, call: &'ast FunctionCall<'arena>, _context: &mut ()) {
        if let Expression::Identifier(identifier) = call.function {
            let name = identifier.value();
            let name = name.strip_prefix(b"\\").unwrap_or(name);
            if DYNAMIC_SCOPE_FUNCTIONS.iter().any(|function| function.eq_ignore_ascii_case(name)) {
                self.memoizable = false;
            }
        }
    }

    fn walk_in_construct(&mut self, construct: &'ast Construct<'arena>, _context: &mut ()) {
        if matches!(
            construct,
            Construct::Eval(_)
                | Construct::Include(_)
                | Construct::IncludeOnce(_)
                | Construct::Require(_)
                | Construct::RequireOnce(_)
        ) {
            self.memoizable = false;
        }
    }

    fn walk_in_goto(&mut self, _goto: &'ast Goto<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_class(&mut self, _class: &'ast Class<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_anonymous_class(&mut self, _class: &'ast AnonymousClass<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_interface(&mut self, _interface: &'ast Interface<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_trait(&mut self, _trait: &'ast Trait<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_enum(&mut self, _enum: &'ast Enum<'arena>, _context: &mut ()) {
        self.memoizable = false;
    }

    fn walk_in_break(&mut self, r#break: &'ast Break<'arena>, _context: &mut ()) {
        self.check_level(r#break.level);
    }

    fn walk_in_continue(&mut self, r#continue: &'ast Continue<'arena>, _context: &mut ()) {
        self.check_level(r#continue.level);
    }

    fn walk_in_foreach(&mut self, _foreach: &'ast Foreach<'arena>, _context: &mut ()) {
        self.depth += 1;
    }

    fn walk_out_foreach(&mut self, _foreach: &'ast Foreach<'arena>, _context: &mut ()) {
        self.depth -= 1;
    }

    fn walk_in_for(&mut self, _for: &'ast For<'arena>, _context: &mut ()) {
        self.depth += 1;
    }

    fn walk_out_for(&mut self, _for: &'ast For<'arena>, _context: &mut ()) {
        self.depth -= 1;
    }

    fn walk_in_while(&mut self, _while: &'ast While<'arena>, _context: &mut ()) {
        self.depth += 1;
    }

    fn walk_out_while(&mut self, _while: &'ast While<'arena>, _context: &mut ()) {
        self.depth -= 1;
    }

    fn walk_in_do_while(&mut self, _do_while: &'ast DoWhile<'arena>, _context: &mut ()) {
        self.depth += 1;
    }

    fn walk_out_do_while(&mut self, _do_while: &'ast DoWhile<'arena>, _context: &mut ()) {
        self.depth -= 1;
    }

    fn walk_in_switch(&mut self, _switch: &'ast Switch<'arena>, _context: &mut ()) {
        self.depth += 1;
    }

    fn walk_out_switch(&mut self, _switch: &'ast Switch<'arena>, _context: &mut ()) {
        self.depth -= 1;
    }
}
