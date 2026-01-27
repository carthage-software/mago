use std::rc::Rc;

use mago_atom::Atom;
use mago_atom::concat_atom;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::property::PropertyMetadata;
use mago_codex::metadata::property_hook::PropertyHookMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_null;
use mago_codex::ttype::get_void;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Return;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::scope::control_action::ControlAction;
use crate::error::AnalysisError;
use crate::utils::docblock::check_docblock_type_incompatibility;
use crate::utils::docblock::get_type_from_var_docblock;
use crate::utils::get_type_diff;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Return<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let inferred_return_type = if let Some(return_value) = self.value.as_ref() {
            block_context.flags.set_inside_return(true);
            return_value.analyze(context, block_context, artifacts)?;
            block_context.flags.set_inside_return(false);

            let inferred_return_type = artifacts.get_rc_expression_type(&return_value).cloned();

            if let Some(inferred_return_type) = &inferred_return_type
                && inferred_return_type.is_never()
            {
                context.collector.report_with_code(
                    IssueCode::NeverReturn,
                    Issue::error("Cannot return value with type 'never' from this function.")
                    .with_annotation(
                        Annotation::primary(return_value.span())
                            .with_message("This expression has type 'never'.")
                    )
                    .with_annotation(
                        Annotation::secondary(self.span())
                            .with_message("This return statement is effectively unreachable.")
                    )
                    .with_note(
                        "A 'never' return type indicates that a function is guaranteed to exit the script, throw an exception, or loop indefinitely. Code following a call to such a function is unreachable."
                    )
                    .with_help(
                        "Since the preceding expression never returns, this 'return' statement cannot be reached. You can likely remove the 'return' keyword entirely."
                    ),
                );
            }

            match (inferred_return_type, get_type_from_var_docblock(context, block_context, artifacts, None, true)) {
                (Some(inferred_type), Some((docblock_type, docblock_type_span))) => {
                    check_docblock_type_incompatibility(
                        context,
                        None,
                        return_value.span(),
                        &inferred_type,
                        &docblock_type,
                        docblock_type_span,
                        None,
                    );

                    Rc::new(docblock_type)
                }
                (None, Some((docblock_type, _))) => Rc::new(docblock_type),
                (Some(inferred_type), None) => inferred_type,
                (None, None) => Rc::new(get_mixed()),
            }
        } else {
            Rc::new(get_void())
        };

        handle_return_value(context, block_context, artifacts, self.value.as_ref(), inferred_return_type, self.span());

        Ok(())
    }
}

pub fn handle_return_value<'ctx>(
    context: &mut Context<'ctx, '_>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    return_value: Option<&Expression>,
    mut inferred_return_type: Rc<TUnion>,
    return_span: Span,
) {
    if inferred_return_type.is_void() {
        inferred_return_type = Rc::new(get_null());
    }

    if let Some(finally_scope) = block_context.finally_scope.clone() {
        let mut finally_scope = (*finally_scope).borrow_mut();
        for (var_id, var_type) in &block_context.locals {
            if let Some(finally_type) = finally_scope.locals.get_mut(var_id) {
                *finally_type = Rc::new(combine_union_types(finally_type, var_type, context.codebase, false));
            } else {
                finally_scope.locals.insert(*var_id, var_type.clone());
            }
        }
    }

    block_context.flags.set_has_returned(true);
    block_context.control_actions.insert(ControlAction::Return);

    // Check for uninitialized properties when returning from a constructor
    if block_context.flags.collect_initializations()
        && let Some(FunctionLikeIdentifier::Method(class_name, method_name)) =
            block_context.scope.get_function_like_identifier()
        && method_name.eq_ignore_ascii_case("__construct")
    {
        check_constructor_early_return(context, block_context, return_span, class_name);
    }

    let mut expansion_options = TypeExpansionOptions {
        self_class: block_context.scope.get_class_like_name(),
        static_class_type: if let Some(calling_class) = block_context.scope.get_class_like_name() {
            StaticClassType::Name(calling_class)
        } else {
            StaticClassType::None
        },
        ..Default::default()
    };

    // Check if we're in a property hook context
    if let Some((property_name, hook_metadata)) = block_context.scope.get_property_hook() {
        handle_property_hook_return(
            context,
            block_context,
            return_value,
            inferred_return_type,
            property_name,
            hook_metadata,
            &expansion_options,
        );

        return;
    }

    let (Some(function_like_metadata), Some(function_like_identifier)) =
        (block_context.scope.get_function_like(), block_context.scope.get_function_like_identifier())
    else {
        // Global return, no function context, exiting.
        return;
    };

    expansion_options.function_is_final = if let Some(method_metadata) = &function_like_metadata.method_metadata {
        method_metadata.is_final
    } else {
        false
    };

    if inferred_return_type.is_expandable() {
        let mut inner_union = (*inferred_return_type).clone();

        expand_union(context.codebase, &mut inner_union, &expansion_options);

        inferred_return_type = Rc::new(inner_union);
    }

    let function_name = function_like_identifier.as_string();

    if let Some(return_value) = return_value
        && function_like_metadata.flags.is_by_reference()
    {
        let is_referenceable = return_value.is_referenceable(false)
            || (return_value.is_referenceable(true) && inferred_return_type.by_reference());

        if !is_referenceable {
            context.collector.report_with_code(
                IssueCode::InvalidReturnStatement,
                Issue::error(format!(
                    "Cannot return a non-referenceable value from function `{function_name}`.",
                ))
                .with_annotation(Annotation::primary(return_value.span()).with_message(
                    "This value cannot be returned by reference.",
                ))
                .with_annotation(
                    Annotation::secondary(function_like_metadata.name_span.unwrap_or(function_like_metadata.span))
                        .with_message("Function is declared to return by reference here."),
                )
                .with_note(
                    "You can only return variables, properties, array elements, or the result of another function call that itself returns a reference."
                )
                .with_help(
                    "To fix this, either return a valid reference or remove the `&` from the function declaration to return by value."
                ),
            );
        }
    }

    let require_return_value;
    let mut expected_return_type =
        if let Some(expected_return_type) = function_like_metadata.return_type_metadata.as_ref() {
            let mut expected_type = expected_return_type.type_union.clone();

            expand_union(
                context.codebase,
                &mut expected_type,
                &TypeExpansionOptions {
                    self_class: block_context.scope.get_class_like_name(),
                    static_class_type: if let Some(calling_class) = block_context.scope.get_class_like_name() {
                        StaticClassType::Name(calling_class)
                    } else {
                        StaticClassType::None
                    },
                    function_is_final: if let Some(method_metadata) = &function_like_metadata.method_metadata {
                        method_metadata.is_final
                    } else {
                        false
                    },
                    ..Default::default()
                },
            );

            if function_like_metadata.return_type_declaration_metadata.is_some() {
                require_return_value = !expected_type.is_void();
            } else {
                require_return_value = !expected_type.has_nullish();
            }

            expected_type
        } else {
            require_return_value = false;

            get_mixed()
        };

    if function_like_metadata.flags.has_yield() {
        if let Some((return_type, is_from_generator)) = get_generator_return_type(context, &expected_return_type) {
            if !is_from_generator {
                let inferred_return_type_str = inferred_return_type.get_id();
                let expected_return_type_str = expected_return_type.get_id();

                let type_declaration_span = function_like_metadata
                    .return_type_metadata
                    .as_ref()
                    .map_or_else(|| function_like_metadata.span, |signature| signature.span);

                if let Some(return_value) = return_value {
                    context.collector.report_with_code(
                        IssueCode::HiddenGeneratorReturn,
                        Issue::warning(format!(
                            "The value returned by generator function `{function_name}` may be inaccessible to callers.",
                        ))
                        .with_annotation(
                            Annotation::primary(return_value.span()).with_message(format!(
                                "This return statement provides a final value of type `{inferred_return_type_str}` for the generator.",
                            )),
                        )
                        .with_annotation(
                            Annotation::secondary(type_declaration_span).with_message(format!(
                                "Function is declared to return `{expected_return_type_str}`, which hides `Generator::getReturn()`."
                            ))
                        )
                        .with_note("Generators can provide a final value via `Generator::getReturn()`.")
                        .with_note(format!("However, the type hint `{expected_return_type_str}` prevents callers from safely accessing this method."))
                        .with_note(format!("Thus, this specific returned value (type `{inferred_return_type_str}`) is effectively inaccessible."))
                        .with_help("Change return hint to `Generator<..., R>` or remove `return <value>` if unused."),
                    );
                }
            }

            expected_return_type = return_type;
        } else {
            // ignore this, it will be handled by the `yield` analyzer
        }
    } else if return_value.is_some() {
        artifacts.inferred_return_types.push(inferred_return_type.clone());
    }

    if let Some(return_value) = return_value {
        if expected_return_type.is_mixed() {
            return;
        }

        if expected_return_type.is_void() {
            context.collector.report_with_code(
                IssueCode::InvalidReturnStatement,
                Issue::error(format!(
                    "Function `{function_name}` is declared to return 'void' but returns a value."
                ))
                .with_annotation(
                    Annotation::primary(return_value.span())
                        .with_message("Value returned here.")
                )
                .with_note(
                    "A 'void' return type means the function should not return any value."
                )
                .with_note(
                    "Use 'return;' without a value, or omit the return statement if it's at the end of the function."
                )
                .with_help(
                    "Remove the return value (e.g., change 'return $value;' to 'return;') or change the function's declared return type if it's intended to return a value."
                ),
            );

            return;
        }

        if inferred_return_type.is_mixed() {
            context.collector.report_with_code(
                IssueCode::MixedReturnStatement,
                Issue::error(format!(
                    "Could not infer a precise return type for function `{}`. Saw type `{}`.",
                    function_name,
                    inferred_return_type.get_id()
                ))
                .with_annotation(
                    Annotation::primary(return_value.span())
                        .with_message("Type inferred as `mixed` here.")
                )
                .with_note(
                    "The analysis could not determine a specific type for the value returned here, resulting in `mixed`. This can happen with complex code paths or unannotated data.".to_string()
                )
                .with_help(
                    "Add specific type hints to variables, parameters, or properties involved in calculating the return value. Consider adding a specific return type declaration to the function signature to catch potential mismatches earlier."
                ),
            );

            return;
        }

        let mut union_comparison_result = ComparisonResult::new();

        let is_contained_by = is_contained_by(
            context.codebase,
            &inferred_return_type,
            &expected_return_type,
            inferred_return_type.ignore_nullable_issues(),
            inferred_return_type.ignore_falsable_issues(),
            false,
            &mut union_comparison_result,
        );

        if is_contained_by {
            return;
        }

        let expected_return_type_str = expected_return_type.get_id();
        let inferred_return_type_str = inferred_return_type.get_id();

        if inferred_return_type.is_nullable()
            && !inferred_return_type.ignore_nullable_issues()
            && !expected_return_type.is_nullable()
            && !expected_return_type.has_template()
        {
            context.collector.report_with_code(
                IssueCode::NullableReturnStatement,
                Issue::error(format!(
                    "Function `{function_name}` is declared to return `{expected_return_type_str}` but possibly returns a nullable value (inferred as `{inferred_return_type_str}`).",
                ))
                .with_annotation(
                    Annotation::primary(return_value.span()).with_message("Nullable value returned here.")
                )
                .with_annotation(
                    Annotation::secondary(function_like_metadata.span)
                        .with_message(format!("Return type declared as non-nullable `{expected_return_type_str}` here."))
                )
                .with_note(
                    "The declared return type does not permit null, but the analysis indicates that 'null' or a nullable type could be returned from this path.".to_string()
                )
                .with_help(
                    format!(
                        "You can either change the return type declaration of `{function_name}` to be nullable (e.g., '?{expected_return_type_str}'), or ensure that this function path always returns a non-null value."
                    )
                ),
            );
        }

        if inferred_return_type.is_falsable()
            && !inferred_return_type.ignore_falsable_issues()
            && !expected_return_type.is_falsable()
            && !expected_return_type.has_template()
        {
            context.collector.report_with_code(
                IssueCode::FalsableReturnStatement,
                Issue::error(format!(
                    "Function `{function_name}` is declared to return `{expected_return_type_str}` but possibly returns 'false' (inferred as `{inferred_return_type_str}`).",
                ))
                .with_annotation(
                    Annotation::primary(return_value.span())
                        .with_message("Potentially 'false' returned here.")
                )
                .with_annotation(
                    Annotation::secondary(function_like_metadata.span)
                        .with_message(format!("Return type declared as non-falsable `{expected_return_type_str}` here")))
                .with_note(
                    "The declared return type does not permit 'false', but the analysis indicates that 'false' or a falsable type could be returned from this path."
                )
                .with_help(
                    format!(
                        "You can either change the return type declaration of `{function_name}` to include 'false' (e.g., '{expected_return_type_str}|false'), or ensure that this function path never returns 'false'.",
                    )
                ),
            );
        }

        if union_comparison_result.type_coerced.unwrap_or(false) {
            if union_comparison_result.type_coerced_from_as_mixed.unwrap_or(false) {
                return;
            }

            if union_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                let mut issue = Issue::error(format!(
                    "Returned type `{inferred_return_type_str}` is less specific than the declared return type `{expected_return_type_str}` for function `{function_name}` due to nested 'mixed'."
                ))
                .with_annotation(
                    Annotation::primary(return_value.span())
                        .with_message("Returned value's type is too general here due to nested mixed")
                )
                .with_note(
                    "The analysis detected 'mixed' within the structure of the returned value, making the overall type less specific than what the function declared."
                )
                .with_help(
                    format!(
                        "Ensure the structure returned by `{function_name}` strictly adheres to the types specified in the `{expected_return_type_str}` return type declaration."
                    )
                );

                if let Some(type_diff) = get_type_diff(context, &expected_return_type, &inferred_return_type) {
                    issue = issue.with_note(type_diff);
                }

                context.collector.report_with_code(IssueCode::LessSpecificNestedReturnStatement, issue);
            } else {
                let mut issue = Issue::error(format!(
                    "Returned type `{inferred_return_type_str}` is less specific than the declared return type `{expected_return_type_str}` for function `{function_name}`."
                ))
                .with_annotation(
                    Annotation::primary(return_value.span())
                        .with_message("Returned type is too general.")
                )
                .with_note(
                    format!(
                        "The inferred type `{inferred_return_type_str}` could be assigned to the declared type `{expected_return_type_str}`, but is wider (less specific)."
                    )
                )
                .with_help(
                    format!(
                        "Consider returning a value that more precisely matches the declared `{expected_return_type_str}` type, or adjust the function's return type declaration if the broader type is intended."
                    )
               );

                if let Some(type_diff) = get_type_diff(context, &expected_return_type, &inferred_return_type) {
                    issue = issue.with_note(type_diff);
                }

                context.collector.report_with_code(IssueCode::LessSpecificReturnStatement, issue);
            }
        } else {
            let mut issue = Issue::error(format!(
                "Invalid return type for function `{function_name}`: expected `{expected_return_type_str}`, but found `{inferred_return_type_str}`."
            ))
            .with_annotation(
                Annotation::primary(return_value.span())
                    .with_message(format!("This has type `{inferred_return_type_str}`"))
            )
            .with_note(
                format!(
                    "The type `{inferred_return_type_str}` returned here is not compatible with the declared return type `{expected_return_type_str}`."
                )
            )
            .with_help(
                format!(
                    "Change the return value to match `{expected_return_type_str}`, or update the function's return type declaration."
                )
            );

            if let Some(type_diff) = get_type_diff(context, &expected_return_type, &inferred_return_type) {
                issue = issue.with_note(type_diff);
            }

            context.collector.report_with_code(IssueCode::InvalidReturnStatement, issue);
        }
    } else if require_return_value
        && !function_like_metadata.flags.has_yield()
        && !matches!(
            block_context.scope.get_function_like_identifier(),
            Some(FunctionLikeIdentifier::Method(_, name))
            if name.eq_ignore_ascii_case("__construct")
        )
    {
        let expected_return_type_str = expected_return_type.get_id();

        context.collector.report_with_code(
            IssueCode::InvalidReturnStatement,
            Issue::error(format!(
                "Function `{function_name}` is declared to return `{expected_return_type_str}` but no return value was specified.",
            ))
            .with_annotation(Annotation::primary(return_span).with_message("No return value specified here."))
            .with_annotation(
                Annotation::secondary(function_like_metadata.span)
                    .with_message(format!("Return type declared as `{expected_return_type_str}` here."))
            )
            .with_note(
                "The declared return type does not permit 'void', but the analysis indicates that this function path does not return a value.".to_string()
            )
            .with_help(
                format!(
                    "You can either change the return type declaration of `{function_name}` to be 'void', or ensure that this function path always returns a value."
                )
            ),
        );
    }
}

fn handle_property_hook_return<'ctx>(
    context: &mut Context<'ctx, '_>,
    block_context: &BlockContext<'ctx>,
    return_value: Option<&Expression>,
    mut inferred_return_type: Rc<TUnion>,
    property_name: Atom,
    hook_metadata: &PropertyHookMetadata,
    expansion_options: &TypeExpansionOptions,
) {
    if !hook_metadata.is_get() {
        return;
    }

    let Some(class_like) = block_context.scope.get_class_like() else {
        return;
    };
    let Some(property) = class_like.properties.get(&property_name) else { return };
    let Some(type_metadata) = &property.type_metadata else { return };

    let mut expected_return_type = type_metadata.type_union.clone();
    expand_union(context.codebase, &mut expected_return_type, expansion_options);

    if inferred_return_type.is_expandable() {
        let mut inner = (*inferred_return_type).clone();
        expand_union(context.codebase, &mut inner, expansion_options);
        inferred_return_type = Rc::new(inner);
    }

    if expected_return_type.is_mixed() {
        return;
    }

    let Some(return_value) = return_value else { return };

    if expected_return_type.is_void() {
        return;
    }

    let hook_name = concat_atom!(class_like.name, "::", property_name, "::get");

    if inferred_return_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::MixedReturnStatement,
            Issue::error(format!(
                "Could not infer a precise return type for property hook `{hook_name}`. Saw type `{}`.",
                inferred_return_type.get_id()
            ))
            .with_annotation(Annotation::primary(return_value.span()).with_message("Type inferred as `mixed` here."))
            .with_note("The analysis could not determine a specific type for the value returned here.")
            .with_help("Add specific type hints to variables or properties involved in calculating the return value."),
        );
        return;
    }

    let mut comparison_result = ComparisonResult::new();
    if is_contained_by(
        context.codebase,
        &inferred_return_type,
        &expected_return_type,
        inferred_return_type.ignore_nullable_issues(),
        inferred_return_type.ignore_falsable_issues(),
        false,
        &mut comparison_result,
    ) {
        return;
    }

    let expected_str = expected_return_type.get_id();
    let inferred_str = inferred_return_type.get_id();

    if inferred_return_type.is_nullable()
        && !inferred_return_type.ignore_nullable_issues()
        && !expected_return_type.is_nullable()
        && !expected_return_type.has_template()
    {
        context.collector.report_with_code(
            IssueCode::NullableReturnStatement,
            Issue::error(format!(
                "Property hook `{hook_name}` returns nullable value `{inferred_str}` but property type is `{expected_str}`.",
            ))
            .with_annotation(Annotation::primary(return_value.span()).with_message("Nullable value returned here."))
            .with_note("The property type does not permit null, but this expression could return null.")
            .with_help(format!(
                "Ensure the hook always returns a non-null value, or change the property type to `?{expected_str}`."
            )),
        );
        return;
    }

    if inferred_return_type.is_falsable()
        && !inferred_return_type.ignore_falsable_issues()
        && !expected_return_type.is_falsable()
        && !expected_return_type.has_template()
    {
        context.collector.report_with_code(
            IssueCode::FalsableReturnStatement,
            Issue::error(format!(
                "Property hook `{hook_name}` returns falsable value `{inferred_str}` but property type is `{expected_str}`.",
            ))
            .with_annotation(Annotation::primary(return_value.span()).with_message("Potentially 'false' returned here."))
            .with_note("The property type does not permit false, but this expression could return false.")
            .with_help(format!("Ensure the hook never returns false, or change the property type to `{expected_str}|false`.")),
        );
        return;
    }

    context.collector.report_with_code(
        IssueCode::InvalidReturnStatement,
        Issue::error(format!(
            "Property hook `{hook_name}` returns `{inferred_str}` but property is typed as `{expected_str}`.",
        ))
        .with_annotation(
            Annotation::primary(return_value.span()).with_message(format!("Expression has type `{inferred_str}`.")),
        )
        .with_note(format!("The get hook must return a value compatible with the property type `{expected_str}`."))
        .with_help("Change the returned expression to match the property type."),
    );
}

/// Check for uninitialized properties when returning early from a constructor.
fn check_constructor_early_return<'ctx>(
    context: &mut Context<'ctx, '_>,
    block_context: &BlockContext<'ctx>,
    return_span: Span,
    class_name: Atom,
) {
    let Some(class_like_metadata) = context.codebase.get_class_like(&class_name) else {
        return;
    };

    // Skip abstract classes, interfaces, traits, enums
    if class_like_metadata.flags.is_abstract()
        || class_like_metadata.kind.is_interface()
        || class_like_metadata.kind.is_trait()
        || class_like_metadata.kind.is_enum()
    {
        return;
    }

    // Check each property that needs initialization
    for (prop_name, declaring_class) in &class_like_metadata.declaring_property_ids {
        // Get the property metadata from the declaring class
        let Some(declaring_meta) = context.codebase.get_class_like(declaring_class) else {
            continue;
        };
        let Some(prop) = declaring_meta.properties.get(prop_name) else {
            continue;
        };

        // Check if this property requires initialization
        if !property_requires_constructor_initialization(prop, class_like_metadata, declaring_meta) {
            continue;
        }

        // Check if property is initialized at this point
        if block_context.definitely_initialized_properties.contains(prop_name) {
            continue;
        }

        // Property not initialized - report error
        let mut issue = Issue::error(format!(
            "Property `{prop_name}` may not be initialized when returning from constructor of class `{}`.",
            class_like_metadata.original_name
        ))
        .with_annotation(
            Annotation::primary(return_span).with_message(format!("Returning without initializing `{prop_name}`")),
        );

        // Add secondary annotation if we have a span for the property
        if let Some(prop_span) = prop.name_span.or(prop.span) {
            issue = issue.with_annotation(Annotation::secondary(prop_span).with_message("Property declared here"));
        }

        issue = issue
            .with_note(
                "Typed properties without default values must be initialized in the constructor before returning.",
            )
            .with_help(format!("Initialize `{prop_name}` before this return statement, or provide a default value."));

        context.collector.report_with_code(IssueCode::UninitializedProperty, issue);
    }
}

/// Determines if a property requires initialization in the constructor.
/// This is a simplified version of the check in initialization.rs, used for early return detection.
fn property_requires_constructor_initialization(
    property: &PropertyMetadata,
    class_like_metadata: &ClassLikeMetadata,
    declaring_class_metadata: &ClassLikeMetadata,
) -> bool {
    // Has default value - doesn't need initialization
    if property.flags.has_default() {
        return false;
    }

    // Promoted property - initialized via constructor parameter
    if property.flags.is_promoted_property() {
        return false;
    }

    // Static property - not initialized in constructor
    if property.flags.is_static() {
        return false;
    }

    // Virtual/hooked property - may not need backing storage
    if property.flags.is_virtual_property() {
        return false;
    }

    // No type declaration - PHP doesn't require initialization
    if property.type_declaration_metadata.is_none() {
        return false;
    }

    // If declared in a different class (inherited)
    if declaring_class_metadata.name != class_like_metadata.name {
        // If declaring class is a concrete class (not abstract and not a trait), it handles initialization.
        // Traits and abstract classes don't handle initialization - the using class must do it.
        if !declaring_class_metadata.flags.is_abstract() && !declaring_class_metadata.kind.is_trait() {
            return false;
        }
    }

    true
}

fn get_generator_return_type(context: &Context, return_type: &TUnion) -> Option<(TUnion, bool)> {
    let mut generator_return = None;
    let mut could_be_array_or_traversable = false;
    for atomic in return_type.types.iter() {
        if !atomic.could_be_array_or_traversable(context.codebase) {
            continue;
        }

        could_be_array_or_traversable = true;
        let Some(generator_parameters) = atomic.get_generator_parameters() else {
            continue;
        };

        match generator_return.as_mut() {
            Some(existing_return) => {
                *existing_return =
                    combine_union_types(existing_return, &generator_parameters.3, context.codebase, false);
            }
            None => {
                generator_return = Some(generator_parameters.3);
            }
        }
    }

    if !could_be_array_or_traversable {
        return None;
    }

    match generator_return {
        Some(generator_return) => Some((generator_return, true)),
        None => Some((get_mixed(), false)),
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::test_analysis;

    test_analysis! {
        name = empty_return_from_generator,
        code = indoc! {r"
            <?php

            /**
             * @param array<string, string> $array
             * @return iterable<string>
             */
            function generator(array $array): iterable
            {
                yield $array['key'] ?? 'default';
                return;
            }
        "}
    }

    test_analysis! {
        name = hidden_return_from_generator,
        code = indoc! {r"
            <?php

            /**
             * @param array<string, string> $array
             * @return iterable<string>
             */
            function generator(array $array): iterable
            {
                yield $array['key'] ?? 'default';

                return 123;
            }
        "},
        issues = [
            IssueCode::HiddenGeneratorReturn,
        ]
    }

    test_analysis! {
        name = return_from_generator,
        code = indoc! {r"
            <?php

            /**
             * @template K
             * @template V
             */
            interface Traversable {}

            /**
             * stub for `Generator`
             *
             * @template K
             * @template V
             * @template S
             * @template R
             *
             * @implements Traversable<K, V>
             */
            class Generator implements Traversable {}

            /**
             * @param array<non-empty-string, non-empty-string> $array
             * @return Generator<'k', non-empty-string, void, 'final value'>
             */
            function generator(array $array): iterable
            {
                yield 'k' => $array['key'] ?? 'default';

                return 'final value';
            }
        "},
        issues = [
            // Traversable: K and V not used in interface body
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
            // Generator: S and R not used
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
        ]
    }

    test_analysis! {
        name = invalid_return_from_generator,
        code = indoc! {r"
            <?php

            /**
             * stub for `Generator`
             *
             * @template TKey
             * @template TValue
             * @template TSend
             * @template TReturn
             */
            class Generator {}

            /**
             * @param array<non-empty-string, non-empty-string> $array
             * @return Generator<'k', non-empty-string, void, 'final value'>
             */
            function generator(array $array): iterable
            {
                yield 'k' => $array['key'] ?? 'default';

                return 'final value 2';
            }
        "},
        issues = [
            IssueCode::InvalidReturnStatement,
            // Generator stub: all 4 template params unused
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
            IssueCode::UnusedTemplateParameter,
        ]
    }

    test_analysis! {
        name = key_of_and_value_of,
        code = indoc! {r"
            <?php

            class A
            {
                /**
                 * @var array<1|2|3, 'a'|'b'|'c'>
                 */
                public const FOO = [1 => 'a', 2 => 'b', 3 => 'c'];
            }

            /**
             * @return value-of<A::FOO>
             */
            function get_val(): string
            {
                return A::FOO[1];
            }

            /**
             * @return key-of<A::FOO>
             */
            function get_k(): int
            {
                return 1;
            }

            /**
             * @return list<list{key-of<A::FOO>, value-of<A::FOO>}>
             */
            function get_list(): array
            {
                return [
                    [1, 'a'],
                    [2, 'b'],
                    [3, 'c'],
                ];
            }

            /**
             * @return key-of<A::FOO>[]
             */
            function get_all_keys(): array
            {
                return [1, 2, 3];
            }

            /**
             * @return key-of<list<int>|array{a: int, b: int}>
             */
            function get_list_or_array_key(bool $as_int)
            {
                if ($as_int) {
                    return 42;
                }

                return 'a';
            }

            /**
             * @template T of array
             *
             * @param T $array
             *
             * @return key-of<T>|null
             */
            function get_a_key($array)
            {
                foreach ($array as $key => $_) {
                    return $key;
                }

                return null;
            }
        "},
    }

    test_analysis! {
        name = return_no_value_from_untyped_functions,
        code = indoc! {r"
            <?php

            function foo() {
                return;
            }
        "},
    }

    test_analysis! {
        name = return_class_string_array,
        code = indoc! {r"
            <?php

            class A {}
            class B extends A {}

            /**
             * @return array<class-string<A>, class-string<A>>
             */
            function example(): array {


                return [
                    A::class => B::class,
                ];
            }
        "},
    }

    test_analysis! {
        name = return_no_value_from_typed_void_functions,
        code = indoc! {r"
            <?php

            function foo(): void {
                return;
            }
        "},
    }

    test_analysis! {
        name = return_no_value_from_mixed_docblock_typed_functions,
        code = indoc! {r"
            <?php

            /**
             * @return mixed
             */
            function foo() {
                return;
            }
        "},
    }

    test_analysis! {
        name = return_no_value_from_null_docblock_typed_functions,
        code = indoc! {r"
            <?php

            /**
             * @return null
             */
            function foo() {
                return;
            }
        "},
    }

    test_analysis! {
        name = return_no_value_from_nullable_docblock_typed_functions,
        code = indoc! {r#"
            <?php

            /**
             * @return null|string
             */
            function foo() {
                if (foo() === "a") {
                    return;
                }

                return "a";
            }
        "#},
    }

    test_analysis! {
        name = expanding_this,
        code = indoc! {r"
            <?php

            /**
             * @template Tk of array-key
             * @template Tv
             */
            final class Map {
                /**
                 * @var array<Tk, Tv> $elements
                 */
                private array $elements;

                /**
                 * @param array<Tk, Tv> $elements
                 */
                public function __construct(array $elements = []) {
                    $this->elements = $elements;
                }

                /** @return array<Tk, Tv> */
                public function getElements(): array { return $this->elements; }

                /**
                 * @return Map<Tk, Tv>
                 */
                public function toStatic(): static {
                    return $this;
                }
            }
        "},
    }

    test_analysis! {
        name = complex_type_return,
        code = indoc! {r"
            <?php

            interface Foo {}

            interface Bar {}

            class Baz implements Foo, Bar {}

            class BarImpl implements Bar {}

            function get_bool(): bool {
                return false;
            }

            /**
             * @return string[][]|int|(Foo&Bar)|Bar[]|list{Bar, list<string>}
             */
            function foo(): mixed {
                if (get_bool()) {
                    return 42; // int
                }

                if (get_bool()) {
                    return new Baz(); // Foo&Bar
                }

                if (get_bool()) {
                    return [new BarImpl(), ['f']]; // list{Bar, list<string>}
                }

                if (get_bool()) {
                    return [new BarImpl()]; // Bar[]
                }

                return [
                    ['a', 'b', 'c'],
                    ['a', 'b', 'c'],
                    ['a', 'b', 'c'],
                    ['a', 'b', 'c'],
                ]; // string[][]
            }
        "},
    }

    test_analysis! {
        name = ignore_falsable_return,
        code = indoc! {r"
            <?php

            /** @ignore-falsable-return */
            function get_foo(): string|false
            {
                return 'foo';
            }

            function get_bar(): string
            {
                return get_foo();
            }

            function get_baz(): string
            {
                return get_foo() ?: 'baz';
            }
        "},
    }

    test_analysis! {
        name = ignore_nullable_return,
        code = indoc! {r"
            <?php

            /** @ignore-nullable-return */
            function get_foo(): string|null
            {
                return 'foo';
            }

            function get_bar(): string
            {
                return get_foo();
            }

            function get_baz(): string
            {
                return get_foo() ?? 'baz';
            }
        "},
    }

    test_analysis! {
        name = resolve_nested_generics_through_inheritance,
        code = indoc! {r"
            <?php

            declare(strict_types=1);

            #[Attribute(Attribute::TARGET_CLASS)]
            class Attribute
            {
            }

            #[Attribute(Attribute::TARGET_METHOD)]
            class Override
            {
            }

            /**
             * @template T
             */
            interface TypeInterface
            {
                /**
                 * @assert-if-true T $value
                 */
                public function matches(mixed $value): bool;
            }

            /**
             * @template R
             * @template L
             *
             * @implements TypeInterface<L|R>
             */
            readonly class UnionType implements TypeInterface
            {
                /**
                 * @param TypeInterface<L> $left
                 * @param TypeInterface<R> $right
                 *
                 * @mutation-free
                 */
                public function __construct(
                    private TypeInterface $left,
                    private TypeInterface $right,
                ) {}

                /**
                 * @assert-if-true R|L $value
                 */
                #[Override]
                public function matches(mixed $value): bool
                {
                    return $this->left->matches($value) || $this->right->matches($value);
                }
            }

            /**
             * @implements TypeInterface<string>
             */
            final readonly class StringType implements TypeInterface
            {
                /**
                 * @assert-if-true string $value
                 */
                #[Override]
                public function matches(mixed $value): bool
                {
                    return $this->matches($value);
                }
            }

            /**
             * @implements TypeInterface<bool>
             */
            final readonly class BoolType implements TypeInterface
            {
                /**
                 * @assert-if-true bool $value
                 */
                #[Override]
                public function matches(mixed $value): bool
                {
                    return $this->matches($value);
                }
            }

            /**
             * @implements TypeInterface<int>
             */
            final readonly class IntType implements TypeInterface
            {
                /**
                 * @assert-if-true int $value
                 */
                #[Override]
                public function matches(mixed $value): bool
                {
                    return $this->matches($value);
                }
            }

            /**
             * @implements TypeInterface<float>
             */
            final readonly class FloatType implements TypeInterface
            {
                /**
                 * @assert-if-true float $value
                 */
                #[Override]
                public function matches(mixed $value): bool
                {
                    return $this->matches($value);
                }
            }

            /**
             * @extends UnionType<string|bool, int|float>
             *
             * @internal
             */
            final readonly class ScalarType extends UnionType
            {
                /**
                 * @mutation-free
                 */
                public function __construct()
                {
                    parent::__construct(
                        new UnionType(new IntType(), new FloatType()),
                        new UnionType(new StringType(), new BoolType()),
                    );
                }
            }

            /**
             * @return TypeInterface<string|bool|int|float>
             */
            function scalar_type(): TypeInterface
            {
                return new ScalarType();
            }
        "},
        issues = [
            // TypeInterface: T is only used in @assert annotation, not in property/param/return
            IssueCode::UnusedTemplateParameter,
        ]
    }
}
