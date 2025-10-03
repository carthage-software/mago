use ahash::HashMap;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::{self};
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::inferred_type_replacer;
use mago_codex::ttype::union::TUnion;
use mago_fixer::FixPlan;
use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_name::report_non_existent_class_like;
use crate::resolver::selector::resolve_member_selector;
use crate::utils::template::get_template_types_for_class_member;
use crate::visibility::check_property_read_visibility;
use crate::visibility::check_property_write_visibility;

/// Represents a successfully resolved instance property.
#[derive(Debug)]
pub struct ResolvedProperty {
    pub property_name: Atom,
    pub declaring_class_id: Option<Atom>,
    pub property_span: Option<Span>,
    pub property_type: TUnion,
    pub is_magic: bool,
}

/// Holds the results of a property resolution attempt.
#[derive(Debug, Default)]
pub struct PropertyResolutionResult {
    pub properties: Vec<ResolvedProperty>,
    pub has_ambiguous_path: bool,
    pub has_error_path: bool,
    pub has_invalid_path: bool,
    pub encountered_null: bool,
    pub encountered_mixed: bool,
    pub has_possibly_defined_property: bool,
}

/// Resolves all possible instance properties from an object expression and a member selector.
pub fn resolve_instance_properties<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    object_expression: &'ast Expression<'arena>,
    property_selector: &'ast ClassLikeMemberSelector<'arena>,
    operator_span: Span,
    is_null_safe: bool,
    for_assignment: bool,
) -> Result<PropertyResolutionResult, AnalysisError> {
    let mut result = PropertyResolutionResult::default();

    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    object_expression.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let selectors = resolve_member_selector(context, block_context, artifacts, property_selector)?;

    let Some(object_type) = artifacts.get_rc_expression_type(object_expression).cloned() else {
        return Ok(result);
    };

    let is_voidable = object_type.is_voidable();
    let is_nullable = object_type.is_nullable() || is_voidable;
    let is_all_void = object_type.is_void();
    let is_all_null = object_type.is_null() || is_all_void;

    if is_null_safe && !is_nullable {
        report_redundant_nullsafe(context, operator_span, object_expression, &object_type);
    }

    let mut property_names = Vec::new();
    for selector in selectors {
        if selector.is_dynamic() {
            result.has_ambiguous_path = true;
        }

        if let Some(name) = selector.name() {
            property_names.push(concat_atom!("$", &name));
        } else {
            result.has_invalid_path = true;
        }
    }

    let mut object_atomics = object_type.types.iter().collect::<Vec<_>>();
    while let Some(object_atomic) = object_atomics.pop() {
        if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = object_atomic {
            object_atomics.extend(constraint.types.iter());

            continue;
        }

        if object_atomic.is_null() || object_atomic.is_void() {
            result.encountered_null = true;

            if !is_null_safe {
                report_access_on_null(
                    context,
                    block_context,
                    object_expression.span(),
                    operator_span,
                    is_all_null,
                    object_atomic.is_void(),
                );
            }

            continue;
        }

        let TAtomic::Object(object) = object_atomic else {
            result.has_invalid_path = true;
            if object_type.is_mixed() {
                result.encountered_mixed = true;
            }

            if !block_context.inside_isset || !object_atomic.is_mixed() {
                report_access_on_non_object(context, object_atomic, property_selector, object_expression.span());
            }

            continue;
        };

        let classname = match object {
            TObject::Any => {
                result.has_ambiguous_path = true;

                if !block_context.inside_isset {
                    report_ambiguous_access(context, property_selector, object_expression.span(), atom("object"));
                }

                continue;
            }
            TObject::WithProperties(object) => {
                for prop_name in &property_names {
                    let key = atom(prop_name.trim_start_matches('$'));
                    let Some((_, value)) = object.known_properties.get_key_value(&key) else {
                        if object.sealed {
                            result.has_invalid_path = true;

                            report_non_existent_property(
                                context,
                                &object_type.get_id(),
                                prop_name,
                                property_selector.span(),
                                object_expression.span(),
                                true,
                            );

                            continue;
                        }

                        result.has_ambiguous_path = true;

                        if !block_context.inside_isset {
                            report_ambiguous_access(
                                context,
                                property_selector,
                                object_expression.span(),
                                object_type.get_id(),
                            );
                        }

                        continue;
                    };

                    let is_optional = value.0;
                    let mut property_type = value.1.clone();

                    if is_optional {
                        if !block_context.inside_isset {
                            report_possibly_non_existent_property(
                                context,
                                &object_type,
                                prop_name,
                                property_selector.span(),
                                object_expression.span(),
                            );
                        }

                        property_type = property_type.as_nullable();
                    }

                    let resolved_property = ResolvedProperty {
                        property_span: None,
                        property_name: *prop_name,
                        declaring_class_id: None,
                        property_type,
                        is_magic: false,
                    };

                    result.properties.push(resolved_property);
                }

                continue;
            }
            TObject::Named(named_object) => named_object.get_name(),
            TObject::Enum(r#enum) => r#enum.get_name(),
        };

        let magic_method_name = if for_assignment { "__set" } else { "__get" };
        let mut magic_method_identifier = MethodIdentifier::new(atom(&classname), atom(magic_method_name));
        magic_method_identifier = context.codebase.get_declaring_method_identifier(&magic_method_identifier);
        let magic_method = context.codebase.get_method_by_id(&magic_method_identifier);

        for prop_name in &property_names {
            let resolved_property = find_property_in_class(
                context,
                block_context,
                &classname,
                prop_name,
                property_selector,
                object_expression,
                object,
                operator_span,
                for_assignment,
                &mut result,
                magic_method.is_some(),
            )?;

            let Some(resolved_property) = resolved_property else {
                result.has_invalid_path = true;

                continue;
            };

            if resolved_property.is_magic {
                if magic_method.is_none() {
                    report_magic_property_without_get_set_method(
                        context,
                        object_expression.span(),
                        property_selector.span(),
                        &classname,
                        prop_name,
                        for_assignment,
                    );
                }

                artifacts
                    .symbol_references
                    .add_reference_for_method_call(&block_context.scope, &magic_method_identifier);
            }

            if let Some(declaring_class_id) = resolved_property.declaring_class_id {
                artifacts.symbol_references.add_reference_for_property_access(
                    &block_context.scope,
                    declaring_class_id,
                    resolved_property.property_name,
                );
            }

            result.properties.push(resolved_property);
        }
    }

    Ok(result)
}

/// Finds a property in a class, gets its type, and handles template localization.
fn find_property_in_class<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    class_id: &Atom,
    prop_name: &Atom,
    selector: &'ast ClassLikeMemberSelector<'arena>,
    object_expr: &'ast Expression<'arena>,
    object: &TObject,
    access_span: Span,
    for_assignment: bool,
    result: &mut PropertyResolutionResult,
    has_magic_method: bool,
) -> Result<Option<ResolvedProperty>, AnalysisError> {
    let declaring_class_id = context.codebase.get_declaring_property_class(class_id, prop_name).unwrap_or(*class_id);

    let Some(declaring_class_metadata) = context.codebase.get_class_like(&declaring_class_id) else {
        report_non_existent_class_like(context, object_expr.span(), &declaring_class_id);

        return Ok(None);
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(prop_name) else {
        if has_magic_method {
            report_non_documented_property(
                context,
                object_expr.span(),
                selector.span(),
                &declaring_class_id,
                prop_name,
                for_assignment,
            );

            return Ok(Some(ResolvedProperty {
                property_span: None,
                property_name: *prop_name,
                declaring_class_id: Some(declaring_class_id),
                property_type: get_mixed(),
                is_magic: true,
            }));
        }

        result.has_invalid_path = true;

        if !declaring_class_metadata.flags.is_final()
            || declaring_class_metadata.kind.is_interface()
            || declaring_class_metadata.kind.is_trait()
        {
            result.has_possibly_defined_property = true;
        }

        report_non_existent_property(context, class_id, prop_name, selector.span(), object_expr.span(), false);
        return Ok(None);
    };

    let mut property_type = property_metadata
        .type_metadata
        .as_ref()
        .map(|type_metadata| &type_metadata.type_union)
        .cloned()
        .unwrap_or_else(get_mixed);

    expander::expand_union(
        context.codebase,
        &mut property_type,
        &TypeExpansionOptions {
            self_class: Some(declaring_class_id),
            static_class_type: StaticClassType::Object(object.clone()),
            parent_class: declaring_class_metadata.direct_parent_class,
            ..Default::default()
        },
    );

    if !declaring_class_metadata.template_types.is_empty()
        && let TObject::Named(named_object) = object
    {
        property_type = localize_property_type(
            context,
            &property_type,
            named_object.get_type_parameters().unwrap_or_default(),
            if class_id.eq_ignore_ascii_case(&declaring_class_id) {
                declaring_class_metadata
            } else {
                context.codebase.get_class_like(class_id).unwrap_or(declaring_class_metadata)
            },
            declaring_class_metadata,
        );
    }

    let is_visible = if for_assignment {
        check_property_write_visibility(
            context,
            block_context,
            &declaring_class_id,
            prop_name,
            access_span,
            Some(selector.span()),
        )
    } else {
        check_property_read_visibility(
            context,
            block_context,
            &declaring_class_id,
            prop_name,
            access_span,
            Some(selector.span()),
        )
    };

    if !is_visible {
        result.has_error_path = true;

        return Ok(None);
    }

    Ok(Some(ResolvedProperty {
        property_span: property_metadata.name_span.or(property_metadata.span),
        property_name: *prop_name,
        declaring_class_id: Some(declaring_class_id),
        property_type,
        is_magic: property_metadata.flags.is_magic_property(),
    }))
}

pub fn localize_property_type(
    context: &Context<'_, '_>,
    class_property_type: &TUnion,
    object_type_parameters: &[TUnion],
    property_class_metadata: &ClassLikeMetadata,
    property_declaring_class_metadata: &ClassLikeMetadata,
) -> TUnion {
    let mut template_types = get_template_types_for_class_member(
        context,
        Some(property_declaring_class_metadata),
        Some(property_declaring_class_metadata.name),
        Some(property_class_metadata),
        &property_class_metadata.template_types,
        &IndexMap::default(),
    );

    update_template_types(
        context,
        &mut template_types,
        property_class_metadata,
        object_type_parameters,
        property_declaring_class_metadata,
    );

    inferred_type_replacer::replace(
        class_property_type,
        &TemplateResult::new(IndexMap::default(), template_types),
        context.codebase,
    )
}

fn update_template_types(
    context: &Context<'_, '_>,
    template_types: &mut IndexMap<Atom, HashMap<GenericParent, TUnion>, RandomState>,
    property_class_metadata: &ClassLikeMetadata,
    lhs_type_params: &[TUnion],
    property_declaring_class_metadata: &ClassLikeMetadata,
) {
    if !template_types.is_empty() && !property_class_metadata.template_types.is_empty() {
        for (param_offset, lhs_param_type) in lhs_type_params.iter().enumerate() {
            let mut i = -1;

            for (calling_param_name, _) in &property_class_metadata.template_types {
                i += 1;

                if i == (param_offset as i32) {
                    template_types.entry(*calling_param_name).or_default().insert(
                        GenericParent::ClassLike(property_class_metadata.name),
                        {
                            let mut lhs_param_type = lhs_param_type.clone();

                            expander::expand_union(
                                context.codebase,
                                &mut lhs_param_type,
                                &TypeExpansionOptions { parent_class: None, ..Default::default() },
                            );

                            lhs_param_type
                        },
                    );
                    break;
                }
            }
        }
    }

    for (type_name, v) in template_types.iter_mut() {
        if let Some(mapped_type) = property_class_metadata
            .template_extended_parameters
            .get(&property_declaring_class_metadata.name)
            .unwrap_or(&IndexMap::default())
            .get(type_name)
        {
            for mapped_type_atomic in mapped_type.types.as_ref() {
                if let TAtomic::GenericParameter(TGenericParameter { parameter_name, .. }) = &mapped_type_atomic {
                    let position = property_class_metadata
                        .template_types
                        .iter()
                        .enumerate()
                        .filter(|(_, (k, _))| k == parameter_name)
                        .map(|(i, _)| i)
                        .next();

                    if let Some(position) = position
                        && let Some(mapped_param) = lhs_type_params.get(position)
                    {
                        v.insert(
                            GenericParent::ClassLike(property_declaring_class_metadata.name),
                            mapped_param.clone(),
                        );
                    }
                }
            }
        }
    }
}

/// Reports an error for a property access on a `null` or `void` value.
fn report_access_on_null<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    object_span: Span,
    operator_span: Span,
    is_always_null: bool,
    from_void: bool,
) {
    match (from_void, is_always_null) {
        (true, true) => {
            context.collector.report_with_code(
                IssueCode::NullPropertyAccess,
                Issue::error("Attempting to access a property on an expression of type `void`.")
                    .with_annotation(
                        Annotation::primary(object_span)
                            .with_message("This expression has type `void`, which is treated as `null` at runtime"),
                    )
                    .with_note("Expressions of type `void` do not produce a value. Accessing a property on this will always result in `null` and raise a warning.")
                    .with_help("This access should be removed. Check the origin of this expression to understand why it results in `void`."),
            );
        }
        (true, false) => {
            context.collector.report_with_code(
                IssueCode::PossiblyNullPropertyAccess,
                Issue::error("Attempting to access a property on an expression that can be `void`.")
                    .with_annotation(
                        Annotation::primary(object_span).with_message("This expression's type includes `void`"),
                    )
                    .with_note("If this expression resolves to `void` at runtime, accessing a property will result in `null` and raise a warning.")
                    .with_note("The `void` type often originates from a function or a method that does not return a value.")
                    .with_help("You must guard this access. Check if the value is an object before accessing the property.")
                ,
            );
        }
        (false, true) => {
            context.collector.report_with_code(
                IssueCode::NullPropertyAccess,
                Issue::error("Attempting to access a property on an expression that is always `null`.")
                    .with_annotation(
                        Annotation::primary(object_span)
                            .with_message("This expression is always `null` here"),
                    )
                    .with_note("In PHP, this will raise a warning and the expression will evaluate to `null`.")
                    .with_help("This code path appears to be an error. You should either ensure this expression can be a valid object or remove the property access entirely."),
            );
        }
        (false, false) => {
            if !block_context.inside_isset {
                if block_context.inside_assignment {
                    context.collector.report_with_code(
                        IssueCode::PossiblyNullPropertyAccess,
                        Issue::error("Attempting to access a property on a possibly `null` value.")
                            .with_annotation(
                                Annotation::primary(object_span)
                                    .with_message("This expression can be `null` here"),
                            )
                            .with_note("If this expression is `null` at runtime, PHP will raise a warning and the property access will result in `null`.")
                            .with_help("Add a check to ensure the value is not `null` (e.g., `if ($obj !== null)`).")
                    );
                } else {
                    context.collector.report_with_code(
                        IssueCode::PossiblyNullPropertyAccess,
                        Issue::error("Attempting to access a property on a possibly `null` value.")
                            .with_annotation(
                                Annotation::primary(object_span)
                                    .with_message("This expression can be `null` here"),
                            )
                            .with_note("If this expression is `null` at runtime, PHP will raise a warning and the property access will result in `null`.")
                            .with_help("Use the nullsafe operator (`?->`) to safely access the property, or add a check to ensure the value is not `null` (e.g., `if ($obj !== null)`).")
                            .with_suggestion(operator_span.file_id, {
                                let mut plan = FixPlan::new();
                                plan.replace(operator_span.to_range(), "?->", SafetyClassification::Safe);

                                plan
                            }),
                    );
                }
            }
        }
    }
}

fn report_redundant_nullsafe<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    operator_span: Span,
    object_expr: &'ast Expression<'arena>,
    object_type: &TUnion,
) {
    let object_type_str = object_type.get_id();

    context.collector.report_with_code(
        IssueCode::RedundantNullsafeOperator,
        Issue::help("Redundant nullsafe operator (`?->`) used on an expression that is never `null`.")
            .with_annotation(
                Annotation::primary(operator_span).with_message("Nullsafe operator `?->` is unnecessary here"),
            )
            .with_annotation(
                Annotation::secondary(object_expr.span())
                    .with_message(format!("This expression (type `{object_type_str}`) is never `null`")),
            )
            .with_note("The nullsafe operator (`?->`) short-circuits the access if the object is `null`. Since this expression is guaranteed not to be `null`, this check is unnecessary.")
            .with_help("Consider using the direct property access operator (`->`) for clarity."),
    );
}

fn report_access_on_non_object(
    context: &mut Context,
    atomic_type: &TAtomic,
    selector: &ClassLikeMemberSelector,
    object_span: Span,
) {
    let type_str = atomic_type.get_id();
    context.collector.report_with_code(
        if atomic_type.is_mixed() { IssueCode::MixedPropertyAccess } else { IssueCode::InvalidPropertyAccess },
        Issue::error(format!("Attempting to access a property on a non-object type (`{type_str}`)."))
            .with_annotation(Annotation::primary(selector.span()).with_message("Cannot access property here"))
            .with_annotation(
                Annotation::secondary(object_span).with_message(format!("This expression has type `{type_str}`")),
            ),
    );
}

fn report_ambiguous_access(
    context: &mut Context,
    selector: &ClassLikeMemberSelector,
    object_span: Span,
    object_type: Atom,
) {
    context.collector.report_with_code(
        IssueCode::AmbiguousObjectPropertyAccess,
        Issue::warning(format!("Cannot statically verify property access on a generic `{}` type.", object_type))
            .with_annotation(Annotation::primary(selector.span()).with_message("Accessing property here"))
            .with_annotation(
                Annotation::secondary(object_span).with_message(format!("This expression has type `{}`", object_type)),
            )
            .with_help("Provide a more specific type hint for the object (e.g., `MyClass`) for robust analysis."),
    );
}

fn report_possibly_non_existent_property(
    context: &mut Context,
    object_type: &TUnion,
    prop_name: &Atom,
    selector_span: Span,
    object_span: Span,
) {
    context.collector.report_with_code(
        IssueCode::PossiblyNonExistentProperty,
        Issue::error(format!("Property `{prop_name}` might not exist on object `{}`.", object_type.get_id()))
            .with_annotation(Annotation::primary(selector_span).with_message("Property might not exist here"))
            .with_annotation(
                Annotation::secondary(object_span).with_message(format!("On instance of `{}`", object_type.get_id())),
            )
            .with_note(
                "If this property does not exist at runtime, PHP will raise a warning and the expression will evaluate to `null`.",
            )
            .with_help(
                "To avoid this, ensure the property is defined on the object or check for its existence before accessing it.",
            ),
    );
}

fn report_non_existent_property(
    context: &mut Context,
    classname: &Atom,
    prop_name: &Atom,
    selector_span: Span,
    object_span: Span,
    is_sealed_object: bool, // `true` if we are accessing undefined prop on `object{foo: string}` type, not an actual class
) {
    let class_kind_str = context.codebase.get_class_like(classname).map_or("class", |m| m.kind.as_str());

    context.collector.report_with_code(
        IssueCode::NonExistentProperty,
        Issue::error(if is_sealed_object {
            format!("Property `{prop_name}` does not exist on sealed object type `{classname}`.")
        } else {
            format!("Property `{prop_name}` does not exist on {class_kind_str} `{classname}`.")
        })
        .with_annotation(Annotation::primary(selector_span).with_message("Property not found here"))
        .with_annotation(Annotation::secondary(object_span).with_message(format!("On instance of `{classname}`")))
        .with_note(if is_sealed_object {
            format!("The type `{classname}` is a sealed object type and does not define the property `{prop_name}`.")
        } else {
            format!("The {class_kind_str} `{classname}` does not define the property `{prop_name}`.")
        })
        .with_help("Define the property in the class or check for its existence before accessing it."),
    );
}

pub(super) fn report_non_documented_property(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: &Atom,
    property_name: &Atom,
    for_assignment: bool,
) {
    let magic_method = if for_assignment { "__set" } else { "__get" };
    let access_type = if for_assignment { "write to" } else { "read from" };

    context.collector.report_with_code(
        IssueCode::NonDocumentedProperty,
        Issue::warning(format!("Ambiguous property access: {property_name} on class `{classname}`."))
        .with_annotation(
            Annotation::primary(selector_span).with_message("This property is not explicitly defined"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("On an object of type `{classname}`")),
        )
        .with_note(
            format!("While this {access_type} might be handled by `{magic_method}()`, Mago cannot determine its type without a corresponding `@property` docblock tag."),
        )
        .with_help(format!(
            "To enable type checking, add a `@property`, `@property-read`, or `@property-write` tag to the docblock of the `{classname}` class. For example: `/** @property string {property_name} */`",
        )),
    );
}

pub(super) fn report_magic_property_without_get_set_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: &Atom,
    property_name: &Atom,
    for_assignment: bool,
) {
    let magic_method_name = if for_assignment { "__set" } else { "__get" };
    let access_type = if for_assignment { "write to" } else { "read from" };

    context.collector.report_with_code(
        IssueCode::MissingMagicMethod,
        Issue::error(format!(
            "Access to documented magic property `{property_name}` on a class that cannot handle it.",
        ))
        .with_annotation(
            Annotation::primary(selector_span)
                .with_message("This magic property is documented but cannot be accessed"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("Class `{classname}` is missing the `{magic_method_name}` method")),
        )
        .with_note(
            format!("The class `{classname}` has a `@property` tag for `{property_name}` but is missing a `{magic_method_name}` method to handle the {access_type}. This will cause a fatal `Error` at runtime.")
        )
        .with_help(
            format!("Add a `public function {magic_method_name}()` to the `{classname}` class to handle magic property access.")
        ),
    );
}
