use mago_bytes::BytesDisplay;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::AnonymousClass;
use mago_syntax::cst::Class;
use mago_syntax::cst::ClassLikeMember;
use mago_syntax::cst::Enum;
use mago_syntax::cst::EnumBackingTypeHint;
use mago_syntax::cst::EnumCaseItem;
use mago_syntax::cst::Hint;
use mago_syntax::cst::Interface;
use mago_syntax::cst::MethodBody;
use mago_syntax::cst::Modifier;
use mago_syntax::cst::ModifierSequenceExt;
use mago_syntax::cst::Property;
use mago_syntax::cst::PropertyHookBody;
use mago_syntax::cst::PropertyItem;
use mago_syntax::cst::Sequence;
use mago_syntax::cst::Trait;

use crate::internal::consts::ANONYMOUS_CLASS_NAME;
use crate::internal::consts::CONSTRUCTOR_MAGIC_METHOD;
use crate::internal::consts::MAGIC_METHODS;
use crate::internal::consts::RESERVED_KEYWORDS;
use crate::internal::consts::SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED;
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
pub fn check_class<'ast, 'arena>(class: &'ast Class<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
    let class_name_bytes: &[u8] = class.name.value;
    let class_fqcn_bytes: &[u8] = context.get_name(class.name.span.start);
    let class_name = BytesDisplay(class_name_bytes);
    let class_fqcn = BytesDisplay(class_fqcn_bytes);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_name_bytes))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(class_name_bytes))
    {
        context.report(
            Issue::error(format!("Class `{class_name}` name cannot be a reserved keyword."))
                .with_annotation(
                    Annotation::primary(class.name.span())
                        .with_message(format!("Class name `{class_name}` conflicts with a reserved keyword.")),
                )
                .with_annotation(
                    Annotation::secondary(class.span()).with_message(format!("Class `{class_fqcn}` declared here.")),
                )
                .with_help("Rename the class to avoid using reserved keywords."),
        );
    }

    let mut last_final = None;
    let mut last_abstract = None;
    let mut last_readonly = None;

    for modifier in &class.modifiers {
        match &modifier {
            Modifier::Static(_) => {
                context.report(
                    Issue::error(format!("Class `{class_name}` cannot have the `static` modifier."))
                        .with_annotation(
                            Annotation::primary(modifier.span()).with_message("`static` modifier applied here."),
                        )
                        .with_annotation(
                            Annotation::secondary(class.span())
                                .with_message(format!("Class `{class_fqcn}` declared here.")),
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
                let visibility_name = BytesDisplay(keyword.value);

                context.report(
                    Issue::error(format!(
                        "Class `{class_name}` cannot have the `{visibility_name}` visibility modifier."
                    ))
                    .with_annotation(
                        Annotation::primary(keyword.span())
                            .with_message(format!("`{visibility_name}` modifier applied here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(class.span())
                            .with_message(format!("Class `{class_fqcn}` declared here.")),
                    )
                    .with_help(format!("Remove the `{visibility_name}` modifier.")),
                );
            }
            Modifier::Final(keyword) => {
                if let Some(span) = last_abstract {
                    context.report(
                        Issue::error(format!("Abstract class `{class_name}` cannot have the `final` modifier."))
                            .with_annotation(
                                Annotation::primary(keyword.span()).with_message("`final` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `abstract` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{class_fqcn}` declared here.")),
                            ])
                            .with_help("Remove the `final` modifier from the abstract class."),
                    );
                }

                if let Some(span) = last_final {
                    context.report(
                        Issue::error(format!("Class `{class_name}` cannot have multiple `final` modifiers."))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `final` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{class_fqcn}` declared here.")),
                            ])
                            .with_help("Remove the duplicate `final` modifier."),
                    );
                }

                last_final = Some(keyword.span);
            }
            Modifier::Abstract(keyword) => {
                if let Some(span) = last_final {
                    context.report(
                        Issue::error(format!("Final class `{class_name}` cannot have the `abstract` modifier."))
                            .with_annotation(
                                Annotation::primary(keyword.span()).with_message("`abstract` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `final` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{class_fqcn}` declared here.")),
                            ])
                            .with_help("Remove the `abstract` modifier from the final class."),
                    );
                }

                if let Some(span) = last_abstract {
                    context.report(
                        Issue::error(format!("Class `{class_name}` cannot have multiple `abstract` modifiers."))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `abstract` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `abstract` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{class_fqcn}` declared here.")),
                            ])
                            .with_help("Remove the duplicate `abstract` modifier."),
                    );
                }

                last_abstract = Some(keyword.span);
            }
            Modifier::Readonly(keyword) => {
                if let Some(span) = last_readonly {
                    context.report(
                        Issue::error(format!("Class `{class_name}` cannot have multiple `readonly` modifiers."))
                            .with_annotation(
                                Annotation::primary(keyword.span())
                                    .with_message("Duplicate `readonly` modifier applied here."),
                            )
                            .with_annotations([
                                Annotation::secondary(span).with_message("Previous `readonly` modifier applied here."),
                                Annotation::secondary(class.span())
                                    .with_message(format!("Class `{class_fqcn}` declared here.")),
                            ])
                            .with_help("Remove the duplicate `readonly` modifier."),
                    );
                }

                last_readonly = Some(keyword.span);
            }
        }
    }

    if !context.version.is_supported(Feature::ReadonlyClasses)
        && let Some(modifier) = last_readonly
    {
        let issue = Issue::error("Readonly classes are only available in PHP 8.2 and above.")
            .with_annotation(Annotation::primary(modifier.span()).with_message("Readonly modifier used here."));

        context.report(issue);
    }

    if let Some(extends) = &class.extends {
        check_extends(extends, class.span(), "class", class_name_bytes, class_fqcn_bytes, true, context);
    }

    if let Some(implements) = &class.implements {
        check_implements(implements, class.span(), "class", class_name_bytes, class_fqcn_bytes, true, context);
    }

    check_members(&class.members, class.span(), "class", class_name_bytes, class_fqcn_bytes, context);

    for member in &class.members {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                context.report(
                    Issue::error(format!("Class `{class_name}` cannot contain enum cases."))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case found in class."))
                        .with_annotation(
                            Annotation::secondary(class.span())
                                .with_message(format!("Class `{class_fqcn}` declared here.")),
                        )
                        .with_help("Remove the enum cases from the class definition."),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name_bytes: &[u8] = method.name.value;
                let method_name = BytesDisplay(method_name_bytes);

                if !class.modifiers.contains_abstract() && method.modifiers.contains_abstract() {
                    context.report(
                        Issue::error(format!(
                            "Class `{class_name}` contains an abstract method `{method_name}`, so the class must be declared abstract."
                        ))
                        .with_annotation(
                            Annotation::primary(class.name.span())
                                .with_message("Class is missing the `abstract` modifier."),
                        )
                        .with_annotation(
                            Annotation::secondary(method.span()).with_message(format!(
                                "Abstract method `{class_name}::{method_name}` declared here."
                            )),
                        )
                        .with_help("Add the `abstract` modifier to the class."),
                    );
                }

                if last_readonly.is_some() && method_name_bytes.eq_ignore_ascii_case(b"__construct") {
                    for parameter in &method.parameter_list.parameters {
                        if let Some(hooks) = &parameter.hooks {
                            let param_name = BytesDisplay(parameter.variable.name);
                            context.report(
                                Issue::error(format!(
                                    "Hooked property `{class_name}::{param_name}` cannot be readonly."
                                ))
                                .with_annotation(
                                    Annotation::primary(hooks.span())
                                        .with_message("Property hooks are defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(parameter.variable.span())
                                        .with_message(format!("Promoted property `{param_name}` is declared here.")),
                                )
                                .with_annotation(
                                    Annotation::secondary(class.span())
                                        .with_message(format!(
                                            "class `{class_fqcn}` is readonly, making all properties implicitly readonly."
                                        )),
                                )
                                .with_note("Hooked properties cannot be readonly, but properties in readonly classes are implicitly readonly."),
                            );
                        }
                    }
                }

                check_method(
                    method,
                    method_name_bytes,
                    class.span(),
                    class_name_bytes,
                    class_fqcn_bytes,
                    "class",
                    false,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                check_property(
                    property,
                    class.span(),
                    "class",
                    class_name_bytes,
                    class_fqcn_bytes,
                    false,
                    class.modifiers.contains_abstract(),
                    last_readonly.is_some(),
                    context,
                );
            }
            ClassLikeMember::Constant(constant) => {
                check_class_like_constant(constant, class.span(), "class", class_name_bytes, class_fqcn_bytes, context);
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_interface<'ast, 'arena>(interface: &'ast Interface<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
    let interface_name_bytes: &[u8] = interface.name.value;
    let interface_fqcn_bytes: &[u8] = context.get_name(interface.name.span.start);
    let interface_name = BytesDisplay(interface_name_bytes);
    let interface_fqcn = BytesDisplay(interface_fqcn_bytes);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(interface_name_bytes))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(interface_name_bytes))
    {
        context.report(
            Issue::error(format!("Interface `{interface_name}` name cannot be a reserved keyword."))
                .with_annotation(
                    Annotation::primary(interface.name.span())
                        .with_message(format!("Interface `{interface_name}` declared here.")),
                )
                .with_annotation(
                    Annotation::secondary(interface.span())
                        .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                )
                .with_help("Rename the interface to avoid using a reserved keyword."),
        );
    }

    if let Some(extends) = &interface.extends {
        check_extends(
            extends,
            interface.span(),
            "interface",
            interface_name_bytes,
            interface_fqcn_bytes,
            false,
            context,
        );
    }

    check_members(
        &interface.members,
        interface.span(),
        "interface",
        interface_name_bytes,
        interface_fqcn_bytes,
        context,
    );

    for member in &interface.members {
        match &member {
            ClassLikeMember::TraitUse(trait_use) => {
                context.report(
                    Issue::error(format!("Interface `{interface_name}` cannot use traits."))
                        .with_annotation(Annotation::primary(trait_use.span()).with_message("Trait use statement."))
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                        )
                        .with_help("Remove the trait use statement."),
                );
            }
            ClassLikeMember::EnumCase(case) => {
                context.report(
                    Issue::error(format!("Interface `{interface_name}` cannot contain enum cases."))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case declared here."))
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                        )
                        .with_note(
                            "Consider moving the enum case to an enum or class if it represents state or constants.",
                        ),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name_bytes: &[u8] = method.name.value;
                let method_name = BytesDisplay(method_name_bytes);

                let mut visibilities = vec![];
                for modifier in &method.modifiers {
                    if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                        visibilities.push(modifier);
                    }
                }

                for visibility in visibilities {
                    let visibility_name = BytesDisplay(visibility.get_keyword().value);

                    context.report(
                        Issue::error(format!(
                            "Interface method `{interface_name}::{method_name}` cannot have `{visibility_name}` modifier."
                        ))
                        .with_annotation(
                            Annotation::primary(visibility.span())
                                .with_message(format!("`{visibility_name}` modifier applied here.")),
                        )
                        .with_annotation(
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                        )
                        .with_help(format!(
                            "Remove the `{visibility_name}` modifier from the method definition as methods in interfaces must always be public."
                        ))
                        .with_note("Interface methods are always public and cannot have non-public visibility modifiers."),
                    );
                }

                if let MethodBody::Concrete(body) = &method.body {
                    context.report(
                        Issue::error(format!("Interface method `{interface_name}::{method_name}` cannot have a body."))
                            .with_annotations([
                                Annotation::primary(body.span()).with_message("Method body declared here."),
                                Annotation::primary(method.name.span()).with_message("Method name defined here."),
                                Annotation::secondary(interface.span())
                                    .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                            ])
                            .with_help("Replace the method body with a `;` to indicate it is abstract.")
                            .with_note("Methods in interfaces cannot have implementations and must be abstract."),
                    );
                }

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.report(
                        Issue::error(format!(
                            "Interface method `{interface_name}::{method_name}` must not be abstract."
                        ))
                        .with_annotation(
                            Annotation::primary(abstract_modifier.span())
                                .with_message("Abstract modifier applied here."),
                        )
                        .with_annotations([
                            Annotation::secondary(interface.span())
                                .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{interface_name}::{method_name}` declared here.")),
                        ])
                        .with_help("Remove the `abstract` modifier as all interface methods are implicitly abstract.")
                        .with_note(
                            "Adding the `abstract` modifier to an interface method is redundant because all interface methods are implicitly abstract.",
                        ),
                    );
                }

                check_method(
                    method,
                    method_name_bytes,
                    interface.span(),
                    interface_name_bytes,
                    interface_fqcn_bytes,
                    "interface",
                    true,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                match &property {
                    Property::Plain(plain_property) => {
                        context.report(
                                    Issue::error(format!(
                                        "Interface `{interface_name}` cannot have non-hooked properties."
                                    ))
                                    .with_annotation(
                                        Annotation::primary(plain_property.span())
                                            .with_message("Non-hooked property declared here."),
                                    )
                                    .with_annotation(
                                        Annotation::secondary(interface.span())
                                            .with_message(format!("Interface `{interface_fqcn}` declared here.")),
                                    )
                                    .with_note("Interfaces are intended to define behavior and cannot include concrete property declarations.")
                                    .with_help("Remove the non-hooked property from the interface or convert it into a hooked property.")
                                );
                    }
                    Property::Hooked(hooked_property) => {
                        let property_name = BytesDisplay(hooked_property.item.variable().name);

                        let mut found_public = false;
                        let mut non_public_read_visibilities = vec![];
                        let mut write_visibilities = vec![];
                        for modifier in &hooked_property.modifiers {
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
                            let visibility_name = BytesDisplay(visibility.get_keyword().value);

                            context.report(
                                        Issue::error(format!(
                                            "Interface virtual property `{interface_name}::{property_name}` must not specify asymmetric visibility.",
                                        ))
                                        .with_annotation(
                                            Annotation::primary(visibility.span())
                                                .with_message(format!("Asymmetric visibility modifier `{visibility_name}` applied here.")),
                                        )
                                        .with_annotation(
                                            Annotation::secondary(interface.span())
                                                .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                        )
                                        .with_help(format!(
                                            "Remove the `{visibility_name}` modifier from the property to make it compatible with interface constraints."
                                        )),
                                    );
                        }

                        for visibility in non_public_read_visibilities {
                            let visibility_name = BytesDisplay(visibility.get_keyword().value);

                            context.report(
                                Issue::error(format!(
                                    "Interface virtual property `{interface_name}::{property_name}` cannot have `{visibility_name}` modifier.",
                                ))
                                .with_annotation(
                                    Annotation::primary(visibility.span()).with_message(format!(
                                        "Visibility modifier `{visibility_name}` applied here."
                                    )),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                )
                                .with_help(format!(
                                    "Remove the `{visibility_name}` modifier from the property to meet interface requirements."
                                )),
                            );
                        }

                        if !found_public {
                            context.report(
                                Issue::error(format!(
                                    "Interface virtual property `{interface_name}::{property_name}` must be declared public."
                                ))
                                .with_annotation(
                                    Annotation::primary(hooked_property.span()).with_message("Property defined here."),
                                )
                                .with_annotation(
                                    Annotation::secondary(interface.span())
                                        .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                )
                                .with_help("Add the `public` visibility modifier to the property."),
                            );
                        }

                        if let Some(abstract_modifier) = hooked_property.modifiers.get_abstract() {
                            context.report(
                                            Issue::error(format!(
                                                "Interface virtual property `{interface_name}::{property_name}` cannot be abstract."
                                            ))
                                            .with_annotation(
                                                Annotation::primary(abstract_modifier.span())
                                                    .with_message("Abstract modifier applied here."),
                                            )
                                            .with_annotations([
                                                Annotation::secondary(hooked_property.span())
                                                    .with_message("Property defined here."),
                                                Annotation::secondary(interface.span())
                                                    .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                            ])
                                            .with_note(
                                                "All interface virtual properties are implicitly abstract and cannot be explicitly declared as abstract.",
                                            ),
                                        );
                        }

                        if let PropertyItem::Concrete(item) = &hooked_property.item {
                            context.report(
                                Issue::error(format!(
                                    "Interface virtual property `{interface_name}::{property_name}` cannot have a default value."
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
                                        .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                )
                                .with_note(
                                    "Interface properties are virtual properties and cannot contain a default value.",
                                ),
                            );
                        }

                        for hook in &hooked_property.hook_list.hooks {
                            if let PropertyHookBody::Concrete(property_hook_concrete_body) = &hook.body {
                                context.report(
                                    Issue::error(format!(
                                        "Interface virtual property `{interface_name}::{property_name}` must be abstract."
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
                                            .with_message(format!("Interface `{interface_fqcn}` defined here.")),
                                    )
                                    .with_note("Abstract hooked properties must not contain a body."),
                                );
                            }
                        }
                    }
                }

                check_property(
                    property,
                    interface.span(),
                    "interface",
                    interface_name_bytes,
                    interface_fqcn_bytes,
                    true,
                    false,
                    false,
                    context,
                );
            }
            ClassLikeMember::Constant(class_like_constant) => {
                let mut non_public_read_visibility = vec![];
                for modifier in &class_like_constant.modifiers {
                    if matches!(modifier, Modifier::Private(_) | Modifier::Protected(_)) {
                        non_public_read_visibility.push(modifier);
                    }
                }

                for visibility in &non_public_read_visibility {
                    let visibility_name = BytesDisplay(visibility.get_keyword().value);

                    context.report(
                        Issue::error(format!(
                            "Interface constant cannot have `{visibility_name}` visibility modifier.",
                        ))
                        .with_annotation(
                            Annotation::primary(visibility.span())
                                .with_message(format!("Visibility modifier `{visibility_name}` applied here.")),
                        )
                        .with_help(format!(
                            "Remove the `{visibility_name}` modifier from the constant to comply with interface requirements."
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
                    interface_name_bytes,
                    interface_fqcn_bytes,
                    context,
                );
            }
        }
    }
}

#[inline]
pub fn check_trait<'ast, 'arena>(r#trait: &'ast Trait<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
    let class_like_name_bytes: &[u8] = r#trait.name.value;
    let class_like_fqcn_bytes: &[u8] = context.get_name(r#trait.name.span.start);
    let class_like_name = BytesDisplay(class_like_name_bytes);
    let class_like_fqcn = BytesDisplay(class_like_fqcn_bytes);

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(class_like_name_bytes))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(class_like_name_bytes))
    {
        context.report(
            Issue::error(format!("Trait `{class_like_name}` name cannot be a reserved keyword."))
                .with_annotation(
                    Annotation::primary(r#trait.name.span())
                        .with_message(format!("Trait `{class_like_name}` declared here.")),
                )
                .with_annotation(
                    Annotation::secondary(r#trait.span())
                        .with_message(format!("Trait `{class_like_fqcn}` defined here.")),
                )
                .with_help("Rename the trait to a non-reserved keyword."),
        );
    }

    check_members(&r#trait.members, r#trait.span(), "trait", class_like_name_bytes, class_like_fqcn_bytes, context);

    for member in &r#trait.members {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                context.report(
                    Issue::error(format!("Trait `{class_like_name}` cannot contain enum cases."))
                        .with_annotation(Annotation::primary(case.span()).with_message("Enum case defined here."))
                        .with_annotation(
                            Annotation::secondary(r#trait.span())
                                .with_message(format!("Trait `{class_like_fqcn}` defined here.")),
                        )
                        .with_help("Remove the enum case from the trait."),
                );
            }
            ClassLikeMember::Method(method) => {
                check_method(
                    method,
                    method.name.value,
                    r#trait.span(),
                    class_like_name_bytes,
                    class_like_fqcn_bytes,
                    "trait",
                    false,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                check_property(
                    property,
                    r#trait.span(),
                    "trait",
                    class_like_name_bytes,
                    class_like_fqcn_bytes,
                    false,
                    false,
                    false,
                    context,
                );
            }
            ClassLikeMember::Constant(class_like_constant) => {
                if !context.version.is_supported(Feature::ConstantsInTraits) {
                    context.report(
                        Issue::error("Constants in traits are only available in PHP 8.2 and above.")
                            .with_annotation(
                                Annotation::primary(class_like_constant.span())
                                    .with_message("Constant defined in trait."),
                            )
                            .with_annotation(
                                Annotation::secondary(r#trait.span())
                                    .with_message(format!("Trait `{class_like_fqcn}` defined here.")),
                            ),
                    );
                }

                check_class_like_constant(
                    class_like_constant,
                    r#trait.span(),
                    "trait",
                    class_like_name_bytes,
                    class_like_fqcn_bytes,
                    context,
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_enum<'ast, 'arena>(r#enum: &'ast Enum<'arena>, context: &mut Context<'_, 'ast, 'arena>) {
    if !context.version.is_supported(Feature::Enums) {
        context.report(
            Issue::error("Enums are only available in PHP 8.1 and above.")
                .with_annotation(Annotation::primary(r#enum.span()).with_message("Enum defined here.")),
        );

        return;
    }

    let enum_name_bytes: &[u8] = r#enum.name.value;
    let enum_fqcn_bytes: &[u8] = context.get_name(r#enum.name.span.start);
    let enum_name = BytesDisplay(enum_name_bytes);
    let enum_fqcn = BytesDisplay(enum_fqcn_bytes);
    let enum_is_backed = r#enum.backing_type_hint.is_some();

    if RESERVED_KEYWORDS.iter().any(|keyword| keyword.eq_ignore_ascii_case(enum_name_bytes))
        || SOFT_RESERVED_KEYWORDS_MINUS_SYMBOL_ALLOWED
            .iter()
            .any(|keyword| keyword.eq_ignore_ascii_case(enum_name_bytes))
    {
        context.report(
            Issue::error(format!("Enum `{enum_name}` name cannot be a reserved keyword."))
                .with_annotation(
                    Annotation::primary(r#enum.name.span())
                        .with_message(format!("Reserved keyword used as the enum name `{enum_name}`.")),
                )
                .with_annotation(
                    Annotation::secondary(r#enum.span()).with_message(format!("Enum `{enum_fqcn}` defined here.")),
                )
                .with_help(format!("Rename the enum `{enum_name}` to a non-reserved keyword.")),
        );
    }

    if let Some(EnumBackingTypeHint { hint, .. }) = &r#enum.backing_type_hint
        && !matches!(hint, Hint::String(_) | Hint::Integer(_))
    {
        let key = BytesDisplay(context.get_code_snippet(hint));

        context.report(
            Issue::error(format!(
                "Enum `{enum_name}` backing type must be either `string` or `int`, but found `{key}`."
            ))
            .with_annotation(
                Annotation::primary(hint.span()).with_message(format!("Invalid backing type `{key}` specified here.")),
            )
            .with_annotation(
                Annotation::secondary(r#enum.name.span()).with_message(format!("Enum `{enum_fqcn}` defined here.")),
            )
            .with_help("Change the backing type to either `string` or `int`."),
        );
    }

    if let Some(implements) = &r#enum.implements {
        check_implements(implements, r#enum.span(), "enum", enum_name_bytes, enum_fqcn_bytes, true, context);
    }

    check_members(&r#enum.members, r#enum.span(), "enum", enum_name_bytes, enum_fqcn_bytes, context);

    for member in &r#enum.members {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                let item_name = BytesDisplay(case.item.name().value);

                match &case.item {
                    EnumCaseItem::Unit(_) => {
                        if enum_is_backed {
                            context.report(
                                Issue::error(format!(
                                    "Case `{item_name}` of backed enum `{enum_name}` must have a value."
                                ))
                                .with_annotation(
                                    Annotation::primary(case.span())
                                        .with_message(format!("Case `{item_name}` defined here.")),
                                )
                                .with_annotation(
                                    Annotation::secondary(r#enum.span())
                                        .with_message(format!("Enum `{enum_fqcn}` defined here.")),
                                )
                                .with_help(format!(
                                    "Add a value to case `{item_name}` or remove the backing from the enum `{enum_name}`."
                                )),
                            );
                        }
                    }
                    EnumCaseItem::Backed(item) => {
                        if !enum_is_backed {
                            context.report(
                                Issue::error(format!(
                                    "Case `{item_name}` of unbacked enum `{enum_name}` must not have a value."
                                ))
                                .with_annotation(
                                    Annotation::primary(item.equals.span().join(item.value.span()))
                                        .with_message("Value assigned to the enum case."),
                                )
                                .with_annotations([
                                    Annotation::secondary(item.name.span())
                                        .with_message(format!("Case `{enum_name}::{item_name}` declared here.")),
                                    Annotation::secondary(r#enum.span())
                                        .with_message(format!("Enum `{enum_fqcn}` defined here.")),
                                ])
                                .with_help(format!(
                                    "Remove the value from case `{item_name}` or make the enum `{enum_name}` backed."
                                )),
                            );
                        }
                    }
                }
            }
            ClassLikeMember::Method(method) => {
                let method_name_bytes: &[u8] = method.name.value;
                let method_name = BytesDisplay(method_name_bytes);

                if let Some(magic_method) =
                    MAGIC_METHODS.iter().find(|magic_method| magic_method.eq_ignore_ascii_case(method_name_bytes))
                {
                    let magic_method = BytesDisplay(magic_method);
                    context.report(
                        Issue::error(format!("Enum `{enum_name}` cannot contain magic method `{magic_method}`."))
                            .with_annotation(
                                Annotation::primary(method.name.span)
                                    .with_message(format!("Magic method `{method_name}` declared here.")),
                            )
                            .with_annotation(
                                Annotation::secondary(r#enum.name.span())
                                    .with_message(format!("Enum `{enum_fqcn}` declared here.")),
                            )
                            .with_help(format!("Remove the magic method `{method_name}` from the enum `{enum_name}`.")),
                    );
                }

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.report(
                        Issue::error(format!("Enum method `{enum_name}::{method_name}` must not be abstract."))
                            .with_annotation(
                                Annotation::primary(abstract_modifier.span())
                                    .with_message("Abstract modifier found here."),
                            )
                            .with_annotations([
                                Annotation::secondary(r#enum.span())
                                    .with_message(format!("Enum `{enum_fqcn}` defined here.")),
                                Annotation::secondary(method.span())
                                    .with_message(format!("Method `{enum_name}::{method_name}` defined here.")),
                            ])
                            .with_help(format!(
                                "Remove the abstract modifier from the method `{method_name}` in enum `{enum_name}`."
                            )),
                    );
                }

                check_method(
                    method,
                    method_name_bytes,
                    r#enum.span(),
                    enum_name_bytes,
                    enum_fqcn_bytes,
                    "enum",
                    false,
                    context,
                );
            }
            ClassLikeMember::Property(property) => {
                context.report(
                    Issue::error(format!("Enum `{enum_name}` cannot have properties."))
                        .with_annotation(Annotation::primary(property.span()).with_message("Property defined here."))
                        .with_annotation(
                            Annotation::secondary(r#enum.span())
                                .with_message(format!("Enum `{enum_fqcn}` defined here.")),
                        )
                        .with_help(format!("Remove the property from the enum `{enum_name}`.")),
                );

                check_property(
                    property,
                    r#enum.span(),
                    "enum",
                    enum_name_bytes,
                    enum_fqcn_bytes,
                    false,
                    false,
                    false,
                    context,
                );
            }
            ClassLikeMember::Constant(class_like_constant) => {
                check_class_like_constant(
                    class_like_constant,
                    r#enum.span(),
                    "enum",
                    enum_name_bytes,
                    enum_fqcn_bytes,
                    context,
                );
            }
            _ => {}
        }
    }
}

#[inline]
pub fn check_anonymous_class<'ast, 'arena>(
    anonymous_class: &'ast AnonymousClass<'arena>,
    context: &mut Context<'_, 'ast, 'arena>,
) {
    let anonymous_class_name = BytesDisplay(ANONYMOUS_CLASS_NAME);
    let mut last_final = None;
    let mut last_readonly = None;

    for modifier in &anonymous_class.modifiers {
        match &modifier {
            Modifier::Static(_)
            | Modifier::Abstract(_)
            | Modifier::PrivateSet(_)
            | Modifier::ProtectedSet(_)
            | Modifier::PublicSet(_)
            | Modifier::Public(_)
            | Modifier::Protected(_)
            | Modifier::Private(_) => {
                let modifier_name = BytesDisplay(modifier.get_keyword().value);

                context.report(
                    Issue::error(format!(
                        "Anonymous class `{anonymous_class_name}` cannot have the `{modifier_name}` modifier."
                    ))
                    .with_annotation(
                        Annotation::primary(modifier.span())
                            .with_message(format!("`{modifier_name}` modifier applied here.")),
                    )
                    .with_annotation(
                        Annotation::secondary(anonymous_class.span())
                            .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
                    )
                    .with_help(format!("Remove the `{modifier_name}` modifier from the class definition.")),
                );
            }
            Modifier::Final(keyword) => {
                if let Some(span) = last_final {
                    context.report(
                        Issue::error(format!(
                            "Anonymous class `{anonymous_class_name}` cannot have multiple `final` modifiers."
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
                                .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
                        )
                        .with_help("Remove the duplicate `final` modifier."),
                    );
                }

                last_final = Some(keyword.span);
            }
            Modifier::Readonly(keyword) => {
                if let Some(span) = last_readonly {
                    context.report(
                        Issue::error(format!(
                            "Anonymous class `{anonymous_class_name}` cannot have multiple `readonly` modifiers."
                        ))
                        .with_annotations([
                            Annotation::primary(keyword.span)
                                .with_message("Duplicate `readonly` modifier applied here."),
                            Annotation::secondary(span).with_message("Previous `readonly` modifier applied here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
                        ])
                        .with_help("Remove the duplicate `readonly` modifier."),
                    );
                }

                last_readonly = Some(keyword.span);

                if !context.version.is_supported(Feature::ReadonlyAnonymousClasses) {
                    context.report(
                        Issue::error("Readonly anonymous classes are only available in PHP 8.3 and above.")
                            .with_annotation(
                                Annotation::primary(keyword.span).with_message("Readonly modifier used here."),
                            )
                            .with_annotation(
                                Annotation::secondary(anonymous_class.span())
                                    .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
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

    for member in &anonymous_class.members {
        match &member {
            ClassLikeMember::EnumCase(case) => {
                context.report(
                    Issue::error(format!("Anonymous class `{anonymous_class_name}` cannot contain enum cases."))
                        .with_annotations([
                            Annotation::primary(case.span()).with_message("Enum case defined here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
                        ])
                        .with_help("Remove the enum case from the anonymous class definition."),
                );
            }
            ClassLikeMember::Method(method) => {
                let method_name_bytes: &[u8] = method.name.value;
                let method_name = BytesDisplay(method_name_bytes);

                if let Some(abstract_modifier) = method.modifiers.get_abstract() {
                    context.report(
                        Issue::error(format!(
                            "Method `{method_name}` in anonymous class `{anonymous_class_name}` must not be abstract."
                        ))
                        .with_annotations([
                            Annotation::primary(abstract_modifier.span())
                                .with_message("Abstract modifier applied here."),
                            Annotation::secondary(anonymous_class.span())
                                .with_message(format!("Anonymous class `{anonymous_class_name}` defined here.")),
                            Annotation::secondary(method.span())
                                .with_message(format!("Method `{method_name}` defined here.")),
                        ])
                        .with_help("Remove the `abstract` modifier from the method."),
                    );
                }

                check_method(
                    method,
                    method_name_bytes,
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
                    false,
                    last_readonly.is_some(),
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
pub fn check_members<'ast, 'arena>(
    members: &'ast Sequence<ClassLikeMember<'arena>>,
    class_like_span: Span,
    class_like_kind: &str,
    class_like_name: &[u8],
    class_like_fqcn: &[u8],
    context: &mut Context<'_, 'ast, 'arena>,
) {
    let class_like_name = BytesDisplay(class_like_name);
    let class_like_fqcn = BytesDisplay(class_like_fqcn);
    let mut method_names: Vec<(Span, Vec<u8>)> = vec![];
    let mut constant_names: Vec<(bool, Vec<u8>, Span)> = vec![];
    let mut property_names: Vec<(bool, &[u8], Span)> = vec![];

    for member in members {
        match &member {
            ClassLikeMember::Property(property) => match &property {
                Property::Plain(plain_property) => {
                    for item in &plain_property.items {
                        let item_name_bytes: &[u8] = item.variable().name;
                        let item_name = BytesDisplay(item_name_bytes);

                        if let Some((is_promoted, _, span)) =
                            property_names.iter().find(|(_, name, _)| item_name_bytes.eq(*name))
                        {
                            let message = if *is_promoted {
                                format!(
                                    "property `{class_like_name}::{item_name}` has already been defined as a promoted property"
                                )
                            } else {
                                format!("property `{class_like_name}::{item_name}` has already been defined")
                            };

                            context.report(
                                Issue::error(message)
                                    .with_annotation(Annotation::primary(item.variable().span()))
                                    .with_annotations([
                                        Annotation::secondary(*span).with_message(format!(
                                            "property `{class_like_name}::{item_name}` previously defined here."
                                        )),
                                        Annotation::secondary(class_like_span.span()).with_message(format!(
                                            "{class_like_kind} `{class_like_fqcn}` defined here."
                                        )),
                                    ])
                                    .with_help("remove the duplicate property"),
                            );
                        } else {
                            property_names.push((false, item_name_bytes, item.variable().span()));
                        }
                    }
                }
                Property::Hooked(hooked_property) => {
                    let item_variable = hooked_property.item.variable();
                    let item_name_bytes: &[u8] = item_variable.name;
                    let item_name = BytesDisplay(item_name_bytes);

                    if let Some((is_promoted, _, span)) =
                        property_names.iter().find(|(_, name, _)| item_name_bytes.eq(*name))
                    {
                        let message = if *is_promoted {
                            format!(
                                "property `{class_like_name}::{item_name}` has already been defined as a promoted property"
                            )
                        } else {
                            format!("property `{class_like_name}::{item_name}` has already been defined")
                        };

                        context.report(
                            Issue::error(message)
                                .with_annotation(Annotation::primary(item_variable.span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "property `{class_like_name}::{item_name}` previously defined here."
                                    )),
                                    Annotation::secondary(class_like_span.span())
                                        .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                                ])
                                .with_help("remove the duplicate property"),
                        );
                    } else {
                        property_names.push((false, item_name_bytes, item_variable.span()));
                    }
                }
            },
            ClassLikeMember::Method(method) => {
                let method_name_bytes: &[u8] = method.name.value;
                let method_name = BytesDisplay(method_name_bytes);
                let lowercase_method_name = method_name_bytes.to_ascii_lowercase();

                if let Some((previous, _)) =
                    method_names.iter().find(|(_, previous_name)| lowercase_method_name.eq(previous_name))
                {
                    context.report(
                        Issue::error(format!(
                            "{class_like_kind} method `{class_like_name}::{method_name}` has already been defined"
                        ))
                        .with_annotation(Annotation::primary(method.name.span()))
                        .with_annotations([
                            Annotation::secondary(*previous).with_message("previous definition"),
                            Annotation::secondary(class_like_span.span())
                                .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                        ]),
                    );
                } else {
                    method_names.push((method.name.span(), lowercase_method_name));
                }

                if method_name_bytes.eq_ignore_ascii_case(CONSTRUCTOR_MAGIC_METHOD) {
                    for parameter in &method.parameter_list.parameters {
                        if parameter.is_promoted_property() {
                            let item_name_bytes: &[u8] = parameter.variable.name;
                            let item_name = BytesDisplay(item_name_bytes);

                            if let Some((is_promoted, _, span)) =
                                property_names.iter().find(|(_, name, _)| item_name_bytes.eq(*name))
                            {
                                let message = if *is_promoted {
                                    format!(
                                        "promoted property `{class_like_name}::{item_name}` has already been defined"
                                    )
                                } else {
                                    format!(
                                        "promoted property `{class_like_name}::{item_name}` has already been defined as a property"
                                    )
                                };

                                context.report(
                                    Issue::error(message)
                                        .with_annotation(Annotation::primary(parameter.variable.span()))
                                        .with_annotations([
                                            Annotation::secondary(*span).with_message(format!(
                                                "property `{class_like_name}::{item_name}` previously defined here."
                                            )),
                                            Annotation::secondary(class_like_span.span()).with_message(format!(
                                                "{class_like_kind} `{class_like_fqcn}` defined here."
                                            )),
                                        ])
                                        .with_help("remove the duplicate property"),
                                );
                            } else {
                                property_names.push((true, item_name_bytes, parameter.variable.span()));
                            }
                        }
                    }
                }
            }
            ClassLikeMember::Constant(class_like_constant) => {
                for item in &class_like_constant.items {
                    let item_name_bytes: &[u8] = item.name.value;

                    if let Some((is_constant, name, span)) =
                        constant_names.iter().find(|t| t.1.as_slice() == item_name_bytes)
                    {
                        let name = BytesDisplay(name.as_slice());
                        if *is_constant {
                            context.report(
                                Issue::error(format!(
                                    "{class_like_kind} constant `{class_like_name}::{name}` has already been defined",
                                ))
                                .with_annotation(Annotation::primary(item.name.span()))
                                .with_annotations([
                                    Annotation::secondary(*span).with_message(format!(
                                        "Constant `{class_like_name}::{name}` previously defined here."
                                    )),
                                    Annotation::secondary(class_like_span.span())
                                        .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                                ]),
                            );
                        } else {
                            context.report(
                                Issue::error(format!(
                                    "{class_like_kind} case `{class_like_name}::{name}` and constant `{class_like_name}::{name}` cannot have the same name"
                                ))
                                .with_annotation(Annotation::primary(item.name.span()))
                                .with_annotations([
                                    Annotation::secondary(*span)
                                        .with_message(format!("case `{class_like_name}::{name}` defined here.")),
                                    Annotation::secondary(class_like_span.span()).with_message(format!(
                                        "{class_like_kind} `{class_like_fqcn}` defined here."
                                    )),
                                ]),
                            );
                        }
                    } else {
                        constant_names.push((true, item_name_bytes.to_vec(), item.name.span()));
                    }
                }
            }
            ClassLikeMember::EnumCase(enum_case) => {
                let case_name_bytes: &[u8] = enum_case.item.name().value;

                if let Some((is_constant, name, span)) =
                    constant_names.iter().find(|t| t.1.as_slice() == case_name_bytes)
                {
                    let name = BytesDisplay(name.as_slice());
                    if *is_constant {
                        context.report(
                            Issue::error(format!(
                                "{class_like_kind} case `{class_like_name}::{name}` and constant `{class_like_name}::{name}` cannot have the same name"
                            ))
                            .with_annotation(Annotation::primary(enum_case.item.name().span()))
                            .with_annotations([
                                Annotation::secondary(*span)
                                    .with_message(format!("Constant `{class_like_name}::{name}` defined here.")),
                                Annotation::secondary(class_like_span.span())
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ]),
                        );
                    } else {
                        context.report(
                            Issue::error(format!(
                                "{class_like_kind} case `{class_like_name}::{name}` has already been defined",
                            ))
                            .with_annotation(Annotation::primary(enum_case.item.name().span()))
                            .with_annotations([
                                Annotation::secondary(*span)
                                    .with_message(format!("case `{class_like_name}::{name}` previously defined here.")),
                                Annotation::secondary(class_like_span.span())
                                    .with_message(format!("{class_like_kind} `{class_like_fqcn}` defined here.")),
                            ]),
                        );
                    }

                    continue;
                }
                constant_names.push((false, case_name_bytes.to_vec(), enum_case.item.name().span()));
            }
            _ => {}
        }
    }
}
