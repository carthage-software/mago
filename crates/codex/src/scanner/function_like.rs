use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;
use mago_docblock::tag::TypeString;
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
use crate::metadata::flags::MetadataFlags;
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
pub fn scan_method<'ctx, 'arena>(
    functionlike_id: (Atom, Atom),
    method: &'arena Method<'arena>,
    class_like_metadata: &ClassLikeMetadata,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
    type_resolution_context: Option<TypeResolutionContext>,
) -> FunctionLikeMetadata {
    let span = method.span();

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if method.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Method, span, flags);
    metadata.attributes = scan_attribute_lists(&method.attribute_lists, context);
    metadata.type_resolution_context = type_resolution_context.filter(|c| !c.is_empty());
    metadata.name = Some(ascii_lowercase_atom(method.name.value));
    metadata.original_name = Some(atom(method.name.value));

    metadata.name_span = Some(method.name.span);
    metadata.parameters = method
        .parameter_list
        .parameters
        .iter()
        .map(|p| scan_function_like_parameter(p, Some(class_like_metadata.name), context))
        .collect();

    if let Some(return_hint) = method.return_type_hint.as_ref() {
        metadata.set_return_type_declaration_metadata(Some(get_type_metadata_from_hint(
            &return_hint.hint,
            Some(class_like_metadata.name),
            context,
        )));
    }

    let method_name_str = method.name.value;

    let mut method_metadata = MethodMetadata {
        is_final: method.modifiers.contains_final(),
        is_abstract: method.modifiers.contains_abstract(),
        is_static: method.modifiers.contains_static(),
        is_constructor: method_name_str.eq_ignore_ascii_case("__construct"),
        visibility: if let Some(v) = method.modifiers.get_first_visibility() {
            Visibility::try_from(v).unwrap_or(Visibility::Public)
        } else {
            Visibility::Public
        },
        where_constraints: Default::default(),
    };

    if let MethodBody::Concrete(block) = &method.body {
        if utils::block_has_yield(block) {
            metadata.flags |= MetadataFlags::HAS_YIELD;
        }

        if utils::block_has_throws(block) {
            metadata.flags |= MetadataFlags::HAS_THROW;
        }
    } else {
        method_metadata.is_abstract = true;
    }

    metadata.method_metadata = Some(method_metadata);

    scan_function_like_docblock(span, functionlike_id, &mut metadata, Some(class_like_metadata.name), context, scope);

    metadata
}

#[inline]
pub fn scan_function<'ctx, 'arena>(
    functionlike_id: (Atom, Atom),
    function: &'arena Function<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if utils::block_has_yield(&function.body) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::block_has_throws(&function.body) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if function.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let name = context.resolved_names.get(&function.name);

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Function, function.span(), flags);

    metadata.name = Some(ascii_lowercase_atom(name));
    metadata.original_name = Some(atom(name));
    metadata.name_span = Some(function.name.span);
    metadata.parameters = function
        .parameter_list
        .parameters
        .iter()
        .map(|p| scan_function_like_parameter(p, classname, context))
        .collect();

    metadata.attributes = scan_attribute_lists(&function.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

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
pub fn scan_closure<'ctx, 'arena>(
    functionlike_id: (Atom, Atom),
    closure: &'arena Closure<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let span = closure.span();

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if utils::block_has_yield(&closure.body) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::block_has_throws(&closure.body) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if closure.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Closure, span, flags).with_parameters(
        closure.parameter_list.parameters.iter().map(|p| scan_function_like_parameter(p, classname, context)),
    );

    metadata.attributes = scan_attribute_lists(&closure.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

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
pub fn scan_arrow_function<'ctx, 'arena>(
    functionlike_id: (Atom, Atom),
    arrow_function: &'arena ArrowFunction<'arena>,
    classname: Option<Atom>,
    context: &mut Context<'ctx, 'arena>,
    scope: &mut NamespaceScope,
    type_resolution_context: TypeResolutionContext,
) -> FunctionLikeMetadata {
    let span = arrow_function.span();

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if utils::expression_has_yield(arrow_function.expression) {
        flags |= MetadataFlags::HAS_YIELD;
    }

    if utils::expression_has_throws(arrow_function.expression) {
        flags |= MetadataFlags::HAS_THROW;
    }

    if arrow_function.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::ArrowFunction, span, flags).with_parameters(
        arrow_function.parameter_list.parameters.iter().map(|p| scan_function_like_parameter(p, classname, context)),
    );

    metadata.attributes = scan_attribute_lists(&arrow_function.attribute_lists, context);
    metadata.type_resolution_context =
        if type_resolution_context.is_empty() { None } else { Some(type_resolution_context) };

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

fn scan_function_like_docblock<'ctx, 'arena>(
    span: Span,
    functionlike_id: (Atom, Atom),
    metadata: &mut FunctionLikeMetadata,
    classname: Option<Atom>,
    context: &mut Context<'ctx, 'arena>,
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

    if docblock.is_deprecated {
        metadata.flags |= MetadataFlags::DEPRECATED;
    }

    if docblock.is_internal {
        metadata.flags |= MetadataFlags::INTERNAL;
    }

    if docblock.must_use {
        metadata.flags |= MetadataFlags::MUST_USE;
    }

    if docblock.is_pure {
        metadata.flags |= MetadataFlags::PURE;
    }

    if docblock.ignore_falsable_return {
        metadata.flags |= MetadataFlags::IGNORE_FALSABLE_RETURN;
    }

    if docblock.ignore_nullable_return {
        metadata.flags |= MetadataFlags::IGNORE_NULLABLE_RETURN;
    }

    if docblock.inherits_docs {
        metadata.flags |= MetadataFlags::INHERITS_DOCS;
    }

    if docblock.no_named_arguments {
        metadata.flags |= MetadataFlags::NO_NAMED_ARGUMENTS;
    }

    if docblock.unchecked {
        metadata.flags |= MetadataFlags::UNCHECKED;
    }

    let mut type_context = metadata.type_resolution_context.clone().unwrap_or_default();
    for template in docblock.templates.iter() {
        let template_name = atom(&template.name);
        let template_as_type = if let Some(type_string) = &template.type_string {
            match builder::get_type_from_string(&type_string.value, type_string.span, scope, &type_context, classname) {
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
        type_context = type_context.with_template_definition(template_name, definition);
    }

    for parameter_tag in docblock.parameters {
        let parameter_name = atom(&parameter_tag.variable.name);
        let param_type_string = &parameter_tag.type_string;
        let is_variadic = parameter_tag.variable.is_variadic;

        let Some(parameter_metadata) = metadata.get_parameter_mut(parameter_name) else {
            metadata.issues.push(
                Issue::error("The @param tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamTag)
                    .with_annotation(Annotation::primary(parameter_tag.span).with_message(format!(
                        "Parameter `{}` is not defined in this function",
                        parameter_tag.variable
                    )))
                    .with_note(
                        "Each `@param` tag in a docblock must correspond to a parameter in the function's signature.",
                    )
                    .with_help("Please check for typos or add the parameter to the function signature."),
            );

            continue;
        };

        let mut variadic_mismatch_issue = None;
        if is_variadic && !parameter_metadata.flags.is_variadic() {
            let parameter_span = parameter_metadata.get_span();
            parameter_metadata.flags |= MetadataFlags::VARIADIC;

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

        match get_type_metadata_from_type_string(param_type_string, classname, &type_context, scope) {
            Ok(mut provided_type) => {
                let resulting_type = if !is_variadic
                    && parameter_metadata.flags.is_variadic()
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
        let param_name = atom(&param_out.variable.name);

        let Some(parameter_metadata) = metadata.get_parameter_mut(param_name) else {
            metadata.issues.push(
                Issue::error("@param-out tag references an unknown parameter.")
                    .with_code(ScanningIssueKind::InvalidParamOutTag)
                    .with_annotation(
                        Annotation::primary(param_out.span)
                            .with_message(format!("Parameter `{}` does not exist", param_out.variable)),
                    )
                    .with_note("The `@param-out` tag specifies the type of a by-reference parameter after the function has executed.")
                    .with_help("Check for typos or ensure this parameter exists in the function signature."),
            );

            continue;
        };

        if !parameter_metadata.flags.is_by_reference() {
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

        match get_type_metadata_from_type_string(&param_out.type_string, classname, &type_context, scope) {
            Ok(parameter_out_type) => {
                parameter_metadata.out_type = Some(parameter_out_type);
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

    if let Some(return_type) = docblock.return_type.as_ref() {
        match get_type_metadata_from_type_string(&return_type.type_string, classname, &type_context, scope) {
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

    for where_tag in docblock.where_constraints {
        let Some(method_metadata) = metadata.get_method_metadata_mut() else {
            metadata.issues.push(
                Issue::error("`@where` tag cannot be used on functions or closures.")
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(
                        Annotation::primary(where_tag.span)
                            .with_message("`@where` is only valid on instance methods"),
                    )
                    .with_note("The `@where` tag constrains template types based on the instance type of `$this`. Functions and closures do not have a `$this` context.")
                    .with_help("Remove the `@where` tag. If you need this logic, consider refactoring it into an instance method on a class."),
            );

            continue;
        };

        if method_metadata.is_static {
            metadata.issues.push(
                Issue::error("`@where` tag cannot be used on static methods.")
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(
                        Annotation::primary(where_tag.span)
                            .with_message("This constraint is not allowed on a static method"),
                    )
                    .with_note("The `@where` tag constrains template types based on the instance type of `$this`. Static methods are not tied to an instance and have no `$this` context.")
                    .with_help("Remove the `@where` tag. To constrain a template type on a static method, use a type bound like `@template T of SomeInterface` instead."),
            );

            continue;
        }

        match get_type_metadata_from_type_string(&where_tag.type_string, classname, &type_context, scope) {
            Ok(constraint_type) => {
                let template_name = atom(&where_tag.name);

                method_metadata.where_constraints.insert(template_name, constraint_type);
            }
            Err(typing_error) => metadata.issues.push(
                Issue::error(format!("Invalid constraint type `{}` in `@where` tag.", where_tag.type_string.value))
                    .with_code(ScanningIssueKind::InvalidWhereTag)
                    .with_annotation(Annotation::primary(typing_error.span()).with_message(typing_error.to_string()))
                    .with_note(typing_error.note())
                    .with_help(typing_error.help()),
            ),
        };
    }

    for thrown in docblock.throws {
        match get_type_metadata_from_type_string(&thrown.type_string, classname, &type_context, scope) {
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
        let assertion_param_name = atom(&assertion_tag.variable.name);

        let assertions = parse_assertion_string(assertion_tag.type_string, classname, &type_context, scope, metadata);

        for assertion in assertions {
            metadata.assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    for assertion_tag in docblock.if_true_assertions {
        let assertion_param_name = atom(&assertion_tag.variable.name);

        let assertions = parse_assertion_string(assertion_tag.type_string, classname, &type_context, scope, metadata);

        for assertion in assertions {
            metadata.if_true_assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    for assertion_tag in docblock.if_false_assertions {
        let assertion_param_name = atom(&assertion_tag.variable.name);

        let assertions = parse_assertion_string(assertion_tag.type_string, classname, &type_context, scope, metadata);

        for assertion in assertions {
            metadata.if_false_assertions.entry(assertion_param_name).or_default().push(assertion);
        }
    }

    metadata.type_resolution_context = Some(type_context);

    if docblock.ignore_nullable_return || docblock.ignore_falsable_return {
        if let Some(return_type) = &mut metadata.return_type_metadata {
            return_type.type_union.ignore_nullable_issues = docblock.ignore_nullable_return;
            return_type.type_union.ignore_falsable_issues = docblock.ignore_falsable_return;
        }

        if let Some(return_type) = &mut metadata.return_type_declaration_metadata {
            return_type.type_union.ignore_nullable_issues = docblock.ignore_nullable_return;
            return_type.type_union.ignore_falsable_issues = docblock.ignore_falsable_return;
        }
    }
}

fn parse_assertion_string(
    mut type_string: TypeString,
    classname: Option<Atom>,
    type_context: &TypeResolutionContext,
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
        type_string.span = type_string.span.from_start(type_string.span.start + 1);
    }

    if type_string.value.starts_with("=") {
        is_equal = true;
        type_string.value = type_string.value[1..].to_string();
        type_string.span = type_string.span.from_start(type_string.span.start + 1);
    }

    match get_type_metadata_from_type_string(&type_string, classname, type_context, scope) {
        Ok(type_metadata) => match (is_equal, is_negation) {
            (true, true) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsNotIdentical(atomic));
                }
            }
            (true, false) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsIdentical(atomic));
                }
            }
            (false, true) => {
                for atomic in type_metadata.type_union.types.into_owned() {
                    assertions.push(Assertion::IsNotType(atomic));
                }
            }
            (false, false) => {
                for atomic in type_metadata.type_union.types.into_owned() {
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
