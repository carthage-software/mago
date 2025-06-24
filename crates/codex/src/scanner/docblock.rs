use mago_docblock::error::ParseError;
use mago_docblock::tag::*;
use mago_names::kind::NameKind;
use mago_names::scope::NamespaceScope;
use serde::Serialize;

use mago_docblock::document::*;
use mago_docblock::parse_trivia;
use mago_span::HasSpan;
use mago_span::Span;

use crate::scanner::Context;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClassLikeDocblockComment {
    pub span: Span,
    pub is_deprecated: bool,
    pub is_final: bool,
    pub is_immutable: bool,
    pub is_internal: bool,
    pub is_mutation_free: bool,
    pub is_external_mutation_free: bool,
    pub is_enum_interface: bool,
    pub allows_private_mutation: bool,
    pub has_consistent_constructor: bool,
    pub has_consistent_templates: bool,
    pub has_sealed_properties: Option<bool>,
    pub has_sealed_methods: Option<bool>,
    pub templates: Vec<TemplateTag>,
    pub template_extends: Vec<TypeString>,
    pub template_implements: Vec<TypeString>,
    pub require_extends: Vec<TypeString>,
    pub require_implements: Vec<TypeString>,
    pub inheritors: Option<TypeString>,
    pub unchecked: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct FunctionLikeDocblockComment {
    pub span: Span,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_pure: bool,
    pub ignore_nullable_return: bool,
    pub ignore_falsable_return: bool,
    pub inherits_docs: bool,
    pub is_mutation_free: bool,
    pub is_external_mutation_free: bool,
    pub allows_named_arguments: bool,
    pub return_type: Option<ReturnTypeTag>,
    pub parameters: Vec<ParameterTag>,
    pub parameters_out: Vec<ParameterOutTag>,
    pub this_out: Option<ThisOutTag>,
    pub if_this_is: Option<IfThisIsTag>,
    pub throws: Vec<ThrowsTag>,
    pub templates: Vec<TemplateTag>,
    pub assertions: Vec<AssertionTag>,
    pub if_true_assertions: Vec<AssertionTag>,
    pub if_false_assertions: Vec<AssertionTag>,
    pub unchecked: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyDocblockComment {
    pub span: Span,
    pub type_string: Option<TypeString>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_readonly: bool,
    pub allows_private_mutation: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConstantDocblockComment {
    pub span: Span,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_final: bool,
}

impl ClassLikeDocblockComment {
    pub fn create(
        context: &Context<'_>,
        class_like: impl HasSpan,
        scope: &mut NamespaceScope,
    ) -> Result<Option<ClassLikeDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(class_like) else {
            return Ok(None);
        };

        let mut is_final = false;
        let mut is_immutable = false;
        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_mutation_free = false;
        let mut is_external_mutation_free = false;
        let mut allows_private_mutation = false;
        let mut has_consistent_constructor = false;
        let mut has_consistent_templates = false;
        let mut has_sealed_properties = None;
        let mut has_sealed_methods = None;
        let mut templates = Vec::new();
        let mut template_extends = Vec::new();
        let mut template_implements = Vec::new();
        let mut require_extends = Vec::new();
        let mut require_implements = Vec::new();
        let mut inheritors = None;
        let mut is_enum_interface = false;
        let mut unchecked = false;

        let parsed_docblock = parse_trivia(context.interner, docblock)?;

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::Unchecked | TagKind::MagoUnchecked => {
                    unchecked = true;
                }
                TagKind::Deprecated => {
                    is_deprecated = true;
                }
                TagKind::NotDeprecated => {
                    is_deprecated = false;
                }
                TagKind::EnumInterface => {
                    is_enum_interface = true;
                }
                TagKind::Final => {
                    is_final = true;
                }
                TagKind::PsalmInternal | TagKind::Internal => {
                    is_internal = true;
                }
                TagKind::PsalmSealProperties | TagKind::SealProperties => {
                    has_sealed_properties = Some(true);
                }
                TagKind::PsalmNoSealProperties | TagKind::NoSealProperties => {
                    has_sealed_properties = Some(false);
                }
                TagKind::PsalmSealMethods | TagKind::SealMethods => {
                    has_sealed_methods = Some(true);
                }
                TagKind::PsalmNoSealMethods | TagKind::NoSealMethods => {
                    has_sealed_methods = Some(false);
                }
                TagKind::Inheritors | TagKind::PsalmInheritors => {
                    if let Some(inheritors_tag) =
                        split_tag_content(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        inheritors = Some(inheritors_tag.0);
                    }
                }
                TagKind::PhpstanTemplate
                | TagKind::PsalmTemplate
                | TagKind::Template
                | TagKind::TemplateInvariant
                | TagKind::PhpstanTemplateInvariant
                | TagKind::PsalmTemplateInvariant => {
                    if let Some(template) = parse_template_tag(
                        context.interner.lookup(&tag.description),
                        tag.description_span,
                        false,
                        false,
                    ) {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::PhpstanTemplateContravariant
                | TagKind::PsalmTemplateContravariant
                | TagKind::TemplateContravariant => {
                    if let Some(template) =
                        parse_template_tag(context.interner.lookup(&tag.description), tag.description_span, false, true)
                    {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::PhpstanTemplateCovariant | TagKind::PsalmTemplateCovariant | TagKind::TemplateCovariant => {
                    if let Some(template) =
                        parse_template_tag(context.interner.lookup(&tag.description), tag.description_span, true, false)
                    {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::TemplateExtends | TagKind::Extends => {
                    template_extends.push(TypeString {
                        value: context.interner.lookup(&tag.description).to_string(),
                        span: tag.description_span,
                    });
                }
                TagKind::TemplateImplements | TagKind::Implements => {
                    template_implements.push(TypeString {
                        value: context.interner.lookup(&tag.description).to_string(),
                        span: tag.description_span,
                    });
                }
                TagKind::PhpstanImmutable | TagKind::PsalmImmutable | TagKind::Immutable => {
                    is_immutable = true;
                }
                TagKind::ConsistentConstructor | TagKind::PsalmConsistentConstructor => {
                    has_consistent_constructor = true;
                }
                TagKind::PsalmConsistentTemplates => {
                    has_consistent_templates = true;
                }
                TagKind::PsalmMutationFree | TagKind::MutationFree => {
                    is_mutation_free = true;
                }
                TagKind::PsalmExternalMutationFree | TagKind::ExternalMutationFree => {
                    is_external_mutation_free = true;
                }
                TagKind::PsalmAllowPrivateMutation => {
                    allows_private_mutation = true;
                }
                TagKind::PhpstanRequireExtends | TagKind::PsalmRequireExtends => {
                    require_extends.push(TypeString {
                        value: context.interner.lookup(&tag.description).to_string(),
                        span: tag.description_span,
                    });
                }
                TagKind::PhpstanRequireImplements | TagKind::PsalmRequireImplements => {
                    require_implements.push(TypeString {
                        value: context.interner.lookup(&tag.description).to_string(),
                        span: tag.description_span,
                    });
                }
                _ => {
                    // Ignore other tags
                }
            }
        }

        Ok(Some(ClassLikeDocblockComment {
            span: docblock.span,
            is_deprecated,
            is_final,
            is_immutable,
            is_internal,
            is_mutation_free,
            is_external_mutation_free,
            is_enum_interface,
            allows_private_mutation,
            has_sealed_properties,
            has_sealed_methods,
            has_consistent_constructor,
            has_consistent_templates,
            templates,
            template_extends,
            template_implements,
            require_extends,
            require_implements,
            inheritors,
            unchecked,
        }))
    }
}

impl FunctionLikeDocblockComment {
    pub fn create(
        context: &Context<'_>,
        function: impl HasSpan,
        scope: &mut NamespaceScope,
    ) -> Option<FunctionLikeDocblockComment> {
        let docblock = context.get_docblock(function)?;

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_pure = false;
        let mut ignore_nullable_return = false;
        let mut ignore_falsable_return = false;
        let mut inherits_docs = false;
        let mut is_mutation_free = false;
        let mut is_external_mutation_free = false;
        let mut allows_named_arguments = true;
        let mut return_type: Option<ReturnTypeTag> = None;
        let mut parameters: Vec<ParameterTag> = Vec::new();
        let mut parameters_out: Vec<ParameterOutTag> = Vec::new();
        let mut this_out: Option<ThisOutTag> = None;
        let mut if_this_is: Option<IfThisIsTag> = None;
        let mut throws: Vec<ThrowsTag> = Vec::new();
        let mut templates: Vec<TemplateTag> = Vec::new();
        let mut assertions: Vec<AssertionTag> = Vec::new();
        let mut if_true_assertions: Vec<AssertionTag> = Vec::new();
        let mut if_false_assertions: Vec<AssertionTag> = Vec::new();
        let mut unchecked = false;

        let Ok(parsed_docblock) = parse_trivia(context.interner, docblock) else {
            tracing::trace!(
                "Failed to parse docblock for function-like in {} at {}:{}",
                context.interner.lookup(&context.source.identifier.0),
                context.source.line_number(docblock.span.start.offset),
                context.source.column_number(docblock.span.start.offset),
            );

            return None;
        };

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::Unchecked | TagKind::MagoUnchecked => {
                    unchecked = true;
                }
                TagKind::Deprecated => {
                    is_deprecated = true;
                }
                TagKind::Internal | TagKind::PsalmInternal => {
                    is_internal = true;
                }
                TagKind::PhpstanParam | TagKind::PsalmParam | TagKind::Param => {
                    if let Some(param) =
                        parse_param_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        parameters.push(param);
                    }
                }
                TagKind::NoNamedArguments => {
                    allows_named_arguments = false;
                }
                TagKind::PhpstanTemplate
                | TagKind::PsalmTemplate
                | TagKind::Template
                | TagKind::TemplateInvariant
                | TagKind::PhpstanTemplateInvariant
                | TagKind::PsalmTemplateInvariant => {
                    if let Some(t) = parse_template_tag(
                        context.interner.lookup(&tag.description),
                        tag.description_span,
                        false,
                        false,
                    ) {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::TemplateCovariant | TagKind::PhpstanTemplateCovariant | TagKind::PsalmTemplateCovariant => {
                    if let Some(t) =
                        parse_template_tag(context.interner.lookup(&tag.description), tag.description_span, true, false)
                    {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::TemplateContravariant
                | TagKind::PhpstanTemplateContravariant
                | TagKind::PsalmTemplateContravariant => {
                    if let Some(t) =
                        parse_template_tag(context.interner.lookup(&tag.description), tag.description_span, false, true)
                    {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::PsalmReturn | TagKind::PhpstanReturn | TagKind::Return => {
                    if let Some(return_tag) =
                        parse_return_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        return_type = Some(return_tag);
                    }
                }
                TagKind::Throws => {
                    if let Some(throws_tag) =
                        parse_throws_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        throws.push(throws_tag);
                    }
                }
                TagKind::NotDeprecated => {
                    is_deprecated = false;
                }
                TagKind::PhpstanImpure => {
                    is_pure = false;
                }
                TagKind::PsalmPure | TagKind::PhpstanPure | TagKind::Pure => {
                    is_pure = true;
                }
                TagKind::PsalmParamOut | TagKind::ParamOut => {
                    if let Some(param_out) =
                        parse_param_out_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        parameters_out.push(param_out);
                    }
                }
                TagKind::Assert | TagKind::PsalmAssert | TagKind::PhpstanAssert => {
                    if let Some(assertion) =
                        parse_assertion_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        assertions.push(assertion);
                    }
                }
                TagKind::AssertIfTrue | TagKind::PsalmAssertIfTrue | TagKind::PhpstanAssertIfTrue => {
                    if let Some(assertion) =
                        parse_assertion_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        if_true_assertions.push(assertion);
                    }
                }
                TagKind::AssertIfFalse | TagKind::PsalmAssertIfFalse | TagKind::PhpstanAssertIfFalse => {
                    if let Some(assertion) =
                        parse_assertion_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        if_false_assertions.push(assertion);
                    }
                }
                TagKind::PsalmIfThisIs => {
                    if let Some(if_this_is_tag) =
                        parse_if_this_is_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        if_this_is = Some(if_this_is_tag);
                    }
                }
                TagKind::PhpstanSelfOut | TagKind::PhpstanThisOut | TagKind::PsalmThisOut => {
                    if let Some(this_out_tag) =
                        parse_this_out_tag(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        this_out = Some(this_out_tag);
                    }
                }
                TagKind::IgnoreNullableReturn | TagKind::PsalmIgnoreNullableReturn => {
                    ignore_nullable_return = true;
                }
                TagKind::IgnoreFalsableReturn | TagKind::PsalmIgnoreFalsableReturn => {
                    ignore_falsable_return = true;
                }
                TagKind::PsalmMutationFree | TagKind::MutationFree => {
                    is_mutation_free = true;
                }
                TagKind::PsalmExternalMutationFree | TagKind::ExternalMutationFree => {
                    is_external_mutation_free = true;
                }
                TagKind::InheritDoc => {
                    inherits_docs = true;
                }
                _ => {
                    // Ignore other tags
                }
            }
        }

        Some(FunctionLikeDocblockComment {
            span: docblock.span,
            is_deprecated,
            is_internal,
            is_pure,
            ignore_nullable_return,
            ignore_falsable_return,
            inherits_docs,
            is_mutation_free,
            is_external_mutation_free,
            allows_named_arguments,
            return_type,
            parameters,
            parameters_out,
            this_out,
            if_this_is,
            throws,
            templates,
            assertions,
            if_true_assertions,
            if_false_assertions,
            unchecked,
        })
    }
}

impl PropertyDocblockComment {
    pub fn create(context: &Context<'_>, property: impl HasSpan) -> Option<PropertyDocblockComment> {
        let docblock = context.get_docblock(property)?;

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_readonly = false;
        let mut type_string: Option<TypeString> = None;
        let mut allows_private_mutation = false;

        let Ok(parsed_docblock) = parse_trivia(context.interner, docblock) else {
            tracing::trace!(
                "Failed to parse docblock for class-like property in {} at {}:{}",
                context.interner.lookup(&context.source.identifier.0),
                context.source.line_number(docblock.span.start.offset),
                context.source.column_number(docblock.span.start.offset),
            );

            return None;
        };

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::Deprecated => {
                    is_deprecated = true;
                }
                TagKind::Internal | TagKind::PsalmInternal => {
                    is_internal = true;
                }
                TagKind::PhpstanReadOnly | TagKind::PsalmReadOnly | TagKind::ReadOnly => {
                    is_readonly = true;
                }
                TagKind::PsalmVar | TagKind::PhpstanVar => {
                    if let Some(type_string_tag) =
                        split_tag_content(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::Var if type_string.is_none() => {
                    if let Some(type_string_tag) =
                        split_tag_content(context.interner.lookup(&tag.description), tag.description_span)
                    {
                        type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::PsalmAllowPrivateMutation => {
                    allows_private_mutation = true;
                }
                TagKind::PsalmReadOnlyAllowPrivateMutation => {
                    is_readonly = true;
                    allows_private_mutation = true;
                }
                _ => {}
            }
        }

        Some(PropertyDocblockComment {
            span: docblock.span,
            type_string,
            is_deprecated,
            is_internal,
            is_readonly,
            allows_private_mutation,
        })
    }
}

impl ConstantDocblockComment {
    pub fn create(context: &Context<'_>, constant: impl HasSpan) -> Option<ConstantDocblockComment> {
        let docblock = context.get_docblock(constant)?;

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_final = false;

        let Ok(parsed_docblock) = parse_trivia(context.interner, docblock) else {
            tracing::trace!(
                "Failed to parse docblock for constant in {} at {}:{}",
                context.interner.lookup(&context.source.identifier.0),
                context.source.line_number(docblock.span.start.offset),
                context.source.column_number(docblock.span.start.offset),
            );

            return None;
        };

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::Deprecated => {
                    is_deprecated = true;
                }
                TagKind::Internal | TagKind::PsalmInternal => {
                    is_internal = true;
                }
                TagKind::Final => {
                    is_final = true;
                }
                _ => {}
            }
        }

        Some(ConstantDocblockComment { span: docblock.span, is_deprecated, is_internal, is_final })
    }
}
