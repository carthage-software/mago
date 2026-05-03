use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::AnonymousClass;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Call;
use mago_syntax::ast::Class;
use mago_syntax::ast::Closure;
use mago_syntax::ast::Enum;
use mago_syntax::ast::Expression;
use mago_syntax::ast::ExpressionStatement;
use mago_syntax::ast::Function;
use mago_syntax::ast::Interface;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Statement;
use mago_syntax::ast::Trait;
use mago_syntax::walker::MutWalker;
use mago_syntax::walker::walk_program_mut;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::call::function_call_matches;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoSideEffectsWithDeclarationsRule {
    meta: &'static RuleMeta,
    cfg: NoSideEffectsWithDeclarationsConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoSideEffectsWithDeclarationsConfig {
    pub level: Level,
    /// When true, conditional declarations (`if (...) { class Foo {} }`)
    /// are allowed alongside declarations. This covers the common pattern
    /// of polyfilling a class depending on an extension or PHP version.
    pub allow_conditional_declarations: bool,
    /// When true, top-level `class_alias(...)` calls are allowed alongside
    /// declarations.
    pub allow_class_alias: bool,
    /// When true, top-level `class_exists(...)` calls are allowed alongside
    /// declarations. This is commonly used to trigger autoloading/preloading.
    pub allow_class_exists: bool,
}

impl Default for NoSideEffectsWithDeclarationsConfig {
    fn default() -> Self {
        Self {
            level: Level::Warning,
            allow_conditional_declarations: true,
            allow_class_alias: true,
            allow_class_exists: true,
        }
    }
}

impl Config for NoSideEffectsWithDeclarationsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoSideEffectsWithDeclarationsRule {
    type Config = NoSideEffectsWithDeclarationsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Mixed Declarations And Side Effects",
            code: "no-side-effects-with-declarations",
            description: indoc! {"
                Enforces that a PHP file either declares symbols (classes, functions,
                constants, interfaces, traits, enums) or causes side-effects, but not
                both.

                Side-effects include `echo`, `print`, top-level function calls,
                assignments, `include`/`require` statements, and any other executable
                code outside of a symbol declaration.

                This follows the PSR-1 basic coding standard: files SHOULD either
                declare symbols or execute logic, but SHOULD NOT do both.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                class UserManager
                {
                    public function find(int $id): ?User
                    {
                        return null;
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                echo 'Loading utility file...';

                class StringHelper
                {
                    public static function slugify(string $input): string
                    {
                        return '';
                    }
                }
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        let mut collector = StatementCollector { cfg: &self.cfg, declarations: Vec::new(), side_effects: Vec::new() };

        walk_program_mut(&mut collector, program, ctx);

        if collector.declarations.is_empty() || collector.side_effects.is_empty() {
            return;
        }

        let first_declaration = collector.declarations[0];
        let first_side_effect = collector.side_effects[0];

        let mut issue = Issue::new(self.cfg.level(), "File mixes symbol declarations and side-effects.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(first_side_effect).with_message("Side-effect here"))
            .with_annotation(Annotation::secondary(first_declaration).with_message("Declaration here"));

        for &span in collector.declarations.iter().skip(1) {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("Declaration here"));
        }

        for &span in collector.side_effects.iter().skip(1) {
            issue = issue.with_annotation(Annotation::secondary(span).with_message("Side-effect here"));
        }

        issue = issue
            .with_note(
                "Files should either declare symbols (classes, functions, constants) or cause side-effects, but not both.",
            )
            .with_help("Move the side-effects into a separate file (e.g. a bootstrap or entry-point script).");

        ctx.collector.report(issue);
    }
}

struct StatementCollector<'cfg> {
    cfg: &'cfg NoSideEffectsWithDeclarationsConfig,
    declarations: Vec<Span>,
    side_effects: Vec<Span>,
}

impl<'ast, 'arena> MutWalker<'ast, 'arena, LintContext<'_, 'arena>> for StatementCollector<'_> {
    fn walk_in_statement(&mut self, statement: &'ast Statement<'arena>, ctx: &mut LintContext<'_, 'arena>) {
        match statement {
            _ if statement.is_declaration()
                && !matches!(statement, Statement::Declare(_) | Statement::Namespace(_)) =>
            {
                self.declarations.push(statement.span());
            }
            Statement::OpeningTag(_)
            | Statement::ClosingTag(_)
            | Statement::Use(_)
            | Statement::Declare(_)
            | Statement::Namespace(_)
            | Statement::Block(_)
            | Statement::Noop(_) => {}
            Statement::If(_) if self.cfg.allow_conditional_declarations => {}
            Statement::Expression(expr_stmt) if self.is_allowed_expression(expr_stmt, ctx) => {}
            _ => {
                self.side_effects.push(statement.span());
            }
        }
    }

    // Stop descent into symbol bodies; we only care about top-level statements.
    fn walk_class(&mut self, _: &'ast Class<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_interface(&mut self, _: &'ast Interface<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_trait(&mut self, _: &'ast Trait<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_enum(&mut self, _: &'ast Enum<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_function(&mut self, _: &'ast Function<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_closure(&mut self, _: &'ast Closure<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_arrow_function(&mut self, _: &'ast ArrowFunction<'arena>, _: &mut LintContext<'_, 'arena>) {}
    fn walk_anonymous_class(&mut self, _: &'ast AnonymousClass<'arena>, _: &mut LintContext<'_, 'arena>) {}
}

impl StatementCollector<'_> {
    fn is_allowed_expression<'arena>(
        &self,
        expr_stmt: &ExpressionStatement<'arena>,
        ctx: &LintContext<'_, 'arena>,
    ) -> bool {
        let Expression::Call(Call::Function(function_call)) = &expr_stmt.expression else {
            return false;
        };

        (self.cfg.allow_class_alias && function_call_matches(ctx, function_call, "class_alias"))
            || (self.cfg.allow_class_exists && function_call_matches(ctx, function_call, "class_exists"))
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    use super::*;

    fn enable(s: &mut Settings) {
        s.rules.no_side_effects_with_declarations.enabled = true;
    }

    fn strict(s: &mut Settings) {
        s.rules.no_side_effects_with_declarations.enabled = true;
        s.rules.no_side_effects_with_declarations.config.allow_conditional_declarations = false;
        s.rules.no_side_effects_with_declarations.config.allow_class_alias = false;
        s.rules.no_side_effects_with_declarations.config.allow_class_exists = false;
    }

    test_lint_success! {
        name = declarations_only,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo {}

            function bar(): void {}

            const BAZ = 1;
        "#}
    }

    test_lint_success! {
        name = side_effects_only,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            echo 'hello';
            require_once 'bootstrap.php';
        "#}
    }

    test_lint_success! {
        name = empty_file,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php
        "#}
    }

    test_lint_success! {
        name = conditional_declaration_allowed_by_default,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            if (!class_exists('Foo')) {
                class Foo {}
            }

            class Bar {}
        "#}
    }

    test_lint_success! {
        name = conditional_if_else_both_declarations,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            if (defined('USE_X')) {
                class Foo implements X {}
            } else {
                class Foo implements Y {}
            }

            class Bar {}
        "#}
    }

    test_lint_failure! {
        name = conditional_if_else_with_side_effect_in_else,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            if (defined('USE_X')) {
                class Foo {}
            } else {
                echo 'fallback';
            }

            class Bar {}
        "#}
    }

    test_lint_failure! {
        name = conditional_if_with_side_effect_in_body,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            if (defined('USE_X')) {
                echo 'loading';
            }

            class Foo {}
        "#}
    }

    test_lint_success! {
        name = class_alias_allowed_by_default,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            class Foo {}

            class_alias('Foo', 'Bar');
        "#}
    }

    test_lint_success! {
        name = class_exists_allowed_by_default,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            class Foo {}

            class_exists('Foo');
        "#}
    }

    test_lint_success! {
        name = use_and_declare_are_neutral,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            declare(strict_types=1);

            namespace App;

            use Some\Other\Klass;

            class Foo {}
        "#}
    }

    test_lint_success! {
        name = braced_namespace_declarations_only,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            namespace App {
                class Foo {}
                function bar(): void {}
            }
        "#}
    }

    test_lint_failure! {
        name = echo_before_class,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            echo 'Loading...';

            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = echo_inside_namespace,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            namespace App;

            echo 'hello';

            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = require_with_declarations,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            require_once 'autoload.php';

            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = arbitrary_function_call_with_declaration,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            some_setup_function();

            class Foo {}
        "#}
    }

    test_lint_failure! {
        name = braced_namespace_with_side_effects,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            namespace App {
                echo 'hello';
                class Foo {}
            }
        "#}
    }

    test_lint_failure! {
        name = conditional_declaration_disallowed_in_strict,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = strict,
        code = indoc! {r#"
            <?php

            if (!class_exists('Foo')) {
                class Foo {}
            }

            class Bar {}
        "#}
    }

    test_lint_failure! {
        name = class_alias_disallowed_in_strict,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = strict,
        code = indoc! {r#"
            <?php

            class Foo {}

            class_alias('Foo', 'Bar');
        "#}
    }

    test_lint_failure! {
        name = class_exists_disallowed_in_strict,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = strict,
        code = indoc! {r#"
            <?php

            class Foo {}

            class_exists('Foo');
        "#}
    }

    test_lint_success! {
        name = echo_inside_function_body_is_not_a_top_level_side_effect,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar(): void {
                    echo 'inside method, not a top-level side-effect';
                }
            }
        "#}
    }

    test_lint_success! {
        name = function_body_side_effects_are_fine,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            function setup(): void {
                require_once 'bootstrap.php';
                echo 'done';
            }

            class Foo {}
        "#}
    }

    test_lint_success! {
        name = namespaced_class_alias_allowed,
        rule = NoSideEffectsWithDeclarationsRule,
        settings = enable,
        code = indoc! {r#"
            <?php

            namespace App;

            class Foo {}

            \class_alias('App\Foo', 'App\Bar');
        "#}
    }
}
