use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoInlineRule {
    meta: &'static RuleMeta,
    cfg: NoInlineConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoInlineConfig {
    pub level: Level,
}

impl Default for NoInlineConfig {
    fn default() -> Self {
        Self { level: Level::Error }
    }
}

impl Config for NoInlineConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

impl LintRule for NoInlineRule {
    type Config = NoInlineConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Inline",
            code: "no-inline",
            description: indoc! {"
                Disallows inline content (text outside of PHP tags) in source files.

                Most modern PHP applications are source-code only and do not use PHP as a templating
                language. Inline content before `<?php`, after `?>`, or between PHP tags is typically
                unintentional and can cause issues such as unexpected output or \"headers already sent\"
                errors.

                This rule is disabled by default and is intended for codebases that do not use PHP
                templates.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                echo "Hello, world!";
            "#},
            bad_example: indoc! {r#"
                Hello
                <?php

                echo "Hello, world!";

                ?>
                Goodbye
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Inline];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Inline(inline) = node else {
            return;
        };

        let issue = Issue::new(self.cfg.level(), "Inline content is not allowed outside of PHP tags.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(inline.span()).with_message("This inline content should be removed"))
            .with_note("Modern PHP applications should not contain content outside of PHP tags.")
            .with_help("Remove the inline content, or move it inside PHP tags using `echo`.");

        ctx.collector.report(issue);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = code_only_not_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function add(int $a, int $b): int
            {
                return $a + $b;
            }
        "#}
    }

    test_lint_success! {
        name = inline_content_within_php_tags_not_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            namespace App;

            echo "Hello, world!";
        "#}
    }

    test_lint_success! {
        name = empty_file_not_flagged,
        rule = NoInlineRule,
        code = ""
    }

    test_lint_success! {
        name = no_code_not_flagged,
        rule = NoInlineRule,
        code = "<?php"
    }

    test_lint_failure! {
        name = inline_content_before_php_tag_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            Hello, world!

            <?php

            echo "Hello, world!";
        "#}
    }

    test_lint_failure! {
        name = inline_content_after_php_tag_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            echo "Hello, world!";

            ?>

            Goodbye, world!
        "#}
    }

    test_lint_failure! {
        name = inline_content_between_php_tags_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            echo "Hello, world!";

            ?>

            This should not be here.

            <?php

            echo "Goodbye, world!";
        "#}
    }

    test_lint_failure! {
        name = multiple_inline_contents_flagged,
        rule = NoInlineRule,
        count = 3,
        code = indoc! {r#"
            Hello, world!

            <?php

            echo "Hello, world!";

            ?>

            This should not be here.

            <?php

            echo "Goodbye, world!";

            ?>

            Goodbye, world!
        "#}
    }

    test_lint_failure! {
        name = whitespace_inline_content_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            echo "Hello, world!";

            ?> <?php
        "#}
    }

    test_lint_failure! {
        name = newline_inline_content_flagged,
        rule = NoInlineRule,
        code = indoc! {r#"
            <?php

            echo "Hello, world!";

            ?>

        "#}
    }
}
