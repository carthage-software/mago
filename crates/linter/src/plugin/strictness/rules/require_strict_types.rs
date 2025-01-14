use indoc::indoc;
use toml::Value;

use mago_ast::ast::*;
use mago_ast::Program;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

const ALLOW_DISABLING: &str = "allow-disabling";
const ALLOW_DISABLING_DEFAULT: bool = false;

const STRICT_TYPES_DIRECTIVE: &str = "strict_types";

#[derive(Clone, Debug)]
pub struct RequireStrictTypesRule;

impl Rule for RequireStrictTypesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Require Strict Types", Level::Warning)
            .with_description(indoc! {"
                Detects missing `declare(strict_types=1);` statement at the beginning of the file.
            "})
            .with_option(RuleOptionDefinition {
                name: ALLOW_DISABLING,
                r#type: "boolean",
                description: "Whether to allow disabling the `strict_types` directive.",
                default: Value::Boolean(ALLOW_DISABLING_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "A program with a `declare(strict_types=1);` statement",
                indoc! {r#"
                    <?php

                    declare(strict_types=1);

                    echo "Hello, World!";
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A program that is missing a `declare(strict_types=1);` statement",
                indoc! {r#"
                    <?php

                    echo "Hello, World!";
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "A program with a disabled `strict_types` directive",
                indoc! {r#"
                    <?php

                    declare(strict_types=0);

                    echo "Hello, World!";
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "A program with an allowed disabled `strict_types` directive",
                    indoc! {r#"
                    <?php

                    declare(strict_types=0);

                    echo "Hello, World!";
                "#},
                )
                .with_option(ALLOW_DISABLING, Value::Boolean(true)),
            )
    }
}

impl<'a> Walker<LintContext<'a>> for RequireStrictTypesRule {
    fn walk_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        if program.statements.len() < 2 {
            return;
        }

        let mut found = false;

        for statement in program.statements.iter() {
            if let Statement::Declare(declare) = statement {
                for item in declare.items.iter() {
                    let name = context.lookup(&item.name.value);

                    if name != STRICT_TYPES_DIRECTIVE {
                        continue;
                    }

                    match &item.value {
                        Expression::Literal(Literal::Integer(integer)) => {
                            let disabled = match &integer.value {
                                Some(val) => *val == 0,
                                None => {
                                    // ignore invalid values, as they will be caught by the semantics checker

                                    continue;
                                }
                            };

                            if disabled
                                && !context
                                    .option(ALLOW_DISABLING)
                                    .and_then(|o| o.as_bool())
                                    .unwrap_or(ALLOW_DISABLING_DEFAULT)
                            {
                                context.report(
                                    Issue::new(context.level(), "The `strict_types` directive is disabled.")
                                        .with_annotation(
                                            Annotation::primary(item.span())
                                                .with_message("The `strict_types` is disabled here."),
                                        )
                                        .with_note("Disabling `strict_types` can lead to type safety issues.")
                                        .with_help("Consider setting `strict_types` to `1` to enforce strict typing."),
                                );
                            }
                        }
                        _ => {
                            // ignore other values, as they will be caught by the semantics checker
                        }
                    };

                    found = true;
                }
            }
        }

        if !found {
            context.report(
                Issue::new(
                    context.level(),
                    "Missing `declare(strict_types=1);` statement at the beginning of the file",
                )
                .with_annotation(Annotation::primary(program.span()))
                .with_note("The `strict_types` directive enforces strict type checking, which can prevent subtle bugs.")
                .with_help("Add `declare(strict_types=1);` at the top of your file."),
            );
        }
    }
}
