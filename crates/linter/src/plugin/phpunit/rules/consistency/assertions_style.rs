use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::plugin::phpunit::rules::utils::find_testing_and_assertions_methods;
use crate::plugin::phpunit::rules::utils::MethodReference;
use crate::rule::Rule;

const STATIC_STYLES: &str = "static";
const SELF_STYLES: &str = "self";
const THIS_STYLES: &str = "this";

const STYLES: [&str; 3] = [STATIC_STYLES, SELF_STYLES, THIS_STYLES];

#[derive(Clone, Debug)]
pub struct AssertionsStyleRule;

impl Rule for AssertionsStyleRule {
    fn get_name(&self) -> &'static str {
        "assertions-style"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Warning)
    }
}

impl<'a> Walker<LintContext<'a>> for AssertionsStyleRule {
    fn walk_in_method(&self, method: &Method, context: &mut LintContext<'a>) {
        let name = context.lookup(&method.name.value);
        if !name.starts_with("test") || name.chars().nth(4).is_none_or(|c| c != '_' && !c.is_uppercase()) {
            return;
        }

        let desired_style = context
            .option("style")
            .and_then(|o| o.as_str())
            .filter(|s| STYLES.contains(&s.to_ascii_lowercase().as_str()))
            .unwrap_or(STATIC_STYLES)
            .to_string();

        let desired_syntax = match desired_style.as_str() {
            STATIC_STYLES => "static::",
            SELF_STYLES => "self::",
            THIS_STYLES => "$this->",
            _ => unreachable!(),
        };

        for reference in find_testing_and_assertions_methods(method, context) {
            let (to_replace, current_style) = match reference {
                MethodReference::MethodCall(c) => (c.object.span().join(c.arrow), THIS_STYLES),
                MethodReference::MethodClosureCreation(c) => (c.object.span().join(c.arrow), THIS_STYLES),
                MethodReference::StaticMethodClosureCreation(StaticMethodClosureCreation {
                    class,
                    double_colon,
                    ..
                }) => match class {
                    Expression::Static(_) => (class.span().join(*double_colon), STATIC_STYLES),
                    Expression::Self_(_) => (class.span().join(*double_colon), SELF_STYLES),
                    _ => continue,
                },
                MethodReference::StaticMethodCall(StaticMethodCall { class, double_colon, .. }) => match class.as_ref()
                {
                    Expression::Static(_) => (class.span().join(*double_colon), STATIC_STYLES),
                    Expression::Self_(_) => (class.span().join(*double_colon), SELF_STYLES),
                    _ => continue,
                },
            };

            if current_style == desired_style {
                continue;
            }

            let current_syntax = match current_style {
                STATIC_STYLES => "static::",
                SELF_STYLES => "self::",
                THIS_STYLES => "$this->",
                _ => unreachable!(),
            };

            let issue = Issue::new(context.level(), "inconsistent assertions style")
                .with_annotations([Annotation::primary(reference.span()).with_message(format!(
                    "assertion style mismatch: expected `{}` style but found `{}` style.",
                    desired_style, current_style
                ))])
                .with_help(format!(
                    "use `{}` instead of `{}` to conform to the `{}` style.",
                    desired_syntax, current_syntax, desired_style,
                ));

            context.report_with_fix(issue, |plan| {
                plan.replace(
                    to_replace.to_range(),
                    desired_syntax.to_string(),
                    SafetyClassification::PotentiallyUnsafe,
                );
            });
        }
    }
}
