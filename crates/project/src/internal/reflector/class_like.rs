use ahash::HashMap;

use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::class_like::constant::ClassLikeConstantReflection;
use mago_reflection::class_like::enum_case::EnumCaseReflection;
use mago_reflection::class_like::inheritance::InheritanceReflection;
use mago_reflection::class_like::member::ClassLikeMemberVisibilityReflection;
use mago_reflection::class_like::property::PropertyDefaultValueReflection;
use mago_reflection::class_like::property::PropertyReflection;
use mago_reflection::function_like::FunctionLikeReflection;
use mago_reflection::identifier::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::context::Context;
use crate::internal::reflector::attribute::reflect_attributes;
use crate::internal::reflector::function_like::reflect_function_like_parameter_list;
use crate::internal::reflector::function_like::reflect_function_like_return_type_hint;
use crate::internal::reflector::r#type::maybe_reflect_hint;
use crate::internal::reflector::r#type::reflect_hint;

use super::should_reflect_element;

#[inline]
pub fn reflect_class<'ast>(class: &'ast Class, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let name = ClassLikeName::Class(Name::new(*context.names.get(&class.name), class.name.span));
    let span = class.span();

    let mut reflection = ClassLikeReflection::new(name, span);
    reflection.is_final = class.modifiers.contains_final();
    reflection.is_readonly = class.modifiers.contains_readonly();
    reflection.is_abstract = class.modifiers.contains_abstract();
    reflection.attribute_reflections = reflect_attributes(&class.attribute_lists, context);
    reflection.inheritance = {
        let mut inheritance_reflection = InheritanceReflection::default();
        if let Some(extends) = &class.extends {
            if let Some(first_parent) = extends.types.first() {
                let parent = Name::new(*context.names.get(first_parent), first_parent.span());
                let parent_lowered = context.interner.lowered(&parent.value);

                inheritance_reflection.direct_extended_class = Some(parent);
                inheritance_reflection.all_extended_classes.insert(parent);
                inheritance_reflection.names.insert(parent_lowered, parent);
            }
        }

        if let Some(impelemnts) = &class.implements {
            for interface in impelemnts.types.iter() {
                let interface = Name::new(*context.names.get(interface), interface.span());
                let interface_lowered = context.interner.lowered(&interface.value);

                inheritance_reflection.direct_implemented_interfaces.insert(interface);
                inheritance_reflection.all_implemented_interfaces.insert(interface);
                inheritance_reflection.names.insert(interface.value, interface);
                inheritance_reflection.names.insert(interface_lowered, interface);
            }
        }

        inheritance_reflection
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

#[inline]
pub fn reflect_anonymous_class<'ast>(
    class: &'ast AnonymousClass,
    context: &'ast mut Context<'_>,
) -> ClassLikeReflection {
    let name = ClassLikeName::AnonymousClass(class.span());
    let span = class.span();

    let mut reflection = ClassLikeReflection::new(name, span);
    reflection.is_anonymous = true;
    reflection.is_final = class.modifiers.contains_final();
    reflection.is_readonly = class.modifiers.contains_readonly();
    reflection.is_abstract = class.modifiers.contains_abstract();
    reflection.attribute_reflections = reflect_attributes(&class.attribute_lists, context);
    reflection.inheritance = {
        let mut inheritance_reflection = InheritanceReflection::default();
        if let Some(extends) = &class.extends {
            if let Some(first_parent) = extends.types.first() {
                let parent = Name::new(*context.names.get(first_parent), first_parent.span());
                let parent_lowered = context.interner.lowered(&parent.value);

                inheritance_reflection.direct_extended_class = Some(parent);
                inheritance_reflection.all_extended_classes.insert(parent);
                inheritance_reflection.names.insert(parent_lowered, parent);
            }
        }

        if let Some(impelemnts) = &class.implements {
            for interface in impelemnts.types.iter() {
                let interface = Name::new(*context.names.get(interface), interface.span());
                let interface_lowered = context.interner.lowered(&interface.value);

                inheritance_reflection.direct_implemented_interfaces.insert(interface);
                inheritance_reflection.all_implemented_interfaces.insert(interface);
                inheritance_reflection.names.insert(interface.value, interface);
                inheritance_reflection.names.insert(interface_lowered, interface);
            }
        }

        inheritance_reflection
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

#[inline]
pub fn reflect_interface<'ast>(interface: &'ast Interface, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let name = ClassLikeName::Interface(Name::new(*context.names.get(&interface.name), interface.name.span()));
    let span = interface.span();

    let mut reflection = ClassLikeReflection::new(name, span);
    reflection.is_abstract = true;
    reflection.attribute_reflections = reflect_attributes(&interface.attribute_lists, context);
    reflection.inheritance = {
        let mut inheritance_reflection = InheritanceReflection::default();
        if let Some(extends) = &interface.extends {
            for interface in extends.types.iter() {
                let interface = Name::new(*context.names.get(interface), interface.span());
                let interface_lowered = context.interner.lowered(&interface.value);

                inheritance_reflection.direct_extended_interfaces.insert(interface);
                inheritance_reflection.all_extended_interfaces.insert(interface);
                inheritance_reflection.names.insert(interface_lowered, interface);
            }
        }

        inheritance_reflection
    };

    reflect_class_like_members(&mut reflection, &interface.members, context);

    reflection
}

#[inline]
pub fn reflect_trait<'ast>(r#trait: &'ast Trait, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let name = ClassLikeName::Trait(Name::new(*context.names.get(&r#trait.name), r#trait.name.span()));
    let span = r#trait.span();

    let mut reflection = ClassLikeReflection::new(name, span);
    reflection.is_abstract = true;
    reflection.attribute_reflections = reflect_attributes(&r#trait.attribute_lists, context);

    reflect_class_like_members(&mut reflection, &r#trait.members, context);

    reflection
}

#[inline]
pub fn reflect_enum<'ast>(r#enum: &'ast Enum, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let name = ClassLikeName::Enum(Name::new(*context.names.get(&r#enum.name), r#enum.name.span()));
    let span = r#enum.span();

    let mut reflection = ClassLikeReflection::new(name, span);
    reflection.is_final = true;
    reflection.is_readonly = true;
    reflection.attribute_reflections = reflect_attributes(&r#enum.attribute_lists, context);
    reflection.inheritance = {
        let mut inheritance_reflection = InheritanceReflection::default();
        if let Some(impelemnts) = &r#enum.implements {
            for interface in impelemnts.types.iter() {
                let interface = Name::new(*context.names.get(interface), interface.span());
                let interface_lowered = context.interner.lowered(&interface.value);

                inheritance_reflection.direct_implemented_interfaces.insert(interface);
                inheritance_reflection.all_implemented_interfaces.insert(interface);
                inheritance_reflection.names.insert(interface_lowered, interface);
            }
        }

        inheritance_reflection
    };

    if let Some(backing_type_hint) = &r#enum.backing_type_hint {
        reflection.backing_type = Some(reflect_hint(&backing_type_hint.hint, context, None));
    }

    reflect_class_like_members(&mut reflection, &r#enum.members, context);

    reflection
}

#[inline]
fn reflect_class_like_members<'ast>(
    reflection: &mut ClassLikeReflection,
    members: &'ast Sequence<ClassLikeMember>,
    context: &'ast mut Context<'_>,
) {
    for member in members.iter() {
        match &member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_name in trait_use.trait_names.iter() {
                    let trait_id = *context.names.get(trait_name);
                    let lower_trait_id = context.interner.lowered(&trait_id);
                    let name = Name::new(lower_trait_id, trait_name.span());

                    reflection.used_traits.insert(name);
                    reflection.used_trait_names.insert(name.value, name);
                }
            }
            ClassLikeMember::Constant(class_like_constant) => {
                let const_refs = reflect_class_like_constant(reflection, class_like_constant, context);
                for const_ref in const_refs {
                    reflection.constants.insert(const_ref.name.member.value, const_ref);
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_ref = reflect_class_like_enum_case(reflection, enum_case, context);

                reflection.cases.insert(case_ref.name.member.value, case_ref);
            }
            ClassLikeMember::Method(method) => {
                if let Some((name, meth_ref)) = reflect_class_like_method(reflection, method, context) {
                    let lowercase_name = context.interner.lowered(&name.value);

                    // `__construct`, `__clone`, and trait methods are always inheritable
                    let name_value = context.interner.lookup(&lowercase_name);
                    if meth_ref.visibility_reflection.map(|v| !v.is_private()).unwrap_or(true)
                        || name_value.eq("__construct")
                        || name_value.eq("__clone")
                        || reflection.is_trait()
                    {
                        reflection.methods.inheritable_members.insert(lowercase_name, reflection.name);
                    }

                    reflection.methods.members.insert(lowercase_name, meth_ref);
                }
            }
            ClassLikeMember::Property(property) => {
                let prop_refs = reflect_class_like_property(reflection, property, context);
                for prop_ref in prop_refs {
                    let name = prop_ref.name.member.value;

                    if prop_ref.read_visibility_reflection.map(|v| !v.is_private()).unwrap_or(true) {
                        reflection.properties.inheritable_members.insert(name, reflection.name);
                    }

                    reflection.properties.members.insert(name, prop_ref);
                }
            }
        }
    }
}

#[inline]
fn reflect_class_like_constant<'ast>(
    class_like: &mut ClassLikeReflection,
    constant: &'ast ClassLikeConstant,
    context: &'ast mut Context<'_>,
) -> Vec<ClassLikeConstantReflection> {
    let attribute_reflections = reflect_attributes(&constant.attribute_lists, context);
    let visibility_reflection = modifier_to_visibility(constant.modifiers.get_first_read_visibility());
    let type_reflection = maybe_reflect_hint(&constant.hint, context, Some(class_like));
    let is_final = constant.modifiers.contains_final();

    constant
        .items
        .iter()
        .map(|item| ClassLikeConstantReflection {
            attribute_reflections: attribute_reflections.clone(),
            visibility_reflection,
            type_reflection: type_reflection.clone(),
            name: ClassLikeMemberName {
                class_like: class_like.name,
                member: Name::new(item.name.value, item.name.span),
            },
            is_final,
            inferred_type_reflection: mago_typing::infere(context.interner, context.source, context.names, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
        })
        .collect()
}

#[inline]
fn reflect_class_like_enum_case<'ast>(
    class_like: &mut ClassLikeReflection,
    case: &'ast EnumCase,
    context: &'ast mut Context<'_>,
) -> EnumCaseReflection {
    let (identifier, type_reflection, is_backed) = match &case.item {
        EnumCaseItem::Unit(enum_case_unit_item) => (
            ClassLikeMemberName {
                class_like: class_like.name,
                member: Name::new(enum_case_unit_item.name.value, enum_case_unit_item.name.span),
            },
            None,
            false,
        ),
        EnumCaseItem::Backed(enum_case_backed_item) => (
            ClassLikeMemberName {
                class_like: class_like.name,
                member: Name::new(enum_case_backed_item.name.value, enum_case_backed_item.name.span),
            },
            Some(mago_typing::infere(context.interner, context.source, context.names, &enum_case_backed_item.value)),
            true,
        ),
    };

    EnumCaseReflection {
        attribut_reflections: reflect_attributes(&case.attribute_lists, context),
        name: identifier,
        type_reflection,
        is_backed,
        span: case.span(),
    }
}

#[inline]
fn reflect_class_like_method<'ast>(
    class_like: &mut ClassLikeReflection,
    method: &'ast Method,
    context: &'ast mut Context<'_>,
) -> Option<(Name, FunctionLikeReflection)> {
    if !should_reflect_element(context, &method.attribute_lists) {
        return None;
    }

    let name = Name::new(method.name.value, method.name.span);

    let (has_yield, has_throws, is_abstract) = match &method.body {
        MethodBody::Abstract(_) => (false, false, true),
        MethodBody::Concrete(block) => {
            (mago_syntax::utils::block_has_yield(block), mago_syntax::utils::block_has_throws(block), false)
        }
    };

    let visibility_reflection = modifier_to_visibility(method.modifiers.get_first_read_visibility());

    Some((
        name,
        FunctionLikeReflection {
            attribute_reflections: reflect_attributes(&method.attribute_lists, context),
            visibility_reflection,
            name: FunctionLikeName::Method(class_like.name, name),
            // TODO: parse docblock to get the template list
            templates: vec![],
            parameters: reflect_function_like_parameter_list(&method.parameter_list, context, Some(class_like)),
            return_type_reflection: reflect_function_like_return_type_hint(
                &method.return_type_hint,
                context,
                Some(class_like),
            ),
            returns_by_reference: method.ampersand.is_some(),
            has_yield,
            has_throws,
            is_anonymous: false,
            // TODO: parse docblock to determine if pure
            is_pure: false,
            is_static: method.modifiers.contains_static(),
            is_final: class_like.is_final || method.modifiers.contains_final(),
            is_abstract,
            is_overriding: false,
            span: method.span(),
            is_populated: false,
            issues: Default::default(),
        },
    ))
}

#[inline]
fn reflect_class_like_property<'ast>(
    class_like: &mut ClassLikeReflection,
    property: &'ast Property,
    context: &'ast mut Context<'_>,
) -> Vec<PropertyReflection> {
    let mut reflections = vec![];

    match &property {
        Property::Plain(plain_property) => {
            let attribut_reflections = reflect_attributes(&plain_property.attribute_lists, context);

            let read_visibility_reflection =
                modifier_to_visibility(plain_property.modifiers.get_first_read_visibility());
            let write_visibility_reflection =
                modifier_to_visibility(plain_property.modifiers.get_first_write_visibility());

            let type_reflection = maybe_reflect_hint(&plain_property.hint, context, Some(class_like));
            let is_readonly = class_like.is_readonly || plain_property.modifiers.contains_readonly();
            let is_final = class_like.is_final || plain_property.modifiers.contains_final();
            let is_static = plain_property.modifiers.contains_static();

            for item in plain_property.items.iter() {
                let (identifier, default_value_reflection) = match &item {
                    PropertyItem::Abstract(item) => (
                        ClassLikeMemberName {
                            class_like: class_like.name,
                            member: Name::new(item.variable.name, item.variable.span),
                        },
                        None,
                    ),
                    PropertyItem::Concrete(item) => (
                        ClassLikeMemberName {
                            class_like: class_like.name,
                            member: Name::new(item.variable.name, item.variable.span),
                        },
                        Some(PropertyDefaultValueReflection {
                            inferred_type_reflection: mago_typing::infere(
                                context.interner,
                                context.source,
                                context.names,
                                &item.value,
                            ),
                            span: item.value.span(),
                        }),
                    ),
                };

                reflections.push(PropertyReflection {
                    attribut_reflections: attribut_reflections.clone(),
                    read_visibility_reflection,
                    write_visibility_reflection,
                    name: identifier,
                    type_reflection: type_reflection.clone(),
                    default_value_reflection,
                    hooks: HashMap::default(),
                    is_readonly,
                    is_final,
                    is_promoted: false,
                    is_static,
                    item_span: item.span(),
                    definition_span: plain_property.span(),
                    is_overriding: false,
                })
            }
        }
        Property::Hooked(hooked_property) => {
            let read_visibility_reflection =
                modifier_to_visibility(hooked_property.modifiers.get_first_read_visibility());
            let write_visibility_reflection =
                modifier_to_visibility(hooked_property.modifiers.get_first_write_visibility());

            let (name, default_value_reflection) = match &hooked_property.item {
                PropertyItem::Abstract(item) => (
                    ClassLikeMemberName {
                        class_like: class_like.name,
                        member: Name::new(item.variable.name, item.variable.span),
                    },
                    None,
                ),
                PropertyItem::Concrete(item) => (
                    ClassLikeMemberName {
                        class_like: class_like.name,
                        member: Name::new(item.variable.name, item.variable.span),
                    },
                    Some(PropertyDefaultValueReflection {
                        inferred_type_reflection: mago_typing::infere(
                            context.interner,
                            context.source,
                            context.names,
                            &item.value,
                        ),
                        span: item.value.span(),
                    }),
                ),
            };

            reflections.push(PropertyReflection {
                attribut_reflections: reflect_attributes(&hooked_property.attribute_lists, context),
                read_visibility_reflection,
                write_visibility_reflection,
                name,
                type_reflection: maybe_reflect_hint(&hooked_property.hint, context, Some(class_like)),
                default_value_reflection,
                hooks: {
                    let mut map = HashMap::default();
                    for hook in hooked_property.hooks.hooks.iter() {
                        let hook_name = Name::new(hook.name.value, hook.name.span);

                        let function_like_name =
                            FunctionLikeName::PropertyHook(name.class_like, name.member, hook_name);

                        let (has_yield, has_throws) = match &hook.body {
                            PropertyHookBody::Abstract(_) => (false, false),
                            PropertyHookBody::Concrete(body) => match &body {
                                PropertyHookConcreteBody::Block(block) => (
                                    mago_syntax::utils::block_has_yield(block),
                                    mago_syntax::utils::block_has_throws(block),
                                ),
                                PropertyHookConcreteBody::Expression(body) => (
                                    mago_syntax::utils::expression_has_yield(&body.expression),
                                    mago_syntax::utils::expression_has_throws(&body.expression),
                                ),
                            },
                        };

                        map.insert(
                            hook_name.value,
                            FunctionLikeReflection {
                                attribute_reflections: reflect_attributes(&hook.attribute_lists, context),
                                name: function_like_name,
                                // TODO: parse docblock to get the template list
                                templates: vec![],
                                parameters: match hook.parameters.as_ref() {
                                    Some(parameters) => {
                                        reflect_function_like_parameter_list(parameters, context, Some(class_like))
                                    }
                                    None => vec![],
                                },
                                return_type_reflection: None,
                                returns_by_reference: hook.ampersand.is_some(),
                                has_yield,
                                has_throws,
                                is_anonymous: false,
                                is_static: false,
                                is_final: true,
                                is_pure: false,
                                is_abstract: false,
                                is_overriding: false,
                                span: hook.span(),
                                visibility_reflection: None,
                                is_populated: false,
                                issues: Default::default(),
                            },
                        );
                    }

                    map
                },
                is_readonly: class_like.is_readonly || hooked_property.modifiers.contains_readonly(),
                is_final: class_like.is_final || hooked_property.modifiers.contains_final(),
                is_promoted: false,
                is_static: false,
                item_span: hooked_property.item.span(),
                definition_span: hooked_property.span(),
                is_overriding: false,
            })
        }
    }

    reflections
}

#[inline]
pub fn modifier_to_visibility(modifier: Option<&Modifier>) -> Option<ClassLikeMemberVisibilityReflection> {
    Some(match modifier? {
        Modifier::Public(m) | Modifier::PublicSet(m) => ClassLikeMemberVisibilityReflection::Public { span: m.span },
        Modifier::Protected(m) | Modifier::ProtectedSet(m) => {
            ClassLikeMemberVisibilityReflection::Protected { span: m.span }
        }
        Modifier::Private(m) | Modifier::PrivateSet(m) => ClassLikeMemberVisibilityReflection::Private { span: m.span },
        _ => {
            unreachable!(
                "Modifier should be one of `Public`, `PublicSet`, `Protected`, `ProtectedSet`, `Private`, or `PrivateSet`."
            )
        }
    })
}
