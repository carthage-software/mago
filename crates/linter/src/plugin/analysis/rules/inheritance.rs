use indoc::indoc;

use mago_ast::*;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct InheritanceRule;

impl Rule for InheritanceRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Inheritance", Level::Error)
            .with_description(indoc! {"
                Checks for invalid inheritance relationships, such as extending a final class,
                referencing non-existent classes/interfaces, or creating circular inheritance.
            "})
            .with_example(RuleUsageExample::valid(
                "Extending a class",
                indoc! {"
                    <?php

                    abstract class AbstractFoo {}

                    class Foo extends AbstractFoo {}
                "},
            ))
            .with_example(RuleUsageExample::valid(
                "Implementing an interface",
                indoc! {"
                    <?php

                    interface FooInterface {}

                    class Foo implements FooInterface {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Extending a final class",
                indoc! {"
                    <?php

                    final class Foo {}

                    class Bar extends Foo {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Extending a non-existent class",
                indoc! {"
                    <?php

                    class Foo extends Bar {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Extending a class with circular inheritance",
                indoc! {"
                    <?php

                    class Foo extends Bar {}

                    class Bar extends Foo {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Extending a readonly class from a non-readonly class",
                indoc! {"
                    <?php

                    readonly class Foo {}

                    class Bar extends Foo {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Extending a non-readonly class from a readonly class",
                indoc! {"
                    <?php

                    class Foo {}

                    readonly class Bar extends Foo {}
                "},
            ))
            .with_example(RuleUsageExample::invalid(
                "Implementing a non-existent interface",
                indoc! {"
                    <?php

                    class Foo implements BarInterface {}
                "},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Interface(interface) => {
                let name_identifier = context.semantics.names.get(&interface.name);
                let Some(reflection) = context.codebase.get_interface(context.interner, name_identifier) else {
                    return LintDirective::default();
                };

                if let Some(extends) = interface.extends.as_ref() {
                    for extended_interface in extends.types.iter() {
                        check_interface_extension(reflection, extended_interface, context);
                    }
                }
            }
            Node::Class(class) => {
                let name_identifier = context.semantics.names.get(&class.name);
                let Some(reflection) = context.codebase.get_class(context.interner, name_identifier) else {
                    return LintDirective::default();
                };

                if let Some(extends) = class.extends.as_ref() {
                    for extended_class in extends.types.iter() {
                        check_class_extension(reflection, extended_class, context);
                    }
                }

                if let Some(implements) = class.implements.as_ref() {
                    for implemented_interface in implements.types.iter() {
                        check_interface_implementation(implemented_interface, context);
                    }
                }
            }
            Node::AnonymousClass(anonymous_class) => {
                let Some(reflection) = context.codebase.get_anonymous_class(&anonymous_class) else {
                    return LintDirective::default();
                };

                if let Some(extends) = anonymous_class.extends.as_ref() {
                    for extended_class in extends.types.iter() {
                        check_class_extension(reflection, extended_class, context);
                    }
                }

                if let Some(implements) = anonymous_class.implements.as_ref() {
                    for implemented_interface in implements.types.iter() {
                        check_interface_implementation(implemented_interface, context);
                    }
                }
            }
            Node::Enum(r#enum) => {
                if let Some(implements) = r#enum.implements.as_ref() {
                    for implemented_interface in implements.types.iter() {
                        check_interface_implementation(implemented_interface, context);
                    }
                }
            }
            _ => (),
        };

        LintDirective::default()
    }
}

#[inline]
fn check_class_extension(
    extender: &ClassLikeReflection,
    extended_identifier: &Identifier,
    context: &mut LintContext<'_>,
) {
    let extended_name_identifier = context.semantics.names.get(extended_identifier);
    let extended_name = context.lookup(&extended_identifier.value()).to_string();
    let extended_fqcn = context.lookup(extended_name_identifier).to_string();

    let Some(extended) = context.codebase.get_class(context.interner, extended_name_identifier) else {
        let issue = Issue::error(format!("Extended class `{}` does not exist.", extended_name))
            .with_annotation(
                Annotation::primary(extended_identifier.span())
                    .with_message(format!("Class `{}` does not exist.", extended_fqcn)),
            )
            .with_help(format!("Ensure the class `{}` is defined or imported before extending it.", extended_fqcn));

        context.report(issue);

        return;
    };

    let extender_name = extender.name.get_key(context.interner);

    if extended.is_final {
        let issue = Issue::error(format!("Cannot extend final class `{}` from `{}`.", extended_name, extender_name))
            .with_annotation(
                Annotation::primary(extended_identifier.span())
                    .with_message(format!("Class `{}` is final.", extended_fqcn)),
            )
            .with_help(format!("Ensure the class `{}` is not final or remove the `extends` clause.", extended_fqcn));

        context.report(issue);
    }

    if extender.is_readonly && !extended.is_readonly {
        let issue = Issue::error(format!(
            "Cannot extend non-readonly class `{}` from readonly class `{}`.",
            extended_name, extender_name
        ))
        .with_annotation(
            Annotation::primary(extended_identifier.span())
                .with_message(format!("Class `{}` is not readonly.", extended_fqcn)),
        )
        .with_annotation(
            Annotation::secondary(extender.name.span()).with_message(format!("Class `{}` is readonly.", extender_name)),
        )
        .with_help(format!("Ensure the class `{}` is readonly or remove the `extends` clause.", extended_fqcn));

        context.report(issue);
    } else if !extender.is_readonly && extended.is_readonly {
        let issue = Issue::new(
            context.level(),
            format!("Extending readonly class `{}` from non-readonly class `{}`.", extended_name, extender_name),
        )
        .with_annotation(
            Annotation::primary(extended_identifier.span())
                .with_message(format!("Class `{}` is readonly.", extended_fqcn)),
        )
        .with_annotation(
            Annotation::secondary(extender.name.span())
                .with_message(format!("Class `{}` is not readonly.", extender_name)),
        )
        .with_help(format!("Mark the class `{}` as readonly or remove the `extends` clause.", extender_name));

        context.report(issue);
    }

    if extended.inheritance.extends_class(context.interner, extender) {
        let issue =
            Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", extender_name, extended_name))
                .with_annotation(
                    Annotation::primary(extended_identifier.span())
                        .with_message(format!("Class `{}` already extends `{}`.", extended_fqcn, extender_name)),
                )
                .with_help(format!(
                    "Ensure there is no circular inheritance between `{}` and `{}`.",
                    extender_name, extended_name
                ));

        context.report(issue);
    }
}

#[inline]
fn check_interface_extension(
    extender: &ClassLikeReflection,
    extended_identifier: &Identifier,
    context: &mut LintContext<'_>,
) {
    let extended_name_identifier = context.semantics.names.get(extended_identifier);
    let extended_name = context.lookup(&extended_identifier.value());
    let extended_fqcn = context.lookup(extended_name_identifier);

    let Some(extended) = context.codebase.get_interface(context.interner, extended_name_identifier) else {
        let issue = Issue::error(format!("Extended interface `{}` does not exist.", extended_name))
            .with_annotation(
                Annotation::primary(extended_identifier.span())
                    .with_message(format!("Interface `{}` does not exist.", extended_fqcn)),
            )
            .with_help(format!("Ensure the interface `{}` is defined or imported before extending it.", extended_fqcn));

        context.report(issue);

        return;
    };

    if extended.inheritance.extends_interface(context.interner, extender) {
        let extender_name = extender.name.get_key(context.interner);

        let issue =
            Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", extender_name, extended_name))
                .with_annotation(
                    Annotation::primary(extended_identifier.span())
                        .with_message(format!("Interface `{}` already extends `{}`.", extended_fqcn, extender_name)),
                )
                .with_help(format!(
                    "Ensure there is no circular inheritance between `{}` and `{}`.",
                    extender_name, extended_name
                ));

        context.report(issue);
    }
}

#[inline]
fn check_interface_implementation(implemented_identifier: &Identifier, context: &mut LintContext<'_>) {
    let implemented_name_identifier = context.semantics.names.get(implemented_identifier);
    let implemented_name = context.lookup(&implemented_identifier.value());
    let implemented_fqcn = context.lookup(implemented_name_identifier);

    if context.codebase.interface_exists(context.interner, implemented_name_identifier) {
        return;
    }

    let issue = Issue::error(format!("Implemented interface `{}` does not exist.", implemented_name))
        .with_annotation(
            Annotation::primary(implemented_identifier.span())
                .with_message(format!("Interface `{}` does not exist.", implemented_fqcn)),
        )
        .with_help(format!(
            "Ensure the interface `{}` is defined or imported before implementing it.",
            implemented_fqcn
        ));

    context.report(issue);
}
