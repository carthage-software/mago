use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Closure;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Function;
use mago_syntax::cst::Method;
use mago_syntax::cst::MethodBody;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;
use mago_syntax::cst::Statement;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::early_exit::EarlyExitPattern;
use crate::rule::utils::early_exit::check_early_exit;
use crate::rule::utils::early_exit::extract_single_if_from_statements;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferEarlyReturnRule {
    meta: &'static RuleMeta,
    cfg: PreferEarlyReturnConfig,
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PreferEarlyReturnConfig {
    pub level: Level,
    pub max_allowed_statements: usize,
}

impl Default for PreferEarlyReturnConfig {
    fn default() -> Self {
        Self { level: Level::Help, max_allowed_statements: 0 }
    }
}

impl Config for PreferEarlyReturnConfig {
    fn level(&self) -> Level {
        self.level
    }

    fn default_enabled() -> bool {
        false
    }
}

#[allow(clippy::unwrap_used)]
impl LintRule for PreferEarlyReturnRule {
    type Config = PreferEarlyReturnConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Early Return",
            code: "prefer-early-return",
            description: indoc! {"
                Suggests using early return pattern when a function body contains only a single if statement.

                This improves code readability by reducing nesting and making the control flow more explicit.
            "},
            good_example: indoc! {r"
                <?php

                function process($condition) {
                    if (!$condition) {
                        return;
                    }
                    doSomething();
                }
            "},
            bad_example: indoc! {r"
                <?php

                function process($condition) {
                    if ($condition) {
                        doSomething();
                    }
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };
        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Function, NodeKind::Method, NodeKind::Closure];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (if_statement, function_span) = match node {
            Node::Function(Function { body, .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            Node::Method(Method { body: MethodBody::Concrete(body), .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            Node::Closure(Closure { body, .. }) => {
                (extract_single_if_from_statements(body.statements.as_slice()), node.span())
            }
            _ => return,
        };

        let Some(if_stmt) = if_statement else { return };

        check_early_exit(
            ctx,
            self.cfg.level(),
            if_stmt,
            function_span,
            self.cfg.max_allowed_statements,
            &EarlyExitPattern {
                keyword: "return",
                code: self.meta.code,
                title: "Consider using early return pattern to reduce nesting.",
                primary_message: "This if statement wraps the entire function body",
                secondary_message: "The function can benefit from early return to improve readability",
                help: "Invert the condition and use `return` to exit early, then place the main logic outside the if block.",
                is_early_exit_statement,
            },
        );
    }
}

fn is_early_exit_statement(stmt: &Statement) -> bool {
    match stmt {
        Statement::Return(_) => true,
        Statement::Expression(expr) => matches!(expr.expression, Expression::Throw(_)),
        Statement::Block(block) => block.statements.len() == 1 && is_early_exit_statement(&block.statements.nodes[0]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferEarlyReturnRule;
    use crate::test_lint_fix;

    test_lint_fix! {
        name = fix_without_comments_keeps_compact_shape,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            function process($node) {
                if (is_array($node)) {
                    handle($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            function process($node) {
                if (!(is_array($node))) { return; }

            handle($node);
            }
        "}
    }

    test_lint_fix! {
        name = fix_preserves_leading_comment,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            function process($node) {
                if (is_array($node)) {
                    // keep only array nodes
                    handle($node);
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            function process($node) {
                if (!(is_array($node))) { return; }

                    // keep only array nodes
                    handle($node);
            }
        "}
    }

    test_lint_fix! {
        name = fix_method_body,
        rule = PreferEarlyReturnRule,
        code = indoc! {r"
            <?php

            class Foo {
                public function bar($condition) {
                    if ($condition) {
                        doSomething();
                    }
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            class Foo {
                public function bar($condition) {
                    if (!($condition)) { return; }

            doSomething();
                }
            }
        "}
    }
}
