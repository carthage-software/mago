use indoc::indoc;
use toml::Value;

use mago_ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::walk_block_mut;
use mago_walker::MutWalker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 7;

#[derive(Clone, Debug)]
pub struct ExcessiveNesting;

impl Rule for ExcessiveNesting {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Excessive Nesting", Level::Warning)
            .with_description(
                "Checks if the nesting level in any block exceeds a configurable `threshold` (default: 7).",
            )
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed nesting depth. If exceeded, a warning is reported.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
            .with_example(RuleUsageExample::valid(
                "Nesting below the default threshold (7)",
                indoc! {r#"
                    <?php

                    // Here we have at most 2 levels of nesting
                    if ($condition) {
                        while ($otherCondition) {
                            echo "Hello";
                        }
                    }
                "#},
            ))
            .with_example(
                RuleUsageExample::valid(
                    "Nesting below a custom threshold of 2",
                    indoc! {r#"
                        <?php

                        if ($condition) {
                            echo "Only 1 level of nesting here.";
                        }
                    "#},
                )
                .with_option(THRESHOLD, Value::Integer(2)),
            )
            .with_example(RuleUsageExample::invalid(
                "Nesting exceeds default threshold (7)",
                indoc! {r#"
                    <?php

                    if ($a) {
                        if ($b) {
                            if ($c) {
                                if ($d) {
                                    if ($e) {
                                        if ($f) {
                                            if ($g) {
                                                if ($h) {
                                                    echo "This is too deeply nested!";
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                "#},
            ))
            .with_example(
                RuleUsageExample::invalid(
                    "Nesting exceeds custom threshold (2)",
                    indoc! {r#"
                        <?php

                        if ($a) {
                            if ($b) {
                                if ($c) {
                                    echo "Too deep for threshold=2!";
                                }
                            }
                        }
                    "#},
                )
                .with_option(THRESHOLD, Value::Integer(2)),
            )
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Program(program) = node else { return LintDirective::default() };

        let threshold = context.option(THRESHOLD).and_then(|value| value.as_integer()).unwrap_or(THRESHOLD_DEFAULT);

        let mut walker = NestingWalker { threshold: threshold as usize, level: 0 };

        walker.walk_program(program, context);

        LintDirective::Abort
    }
}

struct NestingWalker {
    threshold: usize,
    level: usize,
}

impl NestingWalker {
    fn check_indent(&self, block: &Block, context: &mut LintContext) -> bool {
        if self.level > self.threshold {
            let issue = Issue::new(context.level(), "Excessive block nesting.")
                .with_annotation(Annotation::primary(block.span()))
                .with_note(format!(
                    "This block has a nesting level of {} which exceeds the threshold of {}.",
                    self.level, self.threshold
                ))
                .with_note("Excessive nesting can make code harder to read, understand, and maintain.")
                .with_help("Refactor your code to reduce the level of nesting.");

            context.report(issue);

            return true;
        }

        false
    }
}

impl<'a> MutWalker<LintContext<'a>> for NestingWalker {
    fn walk_block<'ast>(&mut self, block: &'ast Block, context: &mut LintContext<'a>) {
        self.level += 1;

        if !self.check_indent(block, context) {
            walk_block_mut(self, block, context);
        }

        self.level -= 1;
    }
}
