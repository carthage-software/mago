use mago_allocator::Arena;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::rc::Rc;

use foldhash::HashSet;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::word;

use mago_codex::ttype;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Hint;
use mago_syntax::cst::Node;
use mago_syntax::cst::Try;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::control_action::ControlActionSet;
use crate::context::scope::finally_scope::FinallyScope;
use crate::context::scope::loop_scope::LoopScope;
use crate::context::utils::inherit_branch_context_properties;
use crate::error::AnalysisError;
use crate::statement::analyze_statements;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Try<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        let mut catch_actions = vec![];
        let mut all_catches_leave = !self.catch_clauses.is_empty();

        for catch_clause in &self.catch_clauses {
            let actions = ControlAction::from_statements(
                catch_clause.block.statements.iter().collect::<Vec<_>>(),
                vec![],
                Some(artifacts),
                true,
            );

            all_catches_leave = all_catches_leave && !actions.contains(ControlAction::None);
            catch_actions.push(actions);
        }

        let existing_thrown_exceptions = std::mem::take(&mut block_context.possibly_thrown_exceptions);
        let old_block_context_locals = block_context.locals.clone();
        let loop_final_actions = artifacts.loop_scope.as_ref().map(snapshot_loop_final_actions);
        let mut try_block_context = block_context.clone();

        if self.finally_clause.is_some() {
            try_block_context.finally_scope = Some(Rc::new(RefCell::new(FinallyScope::new())));
        }

        let assigned_variable_ids = std::mem::take(&mut block_context.assigned_variable_ids);

        let was_inside_try = block_context.flags.inside_try();
        block_context.flags.set_inside_try(true);
        let mut exception_entry_locals: WordMap<Vec<WordSet>> = WordMap::default();
        if self.catch_clauses.is_empty() || !context.settings.check_throws {
            analyze_statements(self.block.statements.as_slice(), context, block_context, artifacts)?;
        } else {
            for statement in &self.block.statements {
                let definitely_defined_locals = block_context
                    .assigned_variable_ids
                    .keys()
                    .filter_map(|variable_id| {
                        block_context.locals.get(variable_id).and_then(|variable_type| {
                            (!variable_type.possibly_undefined() && !variable_type.possibly_undefined_from_try())
                                .then_some(*variable_id)
                        })
                    })
                    .collect::<WordSet>();
                let thrown_exception_counts = block_context
                    .possibly_thrown_exceptions
                    .iter()
                    .map(|(exception, spans)| (*exception, spans.len()))
                    .collect::<WordMap<_>>();

                analyze_statements(std::slice::from_ref(statement), context, block_context, artifacts)?;

                for (exception, spans) in &block_context.possibly_thrown_exceptions {
                    if spans.len() > thrown_exception_counts.get(exception).copied().unwrap_or_default() {
                        exception_entry_locals.entry(*exception).or_default().push(definitely_defined_locals.clone());
                    }
                }
            }
        }
        block_context.flags.set_inside_try(was_inside_try);
        if !self.catch_clauses.is_empty() {
            block_context.flags.set_has_returned(false);
        }

        let try_block_control_actions = ControlAction::from_statements(
            self.block.statements.iter().collect::<Vec<_>>(),
            vec![],
            Some(artifacts),
            true,
        );

        let newly_assigned_variable_ids = std::mem::take(&mut block_context.assigned_variable_ids);
        block_context.assigned_variable_ids.extend(assigned_variable_ids);
        block_context.assigned_variable_ids.extend(newly_assigned_variable_ids.iter().map(|(v, u)| (*v, *u)));

        let post_try_locals = std::mem::take(&mut block_context.locals);

        let invalidated_during_try: Vec<Word> = old_block_context_locals
            .keys()
            .copied()
            .filter(|variable_id| !post_try_locals.contains_key(variable_id))
            .collect();

        for variable_id in &invalidated_during_try {
            try_block_context.locals.remove(variable_id);
        }

        for (variable_id, variable_type) in post_try_locals {
            match try_block_context.locals.entry(variable_id) {
                Entry::Occupied(mut occupied_entry) => {
                    let combined_type = ttype::combine_union_types(
                        occupied_entry.get(),
                        variable_type.as_ref(),
                        context.codebase,
                        CombinerOptions::default(),
                    );

                    occupied_entry.insert(Rc::new(combined_type));

                    block_context.locals.insert(variable_id, variable_type);
                }
                Entry::Vacant(vacant_entry) => {
                    let mut possibly_undefined_type = (*variable_type).clone();
                    possibly_undefined_type.set_possibly_undefined(true, Some(true));

                    vacant_entry.insert(variable_type);
                    block_context.locals.insert(variable_id, Rc::new(possibly_undefined_type));
                }
            }
        }

        if let Some(try_scope) = &try_block_context.finally_scope {
            let mut mutable_try_scope = try_scope.borrow_mut();

            for (variable_id, variable_type) in &try_block_context.locals {
                if let Some(existing_type) = mutable_try_scope.locals.get_mut(variable_id) {
                    let combined_type = ttype::combine_union_types(
                        existing_type,
                        variable_type.as_ref(),
                        context.codebase,
                        CombinerOptions::default(),
                    );

                    *existing_type = Rc::new(combined_type);
                } else {
                    let mut var_type = (**variable_type).clone();
                    if !old_block_context_locals.contains_key(variable_id) {
                        var_type.set_possibly_undefined(true, Some(true));
                    }
                    mutable_try_scope.locals.insert(*variable_id, Rc::new(var_type));
                }
            }
        }

        try_block_context.possibly_thrown_exceptions = block_context.possibly_thrown_exceptions.clone();
        try_block_context.variables_possibly_in_scope = block_context.variables_possibly_in_scope.clone();
        let has_untracked_throw_source = context.settings.check_throws
            && has_untracked_throw_source(context, &self.block, &try_block_context.possibly_thrown_exceptions);

        let try_leaves_loop = artifacts
            .loop_scope
            .as_ref()
            .is_some_and(|loop_scope| !loop_scope.final_actions.contains(ControlAction::None));

        if all_catches_leave {
            for assigned_variable_id in newly_assigned_variable_ids.keys() {
                try_block_context.remove_variable_from_conflicting_clauses(context, *assigned_variable_id, None);
            }
        } else {
            for assigned_variable_id in newly_assigned_variable_ids.keys() {
                block_context.remove_variable_from_conflicting_clauses(context, *assigned_variable_id, None);
            }
        }

        let mut original_block_context = try_block_context.clone();
        let mut definitely_newly_assigned_var_ids = newly_assigned_variable_ids;
        let mut previously_caught_types = WordSet::default();

        for (i, catch_clause) in self.catch_clauses.iter().enumerate() {
            let mut catch_block_context = original_block_context.clone();
            catch_block_context.flags.set_has_returned(false);
            let (caught_classes, has_valid_caught_classes) =
                get_caught_classes(context, &catch_block_context, artifacts, &catch_clause.hint);

            for (variable_id, variable_type) in &mut catch_block_context.locals {
                if let Some(old_type) = old_block_context_locals.get(variable_id) {
                    *variable_type = Rc::new(ttype::combine_union_types(
                        variable_type.as_ref(),
                        old_type,
                        context.codebase,
                        CombinerOptions::default(),
                    ));
                } else {
                    let mut possibly_undefined_type = (**variable_type).clone();
                    possibly_undefined_type.set_possibly_undefined(variable_type.possibly_undefined(), Some(true));

                    *variable_type = Rc::new(possibly_undefined_type);
                }
            }

            for caught in &caught_classes {
                if context.codebase.is_instance_of(caught.as_bytes(), b"Error") {
                    context.collector.report_with_code(
                        IssueCode::AvoidCatchingError,
                        Issue::warning("Avoid catching 'Error' class instances.")
                            .with_annotation(Annotation::primary(catch_clause.hint.span()).with_message(
                                "This throwable is an instance of the `Error` class or one of its sub-classes.",
                            ))
                            .with_annotation(
                                Annotation::secondary(catch_clause.block.span())
                                    .with_message("This catch clause intercepts a critical error."),
                            )
                            .with_note("Catching these errors hides issues that should crash your app.")
                            .with_help("Remove or adjust this catch clause so errors propagate naturally."),
                    );
                }
            }

            let possibly_thrown_exceptions = std::mem::take(&mut catch_block_context.possibly_thrown_exceptions);
            let is_covered_by_previous_catch = caught_classes.iter().all(|caught| {
                previously_caught_types
                    .iter()
                    .any(|previous| context.codebase.is_instance_of(caught.as_bytes(), previous.as_bytes()))
            });
            if context.settings.check_throws
                && context.settings.check_dead_catch
                && has_valid_caught_classes
                && (is_covered_by_previous_catch
                    || (!has_untracked_throw_source
                        && !possibly_thrown_exceptions.keys().any(|thrown| {
                            caught_classes.iter().any(|caught| could_be_caught(context, *thrown, *caught))
                        })))
            {
                context.collector.report_with_code(
                    IssueCode::DeadCatch,
                    Issue::warning(
                        "This `catch` clause cannot catch any exception thrown in its associated `try` block.",
                    )
                    .with_annotation(
                        Annotation::primary(catch_clause.hint.span())
                            .with_message("No possible thrown exception matches this caught type."),
                    )
                    .with_annotation(
                        Annotation::secondary(self.block.span())
                            .with_message("The associated `try` block has no matching thrown exception."),
                    )
                    .with_help(
                        "Remove the unreachable `catch` clause or make the `try` body throw a compatible exception.",
                    ),
                );
            }

            let mut caught_exception_types = WordSet::default();
            let mut definitely_defined_locals: Option<WordSet> = None;
            for caught_class in &caught_classes {
                for possibly_thrown_exception in possibly_thrown_exceptions.keys() {
                    if possibly_thrown_exception.as_bytes().eq_ignore_ascii_case(caught_class.as_bytes())
                        || context
                            .codebase
                            .is_instance_of(possibly_thrown_exception.as_bytes(), caught_class.as_bytes())
                    {
                        if caught_exception_types.insert(*possibly_thrown_exception)
                            && let Some(entry_locals) = exception_entry_locals.get(possibly_thrown_exception)
                        {
                            for entry_locals in entry_locals {
                                intersect_definitely_defined_locals(&mut definitely_defined_locals, entry_locals);
                            }
                        }

                        original_block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                        block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                        catch_block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                    }
                }
            }

            if let Some(definitely_defined_locals) = definitely_defined_locals {
                for variable_id in definitely_defined_locals {
                    let Some(variable_type) = catch_block_context.locals.get_mut(&variable_id) else {
                        continue;
                    };

                    if !variable_type.possibly_undefined() && !variable_type.possibly_undefined_from_try() {
                        continue;
                    }

                    let mut defined_variable_type = (**variable_type).clone();
                    defined_variable_type.set_possibly_undefined(false, Some(false));
                    *variable_type = Rc::new(defined_variable_type);
                }
            }

            catch_block_context.clauses = vec![];
            if let Some(catch_variable) = catch_clause.variable.as_ref() {
                let exception_type = TUnion::new(
                    caught_classes
                        .iter()
                        .map(|caught_class| TAtomic::Object(TObject::Named(TNamedObject::new(*caught_class))))
                        .collect(),
                );

                let catch_var_name = Word::from(catch_variable.name);
                catch_block_context.locals.insert(catch_var_name, Rc::new(exception_type));
                catch_block_context.remove_variable_from_conflicting_clauses(context, catch_var_name, None);
                catch_block_context.variables_possibly_in_scope.insert(catch_var_name);
            }

            let old_catch_assigned_variable_ids = std::mem::take(&mut catch_block_context.assigned_variable_ids);

            analyze_statements(catch_clause.block.statements.as_slice(), context, &mut catch_block_context, artifacts)?;

            // recalculate in case there's a no-return clause
            if let Some(actions) = catch_actions.get_mut(i) {
                *actions = ControlAction::from_statements(
                    catch_clause.block.statements.iter().collect::<Vec<_>>(),
                    vec![],
                    Some(artifacts),
                    true,
                );
            }

            all_catches_leave = catch_actions.iter().all(|actions| !actions.contains(ControlAction::None));

            let new_catch_assigned_variables_ids = catch_block_context.assigned_variable_ids.clone();
            catch_block_context.assigned_variable_ids.extend(old_catch_assigned_variable_ids);

            inherit_branch_context_properties(context, block_context, &catch_block_context);

            let catch_doesnt_leave_parent_scope = catch_actions[i].contains(ControlAction::None);

            if catch_doesnt_leave_parent_scope {
                definitely_newly_assigned_var_ids = new_catch_assigned_variables_ids
                    .iter()
                    .filter(|(key, _)| definitely_newly_assigned_var_ids.contains_key(*key))
                    .map(|(key, value)| (*key, *value))
                    .collect();

                let end_action_only =
                    try_block_control_actions.len() == 1 && try_block_control_actions.contains(ControlAction::End);

                for (variable_id, variable_type) in &catch_block_context.locals {
                    if end_action_only {
                        block_context.locals.insert(*variable_id, Rc::clone(variable_type));
                    } else if let Some(existing_type) = block_context.locals.get(variable_id) {
                        block_context.locals.insert(
                            *variable_id,
                            Rc::new(ttype::combine_union_types(
                                existing_type.as_ref(),
                                variable_type.as_ref(),
                                context.codebase,
                                CombinerOptions::default(),
                            )),
                        );
                    }
                }

                block_context.variables_possibly_in_scope.extend(catch_block_context.variables_possibly_in_scope);
            } else if self.finally_clause.is_some() {
                block_context.variables_possibly_in_scope.extend(catch_block_context.variables_possibly_in_scope);
            }

            if let Some(mut finally_scope) = try_block_context.finally_scope.as_ref().map(|s| s.borrow_mut()) {
                for (variable_id, variable_type) in &catch_block_context.locals {
                    let resulting_type = if let Some(finally_variable_type) = finally_scope.locals.get(variable_id) {
                        let mut combined = ttype::combine_union_types(
                            finally_variable_type.as_ref(),
                            variable_type.as_ref(),
                            context.codebase,
                            CombinerOptions::default(),
                        );
                        if !old_block_context_locals.contains_key(variable_id) {
                            combined.set_possibly_undefined(true, Some(true));
                        }
                        combined
                    } else {
                        let mut finally_variable_type = (**variable_type).clone();
                        if !old_block_context_locals.contains_key(variable_id) {
                            finally_variable_type.set_possibly_undefined(true, Some(true));
                        }

                        finally_variable_type
                    };

                    finally_scope.locals.insert(*variable_id, Rc::new(resulting_type));
                }
            }

            if has_valid_caught_classes {
                previously_caught_types.extend(caught_classes.iter().copied());
            }
        }

        if !try_leaves_loop && let Some(loop_scope) = artifacts.loop_scope.as_mut() {
            loop_scope.final_actions.insert(ControlAction::None);
        }

        let mut finally_has_returned = false;
        if let Some(finally_clause) = self.finally_clause.as_ref() {
            let finally_actions = ControlAction::from_statements(
                finally_clause.block.statements.iter().collect(),
                vec![],
                Some(artifacts),
                true,
            );

            if !finally_actions.contains(ControlAction::None)
                && let (Some(loop_scope), Some(loop_final_actions)) =
                    (artifacts.loop_scope.as_mut(), loop_final_actions.as_deref())
            {
                restore_loop_final_actions(loop_scope, loop_final_actions);
            }

            // SAFETY: when a try statement has a `finally_clause`, the finally scope is always
            // initialized on `try_block_context` before reaching this point, so the `Option` is
            // guaranteed to be `Some`.
            let finally_scope = unsafe {
                try_block_context
                    .finally_scope
                    .take()
                    .map(|scope| scope.as_ref().clone())
                    .map(std::cell::RefCell::into_inner)
                    .unwrap_unchecked()
            };

            let mut finally_block_context = block_context.clone();
            finally_block_context.assigned_variable_ids = WordMap::default();
            finally_block_context.possibly_assigned_variable_ids = WordSet::default();
            finally_block_context.locals = finally_scope.locals;
            for var_id in finally_block_context.locals.keys() {
                if !old_block_context_locals.contains_key(var_id) {
                    finally_block_context.possibly_undefined_variable_ids.insert(*var_id);
                }
            }
            finally_block_context.flags.set_has_returned(false);

            analyze_statements(
                finally_clause.block.statements.as_slice(),
                context,
                &mut finally_block_context,
                artifacts,
            )?;

            finally_has_returned = finally_block_context.flags.has_returned();
            block_context.possibly_undefined_variable_ids.extend(finally_block_context.possibly_undefined_variable_ids);

            for (variable_id, _) in finally_block_context.assigned_variable_ids {
                let finally_variable_type = finally_block_context.locals.remove(&variable_id);
                if let Some(finally_variable_type) = finally_variable_type {
                    block_context.locals.insert(variable_id, finally_variable_type);
                }
            }
        }

        for (variable_id, _) in definitely_newly_assigned_var_ids {
            let Some(variable_type) = block_context.locals.get_mut(&variable_id) else {
                continue;
            };

            if !variable_type.possibly_undefined_from_try() {
                continue;
            }

            let mut defined_variable_type = (**variable_type).clone();
            defined_variable_type.set_possibly_undefined(false, Some(false));

            *variable_type = Rc::new(defined_variable_type);
        }

        for (possibly_thrown_exception, throw_spans) in existing_thrown_exceptions {
            block_context.possibly_thrown_exceptions.entry(possibly_thrown_exception).or_default().extend(throw_spans);
        }

        block_context.flags.set_has_returned(if finally_has_returned {
            true
        } else if !try_block_control_actions.contains(ControlAction::None) {
            self.catch_clauses.is_empty() || all_catches_leave
        } else {
            false
        });

        Ok(())
    }
}

fn snapshot_loop_final_actions(loop_scope: &LoopScope) -> Vec<ControlActionSet> {
    let mut final_actions = Vec::new();
    let mut current_loop_scope = Some(loop_scope);

    while let Some(loop_scope) = current_loop_scope {
        final_actions.push(loop_scope.final_actions.clone());
        current_loop_scope = loop_scope.parent_loop.as_deref();
    }

    final_actions
}

fn restore_loop_final_actions(loop_scope: &mut LoopScope, final_actions: &[ControlActionSet]) {
    let Some((final_actions, parent_final_actions)) = final_actions.split_first() else {
        return;
    };

    loop_scope.final_actions.clone_from(final_actions);

    if let Some(parent_loop) = loop_scope.parent_loop.as_deref_mut() {
        restore_loop_final_actions(parent_loop, parent_final_actions);
    }
}

/// Intersects definitely defined locals from paths that can enter the same catch clause.
fn intersect_definitely_defined_locals(definitely_defined_locals: &mut Option<WordSet>, entry_locals: &WordSet) {
    match definitely_defined_locals {
        Some(definitely_defined_locals) => {
            definitely_defined_locals.retain(|variable_id| entry_locals.contains(variable_id));
        }
        None => {
            *definitely_defined_locals = Some(entry_locals.clone());
        }
    }
}

fn get_caught_classes<'arena, A>(
    context: &mut Context<'_, 'arena, A>,
    block_context: &BlockContext<'_>,
    artifacts: &mut AnalysisArtifacts,
    hint: &Hint<'arena>,
) -> (WordSet, bool)
where
    A: Arena,
{
    let mut caught_identifiers: WordMap<Span> = WordMap::default();

    fn walk<'arena, A>(context: &mut Context<'_, 'arena, A>, hint: &Hint<'arena>, caught: &mut WordMap<Span>)
    where
        A: Arena,
    {
        match hint {
            Hint::Identifier(identifier) => {
                let name_bytes = context.resolved_names.get(identifier);
                let name = mago_bytes::BytesDisplay(name_bytes);
                let id = word(name_bytes);

                if let Some(&first_span) = caught.get(&id) {
                    context.collector.report_with_code(
                        IssueCode::DuplicateCaughtType,
                        Issue::error(format!(
                            "Type `{name}` is caught multiple times in the same `catch` clause.",
                        ))
                        .with_annotation(
                            Annotation::primary(hint.span())
                                .with_message("This type is a duplicate occurrence here"),
                        )
                        .with_annotation(
                            Annotation::secondary(first_span)
                                .with_message(format!("`{name}` was already specified here")),
                        )
                        .with_help("Remove the redundant type from the `catch` union. Each exception type should only be listed once."),
                    );
                } else {
                    caught.insert(id, hint.span());
                }
            }
            Hint::Union(union_hint) => {
                walk(context, union_hint.left, caught);
                walk(context, union_hint.right, caught);
            }
            _ => {
                context.collector.report_with_code(
                    IssueCode::InvalidCatchType,
                    Issue::error("Invalid type used in `catch` declaration. Only class or interface names are allowed.")
                    .with_annotation(
                        Annotation::primary(hint.span())
                            .with_message("This type is not a valid class or interface name for a `catch` block."),
                    )
                    .with_note(
                        "PHP `catch` blocks require a class or interface name to specify the type of exceptions to be caught. Primitive types (e.g., `int`, `string`), arrays, or other non-class types are not permitted here."
                    )
                    .with_help(
                        "Use a valid class or interface name (e.g., `Exception`, `MyCustomError`), or a union of them (e.g., `FooException | BarException`)."
                    ),
                );
            }
        }
    }

    walk(context, hint, &mut caught_identifiers);

    let throwable = word(b"Throwable");
    let mut caught_classes: WordSet =
        WordSet::with_capacity_and_hasher(caught_identifiers.len(), foldhash::fast::FixedState::default());
    for (caught_type, caught_span) in caught_identifiers {
        artifacts.symbol_references.add_reference_to_symbol(
            &block_context.scope,
            caught_type.to_ascii_lowercase(),
            false,
        );

        if caught_type.as_bytes().eq_ignore_ascii_case(b"throwable")
            || caught_type.as_bytes().eq_ignore_ascii_case(b"exception")
            || caught_type.as_bytes().eq_ignore_ascii_case(b"error")
        {
            caught_classes.insert(caught_type);
            continue;
        }

        let Some(class_like_metadata) = context.codebase.get_class_like(caught_type.as_bytes()) else {
            context.collector.report_with_code(
                IssueCode::NonExistentCatchType,
                Issue::error(format!("Attempting to catch an undefined class or interface: `{caught_type}`."))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("Type `{caught_type}` is not defined or cannot be found")),
                )
                .with_note(
                    "Types used in `catch` blocks must be existing and autoloadable classes or interfaces."
                )
                .with_help(
                    "Check for typos in the type name. Ensure the class/interface is correctly defined, namespaced, and that your autoloader can find it."
                ),
            );

            continue;
        };

        if class_like_metadata.kind.is_enum() || class_like_metadata.kind.is_trait() {
            let kind_str = if class_like_metadata.kind.is_enum() { "an enum" } else { "a trait" };

            context.collector.report_with_code(
                IssueCode::InvalidCatchTypeNotClassOrInterface,
                Issue::error(format!(
                    "Only classes or interfaces can be caught, but `{caught_type}` is {kind_str}.",
                ))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("Cannot catch `{caught_type}` because it is {kind_str}")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_metadata.name_span.unwrap_or(class_like_metadata.span))
                        .with_message(format!("`{caught_type}` is defined as {kind_str} here")),
                )
                .with_note("PHP `catch` blocks require a class or interface type. Enums and traits are not valid types for catching exceptions as they cannot be thrown or extend `Throwable`.")
                .with_help("Specify a class or interface that implements `Throwable` (e.g., `Exception`, `Error`, or a custom exception class)."),
            );

            continue;
        }

        let is_interface = class_like_metadata.kind.is_interface();
        let is_throwable =
            is_interface || context.codebase.is_instance_of(caught_type.as_bytes(), throwable.as_bytes());
        if !is_throwable {
            context.collector.report_with_code(
                IssueCode::CatchTypeNotThrowable,
                Issue::error(format!(
                    "The type `{caught_type}` caught in a catch block must implement the `Throwable` interface.",
                ))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("`{caught_type}` is not an instance of `Throwable`")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_metadata.name_span.unwrap_or(class_like_metadata.span))
                        .with_message(format!("`{caught_type}` defined here does not implement `Throwable`")),
                )
                .with_note("In PHP, only objects that implement the `Throwable` interface (this includes `Exception` and `Error` classes and their children) can be caught in a `catch` block.")
                .with_help(format!("Ensure that `{caught_type}` implements the `Throwable` interface, or catch a more general exception type like `Exception` or `Throwable` itself.")),
            );

            continue;
        }

        caught_classes.insert(caught_type);
    }

    let has_valid_caught_classes = !caught_classes.is_empty();
    if !has_valid_caught_classes {
        context.collector.report_with_code(
            IssueCode::NoValidCatchTypeFound,
            Issue::error(
                "None of the types specified in the `catch` declaration are valid catchable exceptions."
            )
            .with_annotation(
                Annotation::primary(hint.span())
                    .with_message("This type declaration does not resolve to any class/interface that can be caught."),
            )
            .with_help(
                "Ensure the type hint contains at least one valid class or interface name that implements `Throwable` (e.g., `\\Exception`, `\\MyCustomError`). If all types in the hint are invalid for catching, this `catch` block will not catch exceptions based on this type hint."
            )
            .with_note(
                "To be caught, a type must be a defined class or interface that implements the `Throwable` interface. This can occur if specified types are undefined, are enums/traits, are primitive types, or are classes/interfaces that do not implement `Throwable`."
            )
            .with_note(
                "For analysis purposes, if no valid types were found, Mago might internally default to treating this as `catch (\\Throwable $e)` for subsequent control flow analysis."
            ),
        );

        caught_classes.insert(throwable);
    }

    (caught_classes, has_valid_caught_classes)
}

/// Returns whether an exception represented by `thrown` could match `caught`.
///
/// The throw set stores declared types, which may include subclasses. Incomplete metadata and
/// interface intersections therefore need conservative handling to avoid false unreachable-catch
/// reports.
fn could_be_caught<A>(context: &Context<'_, '_, A>, thrown: Word, caught: Word) -> bool
where
    A: Arena,
{
    let thrown_name = thrown.as_bytes();
    let caught_name = caught.as_bytes();

    if thrown_name.eq_ignore_ascii_case(caught_name)
        || context.codebase.is_instance_of(thrown_name, caught_name)
        || context.codebase.is_instance_of(caught_name, thrown_name)
    {
        return true;
    }

    let Some(thrown_metadata) = context.codebase.get_class_like(thrown_name) else {
        return true;
    };
    let Some(caught_metadata) = context.codebase.get_class_like(caught_name) else {
        return true;
    };

    let thrown_is_interface = thrown_metadata.kind.is_interface();
    let caught_is_interface = caught_metadata.kind.is_interface();
    if thrown_is_interface && caught_is_interface {
        return true;
    }

    if thrown_is_interface && caught_metadata.kind.is_class() {
        return !caught_metadata.flags.is_final();
    }

    if caught_is_interface && thrown_metadata.kind.is_class() {
        return !thrown_metadata.flags.is_final();
    }

    false
}

fn has_untracked_throw_source<'arena, A>(
    context: &Context<'_, 'arena, A>,
    block: &mago_syntax::cst::Block<'arena>,
    possibly_thrown_exceptions: &WordMap<HashSet<Span>>,
) -> bool
where
    A: Arena,
{
    Node::Block(block)
        .filter_map(|node| match node {
            Node::Call(call) => {
                let span = call.span();
                if has_tracked_throw_span(possibly_thrown_exceptions, span) {
                    return None;
                }

                match call {
                    mago_syntax::cst::Call::Function(function_call) => {
                        let Expression::Identifier(identifier) = function_call.function else {
                            return Some(());
                        };

                        if identifier.last_segment().eq_ignore_ascii_case(b"json_decode") {
                            return Some(());
                        }

                        let Some(function) = context.codebase.get_function(identifier.value()) else {
                            return Some(());
                        };

                        let has_untracked_thrown_types = if function.flags.is_internal() {
                            !function.thrown_types.is_empty()
                        } else {
                            function.thrown_types.is_empty()
                        };

                        has_untracked_thrown_types.then_some(())
                    }
                    _ => Some(()),
                }
            }
            Node::Instantiation(instantiation) => {
                let span = instantiation.span();
                if has_tracked_throw_span(possibly_thrown_exceptions, span) {
                    return None;
                }

                let Expression::Identifier(identifier) = instantiation.class else {
                    return Some(());
                };

                let class_name = identifier.value().strip_prefix(b"\\").unwrap_or(identifier.value());
                if !context.codebase.is_instance_of(class_name, b"Throwable") {
                    return Some(());
                }

                has_untracked_throwable_constructor(context.codebase, class_name).then_some(())
            }
            Node::Foreach(_)
            | Node::Clone(_)
            | Node::ArrayAccess(_)
            | Node::PropertyAccess(_)
            | Node::NullSafePropertyAccess(_) => Some(()),
            _ => None,
        })
        .into_iter()
        .any(|_| true)
}

fn has_tracked_throw_span(possibly_thrown_exceptions: &WordMap<HashSet<Span>>, span: Span) -> bool {
    possibly_thrown_exceptions.values().any(|spans| spans.contains(&span))
}

fn has_untracked_throwable_constructor(codebase: &CodebaseMetadata, class_name: &[u8]) -> bool {
    let Some(class_metadata) = codebase.get_class_like(class_name) else {
        return true;
    };

    let constructor_name = word("__construct");
    let mut pending = vec![class_metadata.name];
    let mut visited = WordSet::default();

    while let Some(class_name) = pending.pop() {
        if !visited.insert(class_name) {
            continue;
        }

        let Some(class_metadata) = codebase.get_class_like_by_name(class_name) else {
            return true;
        };

        if let Some(aliased_method_name) = class_metadata.trait_alias_map.get(&constructor_name).copied() {
            return has_untracked_trait_alias_constructor(
                codebase,
                class_metadata,
                constructor_name,
                aliased_method_name,
            );
        }

        let constructor_id = get_declared_method_identifier(class_metadata, constructor_name);

        if let Some(constructor_id) = constructor_id {
            let Some(constructor) = codebase.get_method_by_id(&constructor_id) else {
                return true;
            };

            let is_implicit = constructor.name_span.is_none()
                && codebase
                    .get_class_like_by_name(constructor_id.get_class_name())
                    .is_some_and(|declaring_class| constructor.span == declaring_class.span);
            if !is_implicit {
                return !constructor.flags.is_built_in();
            }
        }

        if let Some(parent_class) = class_metadata.direct_parent_class {
            pending.push(parent_class);
        }

        pending.extend(class_metadata.used_traits.iter().copied());
    }

    false
}

fn has_untracked_trait_alias_constructor(
    codebase: &CodebaseMetadata,
    class_metadata: &ClassLikeMetadata,
    constructor_name: Word,
    aliased_method_name: Word,
) -> bool {
    if let Some(declaring_method_id) = class_metadata.declaring_method_ids.get(&constructor_name).copied()
        && codebase
            .get_class_like_by_name(declaring_method_id.get_class_name())
            .is_some_and(|declaring_class| declaring_class.kind.is_trait())
    {
        return codebase.get_method_by_id(&declaring_method_id).is_none_or(|method| !method.flags.is_built_in());
    }

    let mut found_method = false;

    for trait_name in class_metadata.direct_trait_names() {
        let Some(trait_metadata) = codebase.get_class_like_by_name(trait_name) else {
            return true;
        };

        let method_id = get_declared_method_identifier(trait_metadata, aliased_method_name);

        let Some(method_id) = method_id else {
            continue;
        };

        found_method = true;
        if codebase.get_method_by_id(&method_id).is_none_or(|method| !method.flags.is_built_in()) {
            return true;
        }
    }

    !found_method
}

fn get_declared_method_identifier(class_metadata: &ClassLikeMetadata, method_name: Word) -> Option<MethodIdentifier> {
    class_metadata.declaring_method_ids.get(&method_name).copied().or_else(|| {
        class_metadata.methods.contains(&method_name).then(|| MethodIdentifier::new(class_metadata.name, method_name))
    })
}
