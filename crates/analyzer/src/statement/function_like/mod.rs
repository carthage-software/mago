use std::rc::Rc;

use ahash::HashMap;

use mago_codex::get_class_like;
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
use mago_codex::ttype::expander;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
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
pub enum FunctionLikeBody<'a> {
    Statements(&'a [Statement]),
    Expression(&'a Expression),
}

pub fn analyze_function_like<'a, 'ast>(
    context: &mut Context<'a>,
    parent_artifacts: &mut AnalysisArtifacts,
    block_context: &mut BlockContext<'a>,
    function_like_metadata: &'a FunctionLikeMetadata,
    parameter_list: &'ast FunctionLikeParameterList,
    body: FunctionLikeBody<'ast>,
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

    if let FunctionLikeBody::Statements(statements) = body {
        for statement in statements {
            let Statement::Global(global) = statement else {
                if statement.is_noop() {
                    continue;
                } else {
                    break;
                }
            };

            for variable in global.variables.iter() {
                if let Some(var_id) = get_variable_id(variable, context.interner) {
                    block_context.conditionally_referenced_variable_ids.insert(var_id);
                }
            }
        }
    }

    if let Some(calling_class) = block_context.scope.get_class_like_name()
        && let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, calling_class)
    {
        add_properties_to_context(context, block_context, class_like_metadata, function_like_metadata)?;
    }

    if !function_like_metadata.flags.is_unchecked() {
        match body {
            FunctionLikeBody::Statements(statements) => {
                analyze_statements(statements, context, block_context, &mut artifacts)?;
            }
            FunctionLikeBody::Expression(value) => {
                block_context.inside_return = true;
                value.analyze(context, block_context, &mut artifacts)?;
                block_context.inside_return = false;
                block_context.conditionally_referenced_variable_ids = Default::default();

                let value_type = artifacts.get_expression_type(value).cloned().unwrap_or_else(get_mixed);

                handle_return_value(context, block_context, &mut artifacts, Some(value), value_type, value.span())?;
            }
        }
    }

    std::mem::swap(&mut context.type_resolution_context, &mut previous_type_resolution_context);
    for (expression_range, expression_type) in std::mem::take(&mut artifacts.expression_types) {
        parent_artifacts.expression_types.insert(expression_range, expression_type);
    }

    Ok(artifacts)
}

fn add_parameter_types_to_context<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    function_like_metadata: &FunctionLikeMetadata,
    parameter_list: &FunctionLikeParameterList,
    mut inferred_parameter_types: Option<HashMap<usize, TUnion>>,
) -> Result<(), AnalysisError> {
    for (i, parameter_metadata) in function_like_metadata.parameters.iter().enumerate() {
        let parameter_variable_str = context.interner.lookup(&parameter_metadata.get_name().0);

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

fn expand_type_metadata<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    function_like_metadata: &FunctionLikeMetadata,
    type_metadata: &TypeMetadata,
) -> TUnion {
    add_symbol_references(
        &type_metadata.type_union,
        block_context.scope.get_function_like_identifier().as_ref(),
        artifacts,
    );

    let signature_union = type_metadata.type_union.clone();

    if !signature_union.is_mixed() {
        let mut parameter_type = signature_union.clone();
        let calling_class = block_context.scope.get_class_like_name();

        expander::expand_union(
            context.codebase,
            context.interner,
            &mut parameter_type,
            &TypeExpansionOptions {
                self_class: calling_class,
                static_class_type: if let Some(calling_class) = calling_class {
                    StaticClassType::Name(*calling_class)
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

        parameter_type
    } else {
        signature_union
    }
}

fn add_properties_to_context<'a>(
    context: &Context<'a>,
    block_context: &mut BlockContext<'a>,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
) -> Result<(), AnalysisError> {
    let Some(calling_class) = block_context.scope.get_class_like_name() else {
        return Ok(());
    };

    for (property_name_id, declaring_class) in &class_like_metadata.declaring_property_ids {
        let Some(property_class_metadata) = get_class_like(context.codebase, context.interner, declaring_class) else {
            return Err(AnalysisError::InternalError(
                format!("Could not load property class metadata for `{}`.", context.interner.lookup(declaring_class)),
                class_like_metadata.span,
            ));
        };

        let Some(property_metadata) = property_class_metadata.properties.get(property_name_id) else {
            return Err(AnalysisError::InternalError(
                format!("Could not load property metadata for `{}`.", context.interner.lookup(property_name_id)),
                class_like_metadata.span,
            ));
        };

        let mut property_type = property_metadata
            .type_metadata
            .as_ref()
            .map(|type_metadata| &type_metadata.type_union)
            .cloned()
            .unwrap_or_else(get_mixed);

        let property_name = context.interner.lookup(property_name_id);
        let raw_property_name = property_name.strip_prefix("$").unwrap_or(property_name);

        let expression_id = if property_metadata.flags.is_static() {
            format!("{}::${raw_property_name}", context.interner.lookup(&class_like_metadata.name),)
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
            context.interner,
            &mut property_type,
            &TypeExpansionOptions {
                self_class: Some(calling_class),
                static_class_type: StaticClassType::Name(*calling_class),
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
    context: &Context<'_>,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
) -> TObject {
    if class_like_metadata.kind.is_enum() {
        return TObject::Enum(TEnum { name: class_like_metadata.original_name, case: None });
    }

    let mut intersections = vec![];
    for required_interface in &class_like_metadata.require_implements {
        let Some(interface_metadata) = get_interface(context.codebase, context.interner, required_interface) else {
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
        let Some(parent_class_metadata) = get_class_like(context.codebase, context.interner, required_class) else {
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

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = properties_added_to_context,
        code = indoc! {r#"
            <?php

            namespace DateTime {
                final class Duration
                {
                    public function isPositive(): bool
                    {
                        return true;
                    }
                }

                /**
                 * @consistent-constructor
                 */
                abstract class AbstractTemporal
                {
                    public static function monotonic(): static
                    {
                        return new static();
                    }
                }

                final class Timestamp extends AbstractTemporal
                {
                    public function plus(Duration $_duration): static
                    {
                        return new static();
                    }

                    public function since(Timestamp $_timestamp): Duration
                    {
                        return new Duration();
                    }
                }
            }

            namespace Example {
                use Closure;
                use DateTime\Duration;
                use DateTime\Timestamp;

                final class OptionalIncrementalTimeout
                {
                    /**
                     * @var ?Timestamp The end time.
                     */
                    private null|Timestamp $end;

                    /**
                     * @var (Closure(): ?Duration) The handler to be called upon timeout.
                     */
                    private Closure $handler;

                    /**
                     * @param null|Duration $timeout The timeout duration. Null to disable timeout.
                     * @param (Closure(): ?Duration) $handler The handler to be executed if the timeout is reached.
                     */
                    public function __construct(null|Duration $timeout, Closure $handler)
                    {
                        $this->handler = $handler;

                        if (null === $timeout) {
                            $this->end = null;

                            return;
                        }

                        if (!$timeout->isPositive()) {
                            $this->end = Timestamp::monotonic();
                            return;
                        }

                        $this->end = Timestamp::monotonic()->plus($timeout);
                    }

                    /**
                     * Retrieves the remaining time until the timeout is reached, or null if no timeout is set.
                     *
                     * If the timeout has already been exceeded, the handler is invoked, and its return value is provided.
                     *
                     * @return Duration|null The remaining time duration, null if no timeout is set, or the handler's return value if the timeout is exceeded.
                     */
                    public function getRemaining(): null|Duration
                    {
                        if ($this->end === null) {
                            return null;
                        }

                        $remaining = $this->end->since(Timestamp::monotonic());

                        return $remaining->isPositive() ? $remaining : ($this->handler)();
                    }
                }
            }
        "#},
    }
}
