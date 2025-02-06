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
pub struct UndefinedConstantOrCaseRule;

impl Rule for UndefinedConstantOrCaseRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Undefined Constant or Enum Case", Level::Error)
            .with_description(indoc! {r#"
                Checks for references to constants or enum cases that aren't declared.
                This includes global constants (e.g., `FOO`), class constants (`Foo::BAR`),
                and enum cases (`MyEnum::CaseName`). If such constants or cases are not defined
                or imported, it flags an error. This helps catch typos or missing declarations.
            "#})
            .with_example(RuleUsageExample::valid(
                "Defining a constant and referencing it",
                indoc! {r#"
                    <?php

                    const GREETING = 'Hello, world!';

                    echo GREETING;
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Defining and referencing a class constant",
                indoc! {r#"
                    <?php

                    class Foo {
                        public const BAR = 42;
                    }

                    echo Foo::BAR;
                "#},
            ))
            .with_example(RuleUsageExample::valid(
                "Referencing an existing enum case",
                indoc! {r#"
                    <?php

                    enum Status {
                        case Open;
                        case Closed;
                    }

                    $current = Status::Open;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Accessing an undefined global constant",
                indoc! {r#"
                    <?php

                    echo GREETING;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Accessing an undefined class constant",
                indoc! {r#"
                    <?php

                    class Foo {
                        public const BAR = 42;
                    }

                    echo Foo::BAZ;
               "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Accessing an undefined enum case",
                indoc! {r#"
                    <?php

                    enum Status {
                        case Open;
                        case Closed;
                    }

                    $invalid = Status::InProgress;
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::ConstantAccess(constant_access) => {
                // Handle global constant usage (e.g. `FOO`).
                let identifier = &constant_access.name;
                let constant_name = context.resolve_constant_name(identifier);
                let constant_name_id = context.interner.intern(constant_name);

                // If the constant is known to exist, prune further checks.
                if context.codebase.constant_exists(context.interner, &constant_name_id) {
                    return LintDirective::Prune;
                }

                // Otherwise, report an undefined global constant usage.
                context.report(
                    Issue::new(context.level(), format!("Use of undefined constant `{constant_name}`."))
                        .with_annotation(
                            Annotation::primary(identifier.span())
                                .with_message(format!("Constant `{constant_name}` does not exist.")),
                        )
                        .with_help(format!(
                            "Ensure the constant `{constant_name}` is defined or imported before using it."
                        )),
                );

                LintDirective::Prune
            }
            Node::ClassConstantAccess(class_constant_access) => {
                // e.g. `Foo::BAR` or `SomeEnum::Case`
                let Expression::Identifier(class_identifier) = class_constant_access.class.as_ref() else {
                    return LintDirective::Prune;
                };

                let ClassLikeConstantSelector::Identifier(constant_identifier) = &class_constant_access.constant else {
                    return LintDirective::Prune;
                };

                let class_id = context.semantics.names.get(class_identifier);
                let class_name = context.interner.lookup(class_id);
                let constant_name = context.interner.lookup(&constant_identifier.value);

                // `Foo::class` is a special built-in usage returning the class’s fully qualified name.
                let accessing_class_name_magic = constant_name.eq_ignore_ascii_case("class");

                let Some(class_reflection) = context.codebase.get_named_class_like(context.interner, class_id) else {
                    // The class doesn't exist at all
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Accessing `{constant_name}` on non-existent class `{class_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(class_identifier.span())
                                .with_message(format!("Class `{class_name}` does not exist.")),
                        )
                        .with_help(format!("Define or import the class `{class_name}` before using it.")),
                    );

                    return LintDirective::Prune;
                };

                // If it's `Foo::class`, skip checks, it's a known usage.
                if accessing_class_name_magic {
                    return LintDirective::Prune;
                }

                // For a valid class, check if the constant exists.
                if class_reflection.has_constant(&constant_identifier.value) {
                    // Class constant is defined, so no issue.
                    return LintDirective::Prune;
                }

                // If the class is an enum:
                if class_reflection.is_enum() {
                    // Check if the enum case exists.
                    if class_reflection.has_enum_case(&constant_identifier.value) {
                        return LintDirective::Prune;
                    }

                    // Report an undefined enum case usage.
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Use of undefined enum case `{class_name}::{constant_name}`."),
                        )
                        .with_annotation(
                            Annotation::primary(class_constant_access.span()).with_message(format!(
                                "Enum `{class_name}` does not have a case named `{constant_name}`."
                            )),
                        )
                        .with_help(format!(
                            "Ensure the enum case `{class_name}::{constant_name}` is defined before using it."
                        )),
                    );
                } else {
                    // It's a class, interface, or trait, so report an undefined class constant usage.
                    context.report(
                        Issue::new(
                            context.level(),
                            format!("Use of undefined class constant `{class_name}::{constant_name}`."),
                        )
                        .with_annotation(Annotation::primary(class_constant_access.span()).with_message(format!(
                            "Class `{class_name}` does not have a constant named `{constant_name}`."
                        )))
                        .with_help(format!(
                            "Ensure the class constant `{class_name}::{constant_name}` is defined before using it."
                        )),
                    );
                }

                LintDirective::Prune
            }
            _ => LintDirective::default(),
        }
    }
}
