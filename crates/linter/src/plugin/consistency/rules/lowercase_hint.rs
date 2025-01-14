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
}

impl<'a> Walker<LintContext<'a>> for LowercaseHintRule {
    fn walk_in_hint<'ast>(&self, hint: &'ast Hint, context: &mut LintContext<'a>) {
        if let Hint::Void(identifier)
        | Hint::Never(identifier)
        | Hint::Float(identifier)
        | Hint::Bool(identifier)
        | Hint::Integer(identifier)
        | Hint::String(identifier)
        | Hint::Object(identifier)
        | Hint::Mixed(identifier)
        | Hint::Iterable(identifier) = hint
        {
            let name = context.interner.lookup(&identifier.value);
            let lowered = name.to_ascii_lowercase();
            if !lowered.eq(&name) {
                let issue = Issue::new(context.level(), format!("Type hint `{}` should be in lowercase.", name))
                    .with_annotation(Annotation::primary(identifier.span()))
                    .with_help(format!("Consider using `{}` instead of `{}`.", lowered, name));

                context.report_with_fix(issue, |p| {
                    p.replace(identifier.span.to_range(), lowered, SafetyClassification::Safe)
                });
            }
        }
    }
}
