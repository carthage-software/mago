use mago_interner::StringIdentifier;
use mago_php_version::feature::Feature;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::internal::consts::*;
use crate::internal::context::Context;

pub use constant::*;
pub use inheritance::*;
pub use method::*;
pub use property::*;

mod constant;
mod inheritance;
mod method;
mod property;

#[inline]
pub fn check_class(class: &Class, context: &mut Context<'_>) {
    let class_name = context.interner.lookup(&class.name.value);
    let class_fqcn = context.get_name(&class.name.span.start);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_name))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_name))
    {
        context.issues.push(
            Issue::error(format!("Class `{}` name cannot be a reserved keyword.", class_name))
                .with_annotation(
                    Annotation::primary(class.name.span())
                        .with_message(format!("Class name `{}` conflicts with a reserved keyword.", class_name)),
                )
                .with_annotation(
                    Annotation::secondary(class.span()).with_message(format!("Class `{}` declared here.", class_fqcn)),
                )
                .with_help("Rename the class to avoid using reserved keywords."),
        );
    }

    let mut last_final = None;
    let mut last_abstract = None;
    let mut last_readonly = None;

    for modifier in class.modifiers.iter() {
        match &modifier {
            Modifier::Static(_) => {
                context.issues.push(
                    Issue::error(format!("Class `{}` cannot have the `static` modifier.", class_name))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("`static` modifier applied here."),
                        )
                        .with_annotation(
                            Annotation::secondary(class.span())
                                .with_message(format!("Class `{}` declared here.", class_fqcn)),
                        )
                        .with_help("Remove the `static` modifier."),
                );
            }
            Modifier::Public(keyword)
            | Modifier::Protected(keyword)
            | Modifier::Private(keyword)
            | Modifier::PublicSet(keyword)
            | Modifier::ProtectedSet(keyword)
            | Modifier::PrivateSet(keyword) => {
                let visibility_name = context.interner.lookup(&keyword.value);

                context.issues.push(
                    Issue::error(format!(
                        "Class `{}` cannot have the `{}` visibility modifier.",
                        class_name, visibility_name
                    ))
                    .with_annotation(
                        Annotation::primary(keyword.span())
                            .with_message(format!("`{}` modifier applied here.", visibility_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(class.span())
                            .with_message(format!("Class `{}` declared here.", class_fqcn)),
                    )
                    .with_help(format!("Remove the `{}` modifier.", visibility_name)),
                );
            }
            Modifier::Final(keyword) => {
                if let Some(span) = last_abstract {
                    context.issues.push(
                        Issue::error(format!("Abstract class `{}` cannot have the `final` modifier.", class_name))
                            .with_annotation(
                                Annotation::primary(keyword.span()).with_message("`final` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `abstract` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            ])
                            .with_help("Remove the `final` modifier from the abstract class."),
                    );
                }

                if let Some(span) = last_final {
                    context.issues.push(
                        Issue::error(format!("Class `{}` cannot have multiple `final` modifiers.", class_name))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `final` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            ])
                            .with_help("Remove the duplicate `final` modifier."),
                    );
                }

                last_final = Some(keyword.span);
            }
            Modifier::Abstract(keyword) => {
                if let Some(span) = last_final {
                    context.issues.push(
                        Issue::error(format!("Final class `{}` cannot have the `abstract` modifier.", class_name))
                            .with_annotation(
                                Annotation::primary(keyword.span()).with_message("`abstract` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            ])
                            .with_help("Remove the `abstract` modifier from the final class."),
                    );
                }

                if let Some(span) = last_abstract {
                    context.issues.push(
                        Issue::error(format!("Class `{}` cannot have multiple `abstract` modifiers.", class_name))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `abstract` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `abstract` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            ])
                            .with_help("Remove the duplicate `abstract` modifier."),
                    );
                }

                last_abstract = Some(keyword.span);
            }
            Modifier::Readonly(keyword) => {
                if let Some(span) = last_readonly {
                    context.issues.push(
                        Issue::error(format!("Class `{}` cannot have multiple `readonly` modifiers.", class_name))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `readonly` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `readonly` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{}` declared here.", class_fqcn)),
                            ])
                            .with_help("Remove the duplicate `readonly` modifier."),
                    );
                }

                last_readonly = Some(keyword.span);
            }
        }
    }

    if !context.version.is_supported(Feature::ReadonlyClasses) {
        if let Some(modifier) = last_readonly {
            let issue = Issue::error("Readonly classes are only available in PHP 8.2 and above.")
                .with_annotation(Annotation::primary(modifier.span()).with_message("Readonly modifier used here."));

            context.issues.push(issue);
        }
    }

    if let Some(extends) = &class.extends {
        check_extends(extends, class.span(), "class", class_name, class_fqcn, true, context);
    }

    if let Some(implements) = &class.implements {
        check_implements(implements, class.span(), "class", class_name, class_fqcn, true, context);
    }

    check_members(&class.members, class.span(), "class", class_name, class_fqcn, context);

    for memeber in class.members.iter() {
        match &memeber {
            ClassLikeMember::EnumCase(case) => {
                context.issues.push(
                    Issue::error(format!("Class `{}` cannot contain enum cases.", class_name))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case found in class."))
                        .with_annotation(
                            Annotation::secondary(class.span())
                                .with_message(format!("Class `{}` declared here.", class_fqcn)),
                        )
                        .with_help("Remove the enum cases from the class definition."),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name = context.interner.lookup(&method.name.value);

                if !class.modifiers.contains_abstract() && method.modifiers.contains_abstract() {
                    context.issues.push(
                        Issue::error(format!(
                            "Class `{}` contains an abstract method `{}`, so the class must be declared abstract.",
                            class_name, method_name
                        ))
                        .with_annotation(
                            Annotation::primary(class.name.span())
                                .with_message("Class is missing the `abstract` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(method.span()).with_message(format!(
                                "Abstract method `{}::{}` declared here.",
                                class_name, method_name
                            )),
                        )
                        .with_help("Add the `abstract` modifier to the class."),
                    );
                }

                check_method(method, method_name, class.span(), class_name, class_fqcn, "class", false, context);
            }
            ClassLikeMember::Property(property) => {
                check_property(property, class.span(), "class", class_name, class_fqcn, false, context);
            }
            ClassLikeMember::Constant(constant) => {
                check_class_like_constant(constant, class.span(), "class", class_name, class_fqcn, context);
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_interface(interface: &Interface, context: &mut Context<'_>) {
    let interface_name = context.interner.lookup(&interface.name.value);
    let interface_fqcn = context.get_name(&interface.name.span.start);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(interface_name))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(interface_name))
    {
        context.issues.push(
            Issue::error(format!("Interface `{}` name cannot be a reserved keyword.", interface_name))
                .with_annotation(
                    Annotation::primary(interface.name.span())
                        .with_message(format!("Interface `{}` declared here.", interface_name)),
                )
                .with_annotation(
                    Annotation::secondary(interface.span())
                        .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                )
                .with_help("Rename the interface to avoid using a reserved keyword."),
        );
    }

    if let Some(extends) = &interface.extends {
        check_extends(extends, interface.span(), "interface", interface_name, interface_fqcn, false, context);
    }

    check_members(&interface.members, interface.span(), "interface", interface_name, interface_fqcn, context);

    for memeber in interface.members.iter() {
        match &memeber {
            ClassLikeMember::TraitUse(trait_use) => {
                context.issues.push(
                    Issue::error(format!("Interface `{}` cannot use traits.", interface_name))
                        .with_annotation(Annotation::primary(trait_use.span()).with_message("Trait use statement."))
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                        )
                        .with_help("Remove the trait use statement."),
                );
            }
            ClassLikeMember::EnumCase(case) => {
                context.issues.push(
                    Issue::error(format!("Interface `{}` cannot contain enum cases.", interface_name))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case declared here."))
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                        )
                        .with_note(
                            "Consider moving the enum case to an enum or class if it represents state or constants.",
                        ),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name_id = method.name.value;
                let method_name = context.interner.lookup(&method_name_id);

                let mut visibilities = vec![];
                for modifier in method.modifiers.iter() {
                    if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                        visibilities.push(modifier);
                    }
                }

                for visibility in visibilities {
                    let visibility_name = visibility.as_str(context.interner);

                    context.issues.push(
                        Issue::error(format!(
                            "Interface method `{}::{}` cannot have `{}` modifier.",
                            interface_name, method_name, visibility_name
                        ))
                        .with_annotation(
                            Annotation::primary(visibility.span())
                                .with_message(format!("`{}` modifier applied here.", visibility_name)),
                        )
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                        )
                        .with_help(format!(
                            "Remove the `{}` modifier from the method definition as methods in interfaces must always be public.",
                            visibility_name
                        ))
                        .with_note("Interface methods are always public and cannot have non-public visibility modifiers."),
                    );
                }

                if let MethodBody::Concrete(body) = &method.body {
                    context.issues.push(
                        Issue::error(format!(
                            "Interface method `{}::{}` cannot have a body.",
                            interface_name, method_name
                        ))
                        .with_annotations([
                            Annotation::primary(body.span()).with_message("Method body declared here."),
                            Annotation::primary(method.name.span()).with_message("Method name defined here."),
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                        ])
                        .with_help("Replace the method body with a `;` to indicate it is abstract.")
                        .with_note("Methods in interfaces cannot have implementations and must be abstract."),
                    );
                }

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.issues.push(
                        Issue::error(format!(
                            "Interface method `{}::{}` must not be abstract.",
                            interface_name, method_name
                        ))
                        .with_annotation(
                            Annotation::primary(abstract_modifier.span())
                                .with_message("Abstract modifier applied here."),
                        )
                        .with_annotations([
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{}::{}` declared here.", interface_name, method_name)),
                        ])
                        .with_help("Remove the `abstract` modifier as all interface methods are implicitly abstract.")
                        .with_note(
                            "Adding the `abstract` modifier to an interface method is redundant because all interface methods are implicitly abstract.",
                        ),
                    );
                }

                check_method(
                    method,
                    method_name,
                    interface.span(),
                    interface_name,
                    interface_fqcn,
                    "interface",
                    true,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                match &property {
                    Property::Plain(plain_property) => {
                        context.issues.push(
                                    Issue::error(format!(
                                        "Interface `{}` cannot have non-hooked properties.",
                                        interface_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(plain_property.span())
                                            .with_message("Non-hooked property declared here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` declared here.", interface_fqcn)),
                                    )
                                    .with_note("Interfaces are intended to define behavior and cannot include concrete property declarations.")
                                    .with_help("Remove the non-hooked property from the interface or convert it into a hooked property.")
                                );
                    }
                    Property::Hooked(hooked_property) => {
                        let property_name_id = hooked_property.item.variable().name;
                        let property_name = context.interner.lookup(&property_name_id);

                        let mut found_public = false;
                        let mut non_public_read_visibilities = vec![];
                        let mut write_visibilities = vec![];
                        for modifier in hooked_property.modifiers.iter() {
                            if matches!(modifier, Modifier::Public(_)) {
                                found_public = true;
                            }

                            if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                                non_public_read_visibilities.push(modifier);
                            }

                            if matches!(modifier, Modifier::PrivateSet(_)) {
                                write_visibilities.push(modifier);
                            }
                        }

                        for visibility in write_visibilities {
                            let visibility_name = visibility.as_str(context.interner);

                            context.issues.push(
                                        Issue::error(format!(
                                            "Interface virtual property `{}::{}` must not specify asymmetric visibility.",
                                            interface_name, property_name,
                                        ))
                                        .with_annotation(
                                            Annotation::primary(visibility.span())
                                                .with_message(format!("Asymmetric visibility modifier `{}` applied here.", visibility_name)),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(interface.span())
                                                .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                        )
                                        .with_help(format!(
                                            "Remove the `{}` modifier from the property to make it compatible with interface constraints.",
                                            visibility_name
                                        )),
                                    );
                        }

                        for visibility in non_public_read_visibilities {
                            let visibility_name = visibility.as_str(context.interner);

                            context.issues.push(
                                Issue::error(format!(
                                    "Interface virtual property `{}::{}` cannot have `{}` modifier.",
                                    interface_name, property_name, visibility_name,
                                ))
                                .with_annotation(
                                    Annotation::primary(visibility.span()).with_message(format!(
                                        "Visibility modifier `{}` applied here.",
                                        visibility_name
                                    )),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                )
                                .with_help(format!(
                                    "Remove the `{}` modifier from the property to meet interface requirements.",
                                    visibility_name
                                )),
                            );
                        }

                        if !found_public {
                            context.issues.push(
                                Issue::error(format!(
                                    "Interface virtual property `{}::{}` must be declared public.",
                                    interface_name, property_name
                                ))
                                .with_annotation(
                                    Annotation::primary(hooked_property.span()).with_message("Property defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                )
                                .with_help("Add the `public` visibility modifier to the property."),
                            );
                        }

                        if let Some(abstract_modifier) = hooked_property.modifiers.get_abstract() {
                            context.issues.push(
                                            Issue::error(format!(
                                                "Interface virtual property `{}::{}` cannot be abstract.",
                                                interface_name, property_name
                                            ))
                                            .with_annotation(
                                                Annotation::primary(abstract_modifier.span())
                                                    .with_message("Abstract modifier applied here."),
                                            )
                                            .with_annotations([
                                                Annotation::secondary(hooked_property.span())
                                                    .with_message("Property defined here."),
                                                Annotation::secondary(interface.span())
                                                    .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                            ])
                                            .with_note(
                                                "All interface virtual properties are implicitly abstract and cannot be explicitly declared as abstract.",
                                            ),
                                        );
                        }

                        if let PropertyItem::Concrete(item) = &hooked_property.item {
                            context.issues.push(
                                Issue::error(format!(
                                    "Interface virtual property `{}::{}` cannot have a default value.",
                                    interface_name, property_name
                                ))
                                .with_annotation(
                                    Annotation::primary(item.equals.join(item.value.span()))
                                        .with_message("Default value assigned here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(hooked_property.span())
                                        .with_message("Property defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                )
                                .with_note(
                                    "Interface properties are virtual properties and cannot contain a default value.",
                                ),
                            );
                        }

                        for hook in hooked_property.hooks.hooks.iter() {
                            if let PropertyHookBody::Concrete(property_hook_concrete_body) = &hook.body {
                                context.issues.push(
                                    Issue::error(format!(
                                        "Interface virtual property `{}::{}` must be abstract.",
                                        interface_name, property_name
                                    ))
                                    .with_annotation(
                                        Annotation::primary(property_hook_concrete_body.span())
                                            .with_message("Body defined here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(hooked_property.item.variable().span())
                                            .with_message("Property declared here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{}` defined here.", interface_fqcn)),
                                    )
                                    .with_note("Abstract hooked properties must not contain a body."),
                                );
                            }
                        }
                    }
                };

                check_property(property, interface.span(), "interface", interface_name, interface_fqcn, true, context);
            }
            ClassLikeMember::Constant(class_like_constant) => {
                let mut non_public_read_visibility = vec![];
                for modifier in class_like_constant.modifiers.iter() {
                    if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                        non_public_read_visibility.push(modifier);
                    }
                }

                for visibility in non_public_read_visibility.iter() {
                    let visibility_name = visibility.as_str(context.interner);

                    context.issues.push(
                        Issue::error(format!(
                            "Interface constant cannot have `{}` visibility modifier.",
                            visibility_name,
                        ))
                        .with_annotation(
                            Annotation::primary(visibility.span())
                                .with_message(format!("Visibility modifier `{}` applied here.", visibility_name)),
                        )
                        .with_help(format!(
                            "Remove the `{}` modifier from the constant to comply with interface requirements.",
                            visibility_name
                        ))
                        .with_note(
                            "Interface constants are implicitly public and cannot have a non-public visibility modifier.",
                        )
                    );
                }

                check_class_like_constant(
                    class_like_constant,
                    interface.span(),
                    "interface",
                    interface_name,
                    interface_fqcn,
                    context,
                );
            }
        }
    }
}

#[inline]
pub fn check_trait(r#trait: &Trait, context: &mut Context<'_>) {
    let class_like_name = context.interner.lookup(&r#trait.name.value);
    let class_like_fqcn = context.get_name(&r#trait.name.span.start);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_like_name))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(class_like_name))
    {
        context.issues.push(
            Issue::error(format!("Trait `{}` name cannot be a reserved keyword.", class_like_name))
                .with_annotation(
                    Annotation::primary(r#trait.name.span())
                        .with_message(format!("Trait `{}` declared here.", class_like_name)),
                )
                .with_annotation(
                    Annotation::secondary(r#trait.span())
                        .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                )
                .with_help("Rename the trait to a non-reserved keyword."),
        );
    }

    check_members(&r#trait.members, r#trait.span(), class_like_name, class_like_fqcn, "trait", context);

    for member in r#trait.members.iter() {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                context.issues.push(
                    Issue::error(format!("Trait `{}` cannot contain enum cases.", class_like_name))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case defined here."))
                        .with_annotation(
                            Annotation::secondary(r#trait.span())
                                .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                        )
                        .with_help("Remove the enum case from the trait."),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name = context.interner.lookup(&method.name.value);

                check_method(
                    method,
                    method_name,
                    r#trait.span(),
                    class_like_name,
                    class_like_fqcn,
                    "trait",
                    false,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                check_property(property, r#trait.span(), "trait", class_like_name, class_like_fqcn, false, context);
            }
            ClassLikeMember::Constant(class_like_constant) => {
                if !context.version.is_supported(Feature::ConstantsInTraits) {
                    context.issues.push(
                        Issue::error("Constants in traits are only available in PHP 8.2 and above.")
                            .with_annotation(
                                Annotation::primary(class_like_constant.span())
                                    .with_message("Constant defined in trait."),
                            )
                            .with_annotation(
                                Annotation::secondary(r#trait.span())
                                    .with_message(format!("Trait `{}` defined here.", class_like_fqcn)),
                            ),
                    );
                }

                check_class_like_constant(
                    class_like_constant,
                    r#trait.span(),
                    "trait",
                    class_like_name,
                    class_like_fqcn,
                    context,
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_enum(r#enum: &Enum, context: &mut Context<'_>) {
    if !context.version.is_supported(Feature::Enums) {
        context.issues.push(
            Issue::error("Enums are only available in PHP 8.1 and above.")
                .with_annotation(Annotation::primary(r#enum.span()).with_message("Enum defined here.")),
        );

        return;
    }

    let enum_name = context.interner.lookup(&r#enum.name.value);
    let enum_fqcn = context.get_name(&r#enum.name.span.start);
    let enum_is_backed = r#enum.backing_type_hint.is_some();

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(enum_name))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED.iter().any(|keyword| keyword.eq_ignore_ascii_case(enum_name))
    {
        context.issues.push(
            Issue::error(format!("Enum `{}` name cannot be a reserved keyword.", enum_name))
                .with_annotation(
                    Annotation::primary(r#enum.name.span())
                        .with_message(format!("Reserved keyword used as the enum name `{}`.", enum_name)),
                )
                .with_annotation(
                    Annotation::secondary(r#enum.span()).with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                )
                .with_help(format!("Rename the enum `{}` to a non-reserved keyword.", enum_name)),
        );
    }

    if let Some(EnumBackingTypeHint { hint, .. }) = &r#enum.backing_type_hint {
        if !matches!(hint, Hint::String(_) | Hint::Integer(_)) {
            let key = context.get_code_snippet(hint);

            context.issues.push(
                Issue::error(format!(
                    "Enum `{}` backing type must be either `string` or `int`, but found `{}`.",
                    enum_name, key
                ))
                .with_annotation(
                    Annotation::primary(hint.span())
                        .with_message(format!("Invalid backing type `{}` specified here.", key)),
                )
                .with_annotation(
                    Annotation::secondary(r#enum.name.span())
                        .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                )
                .with_help("Change the backing type to either `string` or `int`."),
            );
        }
    }

    if let Some(implements) = &r#enum.implements {
        check_implements(implements, r#enum.span(), "enum", enum_name, enum_fqcn, true, context);
    }

    check_members(&r#enum.members, r#enum.span(), enum_name, enum_fqcn, "enum", context);

    for member in r#enum.members.iter() {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                let item_name_id = case.item.name().value;
                let item_name = context.interner.lookup(&item_name_id);

                match &case.item {
                    EnumCaseItem::Unit(_) => {
                        if enum_is_backed {
                            context.issues.push(
                                Issue::error(format!(
                                    "Case `{}` of backed enum `{}` must have a value.",
                                    item_name, enum_name
                                ))
                                .with_annotation(
                                    Annotation::primary(case.span())
                                        .with_message(format!("Case `{}` defined here.", item_name)),
                                )
                                .with_annotation(
                                    Annotation::secondary(r#enum.span())
                                        .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                )
                                .with_help(format!(
                                    "Add a value to case `{}` or remove the backing from the enum `{}`.",
                                    item_name, enum_name
                                )),
                            );
                        }
                    }
                    EnumCaseItem::Backed(item) => {
                        if !enum_is_backed {
                            context.issues.push(
                                Issue::error(format!(
                                    "Case `{}` of unbacked enum `{}` must not have a value.",
                                    item_name, enum_name
                                ))
                                .with_annotation(
                                    Annotation::primary(item.equals.span().join(item.value.span()))
                                        .with_message("Value assigned to the enum case."),
                                )
                                .with_annotations([
                                    Annotation::secondary(item.name.span())
                                        .with_message(format!("Case `{}::{}` declared here.", enum_name, item_name)),
                                    Annotation::secondary(r#enum.span())
                                        .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                ])
                                .with_help(format!(
                                    "Remove the value from case `{}` or make the enum `{}` backed.",
                                    item_name, enum_name
                                )),
                            );
                        }
                    }
                }
            }
            ClassLikeMember::Method(method) => {
                let method_name_id = method.name.value;
                let method_name = context.interner.lookup(&method_name_id);

                if let Some(magic_method) =
                    MAGIC_METHODS.iter().find(|magic_method| magic_method.eq_ignore_ascii_case(method_name))
                {
                    context.issues.push(
                        Issue::error(format!("Enum `{}` cannot contain magic method `{}`.", enum_name, magic_method))
                            .with_annotation(
                                Annotation::primary(method.name.span)
                                    .with_message(format!("Magic method `{}` declared here.", method_name)),
                            )
                            .with_annotation(
                                Annotation::secondary(r#enum.name.span())
                                    .with_message(format!("Enum `{}` declared here.", enum_fqcn)),
                            )
                            .with_help(format!(
                                "Remove the magic method `{}` from the enum `{}`.",
                                method_name, enum_name
                            )),
                    );
                }

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.issues.push(
                        Issue::error(format!("Enum method `{}::{}` must not be abstract.", enum_name, method_name))
                            .with_annotation(
                                Annotation::primary(abstract_modifier.span())
                                    .with_message("Abstract modifier found here."),
                            )
                            .with_annotations([
                                Annotation::secondary(r#enum.span())
                                    .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                                Annotation::secondary(method.span())
                                    .with_message(format!("Method `{}::{}` defined here.", enum_name, method_name)),
                            ])
                            .with_help(format!(
                                "Remove the abstract modifier from the method `{}` in enum `{}`.",
                                method_name, enum_name
                            )),
                    );
                }

                check_method(method, method_name, r#enum.span(), enum_name, enum_fqcn, "enum", false, context);
            }
            ClassLikeMember::Property(property) => {
                context.issues.push(
                    Issue::error(format!("Enum `{}` cannot have properties.", enum_name))
                        .with_annotation(Annotation::primary(property.span()).with_message("Property defined here."))
                        .with_annotation(
                            Annotation::secondary(r#enum.span())
                                .with_message(format!("Enum `{}` defined here.", enum_fqcn)),
                        )
                        .with_help(format!("Remove the property from the enum `{}`.", enum_name)),
                );

                check_property(property, r#enum.span(), "enum", enum_name, enum_fqcn, false, context);
            }
            ClassLikeMember::Constant(class_like_constant) => {
                check_class_like_constant(class_like_constant, r#enum.span(), "enum", enum_name, enum_fqcn, context);
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_anonymous_class(anonymous_class: &AnonymousClass, context: &mut Context<'_>) {
    let mut last_final = None;
    let mut last_readonly = None;

    for modifier in anonymous_class.modifiers.iter() {
        match &modifier {
            Modifier::Static(_)
            | Modifier::Abstract(_)
            | Modifier::PrivateSet(_)
            | Modifier::ProtectedSet(_)
            | Modifier::PublicSet(_)
            | Modifier::Public(_)
            | Modifier::Protected(_)
            | Modifier::Private(_) => {
                let modifier_name = modifier.as_str(context.interner);

                context.issues.push(
                    Issue::error(format!(
                        "Anonymous class `{}` cannot have the `{}` modifier.",
                        ANONYMOUS_CLASS_NAME, modifier_name
                    ))
                    .with_annotation(
                        Annotation::primary(modifier.span())
                            .with_message(format!("`{}` modifier applied here.", modifier_name)),
                    )
                    .with_annotation(
                        Annotation::secondary(anonymous_class.span())
                            .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                    )
                    .with_help(format!("Remove the `{}` modifier from the class definition.", modifier_name)),
                );
            }
            Modifier::Final(keyword) => {
                if let Some(span) = last_final {
                    context.issues.push(
                        Issue::error(format!(
                            "Anonymous class `{}` cannot have multiple `final` modifiers.",
                            ANONYMOUS_CLASS_NAME
                        ))
                        .with_annotation(
                            Annotation::primary(keyword.span())
                                .with_message("Duplicate `final` modifier applied here."),
                        )
                        .with_annotation(
                            Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                        )
                        .with_annotation(
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                        )
                        .with_help("Remove the duplicate `final` modifier."),
                    );
                }

                last_final = Some(keyword.span);
            }
            Modifier::Readonly(keyword) => {
                if let Some(span) = last_readonly {
                    context.issues.push(
                        Issue::error(format!(
                            "Anonymous class `{}` cannot have multiple `readonly` modifiers.",
                            ANONYMOUS_CLASS_NAME
                        ))
                        .with_annotations([
                            Annotation::primary(keyword.span)
                                .with_message("Duplicate `readonly` modifier applied here."),
                            Annotation::secondary(span).with_message("Previous `readonly` modifier applied here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                        ])
                        .with_help("Remove the duplicate `readonly` modifier."),
                    );
                }

                last_readonly = Some(keyword.span);

                if !context.version.is_supported(Feature::ReadonlyAnonymousClasses) {
                    context.issues.push(
                        Issue::error("Readonly anonymous classes are only available in PHP 8.3 and above.")
                            .with_annotation(
                                Annotation::primary(keyword.span).with_message("Readonly modifier used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                            ),
                    );
                }
            }
        }
    }

    if let Some(extends) = &anonymous_class.extends {
        check_extends(
            extends,
            anonymous_class.span(),
            "class",
            ANONYMOUS_CLASS_NAME,
            ANONYMOUS_CLASS_NAME,
            true,
            context,
        );
    }

    if let Some(implements) = &anonymous_class.implements {
        check_implements(
            implements,
            anonymous_class.span(),
            "class",
            ANONYMOUS_CLASS_NAME,
            ANONYMOUS_CLASS_NAME,
            false,
            context,
        );
    }

    check_members(
        &anonymous_class.members,
        anonymous_class.span(),
        "class",
        ANONYMOUS_CLASS_NAME,
        ANONYMOUS_CLASS_NAME,
        context,
    );

    for member in anonymous_class.members.iter() {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                context.issues.push(
                    Issue::error(format!("Anonymous class `{}` cannot contain enum cases.", ANONYMOUS_CLASS_NAME))
                        .with_annotations([
                            Annotation::primary(case.span()).with_message("Enum case defined here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                        ])
                        .with_help("Remove the enum case from the anonymous class definition."),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name = context.interner.lookup(&method.name.value);

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.issues.push(
                        Issue::error(format!(
                            "Method `{}` in anonymous class `{}` must not be abstract.",
                            method_name, ANONYMOUS_CLASS_NAME
                        ))
                        .with_annotations([
                            Annotation::primary(abstract_modifier.span())
                                .with_message("Abstract modifier applied here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{}` defined here.", ANONYMOUS_CLASS_NAME)),
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{}` defined here.", method_name)),
                        ])
                        .with_help("Remove the `abstract` modifier from the method."),
                    );
                }

                check_method(
                    method,
                    method_name,
                    anonymous_class.span(),
                    ANONYMOUS_CLASS_NAME,
                    ANONYMOUS_CLASS_NAME,
                    "class",
                    false,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                check_property(
                    property,
                    anonymous_class.span(),
                    "class",
                    ANONYMOUS_CLASS_NAME,
                    ANONYMOUS_CLASS_NAME,
                    false,
                    context,
                );
            }
            ClassLikeMember::Constant(class_like_constant) => {
                check_class_like_constant(
                    class_like_constant,
                    anonymous_class.span(),
                    "class",
                    ANONYMOUS_CLASS_NAME,
                    ANONYMOUS_CLASS_NAME,
                    context,
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_members(
    members: &Sequence<ClassLikeMember>,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &str,
    class_like_fqcn: &str,
    context: &mut Context<'_>,
) {
    let mut method_names: Vec<(Span, StringIdentifier)> = vec![];
    let mut constant_names: Vec<(bool, std::string::String, Span)> = vec![];
    let mut property_names: Vec<(bool, StringIdentifier, Span)> = vec![];

    for member in members.iter() {
        match &member {
            ClassLikeMember::Property(property) => match &property {
                Property::Plain(plain_property) => {
                    for item in plain_property.items.iter() {
                        let item_name_id = item.variable().name;
                        let item_name = context.interner.lookup(&item_name_id);

                        if let Some((is_promoted, _, span)) =
                            property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                        {
                            let message = if *is_promoted {
                                format!(
                                    "property `{}::{}` has already been defined as a promoted property",
                                    class_like_name, item_name
                                )
                            } else {
                                format!("property `{}::{}` has already been defined", class_like_name, item_name)
                            };

                            context.issues.push(
                                Issue::error(message)
                                    .with_annotation(Annotation::primary(item.variable().span()))
                                    .with_annotations([
                                        Annotation::secondary(*span).with_message(format!(
                                            "property `{}::{}` previously defined here.",
                                            class_like_name, item_name
                                        )),
                                        Annotation::secondary(class_like_span.span()).with_message(format!(
                                            "{} `{}` defined here.",
                                            class_like_kind, class_like_fqcn
                                        )),
                                    ])
                                    .with_help("remove the duplicate property"),
                            );
                        } else {
                            property_names.push((false, item_name_id, item.variable().span()));
                        }
                    }
                }
                Property::Hooked(hooked_property) => {
                    let item_variable = hooked_property.item.variable();
                    let item_name_id = item_variable.name;
                    let item_name = context.interner.lookup(&item_name_id);

                    if let Some((is_promoted, _, span)) =
                        property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                    {
                        let message = if *is_promoted {
                            format!(
                                "property `{}::{}` has already been defined as a promoted property",
                                class_like_name, item_name
                            )
                        } else {
                            format!("property `{}::{}` has already been defined", class_like_name, item_name)
                        };

                        context.issues.push(
                            Issue::error(message)
                                .with_annotation(Annotation::primary(item_variable.span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "property `{}::{}` previously defined here.",
                                        class_like_name, item_name
                                    )),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ])
                                .with_help("remove the duplicate property"),
                        );
                    } else {
                        property_names.push((false, item_name_id, item_variable.span()));
                    }
                }
            },
            ClassLikeMember::Method(method) => {
                let method_name_id = method.name.value;
                let method_name = context.interner.lookup(&method_name_id);
                let method_name_lowered_id = context.interner.intern(method_name.to_ascii_lowercase());

                if let Some((previous, _)) =
                    method_names.iter().find(|(_, previous_name)| method_name_lowered_id.eq(previous_name))
                {
                    context.issues.push(
                        Issue::error(format!(
                            "{} method `{}::{}` has already been defined",
                            class_like_kind, class_like_name, method_name
                        ))
                        .with_annotation(Annotation::primary(method.name.span()))
                        .with_annotations([
                            Annotation::secondary(*previous).with_message("previous definition"),
                            Annotation::secondary(class_like_span.span())
                                .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                        ]),
                    );
                } else {
                    method_names.push((method.name.span(), method_name_lowered_id));
                }

                if method_name.eq_ignore_ascii_case(CONSTRUCTOR_MAGIC_METHOD) {
                    for parameter in method.parameter_list.parameters.iter() {
                        if parameter.is_promoted_property() {
                            let item_name_id = parameter.variable.name;
                            let item_name = context.interner.lookup(&item_name_id);

                            if let Some((is_promoted, _, span)) =
                                property_names.iter().find(|(_, name, _)| item_name_id.eq(name))
                            {
                                let message = if !*is_promoted {
                                    format!(
                                        "promoted property `{}::{}` has already been defined as a property",
                                        class_like_name, item_name
                                    )
                                } else {
                                    format!(
                                        "promoted property `{}::{}` has already been defined",
                                        class_like_name, item_name
                                    )
                                };

                                context.issues.push(
                                    Issue::error(message)
                                        .with_annotation(Annotation::primary(parameter.variable.span()))
                                        .with_annotations([
                                            Annotation::secondary(*span).with_message(format!(
                                                "property `{}::{}` previously defined here.",
                                                class_like_name, item_name
                                            )),
                                            Annotation::secondary(class_like_span.span()).with_message(format!(
                                                "{} `{}` defined here.",
                                                class_like_kind, class_like_fqcn
                                            )),
                                        ])
                                        .with_help("remove the duplicate property"),
                                );
                            } else {
                                property_names.push((true, item_name_id, parameter.variable.span()));
                            }
                        }
                    }
                }
            }
            ClassLikeMember::Constant(class_like_constant) => {
                for item in class_like_constant.items.iter() {
                    let item_name = context.interner.lookup(&item.name.value);

                    if let Some((is_constant, name, span)) = constant_names.iter().find(|t| t.1.eq(&item_name)) {
                        if *is_constant {
                            context.issues.push(
                                Issue::error(format!(
                                    "{} constant `{}::{}` has already been defined",
                                    class_like_kind, class_like_name, name,
                                ))
                                .with_annotation(Annotation::primary(item.name.span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "Constant `{}::{}` previously defined here.",
                                        class_like_name, name
                                    )),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                            );
                        } else {
                            context.issues.push(
                                Issue::error(format!(
                                    "{} case `{}::{}` and constant `{}::{}` cannot have the same name",
                                    class_like_kind, class_like_name, name, class_like_name, name
                                ))
                                .with_annotation(Annotation::primary(item.name.span()))
                                .with_annotations([
                                    Annotation::secondary(*span)
                                        .with_message(format!("case `{}::{}` defined here.", class_like_name, name)),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{} `{}` defined here.",
                                        class_like_kind, class_like_fqcn
                                    )),
                                ]),
                            );
                        }
                    } else {
                        constant_names.push((true, item_name.to_string(), item.name.span()));
                    }
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_name = context.interner.lookup(&enum_case.item.name().value);

                if let Some((is_constant, name, span)) = constant_names.iter().find(|t| t.1.eq(&case_name)) {
                    if *is_constant {
                        context.issues.push(
                            Issue::error(format!(
                                "{} case `{}::{}` and constant `{}::{}` cannot have the same name",
                                class_like_kind, class_like_name, name, class_like_name, name
                            ))
                            .with_annotation(Annotation::primary(enum_case.item.name().span()))
                            .with_annotations([
                                Annotation::secondary(*span)
                                    .with_message(format!("Constant `{}::{}` defined here.", class_like_name, name)),
                                Annotation::secondary(class_like_span.span())
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                            ]),
                        );
                    } else {
                        context.issues.push(
                            Issue::error(format!(
                                "{} case `{}::{}` has already been defined",
                                class_like_kind, class_like_name, name,
                            ))
                            .with_annotation(Annotation::primary(enum_case.item.name().span()))
                            .with_annotations([
                                Annotation::secondary(*span).with_message(format!(
                                    "case `{}::{}` previously defined here.",
                                    class_like_name, name
                                )),
                                Annotation::secondary(class_like_span.span())
                                    .with_message(format!("{} `{}` defined here.", class_like_kind, class_like_fqcn)),
                            ]),
                        );
                    }

                    continue;
                } else {
                    constant_names.push((false, case_name.to_string(), enum_case.item.name().span()));
                }
            }
            _ => {}
        }
    }
}
