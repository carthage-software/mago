use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_php_version::PHPVersion;
use mago_reporting::*;
use mago_span::HasSpan;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct ExplicitOctalNotationRule;

impl Rule for ExplicitOctalNotationRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Explicit Octal Notation", Level::Warning)
            .with_minimum_supported_php_version(PHPVersion::PHP81)
            .with_description(indoc! {"
                Detects implicit octal numeral notation and suggests replacing it with explicit octal numeral notation.
            "})
            .with_example(RuleUsageExample::valid(
                "Using explicit octal numeral notation",
                indoc! {r#"
                    <?php

                    $a = 0o123;
                "#},
            ))
            .with_example(RuleUsageExample::invalid(
                "Using implicit octal numeral notation",
                indoc! {r#"
                    <?php

                    $a = 0123;
                "#},
            ))
    }
    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
        let Node::LiteralInteger(literal_integer) = node else { return LintDirective::default() };

        let literal_text = context.lookup(&literal_integer.raw);
        if !literal_text.starts_with('0') {
            return LintDirective::Prune;
        }

        if !literal_text.as_bytes().get(1).copied().is_some_and(|c| {
            // check for `0o`, `0x`, or `0b` prefix
            c != b'o' && c != b'O' && c != b'x' && c != b'X' && c != b'b' && c != b'B'
        }) {
            return LintDirective::Prune;
        }

        let issue = Issue::new(context.level(), "Use explicit octal numeral notation.")
            .with_annotation(
                Annotation::primary(literal_integer.span())
                    .with_message("Implicit octal numeral notation used here."),
            )
            .with_note("Using `0o` makes the octal intent explicit and avoids confusion with other formats.")
            .with_help("Replace the leading `0` with `0o` to make the octal intent explicit")
            .with_link("https://www.php.net/manual/en/migration81.new-features.php#migration81.new-features.core.octal-literal-prefix")
        ;

        let replacement = format!("0o{}", &literal_text[1..]);

        context.propose(issue, |plan| {
            plan.replace(literal_integer.span().to_range(), replacement, SafetyClassification::Safe);
        });

        LintDirective::Prune
    }
}
