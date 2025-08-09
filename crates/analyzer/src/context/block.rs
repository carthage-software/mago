use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::clause::Clause;
use mago_codex::assertion::Assertion;
use mago_codex::context::ScopeContext;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_collector::Collector;
use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;
use mago_span::Position;
use mago_span::Span;

use crate::context::Context;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::finally_scope::FinallyScope;
use crate::context::scope::var_has_root;
use crate::expression::r#match::subtract_union_types;
use crate::reconciler::ReconciliationContext;
use crate::reconciler::assertion_reconciler;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BreakContext {
    Switch,
    Loop,
}

#[derive(Clone, Debug)]
pub struct BlockContext<'a> {
    pub scope: ScopeContext<'a>,
    pub locals: BTreeMap<String, Rc<TUnion>>,
    pub static_locals: HashSet<String>,
    pub variables_possibly_in_scope: HashSet<String>,
    pub conditionally_referenced_variable_ids: HashSet<String>,
    pub assigned_variable_ids: HashMap<String, usize>,
    pub possibly_assigned_variable_ids: HashSet<String>,
    pub inside_conditional: bool,
    pub inside_coalescing: bool,
    pub inside_isset: bool,
    pub inside_unset: bool,
    pub inside_class_exists: bool,
    pub inside_general_use: bool,
    pub inside_return: bool,
    pub inside_throw: bool,
    pub inside_assignment: bool,
    pub inside_assignment_operation: bool,
    pub inside_loop: bool,
    pub inside_call: bool,
    pub inside_try: bool,
    pub inside_loop_expressions: bool,
    pub inside_negation: bool,
    pub inside_variable_reference: bool,
    pub clauses: Vec<Rc<Clause>>,
    pub reconciled_expression_clauses: Vec<Rc<Clause>>,
    pub branch_point: Option<usize>,
    pub break_types: Vec<BreakContext>,
    pub finally_scope: Option<Rc<RefCell<FinallyScope>>>,
    pub calling_closure_id: Option<Position>,
    pub has_returned: bool,
    pub parent_conflicting_clause_variables: HashSet<String>,
    pub loop_bounds: (usize, usize),
    pub for_loop_init_bounds: (usize, usize),
    pub if_body_context: Option<Rc<RefCell<Self>>>,
    pub control_actions: HashSet<ControlAction>,
    pub possibly_thrown_exceptions: HashMap<StringIdentifier, HashSet<Span>>,
}

impl BreakContext {
    #[inline]
    pub const fn is_loop(&self) -> bool {
        matches!(self, BreakContext::Loop)
    }

    #[inline]
    pub const fn is_switch(&self) -> bool {
        matches!(self, BreakContext::Switch)
    }
}

impl<'a> BlockContext<'a> {
    pub fn new(scope: ScopeContext<'a>) -> Self {
        Self {
            scope,
            locals: BTreeMap::new(),
            static_locals: HashSet::default(),
            variables_possibly_in_scope: HashSet::default(),
            conditionally_referenced_variable_ids: HashSet::default(),
            assigned_variable_ids: HashMap::default(),
            possibly_assigned_variable_ids: HashSet::default(),
            inside_conditional: false,
            inside_coalescing: false,
            inside_isset: false,
            inside_unset: false,
            inside_class_exists: false,
            inside_general_use: false,
            inside_return: false,
            inside_throw: false,
            inside_assignment: false,
            inside_assignment_operation: false,
            inside_loop_expressions: false,
            inside_negation: false,
            inside_call: false,
            inside_try: false,
            inside_variable_reference: false,
            has_returned: false,
            clauses: Vec::new(),
            reconciled_expression_clauses: Vec::new(),
            branch_point: None,
            break_types: Vec::new(),
            inside_loop: false,
            finally_scope: None,
            calling_closure_id: None,
            parent_conflicting_clause_variables: HashSet::default(),
            loop_bounds: (0, 0),
            for_loop_init_bounds: (0, 0),
            if_body_context: None,
            control_actions: HashSet::default(),
            possibly_thrown_exceptions: HashMap::default(),
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.scope.is_global()
    }

    pub fn is_mutation_free(&self) -> bool {
        self.scope.is_mutation_free()
    }

    pub fn is_external_mutation_free(&self) -> bool {
        self.scope.is_external_mutation_free()
    }

    pub fn get_redefined_locals(
        &self,
        new_locals: &BTreeMap<String, Rc<TUnion>>,
        include_new_vars: bool,
        removed_vars: &mut HashSet<String>,
    ) -> HashMap<String, TUnion> {
        let mut redefined_vars = HashMap::default();

        let mut var_ids = self.locals.keys().collect::<Vec<_>>();
        var_ids.extend(new_locals.keys());

        for var_id in var_ids {
            if let Some(this_type) = self.locals.get(var_id) {
                if let Some(new_type) = new_locals.get(var_id) {
                    if new_type != this_type {
                        redefined_vars.insert(var_id.clone(), (**this_type).clone());
                    }
                } else if include_new_vars {
                    redefined_vars.insert(var_id.clone(), (**this_type).clone());
                }
            } else {
                removed_vars.insert(var_id.clone());
            }
        }

        redefined_vars
    }

    pub fn get_new_or_updated_locals(original_context: &Self, new_context: &Self) -> HashSet<String> {
        let mut redefined_var_ids = HashSet::default();

        for (var_id, new_type) in &new_context.locals {
            if let Some(original_type) = original_context.locals.get(var_id) {
                if original_context.assigned_variable_ids.get(var_id).unwrap_or(&0)
                    != new_context.assigned_variable_ids.get(var_id).unwrap_or(&0)
                    || original_type != new_type
                {
                    redefined_var_ids.insert(var_id.clone());
                }
            } else {
                redefined_var_ids.insert(var_id.clone());
            }
        }

        redefined_var_ids
    }

    pub fn remove_reconciled_clause_refs(
        clauses: &Vec<Rc<Clause>>,
        changed_var_ids: &HashSet<String>,
    ) -> (Vec<Rc<Clause>>, Vec<Rc<Clause>>) {
        let mut included_clauses = Vec::new();
        let mut rejected_clauses = Vec::new();

        'outer: for c in clauses {
            if c.wedge {
                included_clauses.push(c.clone());
                continue;
            }

            for key in c.possibilities.keys() {
                for changed_var_id in changed_var_ids {
                    if changed_var_id == key || var_has_root(key, changed_var_id) {
                        rejected_clauses.push(c.clone());
                        continue 'outer;
                    }
                }
            }

            included_clauses.push(c.clone());
        }

        (included_clauses, rejected_clauses)
    }

    pub fn remove_reconciled_clauses(
        clauses: &Vec<Clause>,
        changed_var_ids: &HashSet<String>,
    ) -> (Vec<Clause>, Vec<Clause>) {
        let mut included_clauses = Vec::new();
        let mut rejected_clauses = Vec::new();

        'outer: for c in clauses {
            if c.wedge {
                included_clauses.push(c.clone());
                continue;
            }

            for key in c.possibilities.keys() {
                if changed_var_ids.contains(key) {
                    rejected_clauses.push(c.clone());
                    continue 'outer;
                }
            }

            included_clauses.push(c.clone());
        }

        (included_clauses, rejected_clauses)
    }

    pub(crate) fn filter_clauses(
        interner: &ThreadedInterner,
        codebase: &CodebaseMetadata,
        collector: &mut Collector<'_>,
        remove_var_id: &str,
        clauses: Vec<Rc<Clause>>,
        new_type: Option<&TUnion>,
    ) -> Vec<Rc<Clause>> {
        let mut clauses_to_keep = Vec::new();
        let mut other_clauses = Vec::new();

        'outer: for clause in clauses {
            for var_id in clause.possibilities.keys() {
                if var_has_root(var_id, remove_var_id) {
                    break 'outer;
                }
            }

            let keep_clause = should_keep_clause(&clause, remove_var_id, new_type);

            if keep_clause {
                clauses_to_keep.push(clause.clone())
            } else {
                other_clauses.push(clause);
            }
        }

        if let Some(new_type) = new_type
            && !new_type.is_mixed()
        {
            for clause in other_clauses {
                let mut type_changed = false;
                let Some(possibilities) = clause.possibilities.get(remove_var_id) else {
                    clauses_to_keep.push(clause.clone());

                    continue;
                };

                for (_, assertion) in possibilities {
                    if assertion.is_negation() {
                        type_changed = true;
                        break;
                    }

                    let mut context = ReconciliationContext::new(interner, codebase, collector);
                    let result_type = assertion_reconciler::reconcile(
                        &mut context,
                        assertion,
                        Some(&new_type.clone()),
                        false,
                        None,
                        false,
                        None,
                        false,
                        false,
                    );

                    if result_type != *new_type {
                        type_changed = true;
                        break;
                    }
                }

                if !type_changed {
                    clauses_to_keep.push(clause.clone());
                }
            }
        }

        clauses_to_keep
    }

    pub(crate) fn remove_variable_from_conflicting_clauses(
        &mut self,
        interner: &ThreadedInterner,
        codebase: &CodebaseMetadata,
        collector: &mut Collector<'_>,
        remove_var_id: &str,
        new_type: Option<&TUnion>,
    ) {
        self.clauses =
            BlockContext::filter_clauses(interner, codebase, collector, remove_var_id, self.clauses.clone(), new_type);

        self.parent_conflicting_clause_variables.insert(remove_var_id.to_owned());
    }

    pub(crate) fn remove_descendants(
        &mut self,
        interner: &ThreadedInterner,
        codebase: &CodebaseMetadata,
        collector: &mut Collector<'_>,
        remove_var_id: &str,
        existing_type: &TUnion,
        new_type: Option<&TUnion>,
    ) {
        self.remove_variable_from_conflicting_clauses(
            interner,
            codebase,
            collector,
            remove_var_id,
            if existing_type.is_mixed() {
                None
            } else if let Some(new_type) = new_type {
                Some(new_type)
            } else {
                None
            },
        );

        let keys = self.locals.keys().cloned().collect::<Vec<_>>();

        for var_id in keys {
            if var_has_root(&var_id, remove_var_id) {
                self.locals.remove(&var_id);
            }
        }
    }

    pub fn remove_mutable_object_vars(&mut self) {
        let mut removed_var_ids = Vec::new();

        self.locals.retain(|var_id, _| {
            let retain = !var_id.contains("->") && !var_id.contains("::");
            if !retain {
                removed_var_ids.push(var_id.clone());
            }

            retain
        });

        if removed_var_ids.is_empty() {
            return;
        }

        self.clauses.retain(|clause| {
            let mut retain_clause = true;

            for var_id in clause.possibilities.keys() {
                if var_id.contains("->") || var_id.contains("::") {
                    retain_clause = false;
                }
            }

            retain_clause
        });
    }

    pub fn has_variable(&mut self, var_name: &str) -> bool {
        self.conditionally_referenced_variable_ids.insert(var_name.to_owned());
        self.locals.contains_key(var_name)
    }

    pub(crate) fn remove_variable(
        &mut self,
        var_name: &String,
        remove_descendants: bool,
        interner: &ThreadedInterner,
        codebase: &CodebaseMetadata,
        collector: &mut Collector<'_>,
    ) {
        if let Some(existing_type) = self.locals.remove(var_name)
            && remove_descendants
        {
            self.remove_descendants(interner, codebase, collector, var_name, &existing_type, None);
        }

        self.assigned_variable_ids.remove(var_name);
        self.possibly_assigned_variable_ids.remove(var_name);
        self.conditionally_referenced_variable_ids.remove(var_name);
    }

    pub fn remove_possible_reference(&mut self, remove_var_id: &String) {
        self.locals.remove(remove_var_id);
        self.variables_possibly_in_scope.remove(remove_var_id);
        self.assigned_variable_ids.remove(remove_var_id);
        self.possibly_assigned_variable_ids.remove(remove_var_id);
        self.conditionally_referenced_variable_ids.remove(remove_var_id);
    }

    pub fn update(
        &mut self,
        context: &mut Context<'a>,
        start_block_context: &Self,
        end_block_context: &mut Self,
        has_leaving_statements: bool,
        vars_to_update: HashSet<String>,
        updated_vars: &mut HashSet<String>,
    ) {
        for (variable_id, old_type) in &start_block_context.locals {
            if !vars_to_update.contains(variable_id) {
                continue;
            }

            let new_type = if !has_leaving_statements && end_block_context.has_variable(variable_id) {
                end_block_context.locals.get(variable_id).cloned()
            } else {
                None
            };

            let Some(existing_type) = self.locals.get(variable_id).map(|rc| rc.as_ref()).cloned() else {
                if let Some(new_type) = new_type {
                    self.locals.insert(variable_id.clone(), new_type);
                    updated_vars.insert(variable_id.clone());
                }

                continue;
            };

            let old_type = old_type.as_ref().clone();

            let should_substitute = match &new_type {
                Some(new_type) => !old_type.eq(new_type),
                None => existing_type.types.len() > 1,
            };

            let resulting_type = if should_substitute {
                updated_vars.insert(variable_id.clone());

                substitute_types(context, existing_type, old_type, new_type.as_deref())
            } else {
                existing_type
            };

            self.locals.insert(variable_id.clone(), Rc::new(resulting_type));
        }
    }
}

fn substitute_types(
    context: &mut Context<'_>,
    existing_type: TUnion,
    old_type: TUnion,
    new_type: Option<&TUnion>,
) -> TUnion {
    if existing_type.is_mixed() || existing_type.is_never() {
        return existing_type;
    }

    let updated_type =
        if existing_type.eq(&old_type) { get_mixed() } else { subtract_union_types(context, existing_type, old_type) };

    add_optional_union_type(updated_type, new_type, context.codebase, context.interner)
}

fn should_keep_clause(clause: &Rc<Clause>, remove_var_id: &str, new_type: Option<&TUnion>) -> bool {
    if let Some(possibilities) = clause.possibilities.get(remove_var_id) {
        if possibilities.len() == 1
            && let Some((_, Assertion::IsType(assertion_type))) = possibilities.first()
            && let Some(new_type) = new_type
            && new_type.is_single()
        {
            return new_type.get_single() == assertion_type;
        }

        false
    } else {
        true
    }
}
