use indoc::indoc;
use mago_allocator::Arena;
use mago_text_edit::TextEdit;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct LowercaseKeywordRule {
    meta: &'static RuleMeta,
    cfg: LowercaseKeywordConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct LowercaseKeywordConfig {
    pub level: Level,
}

impl Default for LowercaseKeywordConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for LowercaseKeywordConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for LowercaseKeywordRule {
    type Config = LowercaseKeywordConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Lowercase Keyword",
            code: "lowercase-keyword",
            description: indoc! {"
                Enforces that PHP keywords (like `if`, `else`, `return`, `function`, etc.) be written
                in lowercase. Using uppercase or mixed case is discouraged for consistency and readability.

                When the `drupal` integration is enabled, `TRUE`, `FALSE`, and `NULL` are exempted to
                match Drupal's coding standards (and the `drupal` formatter preset).
            "},
            good_example: indoc! {r#"
                <?php

                if (true) {
                    echo "All keywords in lowercase";
                } else {
                    return;
                }
           "#},
            bad_example: indoc! {r#"
                <?PHP

                IF (TRUE) {
                    ECHO "Keywords not in lowercase";
                } ELSE {
                    RETURN;
                }
           "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Keyword];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Keyword(keyword) = node else {
            return;
        };

        if keyword.value.iter().all(|&b| !b.is_ascii_alphabetic() || b.is_ascii_lowercase()) {
            return; // Already in lowercase, no issue to report
        }

        let lowercase = keyword.value.to_ascii_lowercase();

        if ctx.registry.is_integration_enabled(Integration::Drupal)
            && matches!(lowercase.as_slice(), b"true" | b"false" | b"null")
        {
            return;
        }

        let Some(lowercase_str) = std::str::from_utf8(&lowercase).ok() else { return };
        let keyword_display = mago_bytes::BytesDisplay(keyword.value);

        let issue = Issue::new(self.cfg.level(), format!("Keyword `{keyword_display}` should be in lowercase."))
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(keyword.span()))
            .with_note(format!("The keyword `{keyword_display}` does not follow lowercase convention."))
            .with_help(format!("Consider using `{lowercase_str}` instead of `{keyword_display}`."));

        let lowercase_owned = lowercase_str.to_owned();
        ctx.collector.propose(issue, |edits| {
            edits.push(TextEdit::replace(keyword.span, lowercase_owned));
        });
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::LowercaseKeywordRule;
    use crate::integration::Integration;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    fn drupal(s: &mut Settings) {
        s.integrations.insert(Integration::Drupal);
    }

    test_lint_success! {
        name = drupal_uppercase_constants_allowed,
        rule = LowercaseKeywordRule,
        settings = drupal,
        code = indoc! {r#"
            <?php

            $a = NULL;
            $b = TRUE;
            $c = FALSE;
        "#}
    }

    test_lint_failure! {
        name = drupal_other_keywords_still_flagged,
        rule = LowercaseKeywordRule,
        count = 2,
        settings = drupal,
        code = indoc! {r#"
            <?php

            IF (TRUE) {
                $x = NULL;
            } ELSE {
                $x = FALSE;
            }
        "#}
    }

    test_lint_failure! {
        name = uppercase_constants_flagged_without_drupal,
        rule = LowercaseKeywordRule,
        count = 3,
        code = indoc! {r#"
            <?php

            $a = NULL;
            $b = TRUE;
            $c = FALSE;
        "#}
    }
}
