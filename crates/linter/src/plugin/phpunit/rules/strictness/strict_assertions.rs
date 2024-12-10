use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::plugin::phpunit::rules::utils::find_assertions_methods;
use crate::rule::Rule;

const NON_STRICT_ASSERTIONS: [&str; 4] =
    ["assertAttributeEquals", "assertAttributeNotEquals", "assertEquals", "assertNotEquals"];

/// A PHPUnit rule that enforces the use of strict assertions.
#[derive(Clone, Debug)]
pub struct StrictAssertionsRule;

impl Rule for StrictAssertionsRule {
    fn get_name(&self) -> &'static str {
        "strict-assertions"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for StrictAssertionsRule {
    fn walk_in_method(&self, method: &Method, context: &mut LintContext<'a>) {
        let name = context.lookup(&method.name.value);
        if !name.starts_with("test") || name.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase()) {
            return;
        }

        for reference in find_assertions_methods(method, context) {
            let ClassLikeMemberSelector::Identifier(identifier) = reference.get_selector() else {
                continue;
            };

            let name = context.lookup(&identifier.value);
            if NON_STRICT_ASSERTIONS.contains(&name) {
                let strict_name = name.replacen("Equals", "Same", 1);

                let issue = Issue::new(context.level(), "use strict assertions in PHPUnit tests")
                    .with_annotations([Annotation::primary(reference.span())
                        .with_message(format!("replace `{}` with `{}`", name, strict_name))])
                    .with_help(format!(
                        "replace `{}` with `{}` to enforce strict comparisons in your tests.",
                        name, strict_name
                    ));

                context.report_with_fix(issue, |plan| {
                    plan.replace(
                        reference.get_selector().span().to_range(),
                        strict_name,
                        SafetyClassification::PotentiallyUnsafe,
                    );
                });
            }
        }
    }
}
