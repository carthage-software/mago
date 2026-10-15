use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_bytes::BytesDisplay;
use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::UnaryPrefix;
use mago_syntax::cst::UnaryPrefixOperator;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DeprecatedCastRule {
    meta: &'static RuleMeta,
    cfg: DeprecatedCastConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DeprecatedCastConfig {
    pub level: Level,
}

impl Default for DeprecatedCastConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for DeprecatedCastConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DeprecatedCastRule {
    type Config = DeprecatedCastConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Deprecated Cast",
            code: "deprecated-cast",
            description: indoc! {"
                Detect the usage of deprecated type casts in PHP code.

                In PHP 8.5, the following type casts have been deprecated:

                - `(integer)`: The integer cast has been deprecated in favor of `(int)`.
                - `(boolean)`: The boolean cast has been deprecated in favor of `(bool)`.
                - `(double)`: The double cast has been deprecated in favor of `(float)`.
                - `(binary)`: The binary cast has been deprecated in favor of `(string)`.
            "},
            good_example: indoc! {r"
                <?php

                (int) $value;
            "},
            bad_example: indoc! {r"
                <?php

                (integer) $value;
            "},
            category: Category::Deprecation,
            requirements: RuleRequirements::PHPVersion(PHPVersionRange::from(PHPVersion::PHP85)),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::UnaryPrefix];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::UnaryPrefix(UnaryPrefix { operator, .. }) = node else {
            return;
        };

        let replacement = match operator {
            UnaryPrefixOperator::BooleanCast(_, _) => "(bool)",
            UnaryPrefixOperator::DoubleCast(_, _) => "(float)",
            UnaryPrefixOperator::IntegerCast(_, _) => "(int)",
            UnaryPrefixOperator::BinaryCast(_, _) => "(string)",
            _ => return,
        };

        let cast = BytesDisplay(operator.as_bytes());
        let issue = Issue::new(self.cfg.level(), format!("Usage of deprecated cast `{cast}`."))
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(operator.span()).with_message(format!("Deprecated cast `{cast}` used here.")),
            )
            .with_note(format!("The `{cast}` cast has been deprecated since PHP 8.5."))
            .with_help(format!("Replace `{cast}` with `{replacement}` to resolve this issue."))
            .with_link("https://wiki.php.net/rfc/deprecations_php_8_5");

        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(operator.span(), replacement));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::tests::collect_issues;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    #[test]
    fn deprecated_cast_messages_show_php_syntax_and_keep_the_fix() {
        for (cast, replacement) in [
            ("(integer)", "(int)"),
            ("(boolean)", "(bool)"),
            ("(double)", "(float)"),
            ("(binary)", "(string)"),
            ("( InTeGeR )", "(int)"),
        ] {
            let source = format!("<?php {cast} $value;");
            let issues = collect_issues::<DeprecatedCastRule, fn(&mut Settings)>(&source, None, None);
            assert_eq!(issues.len(), 1);
            for issue in issues {
                assert_eq!(issue.message.as_ref(), format!("Usage of deprecated cast `{cast}`."));
                assert_eq!(
                    issue.annotations[0].message.as_deref(),
                    Some(format!("Deprecated cast `{cast}` used here.").as_str())
                );
                assert_eq!(issue.notes[0].as_ref(), format!("The `{cast}` cast has been deprecated since PHP 8.5."));
                assert_eq!(
                    issue.help.as_deref(),
                    Some(format!("Replace `{cast}` with `{replacement}` to resolve this issue.").as_str())
                );
                let edits: Vec<_> = issue.edits.into_values().flatten().collect();
                assert_eq!(edits.len(), 1);
                assert_eq!(edits[0].new_text, replacement.as_bytes());
                assert_eq!(edits[0].range.start, "<?php ".len() as u32);
                assert_eq!(edits[0].range.end, ("<?php ".len() + cast.len()) as u32);
            }
        }
    }

    test_lint_failure! {
        name = double_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (double) $value;
        "}
    }

    test_lint_failure! {
        name = integer_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (integer) $value;
        "}
    }

    test_lint_failure! {
        name = boolean_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (boolean) $value;
        "}
    }

    test_lint_failure! {
        name = binary_deprecated,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (binary) $value;
        "}
    }

    test_lint_success! {
        name = int_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (int) $value;
        "}
    }

    test_lint_success! {
        name = bool_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (bool) $value;
        "}
    }

    test_lint_success! {
        name = float_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (float) $value;
        "}
    }

    test_lint_success! {
        name = string_cast_not_flagged,
        rule = DeprecatedCastRule,
        code = indoc! {r"
            <?php

            (string) $value;
        "}
    }
}
