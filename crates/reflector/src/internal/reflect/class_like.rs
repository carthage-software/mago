use ahash::HashMap;

use mago_ast::*;
use mago_reflection::class_like::constant::ClassLikeConstantReflection;
use mago_reflection::class_like::enum_case::EnumCaseReflection;
use mago_reflection::class_like::inheritance::InheritanceReflection;
use mago_reflection::class_like::member::ClassLikeMemberVisibilityReflection;
use mago_reflection::class_like::member::MemeberCollection;
use mago_reflection::class_like::property::PropertyDefaultValueReflection;
use mago_reflection::class_like::property::PropertyReflection;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reflection::function_like::FunctionLikeReflection;
use mago_reflection::identifier::ClassLikeMemberName;
use mago_reflection::identifier::ClassLikeName;
use mago_reflection::identifier::FunctionLikeName;
use mago_reflection::identifier::Name;
use mago_span::*;

use crate::internal::context::Context;
use crate::internal::reflect::attribute::reflect_attributes;

use super::function_like::reflect_function_like_parameter_list;
use super::function_like::reflect_function_like_return_type_hint;
use super::r#type::maybe_reflect_hint;
use super::r#type::reflect_hint;

pub fn reflect_class<'ast>(class: &'ast Class, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&class.attributes, context),
        name: {
            let value = *context.names.get(&class.name);

            ClassLikeName::Class(Name { value, lower: context.interner.lowered(&value), span: class.name.span })
        },
        inheritance: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = {
                        let value = *context.names.get(&first_parent);

                        Name { value, lower: context.interner.lowered(&value), span: first_parent.span() }
                    };

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                    reflection.names.insert(parent.lower, parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let interface = {
                        let value = *context.names.get(&interface);

                        Name { value, lower: context.interner.lowered(&value), span: interface.span() }
                    };

                    reflection.direct_implemented_interfaces.insert(interface);
                    reflection.all_implemented_interfaces.insert(interface);
                    reflection.names.insert(interface.lower, interface);
                }
            }

            reflection
        },
        backing_type: None,
        is_final: class.modifiers.contains_final(),
        is_readonly: class.modifiers.contains_readonly(),
        is_abstract: class.modifiers.contains_abstract(),
        span: class.span(),
        constants: Default::default(),
        cases: MemeberCollection::empty(),
        properties: MemeberCollection::empty(),
        methods: MemeberCollection::empty(),
        used_traits: Default::default(),
        used_trait_names: Default::default(),
        is_populated: false,
        is_anonymous: false,
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

pub fn reflect_anonymous_class<'ast>(
    class: &'ast AnonymousClass,
    context: &'ast mut Context<'_>,
) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&class.attributes, context),
        name: ClassLikeName::AnonymousClass(class.span()),
        inheritance: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = {
                        let value = *context.names.get(&first_parent);

                        Name { value, lower: context.interner.lowered(&value), span: first_parent.span() }
                    };

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                    reflection.names.insert(parent.lower, parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let interface = {
                        let value = *context.names.get(&interface);

                        Name { value, lower: context.interner.lowered(&value), span: interface.span() }
                    };

                    reflection.direct_implemented_interfaces.insert(interface);
                    reflection.all_implemented_interfaces.insert(interface);
                    reflection.names.insert(interface.lower, interface);
                }
            }

            reflection
        },
        backing_type: None,
        is_final: class.modifiers.contains_final(),
        is_readonly: class.modifiers.contains_readonly(),
        is_abstract: class.modifiers.contains_abstract(),
        span: class.span(),
        constants: Default::default(),
        cases: MemeberCollection::empty(),
        properties: MemeberCollection::empty(),
        methods: MemeberCollection::empty(),
        used_traits: Default::default(),
        used_trait_names: Default::default(),
        is_populated: false,
        is_anonymous: true,
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

pub fn reflect_interface<'ast>(interface: &'ast Interface, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&interface.attributes, context),
        name: {
            let value = *context.names.get(&interface.name);

            ClassLikeName::Interface(Name {
                value,
                lower: context.interner.lowered(&value),
                span: interface.name.span(),
            })
        },
        inheritance: {
            let mut reflection = InheritanceReflection::default();

            if let Some(extends) = &interface.extends {
                for interface in extends.types.iter() {
                    let interface = {
                        let value = *context.names.get(&interface);

                        Name { value, lower: context.interner.lowered(&value), span: interface.span() }
                    };

                    reflection.direct_implemented_interfaces.insert(interface);
                    reflection.all_implemented_interfaces.insert(interface);
                    reflection.names.insert(interface.lower, interface);
                }
            }

            reflection
        },
        backing_type: None,
        is_final: false,
        is_readonly: false,
        is_abstract: true,
        span: interface.span(),
        constants: Default::default(),
        cases: MemeberCollection::empty(),
        properties: MemeberCollection::empty(),
        methods: MemeberCollection::empty(),
        used_traits: Default::default(),
        used_trait_names: Default::default(),
        is_populated: false,
        is_anonymous: false,
    };

    reflect_class_like_members(&mut reflection, &interface.members, context);

    reflection
}

pub fn reflect_trait<'ast>(r#trait: &'ast Trait, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#trait.attributes, context),
        name: {
            let value = *context.names.get(&r#trait.name);

            ClassLikeName::Trait(Name { value, lower: context.interner.lowered(&value), span: r#trait.name.span() })
        },
        inheritance: InheritanceReflection::default(),
        backing_type: None,
        is_final: false,
        is_readonly: false,
        is_abstract: true,
        span: r#trait.span(),
        constants: Default::default(),
        cases: MemeberCollection::empty(),
        properties: MemeberCollection::empty(),
        methods: MemeberCollection::empty(),
        used_traits: Default::default(),
        used_trait_names: Default::default(),
        is_populated: false,
        is_anonymous: false,
    };

    reflect_class_like_members(&mut reflection, &r#trait.members, context);

    reflection
}

pub fn reflect_enum<'ast>(r#enum: &'ast Enum, context: &'ast mut Context<'_>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#enum.attributes, context),
        name: {
            let value = *context.names.get(&r#enum.name);

            ClassLikeName::Enum(Name { value, lower: context.interner.lowered(&value), span: r#enum.name.span() })
        },
        inheritance: {
            let mut reflection = InheritanceReflection::default();

            if let Some(impelemnts) = &r#enum.implements {
                for interface in impelemnts.types.iter() {
                    let interface = {
                        let value = *context.names.get(&interface);

                        Name { value, lower: context.interner.lowered(&value), span: interface.span() }
                    };

                    reflection.direct_implemented_interfaces.insert(interface);
                    reflection.all_implemented_interfaces.insert(interface);
                    reflection.names.insert(interface.lower, interface);
                }
            }

            reflection
        },
        backing_type: r#enum
            .backing_type_hint
            .as_ref()
            .map(|backing_type_hint| reflect_hint(&backing_type_hint.hint, context, None)),
        is_final: true,
        is_readonly: true,
        is_abstract: false,
        span: r#enum.span(),
        constants: Default::default(),
        cases: MemeberCollection::empty(),
        properties: MemeberCollection::empty(),
        methods: MemeberCollection::empty(),
        used_traits: Default::default(),
        used_trait_names: Default::default(),
        is_populated: false,
        is_anonymous: false,
    };

    reflect_class_like_members(&mut reflection, &r#enum.members, context);

    reflection
}

fn reflect_class_like_members<'ast>(
    reflection: &mut ClassLikeReflection,
    members: &'ast Sequence<ClassLikeMember>,
    context: &'ast mut Context<'_>,
) {
    for member in members.iter() {
        match &member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_name in trait_use.trait_names.iter() {
                    let value = *context.names.get(trait_name);
                    let name = Name { value, lower: context.interner.lowered(&value), span: trait_name.span() };

                    reflection.used_traits.insert(name.value);
                    reflection.used_trait_names.insert(name.lower, name);
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

                reflection.cases.members.insert(case_ref.name.member.value, case_ref);
            }
            ClassLikeMember::Method(method) => {
                let (name, meth_ref) = reflect_class_like_method(reflection, method, context);

                // `__construct`, `__clone`, and trait methods are always inheritable
                let name_value = context.interner.lookup(&name.value);
                if meth_ref.visibility_reflection.map(|v| !v.is_private()).unwrap_or(true)
                    || name_value.eq_ignore_ascii_case("__construct")
                    || name_value.eq_ignore_ascii_case("__clone")
                    || reflection.is_trait()
                {
                    reflection.methods.inheritable_members.insert(name.value, reflection.name);
                }

                reflection.methods.members.insert(name.value, meth_ref);
            }
            ClassLikeMember::Property(property) => {
                let prop_refs = reflect_class_like_property(reflection, property, context);
                for prop_ref in prop_refs {
                    if prop_ref.read_visibility_reflection.map(|v| !v.is_private()).unwrap_or(true) {
                        reflection.properties.inheritable_members.insert(prop_ref.name.member.value, reflection.name);
                    }

                    reflection.properties.members.insert(prop_ref.name.member.value, prop_ref);
                }
            }
        }
    }
}

fn reflect_class_like_constant<'ast>(
    class_like: &mut ClassLikeReflection,
    constant: &'ast ClassLikeConstant,
    context: &'ast mut Context<'_>,
) -> Vec<ClassLikeConstantReflection> {
    let attribute_reflections = reflect_attributes(&constant.attributes, context);
    let visibility_reflection = if let Some(m) = constant.modifiers.get_public() {
        Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
    } else if let Some(m) = constant.modifiers.get_protected() {
        Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
    } else {
        constant.modifiers.get_private().map(|m| ClassLikeMemberVisibilityReflection::Private { span: m.span() })
    };
    let type_reflection = maybe_reflect_hint(&constant.hint, context, Some(class_like));
    let is_final = constant.modifiers.contains_final();

    let mut reflections = vec![];

    for item in constant.items.iter() {
        reflections.push(ClassLikeConstantReflection {
            attribute_reflections: attribute_reflections.clone(),
            visibility_reflection,
            type_reflection: type_reflection.clone(),
            name: ClassLikeMemberName {
                class_like: class_like.name,
                member: {
                    let value = item.name.value;

                    Name { value, lower: context.interner.lowered(&value), span: item.name.span }
                },
            },
            is_final,
            inferred_type_reflection: mago_typing::infere(context.interner, context.source, context.names, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
        });
    }

    reflections
}

fn reflect_class_like_enum_case<'ast>(
    class_like: &mut ClassLikeReflection,
    case: &'ast EnumCase,
    context: &'ast mut Context<'_>,
) -> EnumCaseReflection {
    let (identifier, type_reflection, is_backed) = match &case.item {
        EnumCaseItem::Unit(enum_case_unit_item) => (
            ClassLikeMemberName {
                class_like: class_like.name,
                member: Name {
                    value: enum_case_unit_item.name.value,
                    lower: context.interner.lowered(&enum_case_unit_item.name.value),
                    span: enum_case_unit_item.name.span,
                },
            },
            None,
            false,
        ),
        EnumCaseItem::Backed(enum_case_backed_item) => (
            ClassLikeMemberName {
                class_like: class_like.name,
                member: Name {
                    value: enum_case_backed_item.name.value,
                    lower: context.interner.lowered(&enum_case_backed_item.name.value),
                    span: enum_case_backed_item.name.span,
                },
            },
            Some(mago_typing::infere(context.interner, context.source, context.names, &enum_case_backed_item.value)),
            true,
        ),
    };

    EnumCaseReflection {
        attribut_reflections: reflect_attributes(&case.attributes, context),
        name: identifier,
        type_reflection,
        is_backed,
        span: case.span(),
    }
}

fn reflect_class_like_method<'ast>(
    class_like: &mut ClassLikeReflection,
    method: &'ast Method,
    context: &'ast mut Context<'_>,
) -> (Name, FunctionLikeReflection) {
    let name =
        Name { value: method.name.value, lower: context.interner.lowered(&method.name.value), span: method.name.span };

    let (has_yield, has_throws, is_abstract) = match &method.body {
        MethodBody::Abstract(_) => (false, false, true),
        MethodBody::Concrete(block) => {
            (mago_ast_utils::block_has_yield(block), mago_ast_utils::block_has_throws(block), false)
        }
    };

    let visibility_reflection = if let Some(m) = method.modifiers.get_public() {
        Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
    } else if let Some(m) = method.modifiers.get_protected() {
        Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
    } else {
        method.modifiers.get_private().map(|m| ClassLikeMemberVisibilityReflection::Private { span: m.span() })
    };

    (
        name,
        FunctionLikeReflection {
            attribute_reflections: reflect_attributes(&method.attributes, context),
            visibility_reflection,
            name: FunctionLikeName::Method(class_like.name, name),
            // TODO: parse docblock to get the template list
            templates: vec![],
            parameters: reflect_function_like_parameter_list(&method.parameters, context, Some(class_like)),
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
        },
    )
}

fn reflect_class_like_property<'ast>(
    class_like: &mut ClassLikeReflection,
    property: &'ast Property,
    context: &'ast mut Context<'_>,
) -> Vec<PropertyReflection> {
    let mut reflections = vec![];

    match &property {
        Property::Plain(plain_property) => {
            let attribut_reflections = reflect_attributes(&plain_property.attributes, context);
            let read_visibility_reflection = if let Some(m) = plain_property.modifiers.get_public() {
                Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
            } else if let Some(m) = plain_property.modifiers.get_protected() {
                Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
            } else {
                plain_property
                    .modifiers
                    .get_private()
                    .map(|m| ClassLikeMemberVisibilityReflection::Private { span: m.span() })
            };

            // TODO(azjezz): take `(set)` modifiers into account.
            let write_visibility_reflection = read_visibility_reflection;
            let type_reflection = maybe_reflect_hint(&plain_property.hint, context, Some(class_like));
            let is_readonly = class_like.is_readonly || plain_property.modifiers.contains_readonly();
            let is_final = class_like.is_final || plain_property.modifiers.contains_final();
            let is_static = plain_property.modifiers.contains_static();

            for item in plain_property.items.iter() {
                let (identifier, default_value_reflection) = match &item {
                    PropertyItem::Abstract(item) => (
                        ClassLikeMemberName {
                            class_like: class_like.name,
                            member: Name {
                                value: item.variable.name,
                                lower: item.variable.name,
                                span: item.variable.span,
                            },
                        },
                        None,
                    ),
                    PropertyItem::Concrete(item) => (
                        ClassLikeMemberName {
                            class_like: class_like.name,
                            member: Name {
                                value: item.variable.name,
                                lower: item.variable.name,
                                span: item.variable.span,
                            },
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
                })
            }
        }
        Property::Hooked(hooked_property) => {
            let read_visibility_reflection = if let Some(m) = hooked_property.modifiers.get_public() {
                Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
            } else if let Some(m) = hooked_property.modifiers.get_protected() {
                Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
            } else {
                hooked_property
                    .modifiers
                    .get_private()
                    .map(|m| ClassLikeMemberVisibilityReflection::Private { span: m.span() })
            };

            // TODO(azjezz): take `(set)` modifiers into account.
            let write_visibility_reflection = read_visibility_reflection;

            let (name, default_value_reflection) = match &hooked_property.item {
                PropertyItem::Abstract(item) => (
                    ClassLikeMemberName {
                        class_like: class_like.name,
                        member: Name { value: item.variable.name, lower: item.variable.name, span: item.variable.span },
                    },
                    None,
                ),
                PropertyItem::Concrete(item) => (
                    ClassLikeMemberName {
                        class_like: class_like.name,
                        member: Name { value: item.variable.name, lower: item.variable.name, span: item.variable.span },
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
                attribut_reflections: reflect_attributes(&hooked_property.attributes, context),
                read_visibility_reflection,
                write_visibility_reflection,
                name,
                type_reflection: maybe_reflect_hint(&hooked_property.hint, context, Some(class_like)),
                default_value_reflection,
                hooks: {
                    let mut map = HashMap::default();
                    for hook in hooked_property.hooks.hooks.iter() {
                        let hook_name =
                            Name::new(hook.name.value, context.interner.lowered(&hook.name.value), hook.name.span);

                        let function_like_name =
                            FunctionLikeName::PropertyHook(name.class_like, name.member, hook_name);

                        let (has_yield, has_throws) = match &hook.body {
                            PropertyHookBody::Abstract(_) => (false, false),
                            PropertyHookBody::Concrete(body) => match &body {
                                PropertyHookConcreteBody::Block(block) => {
                                    (mago_ast_utils::block_has_yield(block), mago_ast_utils::block_has_throws(block))
                                }
                                PropertyHookConcreteBody::Expression(body) => (
                                    mago_ast_utils::expression_has_yield(&body.expression),
                                    mago_ast_utils::expression_has_throws(&body.expression),
                                ),
                            },
                        };

                        map.insert(
                            hook_name.value,
                            FunctionLikeReflection {
                                attribute_reflections: reflect_attributes(&hook.attributes, context),
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
            })
        }
    }

    reflections
}
