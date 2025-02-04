use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct InstantiationRule;

impl Rule for InstantiationRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Instantiation", Level::Error)
            .with_description(indoc! {"
                Ensures that only valid, concrete classes are instantiated. Flags attempts to instantiate
                non-existent classes, interfaces, traits, enums, or abstract classes.
            "})
            .with_example(RuleUsageExample::valid(
                "Simple class instantiation",
                indoc! {r#"
                    <?php

                    class Foo {}

                    $instance = new Foo();  // Valid instantiation
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Instantiating a non-existent class",
                indoc! {r#"
                    <?php

                    $instance = new Foo(); // Error: Class `Foo` does not exist.
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Instantiating an interface",
                indoc! {r#"
                    <?php

                    interface FooInterface {}

                    $instance = new FooInterface(); // Error: Cannot instantiate an interface
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Instantiating a trait",
                indoc! {r#"
                    <?php

                    trait FooTrait {}

                    $instance = new FooTrait(); // Error: Cannot instantiate a trait
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Instantiating an enum",
                indoc! {r#"
                    <?php

                    enum Foo {}

                    $instance = new Foo(); // Error: Cannot instantiate an enum
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Instantiating an abstract class",
                indoc! {r#"
                    <?php

                    abstract class AbstractFoo {}

                    $instance = new AbstractFoo(); // Error: Cannot instantiate an abstract class
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Instantiation(instantiation) = node else {
            return LintDirective::Continue;
        };

        let Expression::Identifier(class_identifier) = instantiation.class.as_ref() else {
            return LintDirective::default();
        };

        let class_name_identifier = context.semantics.names.get(class_identifier);
        let class_name = context.lookup(&class_identifier.value()).to_string();
        let class_fqcn = context.lookup(class_name_identifier).to_string();

        let Some(reflection) = context.codebase.get_named_class_like(context.interner, class_name_identifier) else {
            let issue = Issue::error(format!("Instantiated class `{}` does not exist.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Class `{}` does not exist.", class_fqcn)),
                )
                .with_help(format!(
                    "Ensure the class `{}` is defined or imported before instantiating it.",
                    class_fqcn
                ));

            context.report(issue);

            return LintDirective::default();
        };

        if reflection.is_interface() {
            let issue = Issue::error(format!("Cannot instantiate interface `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Interface `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Interfaces are abstract types that cannot be instantiated directly.")
                .with_note("Use a class that implements the interface instead.");

            context.report(issue);

            return LintDirective::default();
        }

        if reflection.is_trait() {
            let issue = Issue::error(format!("Cannot instantiate trait `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Trait `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Traits are abstract types that cannot be instantiated directly.")
                .with_note("Use a class that implements the trait instead.");

            context.report(issue);

            return LintDirective::default();
        }

        if reflection.is_enum() {
            let issue = Issue::error(format!("Cannot instantiate enum `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Enum `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Enums are types that represent a fixed set of constants.")
                .with_note("Use one of the enum cases instead.");

            context.report(issue);

            return LintDirective::default();
        }

        if reflection.is_abstract {
            let issue = Issue::error(format!("Cannot instantiate abstract class `{}`.", class_name))
                .with_annotation(
                    Annotation::primary(class_identifier.span())
                        .with_message(format!("Abstract class `{}` cannot be instantiated.", class_fqcn)),
                )
                .with_note("Abstract classes are incomplete types that cannot be instantiated directly.")
                .with_note("Use a concrete subclass instead.");

            context.report(issue);
        }

        LintDirective::default()
    }
}
