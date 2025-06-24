use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;
use ahash::HashSetExt;

use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::get_class_like;
use mago_codex::is_instance_of;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::context::scope::finally_scope::FinallyScope;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::statement::analyze_statements;

impl Analyzable for Try {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let mut catch_actions = vec![];
        let mut all_catches_leave = !self.catch_clauses.is_empty();

        for catch_clause in self.catch_clauses.iter() {
            let actions =
                ControlAction::from_statements(catch_clause.block.statements.to_vec(), vec![], Some(artifacts), true);

            all_catches_leave = all_catches_leave && !actions.contains(&ControlAction::None);
            catch_actions.push(actions);
        }

        let existing_thrown_exceptions = std::mem::take(&mut block_context.possibly_thrown_exceptions);
        let old_block_context_locals = block_context.locals.clone();
        let mut try_block_context = block_context.clone();

        if self.finally_clause.is_some() {
            try_block_context.finally_scope = Some(Rc::new(RefCell::new(FinallyScope { locals: BTreeMap::new() })));
        }

        let assigned_variable_ids = std::mem::take(&mut block_context.assigned_variable_ids);

        let was_inside_try = block_context.inside_try;
        block_context.inside_try = true;
        analyze_statements(self.block.statements.as_slice(), context, block_context, artifacts)?;
        block_context.inside_try = was_inside_try;
        block_context.has_returned = false;

        let try_block_control_actions =
            ControlAction::from_statements(self.block.statements.to_vec(), vec![], Some(artifacts), true);

        let newly_assigned_variable_ids = std::mem::take(&mut block_context.assigned_variable_ids);
        block_context.assigned_variable_ids.extend(assigned_variable_ids);
        block_context.assigned_variable_ids.extend(newly_assigned_variable_ids.iter().map(|(v, u)| (v.clone(), *u)));

        for (variable_id, variable_type) in std::mem::take(&mut block_context.locals) {
            match try_block_context.locals.entry(variable_id.clone()) {
                Entry::Occupied(mut occupied_entry) => {
                    let combined_type = ttype::combine_union_types(
                        occupied_entry.get(),
                        variable_type.as_ref(),
                        context.codebase,
                        context.interner,
                        false,
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

            for (variable_id, variable_type) in &block_context.locals {
                if let Some(existing_type) = mutable_try_scope.locals.get_mut(variable_id) {
                    let combined_type = ttype::combine_union_types(
                        existing_type,
                        variable_type.as_ref(),
                        context.codebase,
                        context.interner,
                        false,
                    );

                    *existing_type = Rc::new(combined_type);
                } else {
                    mutable_try_scope.locals.insert(variable_id.clone(), variable_type.clone());
                }
            }
        }

        try_block_context.possibly_thrown_exceptions = block_context.possibly_thrown_exceptions.clone();
        try_block_context.variables_possibly_in_scope = block_context.variables_possibly_in_scope.clone();

        let try_leaves_loop = context
            .loop_scope
            .as_ref()
            .map(|loop_scope| !loop_scope.final_actions.contains(&ControlAction::None))
            .unwrap_or(false);

        if !all_catches_leave {
            for assigned_variable_id in newly_assigned_variable_ids.keys() {
                block_context.remove_variable_from_conflicting_clauses(
                    context.interner,
                    context.codebase,
                    &mut context.buffer,
                    artifacts,
                    assigned_variable_id,
                    None,
                );
            }
        } else {
            for assigned_variable_id in newly_assigned_variable_ids.keys() {
                try_block_context.remove_variable_from_conflicting_clauses(
                    context.interner,
                    context.codebase,
                    &mut context.buffer,
                    artifacts,
                    assigned_variable_id,
                    None,
                );
            }
        }

        let mut original_block_context = try_block_context.clone();
        let mut definitely_newly_assigned_var_ids = newly_assigned_variable_ids;

        for (i, catch_clause) in self.catch_clauses.iter().enumerate() {
            let mut catch_block_context = original_block_context.clone();
            catch_block_context.has_returned = false;
            for (variable_id, variable_type) in catch_block_context.locals.iter_mut() {
                match old_block_context_locals.get(variable_id) {
                    Some(old_type) => {
                        *variable_type = Rc::new(ttype::combine_union_types(
                            variable_type.as_ref(),
                            old_type,
                            context.codebase,
                            context.interner,
                            false,
                        ));
                    }
                    None => {
                        let mut possibly_undefined_type = (**variable_type).clone();
                        possibly_undefined_type.set_possibly_undefined(variable_type.possibly_undefined, Some(true));

                        *variable_type = Rc::new(possibly_undefined_type);
                    }
                }
            }

            let caught_classes = get_caught_classes(context, &catch_clause.hint);
            let possibly_thrown_exceptions = std::mem::take(&mut catch_block_context.possibly_thrown_exceptions);
            for caught_class in caught_classes.iter() {
                let caught_class_str = context.interner.lookup(caught_class);

                for (possibly_thrown_exception, _) in possibly_thrown_exceptions.iter() {
                    let possibly_thrown_exception_str = context.interner.lookup(possibly_thrown_exception);

                    if possibly_thrown_exception_str.eq_ignore_ascii_case(caught_class_str)
                        || is_instance_of(context.codebase, context.interner, possibly_thrown_exception, caught_class)
                    {
                        original_block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                        block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                        catch_block_context.possibly_thrown_exceptions.remove(possibly_thrown_exception);
                    }
                }
            }

            catch_block_context.clauses = vec![];
            if let Some(catch_variable) = catch_clause.variable.as_ref() {
                let mut exception_type = TUnion::new(
                    caught_classes
                        .iter()
                        .map(|caught_class| TAtomic::Object(TObject::Named(TNamedObject::new(*caught_class))))
                        .collect(),
                );

                exception_type
                    .parent_nodes
                    .push(DataFlowNode::get_for_lvar(VariableIdentifier(catch_variable.name), catch_variable.span));

                let catch_variable_id = context.interner.lookup(&catch_variable.name);
                catch_block_context.locals.insert(catch_variable_id.to_owned(), Rc::new(exception_type));
                catch_block_context.remove_variable_from_conflicting_clauses(
                    context.interner,
                    context.codebase,
                    &mut context.buffer,
                    artifacts,
                    catch_variable_id,
                    None,
                );
                catch_block_context.variables_possibly_in_scope.insert(catch_variable_id.to_owned());
            }

            let old_catch_assigned_variable_ids = std::mem::take(&mut catch_block_context.assigned_variable_ids);

            analyze_statements(catch_clause.block.statements.as_slice(), context, &mut catch_block_context, artifacts)?;

            // recalculate in case there's a no-return clause
            if let Some(actions) = catch_actions.get_mut(i) {
                *actions = ControlAction::from_statements(
                    catch_clause.block.statements.to_vec(),
                    vec![],
                    Some(artifacts),
                    true,
                );
            }

            let new_catch_assigned_variables_ids = catch_block_context.assigned_variable_ids.clone();
            catch_block_context.assigned_variable_ids.extend(old_catch_assigned_variable_ids);

            for (exception, spans) in catch_block_context.possibly_thrown_exceptions {
                block_context.possibly_thrown_exceptions.entry(exception).or_default().extend(spans);
            }

            let catch_doesnt_leave_parent_scope = {
                let catch_actions = &catch_actions[i];

                if catch_actions.len() == 1 {
                    !catch_actions.contains(&ControlAction::End)
                        && !catch_actions.contains(&ControlAction::Continue)
                        && !catch_actions.contains(&ControlAction::Break)
                } else {
                    true
                }
            };

            if catch_doesnt_leave_parent_scope {
                definitely_newly_assigned_var_ids = new_catch_assigned_variables_ids
                    .iter()
                    .filter(|(key, _)| definitely_newly_assigned_var_ids.contains_key(*key))
                    .map(|(key, value)| (key.clone(), *value))
                    .collect();

                let end_action_only =
                    try_block_control_actions.len() == 1 && try_block_control_actions[0] == ControlAction::End;

                for (variable_id, variable_type) in &catch_block_context.locals {
                    if end_action_only {
                        block_context.locals.insert(variable_id.clone(), variable_type.clone());
                    } else if let Some(existing_type) = block_context.locals.get(variable_id) {
                        block_context.locals.insert(
                            variable_id.clone(),
                            Rc::new(ttype::combine_union_types(
                                existing_type.as_ref(),
                                variable_type.as_ref(),
                                context.codebase,
                                context.interner,
                                false,
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
                        ttype::combine_union_types(
                            finally_variable_type.as_ref(),
                            variable_type.as_ref(),
                            context.codebase,
                            context.interner,
                            false,
                        )
                    } else {
                        let mut finally_variable_type = (**variable_type).clone();
                        finally_variable_type.set_possibly_undefined(true, Some(true));

                        finally_variable_type
                    };

                    finally_scope.locals.insert(variable_id.clone(), Rc::new(resulting_type));
                }
            }
        }

        if !try_leaves_loop && let Some(loop_scope) = context.loop_scope.as_mut() {
            loop_scope.final_actions.insert(ControlAction::None);
        }

        let mut finally_has_returned = false;
        if let Some(finally_clause) = self.finally_clause.as_ref() {
            let finally_scope = unsafe {
                try_block_context
                    .finally_scope
                    .take()
                    .map(|scope| scope.as_ref().clone())
                    .map(|s| s.into_inner())
                    .unwrap_unchecked()
            };

            let mut finally_block_context = block_context.clone();
            finally_block_context.assigned_variable_ids = HashMap::default();
            finally_block_context.possibly_assigned_variable_ids = HashSet::default();
            finally_block_context.locals = finally_scope.locals;

            analyze_statements(
                finally_clause.block.statements.as_slice(),
                context,
                &mut finally_block_context,
                artifacts,
            )?;

            finally_has_returned = finally_block_context.has_returned;

            for (variable_id, _) in finally_block_context.assigned_variable_ids {
                let finally_variable_type = finally_block_context.locals.remove(&variable_id);
                if let Some(finally_variable_type) = finally_variable_type {
                    let resulting_type = match block_context.locals.remove(&variable_id) {
                        Some(existing_type) => {
                            let possibly_undefined =
                                finally_variable_type.possibly_undefined_from_try && existing_type.possibly_undefined;

                            let mut combined_type = ttype::combine_union_types(
                                existing_type.as_ref(),
                                finally_variable_type.as_ref(),
                                context.codebase,
                                context.interner,
                                false,
                            );

                            if possibly_undefined {
                                combined_type.possibly_undefined = false;
                                combined_type.possibly_undefined_from_try = false;
                            }

                            Rc::new(combined_type)
                        }
                        None => finally_variable_type,
                    };

                    block_context.locals.insert(variable_id, resulting_type);
                }
            }
        }

        for (variable_id, _) in definitely_newly_assigned_var_ids {
            let Some(variable_type) = block_context.locals.get_mut(&variable_id) else {
                continue;
            };

            if !variable_type.possibly_undefined_from_try {
                continue;
            }

            let mut defined_variable_type = (**variable_type).clone();
            defined_variable_type.set_possibly_undefined(false, Some(false));

            *variable_type = Rc::new(defined_variable_type);
        }

        for (possibly_thrown_exception, throw_spans) in existing_thrown_exceptions {
            block_context.possibly_thrown_exceptions.entry(possibly_thrown_exception).or_default().extend(throw_spans);
        }

        block_context.has_returned =
            finally_has_returned || (!try_block_control_actions.contains(&ControlAction::None) && all_catches_leave);

        Ok(())
    }
}

fn get_caught_classes(context: &mut Context<'_>, hint: &Hint) -> HashSet<StringIdentifier> {
    let mut caught_identifiers: HashMap<StringIdentifier, Span> = HashMap::default();

    fn walk(context: &mut Context<'_>, hint: &Hint, caught: &mut HashMap<StringIdentifier, Span>) {
        match hint {
            Hint::Identifier(identifier) => {
                let id = *context.resolved_names.get(identifier);

                if let Some(&first_span) = caught.get(&id) {
                    context.buffer.report(
                        TypingIssueKind::DuplicateCaughtType,
                        Issue::error(format!(
                            "Type `{}` is caught multiple times in the same `catch` clause.",
                            context.interner.lookup(&id)
                        ))
                        .with_annotation(
                            Annotation::primary(hint.span())
                                .with_message("This type is a duplicate occurrence here"),
                        )
                        .with_annotation(
                            Annotation::secondary(first_span)
                                .with_message(format!("`{}` was already specified here", context.interner.lookup(&id))),
                        )
                        .with_help("Remove the redundant type from the `catch` union. Each exception type should only be listed once."),
                    );
                } else {
                    caught.insert(id, hint.span());
                }
            }
            Hint::Union(union_hint) => {
                walk(context, &union_hint.left, caught);
                walk(context, &union_hint.right, caught);
            }
            _ => {
                context.buffer.report(
                    TypingIssueKind::InvalidCatchType,
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

    let throwable = context.interner.intern("Throwable");
    let mut caught_classes = HashSet::with_capacity(caught_identifiers.len());
    for (caught_type, caught_span) in caught_identifiers.into_iter() {
        let caught_type_str = context.interner.lookup(&caught_type).to_ascii_lowercase();
        if caught_type_str == "throwable" || caught_type_str == "exception" || caught_type_str == "error" {
            caught_classes.insert(caught_type);
            continue;
        }

        let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, &caught_type) else {
            let caught_type_str = context.interner.lookup(&caught_type);

            context.buffer.report(
                TypingIssueKind::NonExistentCatchType,
                Issue::error(format!("Attempting to catch an undefined class or interface: `{caught_type_str}`."))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("Type `{caught_type_str}` is not defined or cannot be found")),
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

        if class_like_metadata.is_enum() || class_like_metadata.is_trait() {
            let caught_type_str = context.interner.lookup(&caught_type);
            let kind_str = if class_like_metadata.is_enum() { "an enum" } else { "a trait" };

            context.buffer.report(
                TypingIssueKind::InvalidCatchTypeNotClassOrInterface,
                Issue::error(format!(
                    "Only classes or interfaces can be caught, but `{caught_type_str}` is {kind_str}.",
                ))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("Cannot catch `{caught_type_str}` because it is {kind_str}")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_metadata.get_name_span().unwrap_or(class_like_metadata.get_span()))
                        .with_message(format!("`{caught_type_str}` is defined as {kind_str} here")),
                )
                .with_note("PHP `catch` blocks require a class or interface type. Enums and traits are not valid types for catching exceptions as they cannot be thrown or extend `Throwable`.")
                .with_help("Specify a class or interface that implements `Throwable` (e.g., `Exception`, `Error`, or a custom exception class)."),
            );

            continue;
        }

        let is_throwable = is_instance_of(context.codebase, context.interner, &caught_type, &throwable);

        if !is_throwable {
            context.buffer.report(
                TypingIssueKind::CatchTypeNotThrowable,
                Issue::error(format!(
                    "The type `{caught_type_str}` caught in a catch block must implement the `Throwable` interface.",
                ))
                .with_annotation(
                    Annotation::primary(caught_span)
                        .with_message(format!("`{caught_type_str}` is not an instance of `Throwable`")),
                )
                .with_annotation(
                    Annotation::secondary(class_like_metadata.get_name_span().unwrap_or(class_like_metadata.get_span()))
                        .with_message(format!("`{caught_type_str}` defined here does not implement `Throwable`")),
                )
                .with_note("In PHP, only objects that implement the `Throwable` interface (this includes `Exception` and `Error` classes and their children) can be caught in a `catch` block.")
                .with_help(format!("Ensure that `{caught_type_str}` implements the `Throwable` interface, or catch a more general exception type like `Exception` or `Throwable` itself.")),
            );

            continue;
        }

        caught_classes.insert(caught_type);
    }

    if caught_classes.is_empty() {
        context.buffer.report(
            TypingIssueKind::NoValidCatchTypeFound,
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

    caught_classes
}
