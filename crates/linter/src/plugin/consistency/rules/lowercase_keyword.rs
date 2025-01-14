use indoc::indoc;

use mago_ast::ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::*;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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

                    IF (true) {
                        ECHO "Keywords not in lowercase";
                    } ELSE {
                        RETURN;
                    }
               "#},
            ))
    }
}

impl<'a> Walker<LintContext<'a>> for LowercaseKeywordRule {
    fn walk_in_keyword<'ast>(&self, keyword: &'ast Keyword, context: &mut LintContext<'a>) {
        let name = context.lookup(&keyword.value);
        let lowered = name.to_ascii_lowercase();
        if !lowered.eq(&name) {
            let issue = Issue::new(context.level(), format!("Keyword `{}` should be in lowercase.", name))
                .with_annotation(Annotation::primary(keyword.span()))
                .with_note(format!("The keyword `{}` does not follow lowercase convention.", name))
                .with_help(format!("Consider using `{}` instead of `{}`.", lowered, name));

            context.report_with_fix(issue, |p| p.replace(keyword.span.to_range(), lowered, SafetyClassification::Safe));
        }
    }
}
