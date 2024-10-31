use ahash::HashMap;

use fennec_ast::*;
use fennec_reflection::class_like::constant::ClassLikeConstantReflection;
use fennec_reflection::class_like::enum_case::EnumCaseReflection;
use fennec_reflection::class_like::inheritance::InheritanceReflection;
use fennec_reflection::class_like::member::ClassLikeMemberVisibilityReflection;
use fennec_reflection::class_like::member::MemeberCollection;
use fennec_reflection::class_like::property::PropertyDefaultValueReflection;
use fennec_reflection::class_like::property::PropertyReflection;
use fennec_reflection::class_like::ClassLikeReflection;
use fennec_reflection::function_like::FunctionLikeReflection;
use fennec_reflection::identifier::ClassLikeMemberName;
use fennec_reflection::identifier::ClassLikeName;
use fennec_reflection::identifier::FunctionLikeName;
use fennec_reflection::identifier::Name;
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
        name: ClassLikeName::Class(Name::new(context.semantics.names.get(&class.name), class.name.span)),
        inheritance: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = Name::new(context.semantics.names.get(first_parent), first_parent.span());

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                    reflection.names.insert(parent.value, parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let name = Name::new(context.semantics.names.get(interface), interface.span());

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                    reflection.names.insert(name.value, name);
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

pub fn reflect_anonymous_class<'i, 'ast>(
    class: &'ast AnonymousClass,
    context: &'ast mut Context<'i>,
) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&class.attributes, context),
        name: ClassLikeName::AnonymousClass(class.span()),
        inheritance: {
            let mut reflection = InheritanceReflection::default();
            if let Some(extends) = &class.extends {
                if let Some(first_parent) = extends.types.first() {
                    let parent = Name::new(context.semantics.names.get(first_parent), first_parent.span());

                    reflection.direct_extended_class = Some(parent);
                    reflection.all_extended_classes.insert(parent);
                    reflection.names.insert(parent.value, parent);
                }
            }

            if let Some(impelemnts) = &class.implements {
                for interface in impelemnts.types.iter() {
                    let name = Name::new(context.semantics.names.get(interface), interface.span());

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                    reflection.names.insert(name.value, name);
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

pub fn reflect_interface<'i, 'ast>(interface: &'ast Interface, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&interface.attributes, context),
        name: ClassLikeName::Interface(Name::new(context.semantics.names.get(&interface.name), interface.name.span)),
        inheritance: {
            let mut reflection = InheritanceReflection::default();

            if let Some(extends) = &interface.extends {
                for interface in extends.types.iter() {
                    let name = Name::new(context.semantics.names.get(interface), interface.span());

                    reflection.direct_extended_interfaces.insert(name);
                    reflection.all_extended_interfaces.insert(name);
                    reflection.names.insert(name.value, name);
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

pub fn reflect_trait<'i, 'ast>(r#trait: &'ast Trait, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#trait.attributes, context),
        name: ClassLikeName::Trait(Name::new(context.semantics.names.get(&r#trait.name), r#trait.name.span)),
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

pub fn reflect_enum<'i, 'ast>(r#enum: &'ast Enum, context: &'ast mut Context<'i>) -> ClassLikeReflection {
    let mut reflection = ClassLikeReflection {
        attribute_reflections: reflect_attributes(&r#enum.attributes, context),
        name: ClassLikeName::Enum(Name::new(context.semantics.names.get(&r#enum.name), r#enum.name.span)),
        inheritance: {
            let mut reflection = InheritanceReflection::default();

            if let Some(impelemnts) = &r#enum.implements {
                for interface in impelemnts.types.iter() {
                    let name = Name::new(context.semantics.names.get(interface), interface.span());

                    reflection.direct_implemented_interfaces.insert(name);
                    reflection.all_implemented_interfaces.insert(name);
                    reflection.names.insert(name.value, name);
                }
            }

            reflection
        },
        backing_type: match &r#enum.backing_type_hint {
            Some(backing_type_hint) => Some(reflect_hint(&backing_type_hint.hint, context, None)),
            None => None,
        },
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

fn reflect_class_like_members<'i, 'ast>(
    reflection: &mut ClassLikeReflection,
    members: &'ast Sequence<ClassLikeMember>,
    context: &'ast mut Context<'i>,
) {
    for member in members.iter() {
        match &member {
            ClassLikeMember::TraitUse(trait_use) => {
                for trait_name in trait_use.trait_names.iter() {
                    let name = Name::new(context.semantics.names.get(trait_name), trait_name.span());

                    reflection.used_traits.insert(name.value);
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

                reflection.cases.members.insert(case_ref.name.member.value, case_ref);
            }
            ClassLikeMember::Method(method) => {
                let (name, meth_ref) = reflect_class_like_method(reflection, method, context);

                // `__construct`, `__clone`, and trait methods are always inheritable
                let name_value = context.interner.lookup(name.value);
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
    let type_reflection = maybe_reflect_hint(&constant.hint, context, Some(&class_like));
    let is_final = constant.modifiers.contains_final();

    let mut reflections = vec![];

    for item in constant.items.iter() {
        reflections.push(ClassLikeConstantReflection {
            attribute_reflections: attribute_reflections.clone(),
            visibility_reflection: visibility_reflection.clone(),
            type_reflection: type_reflection.clone(),
            name: ClassLikeMemberName {
                class_like: class_like.name,
                member: Name::new(item.name.value, item.name.span),
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
            fennec_inference::infere(&context.interner, &context.semantics, &enum_case_backed_item.value),
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

fn reflect_class_like_method<'i, 'ast>(
    class_like: &mut ClassLikeReflection,
    method: &'ast Method,
    context: &'ast mut Context<'i>,
) -> (Name, FunctionLikeReflection) {
    let name = Name::new(method.name.value, method.name.span);

    let (has_yield, has_throws, is_abstract) = match &method.body {
        MethodBody::Abstract(_) => (false, false, true),
        MethodBody::Concrete(block) => {
            (fennec_ast_utils::block_has_yield(&block), fennec_ast_utils::block_has_throws(&block), false)
        }
    };

    let visibility_reflection = if let Some(m) = method.modifiers.get_public() {
        Some(ClassLikeMemberVisibilityReflection::Public { span: m.span() })
    } else if let Some(m) = method.modifiers.get_protected() {
        Some(ClassLikeMemberVisibilityReflection::Protected { span: m.span() })
    } else if let Some(m) = method.modifiers.get_private() {
        Some(ClassLikeMemberVisibilityReflection::Private { span: m.span() })
    } else {
        None
    };

    (
        name,
        FunctionLikeReflection {
            attribute_reflections: reflect_attributes(&method.attributes, context),
            visibility_reflection,
            name: FunctionLikeName::Method(class_like.name, name),
            parameter_reflections: reflect_function_like_parameter_list(&method.parameters, context, Some(class_like)),
            return_type_reflection: reflect_function_like_return_type_hint(
                &method.return_type_hint,
                context,
                Some(class_like),
            ),
            returns_by_reference: method.ampersand.is_some(),
            has_yield,
            has_throws,
            is_anonymous: false,
            is_static: method.modifiers.contains_static(),
            is_final: class_like.is_final || method.modifiers.contains_final(),
            is_abstract,
            is_overriding: false,
            span: method.span(),
            is_populated: false,
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

            let type_reflection = maybe_reflect_hint(&plain_property.hint, context, Some(&class_like));
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
            } else if let Some(m) = hooked_property.modifiers.get_private() {
                Some(ClassLikeMemberVisibilityReflection::Private { span: m.span() })
            } else {
                None
            };

            // TODO(azjezz): take `(set)` modifiers into account.
            let write_visibility_reflection = read_visibility_reflection.clone();

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
                name,
                type_reflection: maybe_reflect_hint(&hooked_property.hint, context, Some(&class_like)),
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
                            hook_name.value,
                            FunctionLikeReflection {
                                attribute_reflections: reflect_attributes(&hook.attributes, context),
                                name: function_like_name,
                                parameter_reflections: match hook.parameters.as_ref() {
                                    Some(parameters) => {
                                        reflect_function_like_parameter_list(&parameters, context, Some(&class_like))
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
