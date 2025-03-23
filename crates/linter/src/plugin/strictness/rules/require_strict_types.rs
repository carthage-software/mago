use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
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
            .with_minimum_supported_php_version(PHPVersion::PHP70)
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

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::default() };

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
                                let issue = Issue::new(context.level(), "The `strict_types` directive is disabled.")
                                    .with_annotation(
                                        Annotation::primary(item.span())
                                            .with_message("The `strict_types` is disabled here."),
                                    )
                                    .with_note("Disabling `strict_types` can lead to type safety issues.")
                                    .with_help("Consider setting `strict_types` to `1` to enforce strict typing.");

                                context.propose(issue, |plan| {
                                    plan.replace(integer.span.to_range(), "1", SafetyClassification::PotentiallyUnsafe);
                                });
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
            let issue = Issue::new(
                context.level(),
                "Missing `declare(strict_types=1);` statement at the beginning of the file",
            )
            .with_annotation(Annotation::primary(program.span()))
            .with_note("The `strict_types` directive enforces strict type checking, which can prevent subtle bugs.")
            .with_help("Add `declare(strict_types=1);` at the top of your file.");

            context.propose(issue, |plan| {
                let Some(mut first_statement) = program.statements.first() else {
                    // The file is completely empty, insert an opening tag and declare statement
                    // This change is safe because the file is empty.
                    plan.insert(0, "<?php\n\ndeclare(strict_types=1);\n", SafetyClassification::Safe);

                    return;
                };

                // If the first statement is a shebang.
                if let Statement::Inline(Inline { kind: InlineKind::Shebang, value, span, .. }) = first_statement {
                    // Skip the shebang and look for the first PHP statement.
                    first_statement = match program.statements.get(1) {
                        Some(statement) => statement,
                        None => {
                            let ends_in_newline = context.interner.lookup(value).ends_with('\n');

                            // If there are no statements after the shebang, insert an opening tag and declare statement.
                            let content = if ends_in_newline {
                                "<?php\n\ndeclare(strict_types=1);\n"
                            } else {
                                "\n<?php\n\ndeclare(strict_types=1);\n"
                            };

                            // This is safe because the shebang is the only statement in the file.
                            plan.insert(span.end.offset, content, SafetyClassification::Safe);

                            return;
                        }
                    };
                }

                match first_statement {
                    Statement::Inline(inline) => {
                        // If the first statement is an inline statement, insert the declare statement before it.
                        let starts_with_newline = context.interner.lookup(&inline.value).starts_with('\n');
                        let content = if !starts_with_newline {
                            "<?php\n\ndeclare(strict_types=1);\n\n?>\n"
                        } else {
                            "<?php\n\ndeclare(strict_types=1);\n\n?>"
                        };

                        plan.insert(inline.span.start.offset, content, SafetyClassification::PotentiallyUnsafe);
                    }
                    Statement::OpeningTag(opening_tag) => match opening_tag {
                        OpeningTag::Full(FullOpeningTag { span, .. }) | OpeningTag::Short(ShortOpeningTag { span }) => {
                            // If the first statement is an opening tag, insert the declare statement after it.
                            plan.insert(
                                span.end.offset,
                                "\n\ndeclare(strict_types=1);\n",
                                SafetyClassification::PotentiallyUnsafe,
                            );
                        }
                        OpeningTag::Echo(echo_opening_tag) => {
                            // If the first statement is an echo opening tag, insert an opening tag and declare statement
                            // and a closing tag before it.
                            plan.insert(
                                echo_opening_tag.span.start.offset,
                                "<?php\n\ndeclare(strict_types=1);\n\n?>\n",
                                SafetyClassification::PotentiallyUnsafe,
                            );
                        }
                    },
                    _ => unreachable!(),
                }
            });
        }

        LintDirective::Abort
    }
}
