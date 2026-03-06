use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ArrowFunction;
use mago_syntax::ast::Closure;
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
pub struct NoUnusedClosureImportRule {
    meta: &'static RuleMeta,
    cfg: NoUnusedClosureImportConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnusedClosureImportConfig {
    pub level: Level,
}

impl Default for NoUnusedClosureImportConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoUnusedClosureImportConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnusedClosureImportRule {
    type Config = NoUnusedClosureImportConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unused Closure Import",
            code: "no-unused-closure-import",
            description: indoc! {"
                Detects unused variables in closure `use` clauses.

                Variables imported into a closure via `use ($var)` that are never
                referenced in the closure body are unnecessary and should be removed.
            "},
            good_example: indoc! {r"
                <?php

                $greeting = 'Hello';
                $fn = function () use ($greeting) {
                    echo $greeting;
                };
            "},
            bad_example: indoc! {r"
                <?php

                $unused = 'Hello';
                $fn = function () use ($unused) {
                    echo 'World';
                };
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Closure];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::Closure(closure) = node else {
            return;
        };

        let Some(ref use_clause) = closure.use_clause else {
            return;
        };

        for use_var in use_clause.variables.iter() {
            // Skip pass-by-reference imports — they may be used for write-only side effects
            if use_var.ampersand.is_some() {
                continue;
            }

            let var_span = use_var.variable.span();
            let var_name = use_var.variable.name;

            if !node_uses_variable(Node::Block(&closure.body), var_name) {
                let issue = Issue::new(self.cfg.level(), format!("Unused closure import `{}`", var_name))
                    .with_code(self.meta.code)
                    .with_annotation(
                        Annotation::primary(var_span)
                            .with_message("This variable is imported but never used in the closure body"),
                    )
                    .with_help(format!("Remove `{}` from the `use` clause", var_name));

                ctx.collector.report(issue);
            }
        }
    }
}

fn node_uses_variable(node: Node<'_, '_>, var_name: &str) -> bool {
    match node {
        Node::DirectVariable(variable) => variable.name == var_name,
        Node::Closure(closure) => closure_uses_variable(closure, var_name),
        Node::ArrowFunction(arrow_function) => arrow_function_uses_variable(arrow_function, var_name),
        Node::TryCatchClause(catch_clause) => node_uses_variable(Node::Block(&catch_clause.block), var_name),
        Node::StaticAbstractItem(_) => false,
        Node::StaticConcreteItem(item) => node_uses_variable(Node::Expression(item.value), var_name),
        Node::Global(_) | Node::AnonymousClass(_) => false,
        node if node.is_declaration() => false,
        _ => node.children().into_iter().any(|child| node_uses_variable(child, var_name)),
    }
}

#[inline]
fn closure_uses_variable(closure: &Closure<'_>, var_name: &str) -> bool {
    closure
        .use_clause
        .as_ref()
        .is_some_and(|use_clause| use_clause.variables.iter().any(|variable| variable.variable.name == var_name))
}

#[inline]
fn arrow_function_uses_variable(arrow_function: &ArrowFunction<'_>, var_name: &str) -> bool {
    if arrow_function.parameter_list.parameters.iter().any(|parameter| parameter.variable.name == var_name) {
        return false;
    }

    node_uses_variable(Node::Expression(arrow_function.expression), var_name)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnusedClosureImportRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = used_closure_import_is_ok,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $greeting = 'Hello';
            $fn = function () use ($greeting) {
                echo $greeting;
            };
        "}
    }

    test_lint_failure! {
        name = unused_closure_import_is_bad,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $unused = 'Hello';
            $fn = function () use ($unused) {
                echo 'World';
            };
        "}
    }

    test_lint_success! {
        name = by_reference_import_is_ok,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $counter = 0;
            $fn = function () use (&$counter) {
                echo 'World';
            };
        "}
    }

    test_lint_success! {
        name = no_use_clause_is_ok,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $fn = function () {
                echo 'World';
            };
        "}
    }

    test_lint_failure! {
        name = substring_match_does_not_count_as_use,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $id = 1;
            $fn = function () use ($id) {
                echo $id2;
            };
        "}
    }

    test_lint_success! {
        name = nested_arrow_function_counts_as_use,
        rule = NoUnusedClosureImportRule,
        code = indoc! {r"
            <?php

            $id = 1;
            $fn = function () use ($id) {
                return fn() => $id;
            };
        "}
    }
}
