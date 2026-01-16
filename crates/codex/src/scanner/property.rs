use mago_atom::Atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::NullSafePropertyAccess;
use mago_syntax::ast::Property;
use mago_syntax::ast::PropertyAccess;
use mago_syntax::ast::PropertyHook;
use mago_syntax::ast::PropertyHookBody;
use mago_syntax::ast::PropertyItem;
use mago_syntax::ast::Variable;
use mago_syntax::walker::MutWalker;

use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::property_hook::PropertyHookMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::PropertyDocblockComment;
use crate::scanner::docblock::PropertyHookDocblockComment;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::scanner::ttype::merge_type_preserving_nullability;
use crate::ttype::resolution::TypeResolutionContext;
use crate::visibility::Visibility;

#[inline]
pub fn scan_promoted_property<'arena>(
    parameter: &'arena FunctionLikeParameter<'arena>,
    parameter_metadata: &mut FunctionLikeParameterMetadata,
    class_like_metadata: &mut ClassLikeMetadata,
    classname: Atom,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> PropertyMetadata {
    debug_assert!(parameter.is_promoted_property(), "Parameter is not a promoted property");

    let name = parameter_metadata.get_name();
    let name_span = parameter_metadata.get_name_span();

    let mut flags = MetadataFlags::PROMOTED_PROPERTY;
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    if parameter_metadata.flags.has_default() {
        flags |= MetadataFlags::HAS_DEFAULT;
    }

    if parameter.modifiers.contains_readonly() {
        flags |= MetadataFlags::READONLY;
    }

    if parameter.modifiers.contains_abstract() {
        flags |= MetadataFlags::ABSTRACT;
    }

    if parameter.modifiers.contains_static() {
        flags |= MetadataFlags::STATIC;
    }

    let default_type_metadata = parameter_metadata.get_default_type().cloned();

    let read_visibility = match parameter.modifiers.get_first_read_visibility() {
        Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
        None => Visibility::Public,
    };

    let write_visibility = match parameter.modifiers.get_first_write_visibility() {
        Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
        None => {
            if parameter.modifiers.contains_readonly() {
                Visibility::Protected
            } else {
                read_visibility
            }
        }
    };

    let mut property_metadata = PropertyMetadata::new(*name, flags);

    property_metadata.set_default_type_metadata(default_type_metadata);
    property_metadata.set_name_span(Some(name_span));
    property_metadata.set_span(Some(parameter.span()));
    property_metadata.set_visibility(read_visibility, write_visibility);
    property_metadata.set_type_declaration_metadata(
        parameter.hint.as_ref().map(|hint| get_type_metadata_from_hint(hint, Some(class_like_metadata.name), context)),
    );

    if let Some(hook_list) = &parameter.hooks {
        for hook in &hook_list.hooks {
            let mut hook_metadata = scan_property_hook(hook, &property_metadata, context, scope);
            class_like_metadata.issues.extend(hook_metadata.take_issues());
            property_metadata.hooks.insert(hook_metadata.name, hook_metadata);
        }

        let prop_name = name.0.strip_prefix('$').unwrap_or(&name.0);
        property_metadata.set_is_virtual(!hooks_reference_backing_store(&hook_list.hooks, prop_name));
    }

    let mut used_parameter_type_from_docblock = false;
    if let Some(type_metadata) = parameter_metadata.type_metadata.as_ref()
        && type_metadata.from_docblock
    {
        used_parameter_type_from_docblock = true;
        property_metadata.type_metadata = Some(type_metadata.clone());
    }

    // Check for inline @var docblock comment on the parameter
    match PropertyDocblockComment::create(context, parameter) {
        Ok(Some(docblock)) => {
            update_property_metadata_from_docblock(
                &mut property_metadata,
                &docblock,
                classname,
                type_context,
                scope,
                class_like_metadata,
            );

            if let Some(type_metadata) = property_metadata.type_metadata.as_ref()
                && type_metadata.from_docblock
                && !used_parameter_type_from_docblock
            {
                parameter_metadata.type_metadata = Some(type_metadata.clone());
            }
        }
        Ok(None) => {
            // No docblock comment found; do nothing
        }
        Err(parse_error) => {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse promoted property docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );
        }
    }

    property_metadata
}

#[inline]
pub fn scan_properties<'arena>(
    property: &'arena Property<'arena>,
    class_like_metadata: &mut ClassLikeMetadata,
    classname: Atom,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> Vec<PropertyMetadata> {
    let docblock = match PropertyDocblockComment::create(context, property) {
        Ok(docblock) => docblock,
        Err(parse_error) => {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse property docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            None
        }
    };

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    match property {
        Property::Plain(plain_property) => plain_property
            .items
            .iter()
            .map(|item| {
                let (name, name_span, has_default, default_type) = scan_property_item(item, context, scope);

                let mut flags = flags;

                if has_default {
                    flags |= MetadataFlags::HAS_DEFAULT;
                }

                if plain_property.modifiers.contains_readonly() {
                    flags |= MetadataFlags::READONLY;
                }

                if plain_property.modifiers.contains_abstract() {
                    flags |= MetadataFlags::ABSTRACT;
                }

                if plain_property.modifiers.contains_static() {
                    flags |= MetadataFlags::STATIC;
                }

                if plain_property.modifiers.contains_final() {
                    flags |= MetadataFlags::FINAL;
                }

                let read_visibility = match plain_property.modifiers.get_first_read_visibility() {
                    Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                    None => Visibility::Public,
                };

                let write_visibility = match plain_property.modifiers.get_first_write_visibility() {
                    Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                    None => {
                        if plain_property.modifiers.contains_readonly() {
                            Visibility::Protected
                        } else {
                            read_visibility
                        }
                    }
                };

                let mut metadata = PropertyMetadata::new(name, flags);

                metadata.set_name_span(Some(name_span));
                metadata.set_default_type_metadata(default_type);
                metadata.set_visibility(read_visibility, write_visibility);
                metadata.set_type_declaration_metadata(
                    plain_property
                        .hint
                        .as_ref()
                        .map(|hint| get_type_metadata_from_hint(hint, Some(class_like_metadata.name), context)),
                );

                if let Some(docblock) = docblock.as_ref() {
                    update_property_metadata_from_docblock(
                        &mut metadata,
                        docblock,
                        classname,
                        type_context,
                        scope,
                        class_like_metadata,
                    );

                    metadata
                } else {
                    metadata
                }
            })
            .collect(),
        Property::Hooked(hooked_property) => {
            let (name, name_span, has_default, default_type) =
                scan_property_item(&hooked_property.item, context, scope);

            let read_visibility = match hooked_property.modifiers.get_first_read_visibility() {
                Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                None => Visibility::Public,
            };

            let write_visibility = match hooked_property.modifiers.get_first_write_visibility() {
                Some(visibility) => Visibility::try_from(visibility).unwrap_or(Visibility::Public),
                None => read_visibility,
            };

            if has_default {
                flags |= MetadataFlags::HAS_DEFAULT;
            }

            if hooked_property.modifiers.contains_abstract() {
                flags |= MetadataFlags::ABSTRACT;
            }

            if hooked_property.modifiers.contains_final() {
                flags |= MetadataFlags::FINAL;
            }

            let mut metadata = PropertyMetadata::new(name, flags);

            metadata.set_name_span(Some(name_span));
            metadata.set_default_type_metadata(default_type);
            metadata.set_span(Some(hooked_property.span()));
            metadata.set_visibility(read_visibility, write_visibility);
            metadata.set_type_declaration_metadata(
                hooked_property
                    .hint
                    .as_ref()
                    .map(|hint| get_type_metadata_from_hint(hint, Some(class_like_metadata.name), context)),
            );

            if let Some(docblock) = docblock.as_ref() {
                update_property_metadata_from_docblock(
                    &mut metadata,
                    docblock,
                    classname,
                    type_context,
                    scope,
                    class_like_metadata,
                );
            }

            for hook in &hooked_property.hook_list.hooks {
                let mut hook_metadata = scan_property_hook(hook, &metadata, context, scope);
                class_like_metadata.issues.extend(hook_metadata.take_issues());
                metadata.hooks.insert(hook_metadata.name, hook_metadata);
            }

            let prop_name = name.0.strip_prefix('$').unwrap_or(&name.0);
            metadata.set_is_virtual(!hooks_reference_backing_store(&hooked_property.hook_list.hooks, prop_name));

            vec![metadata]
        }
    }
}

fn scan_property_hook<'arena>(
    hook: &'arena PropertyHook<'arena>,
    property_metadata: &PropertyMetadata,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> PropertyHookMetadata {
    let name = atom(hook.name.value);
    let is_get = hook.name.value == "get";
    let is_set = hook.name.value == "set";
    let is_abstract = matches!(hook.body, PropertyHookBody::Abstract(_));
    let has_explicit_parameter = hook.parameter_list.is_some();

    let mut flags = MetadataFlags::empty();
    if hook.modifiers.contains_final() {
        flags |= MetadataFlags::FINAL;
    }

    let mut parameter = if is_set {
        if let Some(param_list) = &hook.parameter_list {
            param_list.parameters.first().map(|p| scan_hook_parameter(p, property_metadata, context))
        } else {
            Some(create_implicit_value_parameter(property_metadata, hook.span()))
        }
    } else {
        None
    };

    let attributes = scan_attribute_lists(&hook.attribute_lists, context);

    let mut has_docblock = false;
    let mut return_type_metadata = None;
    let mut issues = Vec::new();

    match PropertyHookDocblockComment::create(context, hook) {
        Ok(None) => {
            // No docblock, nothing to do
        }
        Ok(Some(docblock)) => {
            has_docblock = true;

            if let Some(param_tag) = &docblock.param_type_string {
                if !has_explicit_parameter {
                    issues.push(
                        Issue::error("The `@param` tag cannot be used on a set hook without an explicit parameter.")
                            .with_code(ScanningIssueKind::InvalidParamTag)
                            .with_annotation(
                                Annotation::primary(param_tag.span)
                                    .with_message("This @param cannot be applied to implicit `$value` parameter"),
                            )
                            .with_note("Set hooks without an explicit parameter use an implicit `$value` parameter that inherits the property type.")
                            .with_help("Either add an explicit parameter `set(Type $value) {}` or remove the @param tag."),
                    );
                } else if let Some(ref mut param) = parameter
                    && let Some(type_string) = &param_tag.type_string
                {
                    let type_context = TypeResolutionContext::new();
                    match get_type_metadata_from_type_string(type_string, None, &type_context, scope) {
                        Ok(docblock_type) => {
                            let native_type = param.type_declaration_metadata.as_ref();
                            let merged = merge_type_preserving_nullability(docblock_type, native_type);
                            param.set_type_metadata(Some(merged));
                        }
                        Err(typing_error) => {
                            issues.push(
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
                }
            }

            if let Some(return_type_string) = &docblock.return_type_string
                && is_get
            {
                let type_context = TypeResolutionContext::new();
                match get_type_metadata_from_type_string(return_type_string, None, &type_context, scope) {
                    Ok(docblock_type) => {
                        return_type_metadata = Some(docblock_type);
                    }
                    Err(typing_error) => {
                        issues.push(
                            Issue::error("Could not resolve the type for the @return tag.")
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
        }
        Err(parse_error) => {
            issues.push(
                Issue::error("Failed to parse property hook docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );
        }
    }

    PropertyHookMetadata::new(name, hook.span())
        .with_flags(flags)
        .with_parameter(parameter)
        .with_returns_by_ref(hook.ampersand.is_some())
        .with_is_abstract(is_abstract)
        .with_attributes(attributes)
        .with_return_type_metadata(return_type_metadata)
        .with_has_docblock(has_docblock)
        .with_issues(issues)
}

fn scan_hook_parameter<'arena>(
    param: &'arena FunctionLikeParameter<'arena>,
    property_metadata: &PropertyMetadata,
    context: &mut Context<'_, 'arena>,
) -> FunctionLikeParameterMetadata {
    let name = VariableIdentifier(atom(param.variable.name));
    let name_span = param.variable.span;

    let mut flags = MetadataFlags::empty();
    if param.ampersand.is_some() {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut param_metadata = FunctionLikeParameterMetadata::new(name, param.span(), name_span, flags);

    if let Some(hint) = &param.hint {
        let type_meta = get_type_metadata_from_hint(hint, None, context);
        param_metadata.set_type_declaration_metadata(Some(type_meta));
    } else if let Some(prop_type) = &property_metadata.type_metadata {
        param_metadata.set_type_declaration_metadata(Some(prop_type.clone()));
    }

    param_metadata
}

fn create_implicit_value_parameter(property_metadata: &PropertyMetadata, span: Span) -> FunctionLikeParameterMetadata {
    let name = VariableIdentifier(atom("$value"));
    let mut param = FunctionLikeParameterMetadata::new(name, span, span, MetadataFlags::empty());

    if let Some(type_meta) = &property_metadata.type_metadata {
        param.set_type_declaration_metadata(Some(type_meta.clone()));
    }

    param
}

#[inline]
pub fn scan_property_item<'arena>(
    property_item: &'arena PropertyItem<'arena>,
    context: &mut Context<'_, 'arena>,
    scope: &NamespaceScope,
) -> (VariableIdentifier, Span, bool, Option<TypeMetadata>) {
    match property_item {
        PropertyItem::Abstract(property_abstract_item) => {
            let name = VariableIdentifier(atom(property_abstract_item.variable.name));
            let name_span = property_abstract_item.variable.span;
            let has_default = false;
            let default_type = None;

            (name, name_span, has_default, default_type)
        }
        PropertyItem::Concrete(property_concrete_item) => {
            let name = VariableIdentifier(atom(property_concrete_item.variable.name));
            let name_span = property_concrete_item.variable.span;
            let has_default = true;
            let default_type = infer(context, scope, &property_concrete_item.value).map(|u| {
                let mut type_metadata = TypeMetadata::new(u, property_concrete_item.value.span());
                type_metadata.inferred = true;
                type_metadata
            });

            (name, name_span, has_default, default_type)
        }
    }
}

fn update_property_metadata_from_docblock(
    property_metadata: &mut PropertyMetadata,
    docblock: &PropertyDocblockComment,
    classname: Atom,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
    class_like_metadata: &mut ClassLikeMetadata,
) {
    if docblock.is_internal {
        property_metadata.flags |= MetadataFlags::INTERNAL;
    }

    if docblock.is_deprecated {
        property_metadata.flags |= MetadataFlags::DEPRECATED;
    }

    if docblock.is_readonly {
        property_metadata.flags |= MetadataFlags::READONLY;
    }

    if let Some(type_string) = &docblock.type_string {
        match get_type_metadata_from_type_string(type_string, Some(classname), type_context, scope) {
            Ok(property_type_metadata) => {
                let real_type = property_metadata.type_declaration_metadata.as_ref();
                let property_type_metadata = merge_type_preserving_nullability(property_type_metadata, real_type);

                property_metadata.set_type_metadata(Some(property_type_metadata));
            }
            Err(typing_error) => class_like_metadata.issues.push(
                Issue::error("Could not resolve the type for the @var tag.")
                    .with_code(ScanningIssueKind::InvalidVarTag)
                    .with_annotation(Annotation::primary(typing_error.span()).with_message(typing_error.to_string()))
                    .with_note(typing_error.note())
                    .with_help(typing_error.help()),
            ),
        }
    }
}

/// Checks if any hook references `$this->propertyName` (indicating a backed property).
fn hooks_reference_backing_store<'arena>(
    hooks: impl IntoIterator<Item = &'arena PropertyHook<'arena>>,
    property_name: &str,
) -> bool {
    struct Walker<'arena> {
        property_name: &'arena str,
        found: bool,
    }

    impl<'arena> Walker<'arena> {
        fn new(property_name: &'arena str) -> Self {
            Self { property_name, found: false }
        }

        fn check_access<'ast>(
            &mut self,
            object: &'ast Expression<'arena>,
            property: &'ast ClassLikeMemberSelector<'arena>,
        ) {
            if self.found {
                return;
            }

            let Expression::Variable(Variable::Direct(direct_variable)) = object else {
                return;
            };

            if direct_variable.name != "$this" {
                return;
            }

            let ClassLikeMemberSelector::Identifier(identifier) = property else {
                return;
            };

            if identifier.value == self.property_name {
                self.found = true;
            }
        }
    }

    impl<'ast, 'arena> MutWalker<'ast, 'arena, ()> for Walker<'arena> {
        fn walk_in_property_access(&mut self, access: &'ast PropertyAccess<'arena>, _: &mut ()) {
            self.check_access(access.object, &access.property);
        }

        fn walk_in_null_safe_property_access(&mut self, access: &'ast NullSafePropertyAccess<'arena>, _: &mut ()) {
            self.check_access(access.object, &access.property);
        }
    }

    let mut walker = Walker::new(property_name);
    for hook in hooks {
        walker.walk_property_hook_body(&hook.body, &mut ());
        if walker.found {
            return true;
        }
    }

    false
}
