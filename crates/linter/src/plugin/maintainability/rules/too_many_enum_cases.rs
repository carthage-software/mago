use indoc::indoc;
use toml::Value;

use mago_ast::ast::*;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleOptionDefinition;
use crate::rule::Rule;

const THRESHOLD: &str = "threshold";
const THRESHOLD_DEFAULT: i64 = 20;

#[derive(Clone, Copy, Debug)]
pub struct TooManyEnumCasesRule;

impl Rule for TooManyEnumCasesRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Too Many Enum Cases", Level::Error)
            .with_description(indoc! {r#"
                Detects enums with too many cases.

                This rule checks the number of cases in enums. If the number of cases exceeds a configurable threshold, an issue is reported.
            "#})
            .with_option(RuleOptionDefinition {
                name: THRESHOLD,
                r#type: "integer",
                description: "The maximum allowed number of enum cases before triggering an issue.",
                default: Value::Integer(THRESHOLD_DEFAULT),
            })
    }
}

impl<'a> Walker<LintContext<'a>> for TooManyEnumCasesRule {
    fn walk_in_enum(&self, r#enum: &Enum, context: &mut LintContext<'a>) {
        let threshold = context.option(THRESHOLD).and_then(|o| o.as_integer()).unwrap_or(THRESHOLD_DEFAULT) as usize;

        let mut cases = 0;
        for member in r#enum.members.iter() {
            if let ClassLikeMember::EnumCase(_) = member {
                cases += 1;
            }
        }

        if cases > threshold {
            let issue =
                Issue::new(context.level(), "Enum has too many cases.")
                    .with_annotation(Annotation::primary(r#enum.span()).with_message(format!(
                        "Enum has {} cases, which exceeds the threshold of {}.",
                        cases, threshold
                    )))
                    .with_note("Large enums can be difficult to read, reason about, or maintain.")
                    .with_help(
                        "Try splitting the enum into smaller logical groups or refactoring to reduce the total number of cases.",
                    );

            context.report(issue);
        }
    }
}
