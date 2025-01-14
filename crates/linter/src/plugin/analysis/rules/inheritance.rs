use indoc::indoc;

use mago_ast::*;
use mago_reflection::class_like::ClassLikeReflection;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
}

impl InheritanceRule {
    fn extend_class(this: &ClassLikeReflection, other_identifier: &Identifier, context: &mut LintContext<'_>) {
        let other_name_identifier = context.semantics.names.get(other_identifier);
        let other_name = context.lookup(&other_identifier.value()).to_string();
        let other_fqcn = context.lookup(other_name_identifier).to_string();

        let Some(other) = context.codebase.get_class(context.interner, other_name_identifier) else {
            let issue = Issue::error(format!("Extended class `{}` does not exist.", other_name))
                .with_annotation(
                    Annotation::primary(other_identifier.span())
                        .with_message(format!("Class `{}` does not exist.", other_fqcn)),
                )
                .with_help(format!("Ensure the class `{}` is defined or imported before extending it.", other_fqcn));

            context.report(issue);

            return;
        };

        let this_name = this.name.get_key(context.interner);

        if other.is_final {
            let issue = Issue::error(format!("Cannot extend final class `{}` from `{}`.", other_name, this_name))
                .with_annotation(
                    Annotation::primary(other_identifier.span())
                        .with_message(format!("Class `{}` is final.", other_fqcn)),
                )
                .with_help(format!("Ensure the class `{}` is not final or remove the `extends` clause.", other_fqcn));

            context.report(issue);
        }

        if this.is_readonly && !other.is_readonly {
            let issue = Issue::error(format!(
                "Cannot extend non-readonly class `{}` from readonly class `{}`.",
                other_name, this_name
            ))
            .with_annotation(
                Annotation::primary(other_identifier.span())
                    .with_message(format!("Class `{}` is not readonly.", other_fqcn)),
            )
            .with_annotation(
                Annotation::secondary(this.name.span()).with_message(format!("Class `{}` is readonly.", this_name)),
            )
            .with_help(format!("Ensure the class `{}` is readonly or remove the `extends` clause.", other_fqcn));

            context.report(issue);
        } else if !this.is_readonly && other.is_readonly {
            let issue = Issue::new(
                context.level(),
                format!("Extending readonly class `{}` from non-readonly class `{}`.", other_name, this_name),
            )
            .with_annotation(
                Annotation::primary(other_identifier.span())
                    .with_message(format!("Class `{}` is readonly.", other_fqcn)),
            )
            .with_annotation(
                Annotation::secondary(this.name.span()).with_message(format!("Class `{}` is not readonly.", this_name)),
            )
            .with_help(format!("Mark the class `{}` as readonly or remove the `extends` clause.", this_name));

            context.report(issue);
        }

        if other.inheritance.extends_class(context.interner, this) {
            let issue =
                Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", this_name, other_name))
                    .with_annotation(
                        Annotation::primary(other_identifier.span())
                            .with_message(format!("Class `{}` already extends `{}`.", other_fqcn, this_name)),
                    )
                    .with_help(format!(
                        "Ensure there is no circular inheritance between `{}` and `{}`.",
                        this_name, other_name
                    ));

            context.report(issue);
        }
    }

    fn extend_interface(this: &ClassLikeReflection, other_identifier: &Identifier, context: &mut LintContext<'_>) {
        let other_name_identifier = context.semantics.names.get(other_identifier);
        let other_name = context.lookup(&other_identifier.value());
        let other_fqcn = context.lookup(other_name_identifier);

        let Some(other) = context.codebase.get_interface(context.interner, other_name_identifier) else {
            let issue = Issue::error(format!("Extended interface `{}` does not exist.", other_name))
                .with_annotation(
                    Annotation::primary(other_identifier.span())
                        .with_message(format!("Interface `{}` does not exist.", other_fqcn)),
                )
                .with_help(format!(
                    "Ensure the interface `{}` is defined or imported before extending it.",
                    other_fqcn
                ));

            context.report(issue);

            return;
        };

        if other.inheritance.extends_interface(context.interner, this) {
            let this_name = this.name.get_key(context.interner);

            let issue =
                Issue::error(format!("Circular inheritance detected between `{}` and `{}`.", this_name, other_name))
                    .with_annotation(
                        Annotation::primary(other_identifier.span())
                            .with_message(format!("Interface `{}` already extends `{}`.", other_fqcn, this_name)),
                    )
                    .with_help(format!(
                        "Ensure there is no circular inheritance between `{}` and `{}`.",
                        this_name, other_name
                    ));

            context.report(issue);
        }
    }

    fn implement_interface(other_identifier: &Identifier, context: &mut LintContext<'_>) {
        let other_name_identifier = context.semantics.names.get(other_identifier);
        let other_name = context.lookup(&other_identifier.value());
        let other_fqcn = context.lookup(other_name_identifier);

        if context.codebase.interface_exists(context.interner, other_name_identifier) {
            return;
        }

        let issue = Issue::error(format!("Implemented interface `{}` does not exist.", other_name))
            .with_annotation(
                Annotation::primary(other_identifier.span())
                    .with_message(format!("Interface `{}` does not exist.", other_fqcn)),
            )
            .with_help(format!("Ensure the interface `{}` is defined or imported before implementing it.", other_fqcn));

        context.report(issue);
    }
}

impl<'a> Walker<LintContext<'a>> for InheritanceRule {
    fn walk_in_interface(&self, interface: &Interface, context: &mut LintContext<'a>) {
        let name_identifier = context.semantics.names.get(&interface.name);
        let Some(reflection) = context.codebase.get_interface(context.interner, name_identifier) else {
            return;
        };

        if let Some(extends) = interface.extends.as_ref() {
            for extended_interface in extends.types.iter() {
                Self::extend_interface(reflection, extended_interface, context);
            }
        }
    }

    fn walk_in_class(&self, class: &Class, context: &mut LintContext<'a>) {
        let name_identifier = context.semantics.names.get(&class.name);
        let Some(reflection) = context.codebase.get_class(context.interner, name_identifier) else {
            return;
        };

        if let Some(extends) = class.extends.as_ref() {
            for extended_class in extends.types.iter() {
                Self::extend_class(reflection, extended_class, context);
            }
        }

        if let Some(implements) = class.implements.as_ref() {
            for implemented_interface in implements.types.iter() {
                Self::implement_interface(implemented_interface, context);
            }
        }
    }

    fn walk_in_anonymous_class(&self, anonymous_class: &AnonymousClass, context: &mut LintContext<'a>) {
        let Some(reflection) = context.codebase.get_anonymous_class(&anonymous_class) else {
            return;
        };

        if let Some(extends) = anonymous_class.extends.as_ref() {
            for extended_class in extends.types.iter() {
                Self::extend_class(reflection, extended_class, context);
            }
        }

        if let Some(implements) = anonymous_class.implements.as_ref() {
            for implemented_interface in implements.types.iter() {
                Self::implement_interface(implemented_interface, context);
            }
        }
    }

    fn walk_in_enum(&self, r#enum: &Enum, context: &mut LintContext<'a>) {
        if let Some(implements) = r#enum.implements.as_ref() {
            for implemented_interface in implements.types.iter() {
                Self::implement_interface(implemented_interface, context);
            }
        }
    }
}
