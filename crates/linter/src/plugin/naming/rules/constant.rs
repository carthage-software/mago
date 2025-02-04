use indoc::indoc;

use mago_ast::*;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct ConstantRule;

impl Rule for ConstantRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Constant", Level::Help)
            .with_description(indoc! {"
                Detects constant declarations that do not follow constant naming convention.
                Constant names should be in constant case, also known as UPPER_SNAKE_CASE.
            "})
            .with_example(RuleUsageExample::valid(
                "A constant name in constant case",
                indoc! {r#"
                    <?php

                    const MY_CONSTANT = 42;

                    class MyClass {
                        public const int MY_CONSTANT = 42;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A constant name not in constant case",
                indoc! {r#"
                    <?php

                    const myConstant = 42;
                    const my_constant = 42;
                    const My_Constant = 42;

                    class MyClass {
                        public const int myConstant = 42;
                        public const int my_constant = 42;
                        public const int My_Constant = 42;
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        match node {
            Node::Constant(constant) => {
                for item in constant.items.iter() {
                    let name = context.lookup(&item.name.value);
                    if !mago_casing::is_constant_case(name) {
                        context.report(
                            Issue::new(
                                context.level(),
                                format!("Constant name `{}` should be in constant case.", name),
                            )
                            .with_annotation(
                                Annotation::primary(item.name.span())
                                    .with_message(format!("Constant item `{}` is declared here.", name)),
                            )
                            .with_note(format!(
                                "The constant name `{}` does not follow constant naming convention.",
                                name
                            ))
                            .with_help(format!(
                                "Consider renaming it to `{}` to adhere to the naming convention.",
                                mago_casing::to_constant_case(name)
                            )),
                        );
                    }
                }

                LintDirective::Prune
            }
            Node::ClassLikeConstant(class_like_constant) => {
                for item in class_like_constant.items.iter() {
                    let name = context.lookup(&item.name.value);

                    if !mago_casing::is_constant_case(name) {
                        context.report(
                            Issue::new(
                                context.level(),
                                format!("Constant name `{}` should be in constant case.", name),
                            )
                            .with_annotation(
                                Annotation::primary(item.name.span())
                                    .with_message(format!("Constant item `{}` is declared here.", name)),
                            )
                            .with_note(format!(
                                "The constant name `{}` does not follow constant naming convention.",
                                name
                            ))
                            .with_help(format!(
                                "Consider renaming it to `{}` to adhere to the naming convention.",
                                mago_casing::to_constant_case(name)
                            )),
                        );
                    }
                }

                LintDirective::Prune
            }
            _ => LintDirective::default(),
        }
    }
}
