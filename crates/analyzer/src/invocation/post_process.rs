use std::collections::BTreeMap;
use std::rc::Rc;

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
use mago_atom::AtomSet;
use mago_codex::assertion::Assertion;

use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::bool::TBool;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Argument;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;

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
use crate::utils::misc::unwrap_expression;

pub fn post_invocation_process<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invoication: &Invocation<'ctx, '_, 'arena>,
    this_variable: Option<&str>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    apply_assertions: bool,
) -> Result<(), AnalysisError> {
    update_by_reference_argument_types(context, block_context, artifacts, invoication, template_result, parameters)?;
    clear_object_argument_property_narrowings(context, block_context, invoication);

    let Some(identifier) = invoication.target.get_function_like_identifier() else {
        return Ok(());
    };

    let Some(metadata) = invoication.target.get_function_like_metadata() else {
        return Ok(());
    };

    let (callable_kind_str, full_callable_name) = match identifier {
        FunctionLikeIdentifier::Function(name) => ("function", format!("`{name}`")),
        FunctionLikeIdentifier::Method(class_name, method_name) => ("method", format!("`{class_name}::{method_name}`")),
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
        for argument in &argument_list.arguments {
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
        let thrown_types = context.codebase.get_function_like_thrown_types(
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

        collect_plugin_throw_types(context, block_context, artifacts, invoication, identifier);
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
        this_variable,
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
        this_variable,
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
        this_variable,
        &metadata.assertions,
        template_result,
        parameters,
    );

    apply_plugin_assertions(
        context,
        block_context,
        artifacts,
        invoication,
        identifier,
        this_variable,
        template_result,
        parameters,
        range,
    );

    Ok(())
}

fn apply_assertion_to_call_context<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    this_variable: Option<&str>,
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

    let referenced_variable_ids: AtomSet = type_assertions.keys().copied().collect();
    let mut changed_variable_ids: AtomSet = AtomSet::default();
    let mut active_type_assertions = IndexMap::new();
    for (variable, type_assertion) in &type_assertions {
        active_type_assertions.insert(*variable, (1..type_assertion.len()).collect());
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

fn update_by_reference_argument_types<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
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
                .map_or_else(get_mixed, |new_type| {
                    resolve_invocation_type(context, invocation, template_result, parameters, new_type)
                });

            new_type.set_by_reference(true);

            if constraint_type && let Some(argument_id) = argument_id {
                if let Some(existing_type) = block_context.locals.get(&argument_id).cloned() {
                    block_context.remove_descendants(context, argument_id, &existing_type, Some(&new_type));
                }

                assign_to_expression(
                    context,
                    block_context,
                    artifacts,
                    argument,
                    Some(argument_id),
                    Some(argument),
                    new_type.clone(),
                    false,
                )?;

                block_context.by_reference_constraints.insert(
                    argument_id,
                    ReferenceConstraint::new(argument.span(), ReferenceConstraintSource::Argument, Some(new_type)),
                );
            } else {
                if let Some(argument_id) = &argument_id
                    && let Some(existing_type) = block_context.locals.get(argument_id).cloned()
                {
                    block_context.remove_descendants(context, *argument_id, &existing_type, Some(&new_type));
                }

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

/// Clears narrowed property types for object arguments passed to an invocation.
///
/// When an object is passed to a method or function, its properties could be modified.
/// This function removes any narrowed property types for object arguments to prevent
/// false positives from the analyzer assuming properties remain unchanged after the call.
fn clear_object_argument_property_narrowings<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    invocation: &Invocation<'ctx, '_, 'arena>,
) {
    let arguments = invocation.arguments_source.get_arguments();

    for argument in arguments {
        let Some(expression) = argument.value() else {
            continue;
        };

        let Some(argument_id) = get_expression_id(
            expression,
            block_context.scope.get_class_like_name(),
            context.resolved_names,
            Some(context.codebase),
        ) else {
            continue;
        };

        let Some(existing_type) = block_context.locals.get(&argument_id).cloned() else {
            continue;
        };

        if !existing_type.has_object_type() {
            continue;
        }

        let keys_to_remove: Vec<_> = block_context
            .locals
            .keys()
            .filter(|var_id| {
                if *var_id == &argument_id {
                    return false;
                }
                if !var_id.starts_with(argument_id.as_str()) {
                    return false;
                }
                let after_root = &var_id.as_str()[argument_id.len()..];
                after_root.starts_with("->") || after_root.starts_with('[')
            })
            .copied()
            .collect();

        for key in keys_to_remove {
            block_context.locals.remove(&key);
        }
    }
}

fn resolve_invocation_assertion<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    this_variable: Option<&str>,
    assertions: &BTreeMap<Atom, Conjunction<Assertion>>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
) -> IndexMap<Atom, AssertionSet> {
    let mut type_assertions: IndexMap<Atom, AssertionSet> = IndexMap::new();
    if assertions.is_empty() {
        return type_assertions;
    }

    for (parameter_id, variable_assertions) in assertions {
        let (assertion_expression, assertion_variable) =
            resolve_argument_or_special_target(context, block_context, invocation, *parameter_id, this_variable);

        match assertion_variable {
            Some(assertion_variable) => {
                let mut new_variable_possibilities: AssertionSet = vec![];
                let mut resolved_or_clause: Disjunction<Assertion> = Vec::new();

                let asserted_type = block_context.locals.get(&assertion_variable);
                let mut any_possible = false;
                let mut has_resolved_types = false;
                let mut all_negated = true;
                let mut always_redundant = true;

                for variable_assertion in variable_assertions {
                    all_negated = all_negated && variable_assertion.is_negation();

                    if variable_assertion.has_equality() {
                        always_redundant = false;
                    }

                    let Some(assertion_atomic) = variable_assertion.get_type() else {
                        add_and_assertion(&mut new_variable_possibilities, variable_assertion.clone());
                        always_redundant = false;

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
                        has_resolved_types = true;

                        if !any_possible
                            && let Some(asserted_type) = &asserted_type
                            && can_expression_types_be_identical(
                                context.codebase,
                                asserted_type,
                                &resolved_assertion_type,
                                false,
                                false,
                            )
                        {
                            any_possible = true;
                        }

                        if always_redundant
                            && let Some(asserted_type) = &asserted_type
                            && !asserted_type.is_mixed()
                            && !resolved_assertion_type.has_template()
                        {
                            let mut comparison_result = ComparisonResult::default();
                            let is_subtype = is_contained_by(
                                context.codebase,
                                asserted_type,
                                &resolved_assertion_type,
                                false,
                                false,
                                false,
                                &mut comparison_result,
                            );

                            if !is_subtype {
                                always_redundant = false;
                            }
                        } else {
                            always_redundant = false;
                        }

                        for resolved_atomic in resolved_assertion_type.types.into_owned() {
                            resolved_or_clause.push(variable_assertion.with_type(resolved_atomic));
                        }
                    } else if let Some(asserted_type) = &asserted_type {
                        always_redundant = false;
                        match variable_assertion {
                            Assertion::IsType(_) => {
                                if !can_expression_types_be_identical(
                                    context.codebase,
                                    asserted_type,
                                    &resolved_assertion_type,
                                    false,
                                    false,
                                ) {
                                    let asserted_type_id = asserted_type.get_id();
                                    let expected_type_id = resolved_assertion_type.get_id();

                                    context.collector.report_with_code(
                                        IssueCode::ImpossibleTypeComparison,
                                        Issue::error(format!(
                                            "Impossible type assertion: `{assertion_variable}` of type `{asserted_type_id}` can never be `{expected_type_id}`."
                                        ))
                                        .with_annotation(
                                            Annotation::primary(invocation.span)
                                                .with_message(format!("Argument `{assertion_variable}` has type `{asserted_type_id}`")),
                                        )
                                        .with_note(format!(
                                            "The assertion expects `{assertion_variable}` to be `{expected_type_id}`, but no value of type `{asserted_type_id}` can satisfy this."
                                        ))
                                        .with_help("Check that the correct variable is being passed, or update the assertion type."),
                                    );
                                }
                            }
                            Assertion::IsIdentical(_) => {
                                let intersection = if let Some(intersection) =
                                    intersect_union_with_union(context, asserted_type, &resolved_assertion_type)
                                {
                                    intersection
                                } else {
                                    let asserted_type_id = asserted_type.get_id();
                                    let expected_type_id = resolved_assertion_type.get_id();

                                    context.collector.report_with_code(
                                        IssueCode::ImpossibleTypeComparison,
                                        Issue::error(format!(
                                            "Impossible type assertion: `{assertion_variable}` of type `{asserted_type_id}` can never be identical to `{expected_type_id}`."
                                        ))
                                        .with_annotation(
                                            Annotation::primary(invocation.span)
                                                .with_message(format!("Argument `{assertion_variable}` has type `{asserted_type_id}`")),
                                        )
                                        .with_note(format!(
                                            "The assertion expects `{assertion_variable}` to be identical to `{expected_type_id}`, but no value of type `{asserted_type_id}` can satisfy this."
                                        ))
                                        .with_help("Check that the correct variable is being passed, or update the assertion type."),
                                    );

                                    get_never()
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

                if has_resolved_types
                    && (!any_possible || always_redundant)
                    && let Some(asserted_type) = &asserted_type
                {
                    let asserted_type_id = asserted_type.get_id();
                    let expected_type_id = resolved_or_clause
                        .iter()
                        .filter_map(|a| a.get_type().map(|t| t.get_id().to_string()))
                        .collect::<Vec<_>>()
                        .join("|");

                    if all_negated || always_redundant {
                        context.collector.report_with_code(
                            IssueCode::RedundantTypeComparison,
                            Issue::warning(format!(
                                "Redundant type assertion: `{assertion_variable}` of type `{asserted_type_id}` is always not `{expected_type_id}`."
                            ))
                            .with_annotation(
                                Annotation::primary(invocation.span)
                                    .with_message(format!("Argument `{assertion_variable}` has type `{asserted_type_id}`")),
                            )
                            .with_note(format!(
                                "The assertion expects `{assertion_variable}` to not be `{expected_type_id}`, which is always true."
                            ))
                            .with_help("Consider removing this assertion as it has no effect."),
                        );
                    } else {
                        context.collector.report_with_code(
                            IssueCode::ImpossibleTypeComparison,
                            Issue::error(format!(
                                "Impossible type assertion: `{assertion_variable}` of type `{asserted_type_id}` can never be `{expected_type_id}`."
                            ))
                            .with_annotation(
                                Annotation::primary(invocation.span)
                                    .with_message(format!("Argument `{assertion_variable}` has type `{asserted_type_id}`")),
                            )
                            .with_note(format!(
                                "The assertion expects `{assertion_variable}` to be `{expected_type_id}`, but no value of type `{asserted_type_id}` can satisfy this."
                            ))
                            .with_help("Check that the correct variable is being passed, or update the assertion type."),
                        );
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
                            &context.settings.algebra_thresholds(),
                            context.settings.formula_size_threshold,
                        ),
                        Assertion::IsNotType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(true) })))
                        | Assertion::IsType(TAtomic::Scalar(TScalar::Bool(TBool { value: Some(false) })))
                        | Assertion::Falsy => get_formula(
                            assertion_expression.span(),
                            assertion_expression.span(),
                            assertion_expression,
                            context.get_assertion_context_from_block(block_context),
                            artifacts,
                            &context.settings.algebra_thresholds(),
                            context.settings.formula_size_threshold,
                        )
                        .map(|clauses| {
                            negate_or_synthesize(
                                clauses,
                                assertion_expression,
                                context.get_assertion_context_from_block(block_context),
                                artifacts,
                                &context.settings.algebra_thresholds(),
                                context.settings.formula_size_threshold,
                            )
                        }),

                        _ => {
                            continue; // Unsupported assertion kind for expression
                        }
                    };

                    let new_clauses = clauses.unwrap_or_default();

                    for clause in &new_clauses {
                        block_context.clauses.push(Rc::new(clause.clone()));
                    }

                    let clauses = saturate_clauses(
                        block_context.clauses.iter().map(Rc::as_ref),
                        &context.settings.algebra_thresholds(),
                    );

                    let (truths, _) = find_satisfying_assignments(&clauses, None, &mut Default::default());
                    for (variable, assertions) in truths {
                        type_assertions.entry(variable).or_default().extend(assertions);
                    }
                }
            }
        }
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
/// A tuple `(Option<&'a Expression>, Option<Atom>)`.
/// * If a special target is resolved, the tuple is `(None, Some(resolved_id))`.
/// * If a regular argument is found, it returns the result from `get_argument_for_parameter`.
/// * If nothing is found, it returns `(None, None)`.
fn resolve_argument_or_special_target<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    parameter_name: Atom,
    this_variable: Option<&str>,
) -> (Option<&'ast Expression<'arena>>, Option<Atom>) {
    // First, check if the name refers to a special assertion target like `$this->...`
    if let Some(resolved_id) = resolve_special_assertion_target(block_context, parameter_name, this_variable) {
        return (None, Some(resolved_id));
    }

    // If not a special target, treat it as a regular parameter and find its argument.
    get_argument_for_parameter(context, block_context, invocation, None, Some(parameter_name))
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
/// * `Some(Atom)`: If the target is a special `$this` or `self` reference, containing the resolved variable ID.
/// * `None`: If the target is not a special reference and should be treated as a regular parameter.
fn resolve_special_assertion_target(
    block_context: &BlockContext<'_>,
    target_name: Atom,
    this_variable: Option<&str>,
) -> Option<Atom> {
    if let Some(this_variable) = this_variable
        && target_name.starts_with("$this")
    {
        return Some(Atom::from(&target_name.replacen("$this", this_variable, 1)));
    }

    if let Some(class) = block_context.scope.get_class_like_name()
        && target_name.starts_with("self::")
    {
        return Some(Atom::from(&target_name.replacen("self::", class.as_str(), 1)));
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
/// * `Option<Atom>`: The unique ID of the argument expression (e.g., a variable name), if it can be determined.
fn get_argument_for_parameter<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    mut parameter_offset: Option<usize>,
    mut parameter_name: Option<Atom>,
) -> (Option<&'ast Expression<'arena>>, Option<Atom>) {
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

    let Some(argument) = argument else {
        // The corresponding argument could not be found.
        return (None, None);
    };

    let Some(argument_expression) = argument.value() else {
        // The argument is a placeholder, no expression to analyze
        return (None, None);
    };

    // If an argument was found, resolve its expression ID.
    let argument_id = get_expression_id(
        argument_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let argument_id = match argument_id {
        Some(id) => Some(id),
        None => {
            if let Expression::Binary(binary) = unwrap_expression(argument_expression)
                && matches!(binary.operator, BinaryOperator::NullCoalesce(_))
                && matches!(unwrap_expression(binary.rhs), Expression::Literal(Literal::Null(_)))
            {
                get_expression_id(
                    binary.lhs,
                    block_context.scope.get_class_like_name(),
                    context.resolved_names,
                    Some(context.codebase),
                )
            } else {
                None
            }
        }
    };

    (Some(argument_expression), argument_id)
}

fn collect_plugin_throw_types<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    identifier: &FunctionLikeIdentifier,
) {
    let exceptions = match identifier {
        FunctionLikeIdentifier::Function(name) => context.plugin_registry.get_function_thrown_exceptions(
            context.codebase,
            block_context,
            artifacts,
            name,
            invocation,
        ),
        FunctionLikeIdentifier::Method(class_name, method_name) => {
            context.plugin_registry.get_method_thrown_exceptions(
                context.codebase,
                block_context,
                artifacts,
                class_name,
                method_name,
                invocation,
            )
        }
        FunctionLikeIdentifier::Closure(_, _) => return,
    };

    for exception in exceptions {
        block_context.possibly_thrown_exceptions.entry(exception).or_default().insert(invocation.span);
    }
}

fn apply_plugin_assertions<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    invocation: &Invocation<'ctx, '_, 'arena>,
    identifier: &FunctionLikeIdentifier,
    this_variable: Option<&str>,
    template_result: &TemplateResult,
    parameters: &AtomMap<TUnion>,
    range: (u32, u32),
) {
    let Some(assertions) = context.plugin_registry.get_function_like_assertions(
        context.codebase,
        block_context,
        artifacts,
        identifier,
        invocation,
    ) else {
        return;
    };

    let resolved_if_true_assertions = resolve_invocation_assertion(
        context,
        block_context,
        artifacts,
        invocation,
        this_variable,
        &assertions.if_true,
        template_result,
        parameters,
    );

    for (variable, assertion_set) in resolved_if_true_assertions {
        artifacts.if_true_assertions.entry(range).or_default().entry(variable).or_default().extend(assertion_set);
    }

    let resolved_if_false_assertions = resolve_invocation_assertion(
        context,
        block_context,
        artifacts,
        invocation,
        this_variable,
        &assertions.if_false,
        template_result,
        parameters,
    );

    for (variable, assertion_set) in resolved_if_false_assertions {
        artifacts.if_false_assertions.entry(range).or_default().entry(variable).or_default().extend(assertion_set);
    }

    apply_assertion_to_call_context(
        context,
        block_context,
        artifacts,
        invocation,
        this_variable,
        &assertions.type_assertions,
        template_result,
        parameters,
    );
}
