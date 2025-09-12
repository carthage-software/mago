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
    pub is_internal: bool,
    pub is_enum_interface: bool,
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
    pub methods: Vec<MethodTag>,
    pub properties: Vec<PropertyTag>,
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
    pub no_named_arguments: bool,
    pub return_type: Option<ReturnTypeTag>,
    pub parameters: Vec<ParameterTag>,
    pub parameters_out: Vec<ParameterOutTag>,
    pub where_constraints: Vec<WhereTag>,
    pub throws: Vec<ThrowsTag>,
    pub templates: Vec<TemplateTag>,
    pub assertions: Vec<AssertionTag>,
    pub if_true_assertions: Vec<AssertionTag>,
    pub if_false_assertions: Vec<AssertionTag>,
    pub must_use: bool,
    pub unchecked: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyDocblockComment {
    pub span: Span,
    pub type_string: Option<TypeString>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConstantDocblockComment {
    pub span: Span,
    pub type_string: Option<TypeString>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub is_final: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TraitUseDocblockComment {
    pub template_extends: Vec<TypeString>,
    pub template_implements: Vec<TypeString>,
    pub template_use: Vec<TypeString>,
}

impl ClassLikeDocblockComment {
    pub fn create(
        context: &Context<'_, '_>,
        class_like: impl HasSpan,
        scope: &mut NamespaceScope,
    ) -> Result<Option<ClassLikeDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(class_like) else {
            return Ok(None);
        };

        let mut is_final = false;
        let mut is_deprecated = false;
        let mut is_internal = false;
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
        let mut methods = Vec::new();
        let mut properties = Vec::new();

        let parsed_docblock = parse_trivia(context.arena, docblock)?;

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
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((inheritors_tag, _)) = split_tag_content(description_str, description_span) {
                        inheritors = Some(inheritors_tag);
                    }
                }
                TagKind::PhpstanTemplate
                | TagKind::PsalmTemplate
                | TagKind::Template
                | TagKind::TemplateInvariant
                | TagKind::PhpstanTemplateInvariant
                | TagKind::PsalmTemplateInvariant => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some(template) = parse_template_tag(description_str, description_span, false, false) {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::PhpstanTemplateContravariant
                | TagKind::PsalmTemplateContravariant
                | TagKind::TemplateContravariant => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some(template) = parse_template_tag(description_str, description_span, false, true) {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::PhpstanTemplateCovariant | TagKind::PsalmTemplateCovariant | TagKind::TemplateCovariant => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some(template) = parse_template_tag(description_str, description_span, true, false) {
                        scope.add(NameKind::Default, &template.name, None as Option<&str>);

                        templates.push(template);
                    }
                }
                TagKind::TemplateExtends | TagKind::Extends => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((extended_type, _)) = split_tag_content(description_str, description_span) {
                        template_extends.push(extended_type);
                    }
                }
                TagKind::TemplateImplements | TagKind::Implements => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((implemented_type, _)) = split_tag_content(description_str, description_span) {
                        template_implements.push(implemented_type);
                    }
                }
                TagKind::ConsistentConstructor | TagKind::PsalmConsistentConstructor => {
                    has_consistent_constructor = true;
                }
                TagKind::PsalmConsistentTemplates => {
                    has_consistent_templates = true;
                }
                TagKind::RequireExtends | TagKind::PhpstanRequireExtends | TagKind::PsalmRequireExtends => {
                    require_extends.push(TypeString { value: tag.description.to_string(), span: tag.description_span });
                }
                TagKind::RequireImplements | TagKind::PhpstanRequireImplements | TagKind::PsalmRequireImplements => {
                    require_implements
                        .push(TypeString { value: tag.description.to_string(), span: tag.description_span });
                }
                TagKind::Method | TagKind::PsalmMethod => {
                    if let Some(method) = parse_method_tag(tag.description, tag.description_span) {
                        methods.push(method);
                    }
                }
                TagKind::Property | TagKind::PsalmProperty => {
                    if let Some(property_tag) = parse_property_tag(tag.description, tag.description_span, true, true) {
                        properties.push(property_tag);
                    }
                }
                TagKind::PropertyRead | TagKind::PsalmPropertyRead => {
                    if let Some(property_tag) = parse_property_tag(tag.description, tag.description_span, true, false) {
                        properties.push(property_tag);
                    }
                }
                TagKind::PropertyWrite | TagKind::PsalmPropertyWrite => {
                    if let Some(property_tag) = parse_property_tag(tag.description, tag.description_span, false, true) {
                        properties.push(property_tag);
                    }
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
            is_internal,
            is_enum_interface,
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
            methods,
            properties,
        }))
    }
}

impl FunctionLikeDocblockComment {
    pub fn create(
        context: &Context<'_, '_>,
        function: impl HasSpan,
        scope: &mut NamespaceScope,
    ) -> Result<Option<FunctionLikeDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(function) else {
            return Ok(None);
        };

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_pure = false;
        let mut ignore_nullable_return = false;
        let mut ignore_falsable_return = false;
        let mut inherits_docs = false;
        let mut no_named_arguments = false;
        let mut generic_return_type: Option<ReturnTypeTag> = None;
        let mut psalm_return_type: Option<ReturnTypeTag> = None;
        let mut phpstan_return_type: Option<ReturnTypeTag> = None;
        let mut parameters: Vec<ParameterTag> = Vec::new();
        let mut parameters_out: Vec<ParameterOutTag> = Vec::new();
        let mut where_constraints: Vec<WhereTag> = Vec::new();
        let mut throws: Vec<ThrowsTag> = Vec::new();
        let mut templates: Vec<TemplateTag> = Vec::new();
        let mut assertions: Vec<AssertionTag> = Vec::new();
        let mut if_true_assertions: Vec<AssertionTag> = Vec::new();
        let mut if_false_assertions: Vec<AssertionTag> = Vec::new();
        let mut unchecked = false;
        let mut must_use = false;

        let parsed_docblock = parse_trivia(context.arena, docblock)?;

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::Unchecked | TagKind::MagoUnchecked => {
                    unchecked = true;
                }
                TagKind::MustUse => {
                    must_use = true;
                }
                TagKind::Deprecated => {
                    is_deprecated = true;
                }
                TagKind::Internal | TagKind::PsalmInternal => {
                    is_internal = true;
                }
                TagKind::PhpstanParam | TagKind::PsalmParam | TagKind::Param => {
                    if let Some(param) = parse_param_tag(tag.description, tag.description_span) {
                        parameters.push(param);
                    }
                }
                TagKind::NoNamedArguments => {
                    no_named_arguments = true;
                }
                TagKind::PhpstanTemplate
                | TagKind::PsalmTemplate
                | TagKind::Template
                | TagKind::TemplateInvariant
                | TagKind::PhpstanTemplateInvariant
                | TagKind::PsalmTemplateInvariant => {
                    if let Some(t) = parse_template_tag(tag.description, tag.description_span, false, false) {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::TemplateCovariant | TagKind::PhpstanTemplateCovariant | TagKind::PsalmTemplateCovariant => {
                    if let Some(t) = parse_template_tag(tag.description, tag.description_span, true, false) {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::TemplateContravariant
                | TagKind::PhpstanTemplateContravariant
                | TagKind::PsalmTemplateContravariant => {
                    if let Some(t) = parse_template_tag(tag.description, tag.description_span, false, true) {
                        scope.add(NameKind::Default, &t.name, None as Option<&str>);

                        templates.push(t);
                    }
                }
                TagKind::Return => {
                    if let Some(return_tag) = parse_return_tag(tag.description, tag.description_span) {
                        generic_return_type = Some(return_tag);
                    }
                }
                TagKind::PhpstanReturn => {
                    if let Some(return_tag) = parse_return_tag(tag.description, tag.description_span) {
                        phpstan_return_type = Some(return_tag);
                    }
                }
                TagKind::PsalmReturn => {
                    if let Some(return_tag) = parse_return_tag(tag.description, tag.description_span) {
                        psalm_return_type = Some(return_tag);
                    }
                }
                TagKind::Throws => {
                    if let Some(throws_tag) = parse_throws_tag(tag.description, tag.description_span) {
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
                    if let Some(param_out) = parse_param_out_tag(tag.description, tag.description_span) {
                        parameters_out.push(param_out);
                    }
                }
                TagKind::Assert | TagKind::PsalmAssert | TagKind::PhpstanAssert => {
                    if let Some(assertion) = parse_assertion_tag(tag.description, tag.description_span) {
                        assertions.push(assertion);
                    }
                }
                TagKind::AssertIfTrue | TagKind::PsalmAssertIfTrue | TagKind::PhpstanAssertIfTrue => {
                    if let Some(assertion) = parse_assertion_tag(tag.description, tag.description_span) {
                        if_true_assertions.push(assertion);
                    }
                }
                TagKind::AssertIfFalse | TagKind::PsalmAssertIfFalse | TagKind::PhpstanAssertIfFalse => {
                    if let Some(assertion) = parse_assertion_tag(tag.description, tag.description_span) {
                        if_false_assertions.push(assertion);
                    }
                }
                TagKind::Where => {
                    if let Some(where_tag) = parse_where_tag(tag.description, tag.description_span) {
                        where_constraints.push(where_tag);
                    }
                }
                TagKind::IgnoreNullableReturn | TagKind::PsalmIgnoreNullableReturn => {
                    ignore_nullable_return = true;
                }
                TagKind::IgnoreFalsableReturn | TagKind::PsalmIgnoreFalsableReturn => {
                    ignore_falsable_return = true;
                }
                TagKind::InheritDoc => {
                    inherits_docs = true;
                }
                _ => {
                    // Ignore other tags
                }
            }
        }

        Ok(Some(FunctionLikeDocblockComment {
            span: docblock.span,
            is_deprecated,
            is_internal,
            is_pure,
            ignore_nullable_return,
            ignore_falsable_return,
            inherits_docs,
            no_named_arguments,
            return_type: psalm_return_type.or(phpstan_return_type).or(generic_return_type),
            parameters,
            parameters_out,
            where_constraints,
            throws,
            templates,
            assertions,
            if_true_assertions,
            if_false_assertions,
            must_use,
            unchecked,
        }))
    }
}

impl PropertyDocblockComment {
    pub fn create(
        context: &Context<'_, '_>,
        property: impl HasSpan,
    ) -> Result<Option<PropertyDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(property) else {
            return Ok(None);
        };

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_readonly = false;
        let mut generic_type_string: Option<TypeString> = None;
        let mut phpstan_type_string: Option<TypeString> = None;
        let mut psalm_type_string: Option<TypeString> = None;

        let parsed_docblock = parse_trivia(context.arena, docblock)?;

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
                TagKind::PsalmVar => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        psalm_type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::PhpstanVar => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        phpstan_type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::Var => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        generic_type_string = Some(type_string_tag.0);
                    }
                }
                _ => {}
            }
        }

        Ok(Some(PropertyDocblockComment {
            span: docblock.span,
            type_string: psalm_type_string.or(phpstan_type_string).or(generic_type_string),
            is_deprecated,
            is_internal,
            is_readonly,
        }))
    }
}

impl ConstantDocblockComment {
    pub fn create(
        context: &Context<'_, '_>,
        constant: impl HasSpan,
    ) -> Result<Option<ConstantDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(constant) else {
            return Ok(None);
        };

        let mut is_deprecated = false;
        let mut is_internal = false;
        let mut is_final = false;

        let mut generic_type_string: Option<TypeString> = None;
        let mut phpstan_type_string: Option<TypeString> = None;
        let mut psalm_type_string: Option<TypeString> = None;

        let parsed_docblock = parse_trivia(context.arena, docblock)?;

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
                TagKind::PsalmVar => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        psalm_type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::PhpstanVar => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        phpstan_type_string = Some(type_string_tag.0);
                    }
                }
                TagKind::Var => {
                    if let Some(type_string_tag) = split_tag_content(tag.description, tag.description_span) {
                        generic_type_string = Some(type_string_tag.0);
                    }
                }
                _ => {}
            }
        }

        Ok(Some(ConstantDocblockComment {
            span: docblock.span,
            is_deprecated,
            is_internal,
            is_final,
            type_string: psalm_type_string.or(phpstan_type_string).or(generic_type_string),
        }))
    }
}

impl TraitUseDocblockComment {
    pub fn create(
        context: &Context<'_, '_>,
        trait_use: impl HasSpan,
    ) -> Result<Option<TraitUseDocblockComment>, ParseError> {
        let Some(docblock) = context.get_docblock(trait_use) else {
            return Ok(None);
        };

        let mut template_extends = Vec::new();
        let mut template_implements = Vec::new();
        let mut template_use = Vec::new();

        let parsed_docblock = parse_trivia(context.arena, docblock)?;

        for element in parsed_docblock.elements {
            let Element::Tag(tag) = element else {
                continue;
            };

            match tag.kind {
                TagKind::TemplateExtends | TagKind::Extends => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((extended_type, _)) = split_tag_content(description_str, description_span) {
                        template_extends.push(extended_type);
                    }
                }
                TagKind::TemplateImplements | TagKind::Implements => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((implemented_type, _)) = split_tag_content(description_str, description_span) {
                        template_implements.push(implemented_type);
                    }
                }
                TagKind::Use | TagKind::TemplateUse => {
                    let description_str = tag.description;
                    let description_span = tag.description_span;

                    if let Some((used_type, _)) = split_tag_content(description_str, description_span) {
                        template_use.push(used_type);
                    }
                }
                _ => {}
            }
        }

        Ok(Some(TraitUseDocblockComment { template_extends, template_implements, template_use }))
    }
}
