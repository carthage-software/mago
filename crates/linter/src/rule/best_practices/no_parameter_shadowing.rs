use foldhash::HashMap;
use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::Span;
use mago_syntax::ast::*;
use mago_syntax::walker::MutWalker;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoParameterShadowingRule {
    meta: &'static RuleMeta,
    cfg: NoParameterShadowingConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoParameterShadowingConfig {
    pub level: Level,
}

impl Default for NoParameterShadowingConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for NoParameterShadowingConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoParameterShadowingRule {
    type Config = NoParameterShadowingConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Parameter Shadowing",
            code: "no-parameter-shadowing",
            description: indoc! {"
                Detects when a function or method parameter is shadowed by a loop variable
                or catch variable, making the original parameter value inaccessible.
            "},
            good_example: indoc! {r"
                <?php

                function read(array $domains, array $locales): void
                {
                    $translations = getTranslations($domains, $locales);

                    foreach ($translations as $namespace => $namespaceLocales) {
                        // $locales is still accessible
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                function read(array $domains, array $locales): void
                {
                    $translations = getTranslations($domains, $locales);

                    foreach ($translations as $namespace => $locales) {
                        // $locales now refers to the loop value, original argument is lost
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

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let (parameter_list, body) = match node {
            Node::Function(Function { parameter_list, body, .. }) => (parameter_list, body),
            Node::Method(Method { parameter_list, body: MethodBody::Concrete(body), .. }) => (parameter_list, body),
            Node::Closure(Closure { parameter_list, body, .. }) => (parameter_list, body),
            _ => return,
        };

        if parameter_list.parameters.is_empty() {
            return;
        }

        let params: HashMap<&str, Span> =
            parameter_list.parameters.iter().map(|p| (p.variable.name, p.variable.span)).collect();

        let mut collector = ShadowCollector { params: &params, shadows: HashMap::default() };

        collector.walk_block(body, &mut ());

        for (param_name, shadow_spans) in &collector.shadows {
            let param_span = params[param_name];

            let mut issue = Issue::new(
                self.cfg.level(),
                format!("Parameter `{param_name}` is shadowed by a loop or catch variable."),
            )
            .with_code(self.meta.code)
            .with_annotation(
                Annotation::primary(param_span).with_message(format!("`{param_name}` is defined as a parameter here")),
            );

            for shadow_span in shadow_spans {
                issue = issue.with_annotation(
                    Annotation::secondary(*shadow_span)
                        .with_message(format!("`{param_name}` is overwritten here, shadowing the parameter")),
                );
            }

            issue = issue
                .with_note(
                    "Reusing a parameter name as a loop or catch variable makes the original value inaccessible.",
                )
                .with_help(format!("Rename the variable to avoid shadowing the `{param_name}` parameter."));

            ctx.collector.report(issue);
        }
    }
}

struct ShadowCollector<'rule, 'arena> {
    params: &'rule HashMap<&'arena str, Span>,
    shadows: HashMap<&'arena str, Vec<Span>>,
}

impl<'rule, 'arena> ShadowCollector<'rule, 'arena> {
    fn check_variable(&mut self, name: &'arena str, span: Span) {
        if self.params.contains_key(name) {
            self.shadows.entry(name).or_default().push(span);
        }
    }

    fn check_expression<'ast>(&mut self, expr: &'ast Expression<'arena>) {
        let Expression::Variable(Variable::Direct(var)) = expr else {
            return;
        };

        self.check_variable(var.name, var.span);
    }
}

impl<'rule, 'ast, 'arena> MutWalker<'ast, 'arena, ()> for ShadowCollector<'rule, 'arena> {
    fn walk_in_foreach(&mut self, foreach: &'ast Foreach<'arena>, _: &mut ()) {
        match &foreach.target {
            ForeachTarget::Value(v) => self.check_expression(v.value),
            ForeachTarget::KeyValue(kv) => {
                self.check_expression(kv.key);
                self.check_expression(kv.value);
            }
        }
    }

    fn walk_in_try_catch_clause(&mut self, clause: &'ast TryCatchClause<'arena>, _: &mut ()) {
        if let Some(var) = &clause.variable {
            self.check_variable(var.name, var.span);
        }
    }

    // Don't descend into nested function-like scopes — they have their own parameters.
    fn walk_function(&mut self, _: &'ast Function<'arena>, _: &mut ()) {}
    fn walk_closure(&mut self, _: &'ast Closure<'arena>, _: &mut ()) {}
    fn walk_method(&mut self, _: &'ast Method<'arena>, _: &mut ()) {}
    fn walk_arrow_function(&mut self, _: &'ast ArrowFunction<'arena>, _: &mut ()) {}
    fn walk_property_hook(&mut self, _: &'ast PropertyHook<'arena>, _: &mut ()) {}
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoParameterShadowingRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = no_shadowing,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function read(array $domains, array $locales): void
            {
                foreach ($domains as $domain) {
                    echo $domain;
                }
            }
        "}
    }

    test_lint_success! {
        name = no_params,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function noParams(): void
            {
                foreach ($items as $item) {
                    echo $item;
                }
            }
        "}
    }

    test_lint_success! {
        name = nested_closure_does_not_trigger,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function outer(string $name): void
            {
                $fn = function (string $name): void {
                    echo $name;
                };
            }
        "}
    }

    test_lint_failure! {
        name = foreach_value_shadows_param,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function read(array $domains, array $locales): void
            {
                $translations = getTranslations($domains, $locales);

                foreach ($translations as $namespace => $locales) {
                    echo $namespace;
                }
            }
        "}
    }

    test_lint_failure! {
        name = foreach_key_shadows_param,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function process(string $key, array $items): void
            {
                foreach ($items as $key => $value) {
                    echo $value;
                }
            }
        "}
    }

    test_lint_failure! {
        name = catch_shadows_param,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            function handle(Exception $e): void
            {
                try {
                    doSomething();
                } catch (RuntimeException $e) {
                    log($e);
                }
            }
        "}
    }

    test_lint_failure! {
        name = multiple_shadows,
        rule = NoParameterShadowingRule,
        count = 2,
        code = indoc! {r"
            <?php

            function process(string $key, array $items): void
            {
                foreach ($items as $key => $items) {
                    echo $key;
                }
            }
        "}
    }

    test_lint_success! {
        name = method_with_abstract_body,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            abstract class Foo
            {
                abstract public function bar(string $name): void;
            }
        "}
    }

    test_lint_failure! {
        name = closure_param_shadowed,
        rule = NoParameterShadowingRule,
        code = indoc! {r"
            <?php

            $fn = function (array $items): void {
                foreach ($items as $key => $items) {
                    echo $key;
                }
            };
        "}
    }
}
