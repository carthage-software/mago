use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Copy, Debug)]
pub struct LowercaseKeywordRule;

impl Rule for LowercaseKeywordRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Lowercase Keyword", Level::Help)
            .with_description(indoc! {"
                   Enforces that PHP keywords (like `if`, `else`, `return`, `function`, etc.) be written
                   in lowercase. Using uppercase or mixed case is discouraged for consistency
                   and readability.
               "})
            .with_example(RuleUsageExample::valid(
                "Lowercase keywords",
                indoc! {r#"
                    <?php

                    if (true) {
                        echo "All keywords in lowercase";
                    } else {
                        return;
                    }
               "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Uppercase or mixed-case keywords",
                indoc! {r#"
                    <?PHP

                    IF (TRUE) {
                        ECHO "Keywords not in lowercase";
                    } ELSE {
                        RETURN;
                    }
               "#},
            ))
    }
    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::Keyword(keyword) = node else { return LintDirective::default() };

        let name = context.lookup(&keyword.value);
        let lowered = name.to_ascii_lowercase();
        if !lowered.eq(&name) {
            let issue = Issue::new(context.level(), format!("Keyword `{}` should be in lowercase.", name))
                .with_annotation(Annotation::primary(keyword.span()))
                .with_note(format!("The keyword `{}` does not follow lowercase convention.", name))
                .with_help(format!("Consider using `{}` instead of `{}`.", lowered, name));

            context.propose(issue, |p| p.replace(keyword.span.to_range(), lowered, SafetyClassification::Safe));
        }

        LintDirective::Prune
    }
}
