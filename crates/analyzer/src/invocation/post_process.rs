use std::collections::BTreeMap;
use std::rc::Rc;

use ahash::HashSet;
use indexmap::IndexMap;

use mago_algebra::assertion_set::AssertionSet;
use mago_algebra::assertion_set::Conjunction;
use mago_algebra::assertion_set::Disjunction;
use mago_algebra::assertion_set::add_and_assertion;
use mago_algebra::assertion_set::add_and_clause;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::saturate_clauses;
use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_codex::assertion::Assertion;
use mago_codex::get_function_like_thrown_types;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::expression::assignment::assign_to_expression;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::resolver::resolve_invocation_type;
use crate::reconciler;
use crate::reconciler::assertion_reconciler::intersect_union_with_union;
use crate::utils::expression::get_expression_id;

pub fn post_invocation_process<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invoication: &Invocation<'ctx, 'ast, 'arena>,
    this_variable: Option<String>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    apply_assertions: bool,
) -> Result<(), AnalysisError> {
    update_by_reference_argument_types(context, block_context, artifacts, invoication, template_result, parameters)?;

    let Some(identifier) = invoication.target.get_function_like_identifier() else {
        return Ok(());
    };

    let Some(metadata) = invoication.target.get_function_like_metadata() else {
        return Ok(());
    };

    let (callable_kind_str, full_callable_name) = match identifier {
        FunctionLikeIdentifier::Function(name) => ("function", format!("`{name}`")),
        FunctionLikeIdentifier::Method(class_name, method_name) => {
            ("method", format!("`{}::{}`", class_name, method_name))
        }
        FunctionLikeIdentifier::Closure(file_id, position) => (
            "closure",
            format!(
                "defined at `{}:{}:{}`",
                if *file_id == context.source_file.id {
                    context.source_file.name.to_string()
                } else {
                    format!("<file:{file_id}>")
                },
                context.source_file.line_number(position.offset),
                context.source_file.column_number(position.offset)
            ),
        ),
    };

    if metadata.flags.is_deprecated() {
        let issue_kind = match identifier {
            FunctionLikeIdentifier::Function(_) => IssueCode::DeprecatedFunction,
            FunctionLikeIdentifier::Method(_, _) => IssueCode::DeprecatedMethod,
            FunctionLikeIdentifier::Closure(_, _) => IssueCode::DeprecatedClosure,
        };

        context.collector.report_with_code(
            issue_kind,
            Issue::warning(format!("Call to deprecated {callable_kind_str}: {full_callable_name}."))
                .with_annotation(
                    Annotation::primary(invoication.target.span()).with_message(format!("This {callable_kind_str} is deprecated")),
                )
                .with_note(format!(
                    "The {callable_kind_str} {full_callable_name} is marked as deprecated and may be removed or its behavior changed in future versions."
                ))
                .with_help(format!(
                    "Consult the documentation for {full_callable_name} for alternatives or migration instructions."
                )),
        );
    }

    // Report if named arguments are used where not allowed
    if metadata.flags.forbids_named_arguments()
        && let InvocationArgumentsSource::ArgumentList(argument_list) = invoication.arguments_source
    {
        for argument in argument_list.arguments.iter() {
            let Argument::Named(_) = argument else {
                continue; // Skip if it's not a named argument
            };

            context.collector.report_with_code(
                IssueCode::NamedArgumentNotAllowed,
                Issue::error(format!("Named arguments are not allowed for {full_callable_name}."))
                    .with_annotation(Annotation::primary(argument.span()).with_message("Named argument used here"))
                    .with_annotation(Annotation::secondary(invoication.target.span()).with_message(format!(
                        "The {callable_kind_str} {full_callable_name} only accepts positional arguments"
                    )))
                    .with_help("Convert this named argument to a positional argument."),
            );
        }
    }

    if context.settings.check_throws {
        let thrown_types = get_function_like_thrown_types(
            context.codebase,
            invoication.target.get_method_context().map(|context| context.class_like_metadata),
            metadata,
        );

        for thrown_exception_type in thrown_types {
            let resolved_exception_type = resolve_invocation_type(
                context,
                invoication,
                template_result,
                parameters,
                thrown_exception_type.type_union.clone(),
            );

            for exception_atomic in resolved_exception_type.types.into_owned() {
                for exception in exception_atomic.get_all_object_names() {
                    block_context.possibly_thrown_exceptions.entry(exception).or_default().insert(invoication.span);
                }
            }
        }
    }

    if !apply_assertions {
        return Ok(());
    }

    let range = (invoication.span.start.offset, invoication.span.end.offset);

    let resolved_if_true_assertions = resolve_invocation_assertion(
        context,
        block_context,
        artifacts,
        invoication,
        &this_variable,
        &metadata.if_true_assertions,
        template_result,
        parameters,
    );

    for (variable, assertions) in resolved_if_true_assertions {
        artifacts.if_true_assertions.entry(range).or_default().entry(variable).or_default().extend(assertions);
    }

    let resolved_if_false_assertions = resolve_invocation_assertion(
        context,
        block_context,
        artifacts,
        invoication,
        &this_variable,
        &metadata.if_false_assertions,
        template_result,
        parameters,
    );

    for (variable, assertions) in resolved_if_false_assertions {
        artifacts.if_false_assertions.entry(range).or_default().entry(variable).or_default().extend(assertions);
    }

    apply_assertion_to_call_context(
        context,
        block_context,
        artifacts,
        invoication,
        &this_variable,
        &metadata.assertions,
        template_result,
        parameters,
    );

    Ok(())
}

fn apply_assertion_to_call_context<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    this_variable: &Option<String>,
    assertions: &BTreeMap<Atom, Conjunction<Assertion>>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) {
    let type_assertions = resolve_invocation_assertion(
        context,
        block_context,
        artifacts,
        invocation,
        this_variable,
        assertions,
        template_result,
        parameters,
    );

    if type_assertions.is_empty() {
        return;
    }

    let referenced_variable_ids: HashSet<String> = type_assertions.keys().cloned().collect();
    let mut changed_variable_ids: HashSet<String> = HashSet::default();
    let mut active_type_assertions = IndexMap::new();
    for (variable, type_assertion) in &type_assertions {
        active_type_assertions.insert(variable.clone(), (1..type_assertion.len()).collect());
    }

    reconciler::reconcile_keyed_types(
        context,
        &type_assertions,
        active_type_assertions,
        block_context,
        &mut changed_variable_ids,
        &referenced_variable_ids,
        &invocation.span,
        true,
        false,
    );
}

fn update_by_reference_argument_types<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) -> Result<(), AnalysisError> {
    let constraint_type = invocation.target.is_method_call();

    for (parameter_offset, parameter_ref) in invocation.target.get_parameters().into_iter().enumerate() {
        if !parameter_ref.is_by_reference() {
            continue;
        }

        let (argument, argument_id) = get_argument_for_parameter(
            context,
            block_context,
            invocation,
            Some(parameter_offset),
            parameter_ref.get_name().map(|name| name.0),
        );

        if let Some(argument) = argument {
            let mut new_type = parameter_ref
                .get_out_type()
                .or_else(|| parameter_ref.get_type())
                .cloned()
                .map(|new_type| resolve_invocation_type(context, invocation, template_result, parameters, new_type))
                .unwrap_or_else(get_mixed);

            new_type.by_reference = true;

            if constraint_type && let Some(argument_id) = argument_id {
                assign_to_expression(
                    context,
                    block_context,
                    artifacts,
                    argument,
                    Some(argument_id.clone()),
                    Some(argument),
                    new_type.clone(),
                    false,
                )?;

                block_context.by_reference_constraints.insert(
                    argument_id,
                    ReferenceConstraint::new(argument.span(), ReferenceConstraintSource::Argument, Some(new_type)),
                );
            } else {
                assign_to_expression(
                    context,
                    block_context,
                    artifacts,
                    argument,
                    argument_id,
                    Some(argument),
                    new_type,
                    false,
                )?;
            }
        }
    }

    Ok(())
}

fn resolve_invocation_assertion<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    this_variable: &Option<String>,
    assertions: &BTreeMap<Atom, Disjunction<Assertion>>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) -> IndexMap<String, AssertionSet> {
    let mut type_assertions: IndexMap<String, AssertionSet> = IndexMap::new();
    if assertions.is_empty() {
        return type_assertions;
    }

    for (parameter_id, variable_assertions) in assertions {
        let (assertion_expression, assertion_variable) =
            resolve_argument_or_special_target(context, block_context, invocation, parameter_id, this_variable);

        match assertion_variable {
            Some(assertion_variable) => {
                let mut new_variable_possibilities: AssertionSet = vec![];
                let mut resolved_or_clause: Disjunction<Assertion> = Vec::new();

                for variable_assertion in variable_assertions {
                    let Some(assertion_atomic) = variable_assertion.get_type() else {
                        add_and_assertion(&mut new_variable_possibilities, variable_assertion.clone());

                        continue;
                    };

                    let resolved_assertion_type = resolve_invocation_type(
                        context,
                        invocation,
                        template_result,
                        parameters,
                        TUnion::from_atomic(assertion_atomic.to_owned()),
                    );

                    if !resolved_assertion_type.is_never() {
                        for resolved_atomic in resolved_assertion_type.types.into_owned() {
                            resolved_or_clause.push(variable_assertion.with_type(resolved_atomic));
                        }
                    } else if let Some(asserted_type) = block_context.locals.get(&assertion_variable) {
                        match variable_assertion {
                            Assertion::IsType(_) => {
                                if !can_expression_types_be_identical(
                                    context.codebase,
                                    asserted_type,
                                    &resolved_assertion_type,
                                    false,
                                    false,
                                ) {
                                    // TODO(azejzz): report this as an issue.
                                    //
                                    // e.g:
                                    //
                                    // $foo = new Foo;
                                    // assert_is_bar_or_baz($foo);
                                    //                      ^- impossible
                                }
                            }
                            Assertion::IsIdentical(_) => {
                                let intersection = match intersect_union_with_union(
                                    context,
                                    asserted_type,
                                    &resolved_assertion_type,
                                ) {
                                    Some(intersection) => intersection,
                                    None => {
                                        // TODO: impossible assertion
                                        get_never()
                                    }
                                };

                                for intersection_atomic in intersection.types.into_owned() {
                                    add_and_assertion(
                                        &mut new_variable_possibilities,
                                        Assertion::IsIdentical(intersection_atomic),
                                    );
                                }
                            }
                            _ => {
                                // ignore
                            }
                        }
                    }
                }

                if !resolved_or_clause.is_empty() {
                    add_and_clause(&mut new_variable_possibilities, &resolved_or_clause);
                }

                if !new_variable_possibilities.is_empty() {
                    type_assertions.entry(assertion_variable).or_default().extend(new_variable_possibilities);
                }
            }
            None => {
                if let Some(assertion_expression) = assertion_expression {
                    if variable_assertions.len() != 1 {
                        continue; // We only support single assertions for expressions
                        // maybe we should support more? idk for now, we are following
                        // psalm implementation.
                    }

                    let variable_assertion = &variable_assertions[0];

                    let clauses = match variable_assertion {
                        Assertion::IsNotType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) })))
                        | Assertion::IsType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(true) })))
                        | Assertion::Truthy => get_formula(
                            assertion_expression.span(),
                            assertion_expression.span(),
                            assertion_expression,
                            context.get_assertion_context_from_block(block_context),
                            artifacts,
                        ),
                        Assertion::IsNotType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(true) })))
                        | Assertion::IsType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) })))
                        | Assertion::Falsy => get_formula(
                            assertion_expression.span(),
                            assertion_expression.span(),
                            assertion_expression,
                            context.get_assertion_context_from_block(block_context),
                            artifacts,
                        )
                        .map(|clauses| {
                            negate_or_synthesize(
                                clauses,
                                assertion_expression,
                                context.get_assertion_context_from_block(block_context),
                                artifacts,
                            )
                        }),

                        _ => {
                            continue; // Unsupported assertion kind for expression
                        }
                    };

                    let clauses = saturate_clauses(
                        block_context.clauses.iter().map(Rc::as_ref).chain(clauses.unwrap_or_default().iter()),
                    );

                    let (truths, _) = find_satisfying_assignments(&clauses, None, &mut Default::default());
                    for (variable, assertions) in truths {
                        type_assertions.entry(variable).or_default().extend(assertions);
                    }
                }
            }
        };
    }

    type_assertions
}

/// Resolves an argument or a special assertion target from an invocation.
///
/// This function serves as a convenient wrapper that orchestrates the logic for handling
/// both special assertion targets (like `$this`) and standard function arguments.
///
/// It first attempts to resolve the target as a special `$this` or `self::` reference
/// using `resolve_special_assertion_target`. If successful, it returns the resolved ID,
/// and the expression part of the tuple will be `None`.
///
/// If the target is not a special reference, it then calls `get_argument_for_parameter`
/// to find the corresponding argument passed to the function call and returns its result.
///
/// # Arguments
/// * `context`: The analysis context.
/// * `block_context`: The context of the current block.
/// * `invocation`: The invocation being analyzed.
/// * `parameter_offset`: The zero-based index of the parameter.
/// * `parameter_name`: The name of the parameter or assertion target.
/// * `this_variable`: The name of the variable holding the object instance (`$this`), if any.
///
/// # Returns
/// A tuple `(Option<&'a Expression>, Option<String>)`.
/// * If a special target is resolved, the tuple is `(None, Some(resolved_id))`.
/// * If a regular argument is found, it returns the result from `get_argument_for_parameter`.
/// * If nothing is found, it returns `(None, None)`.
fn resolve_argument_or_special_target<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    parameter_name: &Atom,
    this_variable: &Option<String>,
) -> (Option<&'ast Expression<'arena>>, Option<String>) {
    // First, check if the name refers to a special assertion target like `$this->...`
    if let Some(resolved_id) = resolve_special_assertion_target(block_context, parameter_name, this_variable) {
        return (None, Some(resolved_id));
    }

    // If not a special target, treat it as a regular parameter and find its argument.
    get_argument_for_parameter(context, block_context, invocation, None, Some(*parameter_name))
}

/// Resolves special assertion targets like `$this->...` or `self::...`.
///
/// This function checks if the provided target name corresponds to a property or constant
/// on the current object (`$this`) or class (`self`). If it matches, it rewrites the
/// target string with the appropriate contextual variable name (e.g., `$instance->...`).
/// It should be called before attempting to find an argument for a parameter, as these
/// targets do not correspond to passed arguments.
///
/// # Arguments
/// * `block_context`: The context of the current block, used to get class scope information.
/// * `target_name`: The string identifier for the assertion target (e.g., `'$this->prop'`).
/// * `this_variable`: The name of the variable holding the object instance (`$this`), if any.
///
/// # Returns
/// * `Some(String)`: If the target is a special `$this` or `self` reference, containing the resolved variable ID.
/// * `None`: If the target is not a special reference and should be treated as a regular parameter.
fn resolve_special_assertion_target<'ctx>(
    block_context: &BlockContext<'ctx>,
    target_name: &Atom,
    this_variable: &Option<String>,
) -> Option<String> {
    if let Some(this_variable) = this_variable
        && target_name.starts_with("$this")
    {
        return Some(target_name.replacen("$this", this_variable, 1));
    }

    if let Some(class) = block_context.scope.get_class_like_name()
        && target_name.starts_with("self::")
    {
        return Some(target_name.replacen("self::", &class, 1));
    }

    None
}

/// Finds the argument expression passed to a function for a specific parameter.
///
/// This function is designed to robustly identify the argument for a given parameter,
/// mirroring PHP's own argument resolution rules. The caller can provide the parameter's
/// name, its zero-based offset, or both.
///
/// # Arguments
///
/// * `context`: The analysis context, needed for `get_expression_id`.
/// * `block_context`: The context of the current block, needed for `get_expression_id`.
/// * `invocation`: The invocation being analyzed, which contains the arguments.
/// * `metadata`: The metadata of the invoked function, used to look up parameter details.
/// * `parameter_offset`: An optional zero-based index of the parameter.
/// * `parameter_name`: An optional name of the parameter.
///
/// # Returns
/// A tuple containing:
/// * `Option<&'a Expression>`: The argument's expression AST node, if found.
/// * `Option<String>`: The unique ID of the argument expression (e.g., a variable name), if it can be determined.
fn get_argument_for_parameter<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    mut parameter_offset: Option<usize>,
    mut parameter_name: Option<Atom>,
) -> (Option<&'ast Expression<'arena>>, Option<String>) {
    // If neither name nor offset is provided, we can't do anything.
    if parameter_name.is_none() && parameter_offset.is_none() {
        return (None, None);
    }

    let parameter_refs = invocation.target.get_parameters();

    // Step 1: Ensure we have both the name and offset for the parameter.
    if parameter_name.is_none() {
        if let Some(parameter_ref) = parameter_offset.and_then(|offset| parameter_refs.get(offset)) {
            parameter_name = parameter_ref.get_name().map(|name| name.0);
        }
    } else if parameter_offset.is_none()
        && let Some(name) = parameter_name
    {
        parameter_offset =
            parameter_refs.iter().position(|p| p.get_name().is_some_and(|name_variable| name_variable.0 == name));
    }

    // After attempting to fill in missing info, if we still lack a name or an offset,
    // the parameter is invalid for this function.
    let (_, Some(offset)) = (parameter_name, parameter_offset) else {
        return (None, None);
    };

    // Step 2: Resolve the argument with the correct precedence.
    let arguments = invocation.arguments_source.get_arguments();

    // a. Look for a named argument first.
    let find_by_name = || {
        let variable = parameter_name?;
        let variable_name = if let Some(variable) = variable.strip_prefix('$') { variable } else { variable.as_str() };

        arguments.iter().find(|argument| {
            if let Some(named_argument) = argument.get_named_argument() {
                named_argument.name.value == variable_name
            } else {
                false
            }
        })
    };

    // b. If not found by name, look for a positional argument at the correct offset.
    let find_by_position = || arguments.get(offset).filter(|argument| argument.is_positional());

    let argument = find_by_name().or_else(find_by_position);

    let Some(argument_expression) = argument.map(|argument| argument.value()) else {
        // The corresponding argument could not be found.
        return (None, None);
    };

    // If an argument was found, resolve its expression ID.
    let argument_id = get_expression_id(
        argument_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    (Some(argument_expression), argument_id)
}
