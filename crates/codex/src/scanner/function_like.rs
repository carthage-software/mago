use mago_docblock::tag::TypeString;
use mago_interner::StringIdentifier;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::utils;

use crate::assertion::Assertion;
use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::function_like::MethodMetadata;
use crate::misc::GenericParent;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::FunctionLikeDocblockComment;
use crate::scanner::parameter::scan_function_like_parameter;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::ttype::builder;
use crate::ttype::get_mixed;
use crate::ttype::resolution::TypeResolutionContext;
use crate::visibility::Visibility;

#[inline]
pub fn scan_method(
    functionlike_id: (StringIdentifier, StringIdentifier),
    method: &Method,
    class_like_metadata: &ClassLikeMetadata,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
    type_resolution_context: Option<TypeResolutionContext>,
) -> FunctionLikeMetadata {
    let span = method.span();

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Method, span)
        .with_name(Some(method.name.value), Some(method.name.span))
        .with_parameters(
            method
                .parameter_list
                .parameters
                .iter()
                .map(|p| scan_function_like_parameter(p, Some(&class_like_metadata.name), context)),
        );

    metadata.attributes = scan_attribute_lists(&method.attribute_lists, context);
    metadata.type_resolution_context = type_resolution_context.filter(|c| !c.is_empty());

    if let Some(return_hint) = method.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            Some(&class_like_metadata.name),
            context,
        )));
    }

    let mut method_metadata = MethodMetadata::new(if let Some(v) = method.modifiers.get_first_visibility() {
        Visibility::try_from(v).unwrap_or(Visibility::Public)
    } else {
        Visibility::Public
    })
    .with_final(method.modifiers.contains_final())
    .with_abstract(method.modifiers.contains_abstract())
    .with_static(method.modifiers.contains_static())
    .as_constructor(context.interner.lookup(&method.name.value).eq_ignore_ascii_case("__construct"));

    if let MethodBody::Concrete(block) = &method.body {
        metadata.has_yield = utils::block_has_yield(block);
        metadata.has_throw = utils::block_has_throws(block);
    } else {
        method_metadata = method_metadata.with_abstract(true);
    }

    metadata.method_metadata = Some(method_metadata);

    scan_function_like_docblock(span, functionlike_id, &mut metadata, Some(&class_like_metadata.name), context, scope);

    metadata
}

#[inline]
pub fn scan_function(
    functionlike_id: (StringIdentifier, StringIdentifier),
    function: &Function,
    classname: Option<&StringIdentifier>,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Function, function.span())
        .with_name(Some(functionlike_id.1), Some(function.name.span))
        .with_parameters(
            function.parameter_list.parameters.iter().map(|p| scan_function_like_parameter(p, classname, context)),
        );

    metadata.attributes = scan_attribute_lists(&function.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };
    metadata.has_yield = utils::block_has_yield(&function.body);
    metadata.has_throw = utils::block_has_throws(&function.body);

    if let Some(return_hint) = function.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(function.span(), functionlike_id, &mut metadata, classname, context, scope);

    metadata
}

#[inline]
pub fn scan_closure(
    functionlike_id: (StringIdentifier, StringIdentifier),
    closure: &Closure,
    classname: Option<&StringIdentifier>,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let span = closure.span();

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Closure, span).with_parameters(
        closure.parameter_list.parameters.iter().map(|p| scan_function_like_parameter(p, classname, context)),
    );

    metadata.attributes = scan_attribute_lists(&closure.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };
    metadata.has_yield = utils::block_has_yield(&closure.body);
    metadata.has_throw = utils::block_has_throws(&closure.body);

    if let Some(return_hint) = closure.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(span, functionlike_id, &mut metadata, classname, context, scope);

    metadata
}

#[inline]
pub fn scan_arrow_function(
    functionlike_id: (StringIdentifier, StringIdentifier),
    arrow_function: &ArrowFunction,
    classname: Option<&StringIdentifier>,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let span = arrow_function.span();

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::ArrowFunction, span).with_parameters(
        arrow_function.parameter_list.parameters.iter().map(|p| scan_function_like_parameter(p, classname, context)),
    );

    metadata.attributes = scan_attribute_lists(&arrow_function.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };
    metadata.has_yield = utils::expression_has_yield(&arrow_function.expression);
    metadata.has_throw = utils::expression_has_throws(&arrow_function.expression);

    if let Some(return_hint) = arrow_function.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            classname,
            context,
        )));
    }

    scan_function_like_docblock(span, functionlike_id, &mut metadata, classname, context, scope);

    metadata
}

fn scan_function_like_docblock(
    span: Span,
    functionlike_id: (StringIdentifier, StringIdentifier),
    metadata: &mut FunctionLikeMetadata,
    classname: Option<&StringIdentifier>,
    context: &mut Context<'_>,
    scope: &mut NamespaceScope,
) {
    let docblock = match FunctionLikeDocblockComment::create(context, span, scope) {
        Ok(Some(docblock)) => docblock,
        Ok(None) => return,
        Err(parse_error) => {
            metadata.issues.push(
                Issue::error("Failed to parse function-like docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            return;
        }
    };

    metadata.is_deprecated |= docblock.is_deprecated;
    metadata.is_internal |= docblock.is_internal;
    metadata.is_pure |= docblock.is_pure;
    metadata.is_mutation_free |= docblock.is_mutation_free;
    metadata.is_external_mutation_free |= docblock.is_external_mutation_free;
    metadata.ignore_falsable_return |= docblock.ignore_falsable_return;
    metadata.ignore_nullable_return |= docblock.ignore_nullable_return;
    metadata.inherits_docs |= docblock.inherits_docs;
    metadata.allows_named_arguments |= docblock.allows_named_arguments;

    let mut type_context = metadata.get_type_resolution_context().cloned().unwrap_or_default();
    for template in docblock.templates.iter() {
        let template_name = context.interner.intern(&template.name);
        let template_as_type = if let Some(type_string) = &template.type_string {
            match builder::get_type_from_string(
                &type_string.value,
                type_string.span,
                scope,
                &type_context,
                classname,
                context.interner,
            ) {
                Ok(tunion) => tunion,
                Err(typing_error) => {
                    metadata.issues.push(
                        Issue::error("Invalid `@template` type string.")
                            .with_code(ScanningIssueKind::InvalidTemplateTag)
                            .with_annotation(
                                Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                            )
                            .with_note(typing_error.note())
                            .with_help(typing_error.help()),
                    );

                    continue;
                }
            }
        } else {
            get_mixed()
        };

        let definition = vec![(GenericParent::FunctionLike(functionlike_id), template_as_type)];

        metadata.add_template_type((template_name, definition.clone()));
        type_context = type_context.with_template_definition(template.name.clone(), definition);
    }

    for parameter_tag in docblock.parameters {
        let parameter_name;
        let parameter_name_str;
        let is_variadic = parameter_tag.name.starts_with("...");
        if is_variadic {
            parameter_name_str = &parameter_tag.name[3..];
            parameter_name = context.interner.intern(parameter_name_str);
        } else {
            parameter_name_str = &parameter_tag.name;
            parameter_name = context.interner.intern(parameter_name_str);
        }

        let param_type_string = &parameter_tag.type_string;

        let Some(parameter_metadata) = metadata.get_parameter_mut(parameter_name) else {
            metadata.issues.push(
                Issue::error("The @param tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(
                        Annotation::primary(parameter_tag.span)
                            .with_message(format!("Parameter `{parameter_name_str}` is not defined in this function")),
                    )
                    .with_note(
                        "Each `@param` tag in a docblock must correspond to a parameter in the function's signature.",
                    )
                    .with_help("Please check for typos or add the parameter to the function signature."),
            );

            continue;
        };

        let mut variadic_mismatch_issue = None;
        if is_variadic && !parameter_metadata.is_variadic() {
            let parameter_span = parameter_metadata.get_span();
            parameter_metadata.is_variadic = true;

            variadic_mismatch_issue = Some(
                Issue::error("@param tag has a variadic mismatch.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(Annotation::primary(parameter_tag.span).with_message(
                        "This docblock declares the parameter as variadic, but the function signature does not",
                    ))
                    .with_annotation(
                        Annotation::secondary(parameter_span)
                            .with_message("The parameter is declared here without being variadic"),
                    )
                    .with_note("The use of `...` in the `@param` tag must match the function's parameter declaration.")
                    .with_help("Either add `...` to the parameter in the function signature or remove it from the `@param` tag."),
            );
        }

        match get_type_metadata_from_type_string(param_type_string, classname, &type_context, context, scope) {
            Ok(mut provided_type) => {
                let resulting_type = if !is_variadic
                    && parameter_metadata.is_variadic()
                    && let Some(array_value) = provided_type.type_union.get_single_value_of_array_like()
                {
                    provided_type.type_union = array_value.into_owned();
                    provided_type
                } else {
                    provided_type
                };

                parameter_metadata.set_type_signature(Some(resulting_type));
            }
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Could not resolve the type for the @param tag.")
                        .with_code(ScanningIssueKind::InvalidParamTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }

        if let Some(variadic_mismatch_issue) = variadic_mismatch_issue {
            metadata.issues.push(variadic_mismatch_issue);
        }
    }

    for param_out in docblock.parameters_out {
        let param_name = context.interner.intern(&param_out.name);

        let Some(parameter_metadata) = metadata.get_parameter_mut(param_name) else {
            metadata.issues.push(
                Issue::error("@param-out tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamOutTag)
                    .with_annotation(
                        Annotation::primary(param_out.span)
                            .with_message(format!("Parameter `{}` does not exist", param_out.name)),
                    )
                    .with_note("The `@param-out` tag specifies the type of a by-reference parameter after the function has executed.")
                    .with_help("Check for typos or ensure this parameter exists in the function signature."),
            );

            continue;
        };

        if !parameter_metadata.is_by_reference {
            metadata.issues.push(
                Issue::error("@param-out tag used on a non-by-reference parameter")
                    .with_code(ScanningIssueKind::InvalidParamOutTag)
                    .with_annotation(
                        Annotation::primary(param_out.span)
                            .with_message("This parameter is not declared as by-reference"),
                    )
                    .with_note("The `@param-out` tag can only be used with parameters that are passed by reference.")
                    .with_help("Ensure the parameter is declared with `&` in the function signature."),
            );

            continue;
        }

        match get_type_metadata_from_type_string(&param_out.type_string, classname, &type_context, context, scope) {
            Ok(parameter_out_type) => {
                parameter_metadata.set_out_type(Some(parameter_out_type));
            }
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Invalid `@param-out` type string.")
                        .with_code(ScanningIssueKind::InvalidParamOutTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }
    }

    'this_out: {
        let Some(this_out) = docblock.this_out else {
            // If there is no `@this-out` tag, we can skip this section.
            break 'this_out;
        };

        let Some(method_metadata) = metadata.get_method_metadata_mut() else {
            metadata.issues.push(
                Issue::error("`@this-out` tag used in a non-method function-like.")
                    .with_code(ScanningIssueKind::InvalidThisOutTag)
                    .with_annotation(
                        Annotation::primary(this_out.type_string.span)
                            .with_message("`@this-out` can only be used in methods, not in functions or closures"),
                    )
                    .with_note("The `@this-out` tag is specific to methods that modify the instance state.")
                    .with_help("Ensure this tag is only used in method docblocks."),
            );

            break 'this_out;
        };

        if method_metadata.is_static() {
            metadata.issues.push(
                Issue::error("`@this-out` tag used in a static method.")
                    .with_code(ScanningIssueKind::InvalidThisOutTag)
                    .with_annotation(
                        Annotation::primary(this_out.type_string.span)
                            .with_message("`@this-out` cannot be used in static methods"),
                    )
                    .with_note("The `@this-out` tag is intended for instance methods that modify the instance state.")
                    .with_help("Remove the `@this-out` tag from static method docblocks."),
            );

            break 'this_out;
        }

        match get_type_metadata_from_type_string(&this_out.type_string, classname, &type_context, context, scope) {
            Ok(out_type_metadata) => {
                metadata.this_out_type = Some(out_type_metadata);
            }
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Failed to resolve `@this-out` type string.")
                        .with_code(ScanningIssueKind::InvalidThisOutTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }
    }

    if let Some(return_type) = docblock.return_type.as_ref() {
        match get_type_metadata_from_type_string(&return_type.type_string, classname, &type_context, context, scope) {
            Ok(return_type_signature) => metadata.set_return_type_metadata(Some(return_type_signature)),
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Failed to resolve `@return` type string.")
                        .with_code(ScanningIssueKind::InvalidReturnTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }
    }

    'if_this_is: {
        let Some(if_this_is) = docblock.if_this_is else {
            // If there is no `@if-this-is` tag, we can skip this section.
            break 'if_this_is;
        };

        let Some(method_metadata) = metadata.get_method_metadata_mut() else {
            metadata.issues.push(
                Issue::error("`@if-this-is` tag used in a non-method function-like.")
                    .with_code(ScanningIssueKind::InvalidIfThisIsTag)
                    .with_annotation(
                        Annotation::primary(if_this_is.type_string.span)
                            .with_message("`@if-this-is` can only be used in methods, not in functions or closures"),
                    )
                    .with_note("The `@if-this-is` tag is specific to methods that check the type of `this`.")
                    .with_help("Ensure this tag is only used in method docblocks."),
            );

            break 'if_this_is;
        };

        if method_metadata.is_static() {
            metadata.issues.push(
                Issue::error("`@if-this-is` tag used in a static method.")
                    .with_code(ScanningIssueKind::InvalidIfThisIsTag)
                    .with_annotation(
                        Annotation::primary(if_this_is.type_string.span)
                            .with_message("`@if-this-is` cannot be used in static methods"),
                    )
                    .with_note("The `@if-this-is` tag is intended for instance methods that check the type of `this`.")
                    .with_help("Remove the `@if-this-is` tag from static method docblocks."),
            );

            break 'if_this_is;
        }

        match get_type_metadata_from_type_string(&if_this_is.type_string, classname, &type_context, context, scope) {
            Ok(constraint_type) => {
                metadata.if_this_is_type = Some(constraint_type);
            }
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Failed to resolve `@if-this-is` type string.")
                        .with_code(ScanningIssueKind::InvalidIfThisIsTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }
    }

    for thrown in docblock.throws {
        match get_type_metadata_from_type_string(&thrown.type_string, classname, &type_context, context, scope) {
            Ok(thrown_type) => {
                metadata.thrown_types.push(thrown_type);
            }
            Err(typing_error) => {
                metadata.issues.push(
                    Issue::error("Invalid `@throws` type string.")
                        .with_code(ScanningIssueKind::InvalidThrowsTag)
                        .with_annotation(
                            Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                        )
                        .with_note(typing_error.note())
                        .with_help(typing_error.help()),
                );
            }
        }
    }

    for assertion_tag in docblock.assertions {
        let assertion_param_name = context.interner.intern(&assertion_tag.parameter_name);

        let assertions =
            parse_assertion_string(assertion_tag.type_string, classname, &type_context, context, scope, metadata);

        for assertion in assertions {
            metadata.assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    for assertion_tag in docblock.if_true_assertions {
        let assertion_param_name = context.interner.intern(&assertion_tag.parameter_name);

        let assertions =
            parse_assertion_string(assertion_tag.type_string, classname, &type_context, context, scope, metadata);

        for assertion in assertions {
            metadata.if_true_assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    for assertion_tag in docblock.if_false_assertions {
        let assertion_param_name = context.interner.intern(&assertion_tag.parameter_name);

        let assertions =
            parse_assertion_string(assertion_tag.type_string, classname, &type_context, context, scope, metadata);

        for assertion in assertions {
            metadata.if_false_assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    metadata.type_resolution_context = Some(type_context);

    if let Some(return_type) = metadata.get_return_type_metadata_mut() {
        return_type.type_union.ignore_nullable_issues = docblock.ignore_nullable_return;
        return_type.type_union.ignore_falsable_issues = docblock.ignore_falsable_return;
    }
}

fn parse_assertion_string(
    mut type_string: TypeString,
    classname: Option<&StringIdentifier>,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_>,
    scope: &NamespaceScope,
    function_like_metadata: &mut FunctionLikeMetadata,
) -> Vec<Assertion> {
    let mut assertions = Vec::new();
    if type_string.value.eq_ignore_ascii_case("truthy") || type_string.value.eq_ignore_ascii_case("!falsy") {
        assertions.push(Assertion::Truthy);

        return assertions;
    }

    if type_string.value.eq_ignore_ascii_case("falsy") || type_string.value.eq_ignore_ascii_case("!truthy") {
        assertions.push(Assertion::Falsy);

        return assertions;
    }

    if type_string.value.eq_ignore_ascii_case("empty") || type_string.value.eq_ignore_ascii_case("!non-empty") {
        assertions.push(Assertion::Empty);

        return assertions;
    }

    if type_string.value.eq_ignore_ascii_case("non-empty") || type_string.value.eq_ignore_ascii_case("!empty") {
        assertions.push(Assertion::NonEmpty);

        return assertions;
    }

    let mut is_equal = false;
    let mut is_negation = false;
    if type_string.value.starts_with("!") {
        is_negation = true;
        type_string.value = type_string.value[1..].to_string();
        type_string.span = type_string.span.with_start(type_string.span.start + 1);
    }

    if type_string.value.starts_with("=") {
        is_equal = true;
        type_string.value = type_string.value[1..].to_string();
        type_string.span = type_string.span.with_start(type_string.span.start + 1);
    }

    match get_type_metadata_from_type_string(&type_string, classname, type_context, context, scope) {
        Ok(type_metadata) => match (is_equal, is_negation) {
            (true, true) => {
                for atomic in type_metadata.type_union.types {
                    assertions.push(Assertion::IsNotIdentical(atomic));
                }
            }
            (true, false) => {
                for atomic in type_metadata.type_union.types {
                    assertions.push(Assertion::IsIdentical(atomic));
                }
            }
            (false, true) => {
                for atomic in type_metadata.type_union.types {
                    assertions.push(Assertion::IsNotType(atomic));
                }
            }
            (false, false) => {
                for atomic in type_metadata.type_union.types {
                    assertions.push(Assertion::IsType(atomic));
                }
            }
        },
        Err(typing_error) => {
            function_like_metadata.issues.push(
                Issue::error("Failed to resolve assertion type string.")
                    .with_code(ScanningIssueKind::InvalidAssertionTag)
                    .with_annotation(Annotation::primary(typing_error.span()).with_message(typing_error.to_string()))
                    .with_note(typing_error.note())
                    .with_help(typing_error.help()),
            );
        }
    }

    assertions
}
