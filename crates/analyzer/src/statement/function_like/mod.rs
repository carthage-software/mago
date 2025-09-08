use std::rc::Rc;

use ahash::HashMap;

use mago_atom::concat_atom;
use mago_codex::get_class_like;
use mago_codex::get_function_like_thrown_types;
use mago_codex::get_interface;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::ttype::TypeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::TypeRef;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::reference::TReference;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraint;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::resolver::property::localize_property_type;
use crate::statement::analyze_statements;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;
use crate::statement::r#return::handle_return_value;
use crate::utils::expression::get_variable_id;

pub mod function;

#[derive(Debug, Clone, Copy)]
pub enum FunctionLikeBody<'ast, 'arena> {
    Statements(&'ast [Statement<'arena>], Span),
    Expression(&'ast Expression<'arena>),
}

impl HasSpan for FunctionLikeBody<'_, '_> {
    fn span(&self) -> Span {
        match self {
            FunctionLikeBody::Statements(_, span) => *span,
            FunctionLikeBody::Expression(expr) => expr.span(),
        }
    }
}

pub fn analyze_function_like<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    parent_artifacts: &mut AnalysisArtifacts,
    block_context: &mut BlockContext<'ctx>,
    function_like_metadata: &'ctx FunctionLikeMetadata,
    parameter_list: &'ast FunctionLikeParameterList<'arena>,
    body: FunctionLikeBody<'ast, 'arena>,
    inferred_parameter_types: Option<HashMap<usize, TUnion>>,
) -> Result<AnalysisArtifacts, AnalysisError> {
    let mut previous_type_resolution_context = std::mem::replace(
        &mut context.type_resolution_context,
        function_like_metadata.type_resolution_context.clone().unwrap_or_default(),
    );

    let mut artifacts = AnalysisArtifacts::new();

    add_parameter_types_to_context(
        context,
        block_context,
        &mut artifacts,
        function_like_metadata,
        parameter_list,
        inferred_parameter_types,
    )?;

    if !block_context.scope.is_static()
        && let Some(class_like_metadata) = block_context.scope.get_class_like()
    {
        block_context.locals.insert(
            "$this".to_string(),
            Rc::new(wrap_atomic(TAtomic::Object(get_this_type(context, class_like_metadata, function_like_metadata)))),
        );
    }

    if let FunctionLikeBody::Statements(statements, _) = body {
        for statement in statements {
            let Statement::Global(global) = statement else {
                if statement.is_noop() {
                    continue;
                } else {
                    break;
                }
            };

            for variable in global.variables.iter() {
                if let Some(var_id) = get_variable_id(variable) {
                    block_context.conditionally_referenced_variable_ids.insert(var_id.to_string());
                }
            }
        }
    }

    if let Some(calling_class) = block_context.scope.get_class_like_name()
        && let Some(class_like_metadata) = get_class_like(context.codebase, &calling_class)
    {
        add_properties_to_context(context, block_context, class_like_metadata, function_like_metadata)?;
    }

    if !function_like_metadata.flags.is_unchecked() {
        match body {
            FunctionLikeBody::Statements(statements, _) => {
                analyze_statements(statements, context, block_context, &mut artifacts)?;
            }
            FunctionLikeBody::Expression(value) => {
                block_context.inside_return = true;
                value.analyze(context, block_context, &mut artifacts)?;
                block_context.inside_return = false;
                block_context.conditionally_referenced_variable_ids = Default::default();

                let value_type =
                    artifacts.get_rc_expression_type(value).cloned().unwrap_or_else(|| Rc::new(get_mixed()));

                handle_return_value(context, block_context, &mut artifacts, Some(value), value_type, value.span())?;
            }
        }
    }

    if let Some(function_metadata) = block_context.scope.get_function_like()
        && !block_context.has_returned
        && let Some(return_type) = &function_metadata.return_type_metadata
        && !return_type.type_union.is_void()
        && !function_like_metadata.flags.has_yield()
    {
        let expanded_type =
            expand_type_metadata(context, block_context, &mut artifacts, function_like_metadata, return_type);
        let expected_return_type_id = expanded_type.get_id();

        let help_message = if expanded_type.is_nullable() {
            "Ensure all code paths end with a `return` statement. You may need to add `return null;` to the paths that currently don't return a value.".to_string()
        } else {
            format!(
                "Add a `return` statement that provides a value of type '{}' to all paths, or change the function's return type to '{}|null' and return `null` explicitly.",
                expected_return_type_id, expected_return_type_id
            )
        };

        context.collector.report_with_code(
            IssueCode::MissingReturnStatement,
            Issue::error(match function_metadata.name {
                Some(name) => format!("Missing return statement in function '{name}'"),
                None => "Missing return statement in closure".to_string(),
            })
            .with_annotation(
                Annotation::primary(function_metadata.name_span.unwrap_or(function_metadata.span))
                    .with_message(format!("This function is declared to return '{}'...", expected_return_type_id)),
            )
            .with_annotation(
                Annotation::secondary(body.span()).with_message("...but this path can exit without returning a value."),
            )
            .with_note("A function that does not explicitly return a value will implicitly return `null`.")
            .with_help(help_message),
        );
    }

    check_thrown_types(context, block_context, &mut artifacts, function_like_metadata)?;

    std::mem::swap(&mut context.type_resolution_context, &mut previous_type_resolution_context);
    parent_artifacts.expression_types.extend(std::mem::take(&mut artifacts.expression_types));

    Ok(artifacts)
}

fn add_parameter_types_to_context<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    function_like_metadata: &'ctx FunctionLikeMetadata,
    parameter_list: &FunctionLikeParameterList<'arena>,
    mut inferred_parameter_types: Option<HashMap<usize, TUnion>>,
) -> Result<(), AnalysisError> {
    for (i, parameter_metadata) in function_like_metadata.parameters.iter().enumerate() {
        let parameter_variable_str = parameter_metadata.get_name().0;

        let declared_parameter_type = if let Some(type_metadata) = parameter_metadata.get_type_metadata() {
            expand_type_metadata(context, block_context, artifacts, function_like_metadata, type_metadata)
        } else {
            get_mixed()
        };

        // Now, decide which type to use: the inferred one or the declared one.
        let mut final_parameter_type = if let Some(inferred_map) = inferred_parameter_types.as_mut() {
            // If an inferred type exists for this parameter index, take it.
            // Otherwise, fall back to the type we derived from the signature.
            inferred_map.remove(&i).unwrap_or(declared_parameter_type)
        } else {
            declared_parameter_type
        };

        if parameter_metadata.flags.is_by_reference() {
            final_parameter_type.by_reference = parameter_metadata.flags.is_by_reference();

            let constraint_type = parameter_metadata
                .out_type
                .as_ref()
                .map(|type_metadata| {
                    expand_type_metadata(context, block_context, artifacts, function_like_metadata, type_metadata)
                })
                .unwrap_or_else(|| final_parameter_type.clone());

            block_context.by_reference_constraints.insert(
                parameter_variable_str.to_string(),
                ReferenceConstraint::new(
                    parameter_metadata.span,
                    ReferenceConstraintSource::Parameter,
                    Some(constraint_type),
                ),
            );
        }

        let parameter_node = if let Some(parameter_node) = parameter_list.parameters.get(i) {
            parameter_node
        } else {
            continue;
        };

        analyze_attributes(
            context,
            block_context,
            artifacts,
            parameter_node.attribute_lists.as_slice(),
            if parameter_node.is_promoted_property() {
                AttributeTarget::PromotedProperty
            } else {
                AttributeTarget::Parameter
            },
        )?;

        if let Some(default_value) = parameter_node.default_value.as_ref() {
            default_value.value.analyze(context, block_context, artifacts)?;
        }

        let final_parameter_type = if parameter_metadata.flags.is_variadic() {
            wrap_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(final_parameter_type)))))
        } else {
            final_parameter_type
        };

        block_context.locals.insert(parameter_variable_str.to_string(), Rc::new(final_parameter_type));
    }

    Ok(())
}

fn expand_type_metadata<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    function_like_metadata: &FunctionLikeMetadata,
    type_metadata: &TypeMetadata,
) -> TUnion {
    add_symbol_references(
        &type_metadata.type_union,
        block_context.scope.get_function_like_identifier().as_ref(),
        artifacts,
    );

    let mut signature_union = type_metadata.type_union.clone();

    let calling_class = block_context.scope.get_class_like_name();

    expander::expand_union(
        context.codebase,
        &mut signature_union,
        &TypeExpansionOptions {
            self_class: calling_class,
            static_class_type: if let Some(calling_class) = calling_class {
                StaticClassType::Name(calling_class)
            } else {
                StaticClassType::None
            },
            evaluate_class_constants: true,
            evaluate_conditional_types: true,
            function_is_final: if let Some(method_metadata) = &function_like_metadata.method_metadata {
                method_metadata.is_final
            } else {
                false
            },
            expand_generic: true,
            expand_templates: true,
            ..Default::default()
        },
    );

    signature_union
}

fn add_properties_to_context<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    function_like_metadata: &'ctx FunctionLikeMetadata,
) -> Result<(), AnalysisError> {
    let Some(calling_class) = block_context.scope.get_class_like_name() else {
        return Ok(());
    };

    for (property_name, declaring_class) in &class_like_metadata.declaring_property_ids {
        let Some(property_class_metadata) = get_class_like(context.codebase, declaring_class) else {
            return Err(AnalysisError::InternalError(
                format!("Could not load property class metadata for `{}`.", declaring_class),
                class_like_metadata.span,
            ));
        };

        let Some(property_metadata) = property_class_metadata.properties.get(property_name) else {
            return Err(AnalysisError::InternalError(
                format!("Could not load property metadata for `{}`.", property_name),
                class_like_metadata.span,
            ));
        };

        let mut property_type = property_metadata
            .type_metadata
            .as_ref()
            .map(|type_metadata| &type_metadata.type_union)
            .cloned()
            .unwrap_or_else(get_mixed);

        let raw_property_name = property_name.strip_prefix("$").unwrap_or(property_name);

        let expression_id = if property_metadata.flags.is_static() {
            format!("{}::${raw_property_name}", class_like_metadata.name)
        } else {
            let this_type = get_this_type(context, class_like_metadata, function_like_metadata);

            property_type = localize_property_type(
                context,
                &property_type,
                this_type.get_type_parameters().unwrap_or_default(),
                class_like_metadata,
                property_class_metadata,
            );

            format!("$this->{raw_property_name}")
        };

        expander::expand_union(
            context.codebase,
            &mut property_type,
            &TypeExpansionOptions {
                self_class: Some(calling_class),
                static_class_type: StaticClassType::Name(calling_class),
                function_is_final: if let Some(method_metadata) = &function_like_metadata.method_metadata {
                    method_metadata.is_final
                } else {
                    false
                },
                expand_generic: true,
                ..Default::default()
            },
        );

        block_context.locals.insert(expression_id, Rc::new(property_type));
    }

    Ok(())
}

fn get_this_type(
    context: &Context<'_, '_>,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
) -> TObject {
    if class_like_metadata.kind.is_enum() {
        return TObject::Enum(TEnum { name: class_like_metadata.original_name, case: None });
    }

    let mut intersections = vec![];
    for required_interface in &class_like_metadata.require_implements {
        let Some(interface_metadata) = get_interface(context.codebase, required_interface) else {
            continue;
        };

        let TObject::Named(mut interface_type) = get_this_type(context, interface_metadata, function_like_metadata)
        else {
            continue;
        };

        let interface_intersactions = std::mem::take(&mut interface_type.intersection_types);

        interface_type.is_this = false;
        intersections.push(TAtomic::Object(TObject::Named(interface_type)));
        if let Some(interface_intersactions) = interface_intersactions {
            intersections.extend(interface_intersactions);
        }
    }

    for required_class in &class_like_metadata.require_extends {
        let Some(parent_class_metadata) = get_class_like(context.codebase, required_class) else {
            continue;
        };

        let TObject::Named(mut parent_type) = get_this_type(context, parent_class_metadata, function_like_metadata)
        else {
            continue;
        };

        let parent_intersections = std::mem::take(&mut parent_type.intersection_types);

        parent_type.is_this = false;
        intersections.push(TAtomic::Object(TObject::Named(parent_type)));
        if let Some(parent_intersections) = parent_intersections {
            intersections.extend(parent_intersections);
        }
    }

    let mut type_parameters = vec![];
    for (template_name, template_map) in &class_like_metadata.template_types {
        if let Some(constraint) = function_like_metadata
            .method_metadata
            .as_ref()
            .and_then(|method_metadata| method_metadata.where_constraints.get(template_name))
        {
            type_parameters.push(constraint.type_union.clone());
        } else {
            let (defining_entry, constraint) = unsafe {
                // SAFETY: This is safe because we are guaranteed that the template_map is not empty
                template_map.iter().next().unwrap_unchecked()
            };

            type_parameters.push(wrap_atomic(TAtomic::GenericParameter(TGenericParameter {
                parameter_name: *template_name,
                constraint: Box::new(constraint.clone()),
                defining_entity: *defining_entry,
                intersection_types: None,
            })));
        }
    }

    TObject::Named(TNamedObject {
        name: class_like_metadata.original_name,
        type_parameters: if !type_parameters.is_empty() { Some(type_parameters) } else { None },
        is_this: true,
        intersection_types: if intersections.is_empty() { None } else { Some(intersections) },
        remapped_parameters: false,
    })
}

fn add_symbol_references(
    parameter_type: &TUnion,
    calling_function_like_id: Option<&FunctionLikeIdentifier>,
    artifacts: &mut AnalysisArtifacts,
) {
    for type_node in parameter_type.get_all_child_nodes() {
        if let TypeRef::Atomic(atomic) = type_node {
            match atomic {
                TAtomic::Reference(TReference::Symbol { name, .. })
                | TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Function(name))) => {
                    match calling_function_like_id {
                        Some(FunctionLikeIdentifier::Function(calling_function)) => {
                            artifacts.symbol_references.add_symbol_reference_to_symbol(*calling_function, *name, true);
                        }
                        Some(FunctionLikeIdentifier::Method(calling_classlike, calling_function)) => {
                            artifacts.symbol_references.add_class_member_reference_to_symbol(
                                (*calling_classlike, *calling_function),
                                *name,
                                true,
                            );
                        }
                        _ => {}
                    }
                }
                TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Method(name, member_name))) => {
                    match calling_function_like_id {
                        Some(FunctionLikeIdentifier::Function(calling_function)) => {
                            artifacts.symbol_references.add_symbol_reference_to_class_member(
                                *calling_function,
                                (*name, *member_name),
                                true,
                            );
                        }
                        Some(FunctionLikeIdentifier::Method(calling_classlike, calling_function)) => {
                            artifacts.symbol_references.add_class_member_reference_to_class_member(
                                (*calling_classlike, *calling_function),
                                (*name, *member_name),
                                true,
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn check_thrown_types<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    function_like_metadata: &'ctx FunctionLikeMetadata,
) -> Result<(), AnalysisError> {
    if !context.settings.check_throws {
        // If the setting is disabled, we skip the check.
        return Ok(());
    }

    if block_context.possibly_thrown_exceptions.is_empty() {
        // No exceptions are thrown in this block, so we can skip the check.
        return Ok(());
    }

    let Some(function_name) = function_like_metadata.original_name.as_ref() else {
        return Ok(());
    };

    let (function_kind, function_name) = match function_like_metadata.kind.is_method() {
        true => {
            let Some(class_like_metadata) = block_context.scope.get_class_like() else {
                return Ok(());
            };

            let name = concat_atom!(&class_like_metadata.original_name, "::", function_name);

            ("method", name)
        }
        false => ("function", *function_name),
    };

    let expected_throw_types =
        get_function_like_thrown_types(context.codebase, block_context.scope.get_class_like(), function_like_metadata)
            .iter()
            .map(|thrown_type| {
                expand_type_metadata(context, block_context, artifacts, function_like_metadata, thrown_type)
            })
            .collect::<Vec<_>>();

    for (thrown_type, thrown_spans) in &block_context.possibly_thrown_exceptions {
        let thrown_type_union = TUnion::from_atomic(TAtomic::Object(TObject::new_named(*thrown_type)));

        let mut is_expected = false;
        for expected_type in &expected_throw_types {
            if union_comparator::is_contained_by(
                context.codebase,
                &thrown_type_union,
                expected_type,
                false,
                false,
                false,
                &mut Default::default(),
            ) {
                is_expected = true;
                break;
            }
        }

        if is_expected {
            continue;
        }

        let mut issue =
            Issue::error(format!("Potentially unhandled exception `{}` in `{}`.", thrown_type, function_name));

        for span in thrown_spans {
            issue = issue.with_annotation(Annotation::primary(*span).with_message("Exception may be thrown here"));
        }

        issue = issue
            .with_annotation(
                Annotation::secondary(function_like_metadata.span)
                    .with_message(format!("This {function_kind} does not declare that it throws `{}`", thrown_type)),
            )
            .with_note(format!(
                "All possible exceptions must be caught or declared in a `@throws` tag in the {function_kind}'s docblock.",
            ))
            .with_help(format!(
                "You can add `@throws {}` to the {function_kind}'s docblock or wrap the throwing code in a `try-catch` block.",
                thrown_type
            ));

        context.collector.report_with_code(IssueCode::UnhandledThrownType, issue);
    }

    Ok(())
}
