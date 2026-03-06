use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::DeclareBody;
use mago_syntax::ast::ForBody;
use mago_syntax::ast::ForeachBody;
use mago_syntax::ast::IfBody;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::SwitchBody;
use mago_syntax::ast::WhileBody;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoAlternativeSyntaxRule {
    meta: &'static RuleMeta,
    cfg: NoAlternativeSyntaxConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoAlternativeSyntaxConfig {
    pub level: Level,
}

impl Default for NoAlternativeSyntaxConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoAlternativeSyntaxConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoAlternativeSyntaxRule {
    type Config = NoAlternativeSyntaxConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Alternative Syntax",
            code: "no-alternative-syntax",
            description: indoc! {"
                Detects the use of alternative syntax for control structures
                (`endif`, `endwhile`, `endfor`, `endforeach`, `endswitch`).

                The brace-style syntax is preferred for consistency with the rest
                of the codebase and is the convention used by the Symfony coding standards.
            "},
            good_example: indoc! {r"
                <?php

                if ($condition) {
                    echo 'yes';
                }
            "},
            bad_example: indoc! {r"
                <?php

                if ($condition):
                    echo 'yes';
                endif;
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] =
            &[NodeKind::If, NodeKind::While, NodeKind::For, NodeKind::Foreach, NodeKind::Switch, NodeKind::Declare];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        match node {
            Node::If(if_stmt) => {
                if let IfBody::ColonDelimited(body) = &if_stmt.body {
                    self.report(ctx, "if", body.endif.span());
                }
            }
            Node::While(while_stmt) => {
                if let WhileBody::ColonDelimited(body) = &while_stmt.body {
                    self.report(ctx, "while", body.end_while.span());
                }
            }
            Node::For(for_stmt) => {
                if let ForBody::ColonDelimited(body) = &for_stmt.body {
                    self.report(ctx, "for", body.end_for.span());
                }
            }
            Node::Foreach(foreach_stmt) => {
                if let ForeachBody::ColonDelimited(body) = &foreach_stmt.body {
                    self.report(ctx, "foreach", body.end_foreach.span());
                }
            }
            Node::Switch(switch_stmt) => {
                if let SwitchBody::ColonDelimited(body) = &switch_stmt.body {
                    self.report(ctx, "switch", body.end_switch.span());
                }
            }
            Node::Declare(declare_stmt) => {
                if let DeclareBody::ColonDelimited(body) = &declare_stmt.body {
                    self.report(ctx, "declare", body.end_declare.span());
                }
            }
            _ => {}
        }
    }
}

impl NoAlternativeSyntaxRule {
    fn report(&self, ctx: &mut LintContext, keyword: &str, end_keyword_span: mago_span::Span) {
        let issue = Issue::new(self.cfg.level(), format!("Do not use alternative syntax `end{}`", keyword))
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(end_keyword_span)
                    .with_message(format!("`end{}` — use brace syntax instead", keyword)),
            )
            .with_help(format!(
                "Replace the colon-delimited `{}:...end{};` with brace syntax `{} {{ ... }}`",
                keyword, keyword, keyword
            ));

        ctx.collector.report(issue);
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoAlternativeSyntaxRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = brace_if_is_ok,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            if ($a) {
                echo 'yes';
            }
        "}
    }

    test_lint_failure! {
        name = alternative_if_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            if ($a):
                echo 'yes';
            endif;
        "}
    }

    test_lint_failure! {
        name = alternative_while_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            while ($a):
                echo 'yes';
            endwhile;
        "}
    }

    test_lint_failure! {
        name = alternative_foreach_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            foreach ($items as $item):
                echo $item;
            endforeach;
        "}
    }

    test_lint_failure! {
        name = alternative_for_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            for ($i = 0; $i < 10; $i++):
                echo $i;
            endfor;
        "}
    }

    test_lint_failure! {
        name = alternative_switch_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            switch ($value):
                case 1:
                    echo 'one';
                    break;
            endswitch;
        "}
    }

    test_lint_failure! {
        name = alternative_declare_is_bad,
        rule = NoAlternativeSyntaxRule,
        code = indoc! {r"
            <?php

            declare(ticks=1):
                echo 'hello';
            enddeclare;
        "}
    }
}
