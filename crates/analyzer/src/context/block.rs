use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::clause::Clause;
use mago_atom::AtomMap;
use mago_codex::assertion::Assertion;
use mago_codex::context::ScopeContext;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_span::Span;

use crate::context::Context;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::finally_scope::FinallyScope;
use crate::context::scope::var_has_root;
use crate::reconciler::assertion_reconciler;
use crate::reconciler::negated_assertion_reconciler;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BreakContext {
    Switch,
    Loop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceConstraintSource {
    Global,
    Static,
    Parameter,
    Argument,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceConstraint {
    pub constraint_span: Span,
    pub source: ReferenceConstraintSource,
    pub constraint_type: Option<TUnion>,
}

#[derive(Clone, Debug)]
pub struct BlockContext<'ctx> {
    pub scope: ScopeContext<'ctx>,
    pub locals: BTreeMap<String, Rc<TUnion>>,
    pub static_locals: HashSet<String>,
    pub variables_possibly_in_scope: HashSet<String>,
    pub conditionally_referenced_variable_ids: HashSet<String>,
    pub assigned_variable_ids: HashMap<String, u32>,
    pub possibly_assigned_variable_ids: HashSet<String>,

    /// Maps variable names to the number of times they have been referenced in the current scope.
    ///
    /// This might not contain all variables in `locals`, as it is only updated when a variable is referenced.
    pub referenced_counts: HashMap<String, u32>,

    /// Maps variable names to the local variable that they reference.
    ///
    /// All keys and values in this map are guaranteed to be set in `locals`.
    pub references_in_scope: HashMap<String, String>,

    /// Set of references to variables in another scope. These references will be marked as used if they are assigned to.
    pub references_to_external_scope: HashSet<String>,

    /// A set of references that might still be in scope from a scope likely to cause confusion. This applies
    /// to references set inside a loop or if statement, since it's easy to forget about PHP's weird scope
    /// rules, and assigning to a reference will change the referenced variable rather than shadowing it.
    pub references_possibly_from_confusing_scope: HashSet<String>,

    /// A map of variable names to their reference constraints,
    /// where the key is the variable name and the value is the reference constraint.
    pub by_reference_constraints: HashMap<String, ReferenceConstraint>,

    pub inside_conditional: bool,
    pub inside_isset: bool,
    pub inside_unset: bool,
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
    pub break_types: Vec<BreakContext>,
    pub finally_scope: Option<Rc<RefCell<FinallyScope>>>,
    pub has_returned: bool,
    pub parent_conflicting_clause_variables: HashSet<String>,
    pub loop_bounds: (u32, u32),
    pub if_body_context: Option<Rc<RefCell<Self>>>,
    pub control_actions: HashSet<ControlAction>,
    pub possibly_thrown_exceptions: AtomMap<HashSet<Span>>,
}

impl BreakContext {
    #[inline]
    pub const fn is_switch(&self) -> bool {
        matches!(self, BreakContext::Switch)
    }
}

impl ReferenceConstraint {
    pub fn new(constraint_span: Span, source: ReferenceConstraintSource, constraint_type: Option<TUnion>) -> Self {
        let constraint_type = match constraint_type {
            Some(mut constraint_type) => {
                if constraint_type.has_literal_string() {
                    constraint_type.types.to_mut().push(TAtomic::Scalar(TScalar::string()));
                }

                if constraint_type.has_literal_int() {
                    constraint_type.types.to_mut().push(TAtomic::Scalar(TScalar::int()));
                }

                if constraint_type.has_literal_float() {
                    constraint_type.types.to_mut().push(TAtomic::Scalar(TScalar::float()));
                }

                Some(constraint_type)
            }
            None => None,
        };

        Self { constraint_span, source, constraint_type }
    }
}

impl<'ctx> BlockContext<'ctx> {
    pub fn new(scope: ScopeContext<'ctx>) -> Self {
        Self {
            scope,
            locals: BTreeMap::new(),
            static_locals: HashSet::default(),
            variables_possibly_in_scope: HashSet::default(),
            conditionally_referenced_variable_ids: HashSet::default(),
            assigned_variable_ids: HashMap::default(),
            possibly_assigned_variable_ids: HashSet::default(),
            referenced_counts: HashMap::default(),
            references_in_scope: HashMap::default(),
            references_to_external_scope: HashSet::default(),
            references_possibly_from_confusing_scope: HashSet::default(),
            by_reference_constraints: HashMap::default(),
            inside_conditional: false,
            inside_isset: false,
            inside_unset: false,
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
            break_types: Vec::new(),
            inside_loop: false,
            finally_scope: None,
            parent_conflicting_clause_variables: HashSet::default(),
            loop_bounds: (0, 0),
            if_body_context: None,
            control_actions: HashSet::default(),
            possibly_thrown_exceptions: AtomMap::default(),
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.scope.is_global()
    }

    pub fn update_references_possibly_from_confusing_scope(&mut self, confusing_scope_context: &BlockContext<'ctx>) {
        let references = confusing_scope_context
            .references_in_scope
            .keys()
            .chain(confusing_scope_context.references_to_external_scope.iter());

        for reference_id in references {
            if !self.references_in_scope.contains_key(reference_id)
                && !self.references_to_external_scope.contains(reference_id)
            {
                self.references_possibly_from_confusing_scope.insert(reference_id.to_owned());
            }
        }

        self.references_possibly_from_confusing_scope
            .extend(confusing_scope_context.references_possibly_from_confusing_scope.iter().cloned());
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

    pub(crate) fn filter_clauses<'arena>(
        context: &mut Context<'ctx, 'arena>,
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

                    let result_type = assertion_reconciler::reconcile(
                        context,
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

    pub(crate) fn remove_variable_from_conflicting_clauses<'arena>(
        &mut self,
        context: &mut Context<'ctx, 'arena>,
        remove_var_id: &str,
        new_type: Option<&TUnion>,
    ) {
        self.clauses = BlockContext::filter_clauses(context, remove_var_id, self.clauses.clone(), new_type);

        self.parent_conflicting_clause_variables.insert(remove_var_id.to_owned());
    }

    pub(crate) fn remove_descendants<'arena>(
        &mut self,
        context: &mut Context<'ctx, 'arena>,
        remove_var_id: &str,
        existing_type: &TUnion,
        new_type: Option<&TUnion>,
    ) {
        self.remove_variable_from_conflicting_clauses(
            context,
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

    /// Registers a variable that is referenced conditionally, like in a property
    /// or array access (`$foo->bar`, `$foo[0]`).
    pub fn add_conditionally_referenced_variable(&mut self, var_name: &str) {
        /// Strips an accessor suffix (from the first `->` or `[`) from a variable name.
        /// Returns the original slice if no accessor is found.
        fn strip_accessor_suffix(var_name: &str) -> &str {
            let first_separator_pos = var_name
                .find("->")
                .map(|pos| {
                    // If we find '->', see if '[' comes before it.
                    var_name.find('[').map_or(pos, |bracket_pos| pos.min(bracket_pos))
                })
                .or_else(|| {
                    // If '->' wasn't found, just look for '['.
                    var_name.find('[')
                });

            if let Some(pos) = first_separator_pos { &var_name[..pos] } else { var_name }
        }

        let stripped_var = strip_accessor_suffix(var_name);

        // A variable is conditionally referenced if it's part of an access chain
        // (i.e., its suffix was stripped) and the base variable is not `$this`.
        if stripped_var != "$this" || stripped_var != var_name {
            self.conditionally_referenced_variable_ids.insert(var_name.to_owned());
        }
    }

    /// Checks if a variable exists in the local scope, while also registering it
    /// as a conditionally referenced variable if it's part of an access chain.
    #[must_use]
    pub fn has_variable(&mut self, var_name: &str) -> bool {
        self.add_conditionally_referenced_variable(var_name);
        self.locals.contains_key(var_name)
    }

    pub(crate) fn remove_variable<'arena>(
        &mut self,
        var_name: &String,
        remove_descendants: bool,
        context: &mut Context<'ctx, 'arena>,
    ) {
        if let Some(existing_type) = self.locals.remove(var_name)
            && remove_descendants
        {
            self.remove_descendants(context, var_name, &existing_type, None);
        }

        self.assigned_variable_ids.remove(var_name);
        self.possibly_assigned_variable_ids.remove(var_name);
        self.conditionally_referenced_variable_ids.remove(var_name);
    }

    pub fn remove_possible_reference(&mut self, remove_var_id: &String) {
        if let Some(reference_count) = self.referenced_counts.get(remove_var_id)
            && *reference_count > 0
        {
            // If a referenced variable goes out of scope, we need to update the references.
            // All of the references to this variable are still references to the same value,
            // so we pick the first one and make the rest of the references point to it.
            let mut references = vec![];
            for (reference, referenced) in &self.references_in_scope {
                if referenced == remove_var_id {
                    references.push(reference.to_owned());
                }
            }

            for reference in references.iter() {
                self.references_in_scope.remove(reference);
            }

            debug_assert!(
                !references.is_empty(),
                "No references found for variable {}, even though it has a reference count of {}",
                remove_var_id,
                reference_count
            );

            if !references.is_empty() {
                let first_reference = references.remove(0);
                if !references.is_empty() {
                    self.referenced_counts.insert(first_reference.to_owned(), references.len() as u32);
                    for reference in references {
                        self.references_in_scope.insert(reference.to_owned(), first_reference.to_owned());
                    }
                }
            }
        }

        if self.references_in_scope.contains_key(remove_var_id) {
            self.decrement_reference_count(remove_var_id);
        }

        self.locals.remove(remove_var_id);
        self.variables_possibly_in_scope.remove(remove_var_id);
        self.assigned_variable_ids.remove(remove_var_id);
        self.possibly_assigned_variable_ids.remove(remove_var_id);
        self.conditionally_referenced_variable_ids.remove(remove_var_id);
    }

    pub fn update(
        &mut self,
        context: &mut Context<'ctx, '_>,
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

    /// Decrement the reference count of the variable that $ref_id is referring to. This needs to
    /// be done before $ref_id is changed to no longer reference its currently referenced variable,
    /// for example by unsetting, reassigning to another reference, or being shadowed by a global.
    pub fn decrement_reference_count(&mut self, ref_id: &str) -> bool {
        let Some(ref_id) = self.references_in_scope.get(ref_id) else {
            return false;
        };

        let Some(reference_count) = self.referenced_counts.get_mut(ref_id) else {
            return false;
        };

        if *reference_count < 1 {
            return false;
        }

        *reference_count -= 1;

        true
    }
}

impl std::fmt::Display for ReferenceConstraintSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReferenceConstraintSource::Global => write!(f, "global"),
            ReferenceConstraintSource::Static => write!(f, "static"),
            ReferenceConstraintSource::Parameter => write!(f, "parameter"),
            ReferenceConstraintSource::Argument => write!(f, "argument"),
        }
    }
}

fn substitute_types<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    existing_type: TUnion,
    old_type: TUnion,
    new_type: Option<&TUnion>,
) -> TUnion {
    if existing_type.is_mixed() || existing_type.is_never() {
        return existing_type;
    }

    let updated_type =
        if existing_type.eq(&old_type) { get_mixed() } else { subtract_union_types(context, existing_type, old_type) };

    add_optional_union_type(updated_type, new_type, context.codebase)
}

/// Subtracts the types in `type_to_remove` from `existing_type`.
///
/// This function iterates through each atomic type in `existing_type`. For each of these,
/// it iteratively applies the logic of "is not `atomic_from_remove_set`" for every
/// atomic type in `type_to_remove`. This effectively refines each part of `existing_type`
/// to exclude any possibilities covered by `type_to_remove`.
///
/// This is primarily useful for determining remaining possible types for a match subject
/// after some conditional arms have been considered.
///
/// # Arguments
///
/// * `context` - The reconciliation context, and providing access to codebase.
/// * `existing_type` - The initial `TUnion` type (the minuend).
/// * `type_to_remove` - The `TUnion` type whose components should be subtracted from `existing_type`.
///
/// # Returns
///
/// A new `TUnion` representing `existing_type - type_to_remove`.
pub fn subtract_union_types<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    existing_type: TUnion,
    type_to_remove: TUnion,
) -> TUnion {
    if existing_type == type_to_remove {
        return get_never();
    }

    if !(existing_type.has_literal_value() && type_to_remove.has_literal_value())
        && union_comparator::is_contained_by(
            context.codebase,
            &existing_type,
            &type_to_remove,
            false,
            false,
            true,
            &mut ComparisonResult::new(),
        )
    {
        return existing_type;
    }

    let mut result = existing_type;
    for atomic in type_to_remove.types.into_owned() {
        let assertion = Assertion::IsNotType(atomic);
        let key = result.get_id();

        result = negated_assertion_reconciler::reconcile(context, &assertion, &result, false, None, key, None, true);
        if result.is_never() {
            break;
        }
    }

    result
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
