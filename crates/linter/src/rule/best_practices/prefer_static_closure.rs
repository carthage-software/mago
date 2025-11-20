use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_fixer::SafetyClassification;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferStaticClosureRule {
    meta: &'static RuleMeta,
    cfg: PreferStaticClosureConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct PreferStaticClosureConfig {
    pub level: Level,
}

impl Default for PreferStaticClosureConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for PreferStaticClosureConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferStaticClosureRule {
    type Config = PreferStaticClosureConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Static Closure",
            code: "prefer-static-closure",
            description: indoc! {"
                Suggests adding the `static` keyword to closures and arrow functions that don't use `$this`.

                Static closures don't bind `$this`, making them more memory-efficient and their intent clearer.
            "},
            good_example: indoc! {r#"
                <?php

                class Foo {
                    public function bar() {
                        // Static closure - doesn't use $this
                        $fn = static fn($x) => $x * 2;

                        // Non-static - uses $this
                        $fn2 = fn() => $this->doSomething();

                        // Static function - doesn't use $this
                        $closure = static function($x) {
                            return $x * 2;
                        };
                    }
                }
            "#},
            bad_example: indoc! {r#"
                <?php

                class Foo {
                    public function bar() {
                        // Missing static - doesn't use $this
                        $fn = fn($x) => $x * 2;

                        // Missing static - doesn't use $this
                        $closure = function($x) {
                            return $x * 2;
                        };
                    }
                }
            "#},
            category: Category::BestPractices,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Closure, NodeKind::ArrowFunction];
        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        // Must be inside a class to have $this available
        if ctx.scope.get_class_like_scope().is_none() {
            return;
        }

        match node {
            Node::Closure(closure) => {
                // Already static - skip
                if closure.r#static.is_some() {
                    return;
                }

                // Check if body contains $this
                if contains_this_reference(Node::Block(&closure.body)) {
                    return;
                }

                self.report_issue(ctx, closure.function.span(), "closure");
            }
            Node::ArrowFunction(arrow) => {
                // Already static - skip
                if arrow.r#static.is_some() {
                    return;
                }

                // Check if expression contains $this
                if contains_this_reference(Node::Expression(arrow.expression)) {
                    return;
                }

                self.report_issue(ctx, arrow.r#fn.span(), "arrow function");
            }
            _ => {}
        }
    }
}

impl PreferStaticClosureRule {
    fn report_issue(&self, ctx: &mut LintContext, keyword_span: Span, kind: &str) {
        let issue =
            Issue::new(self.cfg.level(), format!("This {} does not use `$this` and should be declared static.", kind))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(keyword_span)
                        .with_message(format!("add `static` before this {} keyword", kind)),
                )
                .with_note("Static closures are more memory-efficient and make it clear that `$this` is not used.")
                .with_help(format!(
                    "Add the `static` keyword before `{}` to make this {} static.",
                    if kind == "closure" { "function" } else { "fn" },
                    kind
                ));

        ctx.collector.propose(issue, |plan| {
            // Insert "static " before the function/fn keyword
            plan.insert(keyword_span.start_position().offset, "static ", SafetyClassification::Safe);
        });
    }
}

fn contains_this_reference<'ast, 'arena>(node: Node<'ast, 'arena>) -> bool {
    // Check current node
    if let Node::Expression(Expression::Variable(Variable::Direct(var))) = node
        && var.name == "$this"
    {
        return true;
    }

    // Don't recurse into anonymous classes or nested declarations (they have their own $this binding)
    // Note: Non-static closures and arrow functions inherit $this from their parent scope, so we DO recurse into them
    match node {
        Node::Closure(closure) if closure.r#static.is_some() => return false,
        Node::ArrowFunction(arrow_function) if arrow_function.r#static.is_some() => return false,
        Node::AnonymousClass(_) => return false,
        node if node.is_declaration() => return false,
        _ => {}
    }

    // Recursively check children
    for child in node.children() {
        if contains_this_reference(child) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::PreferStaticClosureRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    // Success cases - code should NOT produce lint issues

    test_lint_success! {
        name = closure_uses_this_directly,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    $fn = function() {
                        return $this->value;
                    };
                }
            }
        "#}
    }

    test_lint_success! {
        name = arrow_function_uses_this_directly,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    $fn = fn() => $this->value;
                }
            }
        "#}
    }

    test_lint_success! {
        name = nested_arrow_function_uses_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    $fn = fn() => fn() => $this->value;
                }
            }
        "#}
    }

    test_lint_success! {
        name = nested_closure_uses_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    $fn = function() {
                        return function() {
                            return $this->value;
                        };
                    };
                }
            }
        "#}
    }

    test_lint_success! {
        name = mixed_nested_closures_use_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    // Arrow function containing closure that uses $this
                    $fn1 = fn() => function() {
                        return $this->value;
                    };

                    // Closure containing arrow function that uses $this
                    $fn2 = function() {
                        return fn() => $this->value;
                    };
                }
            }
        "#}
    }

    test_lint_success! {
        name = deeply_nested_closures_use_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private int $value = 42;

                public function bar() {
                    $fn = fn() => fn() => fn() => $this->value;
                }
            }
        "#}
    }

    test_lint_success! {
        name = already_static_closure,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = static function($x) {
                        return $x * 2;
                    };
                }
            }
        "#}
    }

    test_lint_success! {
        name = already_static_arrow_function,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = static fn($x) => $x * 2;
                }
            }
        "#}
    }

    test_lint_success! {
        name = outside_class_context,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            function foo() {
                $fn = fn($x) => $x * 2;
            }
        "#}
    }

    test_lint_success! {
        name = anonymous_class_has_own_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    // The outer closure should be static even though the inner
                    // anonymous class uses $this, because they have different $this
                    $fn = static function() {
                        return new class {
                            private int $value = 42;

                            public function getValue() {
                                return $this->value;
                            }
                        };
                    };
                }
            }
        "#}
    }

    test_lint_failure! {
        name = closure_does_not_use_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = function($x) {
                        return $x * 2;
                    };
                }
            }
        "#}
    }

    test_lint_failure! {
        name = arrow_function_does_not_use_this,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = fn($x) => $x * 2;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = nested_closures_do_not_use_this,
        rule = PreferStaticClosureRule,
        count = 2,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = fn() => fn($x) => $x * 2;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = multiple_closures_without_this,
        rule = PreferStaticClosureRule,
        count = 3,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn1 = fn($x) => $x * 2;
                    $fn2 = function($x) { return $x + 1; };
                    $fn3 = fn($x) => $x - 1;
                }
            }
        "#}
    }

    test_lint_failure! {
        name = outer_closure_static_but_nested_not,
        rule = PreferStaticClosureRule,
        code = indoc! {r#"
            <?php

            class Foo {
                public function bar() {
                    $fn = static fn() => fn($x) => $x * 2;
                }
            }
        "#}
    }
}
