use ahash::HashMap;

use fennec_ast::*;
use fennec_interner::StringIdentifier;
use fennec_reflection::class_like::constant::ClassLikeConstantReflection;
use fennec_reflection::class_like::enum_case::EnumCaseReflection;
use fennec_reflection::class_like::inheritance::InheritanceReflection;
use fennec_reflection::class_like::member::ClassLikeMemberVisibilityReflection;
use fennec_reflection::class_like::member::MemeberCollection;
use fennec_reflection::class_like::property::PropertyDefaultValueReflection;
use fennec_reflection::class_like::property::PropertyReflection;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::function_like::FunctionLikeReflection;
use fennec_reflection::identifier::ClassLikeIdentifier;
use fennec_reflection::identifier::ClassLikeMemberIdentifier;
use fennec_reflection::identifier::FunctionLikeIdentifier;
use fennec_span::*;

use crate::internal::context::Context;
use crate::internal::reflect::attribute::reflect_attributes;

use super::function_like::reflect_function_like_parameter_list;
use super::function_like::reflect_function_like_return_type_hint;
use super::r#type::maybe_reflect_hint;
use super::r#type::reflect_hint;

pub fn reflect_class<'i, 'ast>(class: &'ast Class, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&class.attributes, context),
        identifier: ClassLikeIdentifier::Class(context.semantics.names.get(&class.name), class.name.span),
        inheritance_reflection: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = context.semantics.names.get(first_parent);

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let name = context.semantics.names.get(interface);

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                }
            }

            reflection
        },
        backing_type_reflection: None,
        is_final: class.modifiers.contains_final(),
        is_readonly: class.modifiers.contains_readonly(),
        is_abstract: class.modifiers.contains_abstract(),
        span: class.span(),
        constant_reflections: MemeberCollection::empty(),
        case_reflections: MemeberCollection::empty(),
        property_reflections: MemeberCollection::empty(),
        method_reflections: MemeberCollection::empty(),
        used_traits: Default::default(),
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

pub fn reflect_anonymous_class<'i, 'ast>(
    class: &'ast AnonymousClass,
    context: &'ast mut Context<'i>,
) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&class.attributes, context),
        identifier: ClassLikeIdentifier::AnonymousClass(class.span()),
        inheritance_reflection: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = context.semantics.names.get(first_parent);

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let name = context.semantics.names.get(interface);

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                }
            }

            reflection
        },
        backing_type_reflection: None,
        is_final: class.modifiers.contains_final(),
        is_readonly: class.modifiers.contains_readonly(),
        is_abstract: class.modifiers.contains_abstract(),
        span: class.span(),
        constant_reflections: MemeberCollection::empty(),
        case_reflections: MemeberCollection::empty(),
        property_reflections: MemeberCollection::empty(),
        method_reflections: MemeberCollection::empty(),
        used_traits: Default::default(),
    };

    reflect_class_like_members(&mut reflection, &class.members, context);

    reflection
}

pub fn reflect_interface<'i, 'ast>(interface: &'ast Interface, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&interface.attributes, context),
        identifier: ClassLikeIdentifier::Interface(context.semantics.names.get(&interface.name), interface.name.span),
        inheritance_reflection: {
            let mut reflection = InheritanceReflection::default();

            if let Some(extends) = &interface.extends {
                for interface in extends.types.iter() {
                    let name = context.semantics.names.get(interface);

                    reflection.direct_extended_interfaces.insert(name);
                    reflection.all_extended_interfaces.insert(name);
                }
            }

            reflection
        },
        backing_type_reflection: None,
        is_final: false,
        is_readonly: false,
        is_abstract: true,
        span: interface.span(),
        constant_reflections: MemeberCollection::empty(),
        case_reflections: MemeberCollection::empty(),
        property_reflections: MemeberCollection::empty(),
        method_reflections: MemeberCollection::empty(),
        used_traits: Default::default(),
    };

    reflect_class_like_members(&mut reflection, &interface.members, context);

    reflection
}

pub fn reflect_trait<'i, 'ast>(r#trait: &'ast Trait, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#trait.attributes, context),
        identifier: ClassLikeIdentifier::Interface(context.semantics.names.get(&r#trait.name), r#trait.name.span),
        inheritance_reflection: InheritanceReflection::default(),
        backing_type_reflection: None,
        is_final: false,
        is_readonly: false,
        is_abstract: true,
        span: r#trait.span(),
        constant_reflections: MemeberCollection::empty(),
        case_reflections: MemeberCollection::empty(),
        property_reflections: MemeberCollection::empty(),
        method_reflections: MemeberCollection::empty(),
        used_traits: Default::default(),
    };

    reflect_class_like_members(&mut reflection, &r#trait.members, context);

    reflection
}

pub fn reflect_enum<'i, 'ast>(r#enum: &'ast Enum, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#enum.attributes, context),
        identifier: ClassLikeIdentifier::Interface(context.semantics.names.get(&r#enum.name), r#enum.name.span),
        inheritance_reflection: {
            let mut reflection = InheritanceReflection::default();

            if let Some(impelemnts) = &r#enum.implements {
                for interface in impelemnts.types.iter() {
                    let name = context.semantics.names.get(interface);

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                }
            }

            reflection
        },
        backing_type_reflection: match &r#enum.backing_type_hint {
            Some(backing_type_hint) => Some(reflect_hint(&backing_type_hint.hint, context)),
            None => None,
        },
        is_final: true,
        is_readonly: true,
        is_abstract: false,
        span: r#enum.span(),
        constant_reflections: MemeberCollection::empty(),
        case_reflections: MemeberCollection::empty(),
        property_reflections: MemeberCollection::empty(),
        method_reflections: MemeberCollection::empty(),
        used_traits: Default::default(),
    };

    reflect_class_like_members(&mut reflection, &r#enum.members, context);

    reflection
}

fn reflect_class_like_members<'i, 'ast>(
    reflection: &mut ClassLikeReflection,
    members: &'ast Sequence<ClassLikeMember>,
    context: &'ast mut Context<'i>,
) {
    for member in members.iter() {
        match &member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_name in trait_use.trait_names.iter() {
                    let name = context.semantics.names.get(trait_name);

                    reflection.used_traits.insert(name);
                }
            }
            ClassLikeMember::Constant(class_like_constant) => {
                let const_refs = reflect_class_like_constant(reflection, class_like_constant, context);
                for const_ref in const_refs {
                    if const_ref.visibility_reflection.map(|v| !v.is_private()).unwrap_or(true) {
                        reflection.constant_reflections.inheritable_members.insert(const_ref.identifier.name);
                    }

                    reflection.constant_reflections.appering_members.insert(const_ref.identifier.name);
                    reflection.constant_reflections.declared_members.insert(const_ref.identifier.name);
                    reflection.constant_reflections.members.insert(const_ref.identifier.name, const_ref);
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_ref = reflect_class_like_enum_case(reflection, enum_case, context);

                reflection.case_reflections.appering_members.insert(case_ref.identifier.name);
                reflection.case_reflections.declared_members.insert(case_ref.identifier.name);
                reflection.case_reflections.members.insert(case_ref.identifier.name, case_ref);
            }
            ClassLikeMember::Method(method) => {
                let (name, meth_ref) = reflect_class_like_method(reflection, method, context);

                reflection.method_reflections.appering_members.insert(name);
                reflection.method_reflections.declared_members.insert(name);
                reflection.method_reflections.members.insert(name, meth_ref);
            }
            ClassLikeMember::Property(property) => {
                let prop_refs = reflect_class_like_property(reflection, property, context);
                for prop_ref in prop_refs {
                    if prop_ref.read_visibility_reflection.map(|v| !v.is_private()).unwrap_or(true) {
                        reflection.property_reflections.inheritable_members.insert(prop_ref.identifier.name);
                    }

                    reflection.property_reflections.appering_members.insert(prop_ref.identifier.name);
                    reflection.property_reflections.declared_members.insert(prop_ref.identifier.name);
                    reflection.property_reflections.members.insert(prop_ref.identifier.name, prop_ref);
                }
            }
        }
    }
}

fn reflect_class_like_constant<'i, 'ast>(
    class_like: &mut ClassLikeReflection,
    constant: &'ast ClassLikeConstant,
    context: &'ast mut Context<'i>,
) -> Vec<ClassLikeConstantReflection> {
    let attribute_reflections = reflect_attributes(&constant.attributes, context);
    let visibility_reflection = if let Some(m) = constant.modifiers.get_public() {
        Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
    } else if let Some(m) = constant.modifiers.get_protected() {
        Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
    } else if let Some(m) = constant.modifiers.get_private() {
        Some(ClassLikeMemberVisibilityReflection::Private { span: m.span() })
    } else {
        None
    };
    let type_reflection = maybe_reflect_hint(&constant.hint, context);
    let is_final = constant.modifiers.contains_final();

    let mut reflections = vec![];

    for item in constant.items.iter() {
        reflections.push(ClassLikeConstantReflection {
            attribute_reflections: attribute_reflections.clone(),
            visibility_reflection: visibility_reflection.clone(),
            type_reflection: type_reflection.clone(),
            identifier: ClassLikeMemberIdentifier {
                class_like: class_like.identifier,
                name: context.semantics.names.get(&item.name),
                span: item.name.span,
            },
            is_final,
            inferred_type_reflection: fennec_inference::infere(&context.interner, &context.semantics, &item.value),
            item_span: item.span(),
            definition_span: constant.span(),
        });
    }

    reflections
}

fn reflect_class_like_enum_case<'i, 'ast>(
    class_like: &mut ClassLikeReflection,
    case: &'ast EnumCase,
    context: &'ast mut Context<'i>,
) -> EnumCaseReflection {
    let (identifier, type_reflection, is_backed) = match &case.item {
        EnumCaseItem::Unit(enum_case_unit_item) => (
            ClassLikeMemberIdentifier {
                class_like: class_like.identifier,
                name: context.semantics.names.get(&enum_case_unit_item.name),
                span: enum_case_unit_item.name.span,
            },
            None,
            false,
        ),
        EnumCaseItem::Backed(enum_case_backed_item) => (
            ClassLikeMemberIdentifier {
                class_like: class_like.identifier,
                name: context.semantics.names.get(&enum_case_backed_item.name),
                span: enum_case_backed_item.name.span,
            },
            fennec_inference::infere(&context.interner, &context.semantics, &enum_case_backed_item.value),
            true,
        ),
    };

    EnumCaseReflection {
        attribut_reflections: reflect_attributes(&case.attributes, context),
        identifier,
        type_reflection,
        is_backed,
        span: case.span(),
    }
}

fn reflect_class_like_method<'i, 'ast>(
    class_like: &mut ClassLikeReflection,
    method: &'ast Method,
    context: &'ast mut Context<'i>,
) -> (StringIdentifier, FunctionLikeReflection) {
    let name = context.semantics.names.get(&method.name);

    let (has_yield, has_throws, is_abstract) = match &method.body {
        MethodBody::Abstract(_) => (false, false, true),
        MethodBody::Concrete(block) => {
            (fennec_ast_utils::block_has_yield(&block), fennec_ast_utils::block_has_throws(&block), false)
        }
    };

    (
        name,
        FunctionLikeReflection {
            attribute_reflections: reflect_attributes(&method.attributes, context),
            identifier: FunctionLikeIdentifier::Method(class_like.identifier, name, method.name.span),
            parameter_reflections: reflect_function_like_parameter_list(&method.parameters, context),
            return_type_reflection: reflect_function_like_return_type_hint(&method.return_type_hint, context),
            returns_by_reference: method.ampersand.is_some(),
            has_yield,
            has_throws,
            is_anonymous: false,
            is_static: method.modifiers.contains_static(),
            is_final: class_like.is_final || method.modifiers.contains_final(),
            is_abstract,
            is_overriding: false,
            span: method.span(),
        },
    )
}

fn reflect_class_like_property<'i, 'ast>(
    class_like: &mut ClassLikeReflection,
    property: &'ast Property,
    context: &'ast mut Context<'i>,
) -> Vec<PropertyReflection> {
    let mut reflections = vec![];

    match &property {
        Property::Plain(plain_property) => {
            let attribut_reflections = reflect_attributes(&plain_property.attributes, context);
            let read_visibility_reflection = if let Some(m) = plain_property.modifiers.get_public() {
                Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
            } else if let Some(m) = plain_property.modifiers.get_protected() {
                Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
            } else if let Some(m) = plain_property.modifiers.get_private() {
                Some(ClassLikeMemberVisibilityReflection::Private { span: m.span() })
            } else {
                None
            };

            // TODO(azjezz): take `(set)` modifiers into account.
            let write_visibility_reflection = read_visibility_reflection.clone();

            let type_reflection = maybe_reflect_hint(&plain_property.hint, context);
            let is_readonly = class_like.is_readonly || plain_property.modifiers.contains_readonly();
            let is_final = class_like.is_final || plain_property.modifiers.contains_final();
            let is_static = plain_property.modifiers.contains_static();

            for item in plain_property.items.iter() {
                let (identifier, default_value_reflection) = match &item {
                    PropertyItem::Abstract(item) => (
                        ClassLikeMemberIdentifier {
                            class_like: class_like.identifier,
                            name: item.variable.name,
                            span: item.variable.span,
                        },
                        None,
                    ),
                    PropertyItem::Concrete(item) => (
                        ClassLikeMemberIdentifier {
                            class_like: class_like.identifier,
                            name: item.variable.name,
                            span: item.variable.span,
                        },
                        Some(PropertyDefaultValueReflection {
                            inferred_type_reflection: fennec_inference::infere(
                                &context.interner,
                                context.semantics,
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
                    identifier,
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
            } else if let Some(m) = hooked_property.modifiers.get_private() {
                Some(ClassLikeMemberVisibilityReflection::Private { span: m.span() })
            } else {
                None
            };

            // TODO(azjezz): take `(set)` modifiers into account.
            let write_visibility_reflection = read_visibility_reflection.clone();

            let (identifier, default_value_reflection) = match &hooked_property.item {
                PropertyItem::Abstract(item) => (
                    ClassLikeMemberIdentifier {
                        class_like: class_like.identifier,
                        name: item.variable.name,
                        span: item.variable.span,
                    },
                    None,
                ),
                PropertyItem::Concrete(item) => (
                    ClassLikeMemberIdentifier {
                        class_like: class_like.identifier,
                        name: item.variable.name,
                        span: item.variable.span,
                    },
                    Some(PropertyDefaultValueReflection {
                        inferred_type_reflection: fennec_inference::infere(
                            &context.interner,
                            context.semantics,
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
                identifier,
                type_reflection: maybe_reflect_hint(&hooked_property.hint, context),
                default_value_reflection,
                hooks: {
                    let mut map = HashMap::default();
                    for hook in hooked_property.hooks.hooks.iter() {
                        let name = hook.name.value;
                        let identifier = FunctionLikeIdentifier::PropertyHook(
                            identifier.class_like,
                            identifier.name,
                            hook.name.value,
                            hook.name.span,
                        );

                        let (has_yield, has_throws) = match &hook.body {
                            PropertyHookBody::Abstract(_) => (false, false),
                            PropertyHookBody::Concrete(body) => match &body {
                                PropertyHookConcreteBody::Block(block) => (
                                    fennec_ast_utils::block_has_yield(block),
                                    fennec_ast_utils::block_has_throws(block),
                                ),
                                PropertyHookConcreteBody::Expression(body) => (
                                    fennec_ast_utils::expression_has_yield(&body.expression),
                                    fennec_ast_utils::expression_has_throws(&body.expression),
                                ),
                            },
                        };

                        map.insert(
                            name,
                            FunctionLikeReflection {
                                attribute_reflections: reflect_attributes(&hook.attributes, context),
                                identifier,
                                parameter_reflections: match hook.parameters.as_ref() {
                                    Some(parameters) => reflect_function_like_parameter_list(&parameters, context),
                                    None => vec![],
                                },
                                return_type_reflection: None,
                                returns_by_reference: hook.ampersand.is_some(),
                                has_yield,
                                has_throws,
                                is_anonymous: false,
                                is_static: false,
                                is_final: true,
                                is_abstract: false,
                                is_overriding: false,
                                span: hook.span(),
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
