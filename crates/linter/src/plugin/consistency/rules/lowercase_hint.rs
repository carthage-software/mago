use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct LowercaseHintRule;

impl Rule for LowercaseHintRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Lowercase Hint", Level::Help)
            .with_description(indoc! {"
                Enforces that PHP type hints (like `void`, `bool`, `int`, `float`, etc.) be written
                in lowercase. Using uppercase or mixed case is discouraged for consistency
                and readability.
            "})
            .with_example(RuleUsageExample::valid(
                "Using lowercase type hints",
                indoc! {r#"
                    <?php

                    function example(int $param): void {
                        return;
                    }
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using uppercase or mixed case type hints",
                indoc! {r#"
                    <?php

                    function example(Int $param): VOID {
                        return;
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Hint(hint) = node else { return LintDirective::default() };

        match hint {
            Hint::Void(identifier)
            | Hint::Never(identifier)
            | Hint::Float(identifier)
            | Hint::Bool(identifier)
            | Hint::Integer(identifier)
            | Hint::String(identifier)
            | Hint::Object(identifier)
            | Hint::Mixed(identifier)
            | Hint::Iterable(identifier) => {
                let name = context.interner.lookup(&identifier.value);
                let lowered = name.to_ascii_lowercase();
                if !lowered.eq(&name) {
                    let issue = Issue::new(context.level(), format!("Type hint `{}` should be in lowercase.", name))
                        .with_annotation(Annotation::primary(identifier.span()))
                        .with_help(format!("Consider using `{}` instead of `{}`.", lowered, name));

                    context
                        .propose(issue, |p| p.replace(identifier.span.to_range(), lowered, SafetyClassification::Safe));
                }

                // No need to continue linting the children of this node.
                LintDirective::Prune
            }
            _ => {
                if hint.is_complex() {
                    // These are compound hints, so we need to continue linting the children.
                    LintDirective::Continue
                } else {
                    // We don't care about other hints.
                    LintDirective::Prune
                }
            }
        }
    }
}
