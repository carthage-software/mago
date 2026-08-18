use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::cst::Expression;
use mago_syntax::cst::Node;
use mago_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct AmbiguousFunctionCallRule {
    meta: &'static RuleMeta,
    cfg: AmbiguousFunctionCallConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct AmbiguousFunctionCallConfig {
    pub level: Level,
}

impl Default for AmbiguousFunctionCallConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for AmbiguousFunctionCallConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for AmbiguousFunctionCallRule {
    type Config = AmbiguousFunctionCallConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Ambiguous Function Call",
            code: "ambiguous-function-call",
            description: indoc! {"
                Enforces that all function calls made from within a namespace are explicit.

                When an unqualified function like `strlen()` is called from within a namespace, PHP
                performs a runtime fallback check (current namespace -> global namespace). This
                ambiguity prevents PHP from performing powerful compile-time optimizations,
                such as replacing a call to `strlen()` with the highly efficient `STRLEN` opcode.

                Making calls explicit improves readability, prevents bugs, and allows for significant
                performance gains in some cases.
            "},
            good_example: indoc! {r#"
                <?php

                namespace App;

                use function strlen;

                // OK: Explicitly imported
                $length1 = strlen("hello");

                // OK: Explicitly global
                $length2 = \strlen("hello");

                // OK: Explicitly namespaced
                $value = namespace\my_function();
            "#},
            bad_example: indoc! {r#"
                <?php

                namespace App;

                // Ambiguous: could be App\strlen or \strlen
                $length = strlen("hello");
            "#},
            category: Category::Consistency,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::FunctionCall];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        if ctx.scope.get_namespace().is_empty() {
            return;
        }

        let Node::FunctionCall(call) = node else {
            return;
        };

        let Expression::Identifier(identifier) = call.function else {
            return;
        };

        if identifier.is_qualified() || identifier.is_fully_qualified() {
            return;
        }

        if identifier.value().eq_ignore_ascii_case(b"clone")
            || identifier.value().eq_ignore_ascii_case(b"exit")
            || identifier.value().eq_ignore_ascii_case(b"die")
        {
            return;
        }

        if ctx.is_name_imported(identifier) {
            return;
        }

        let function_name = mago_bytes::BytesDisplay(identifier.value());

        ctx.collector.report(
            Issue::new(self.cfg.level, "Ambiguous function call detected.")
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(identifier.span())
                        .with_message(format!("The call to `{function_name}` is ambiguous")),
                )
                .with_note("At compile time, PHP cannot be sure if this refers to a namespaced function or the global one, preventing significant performance optimizations in some cases.")
                .with_note("Making calls explicit improves code clarity and prevents bugs if a function with the same name is later added to the namespace.")
                .with_help(format!("Make the call explicit: for global functions, use `\\{function_name}(...)` or add a `use function {function_name};` statement.")),
        );
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::AmbiguousFunctionCallRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = language_constructs_are_not_ambiguous_function_calls,
        rule = AmbiguousFunctionCallRule,
        code = indoc! {r#"
            <?php

            namespace App;

            final readonly class Value
            {
                public function __construct(public string $value) {}

                public function withValue(string $value): self
                {
                    return clone($this, ['value' => $value]);
                }
            }

            $exit = exit('done');
            $die = die('done');
        "#}
    }

    test_lint_success! {
        name = language_construct_names_are_case_insensitive,
        rule = AmbiguousFunctionCallRule,
        code = indoc! {r#"
            <?php

            namespace App;

            function copy(object $value): object
            {
                return ClOnE($value);
            }

            $exit = EXIT('done');
            $die = DiE('done');
        "#}
    }

    test_lint_failure! {
        name = similarly_named_functions_remain_ambiguous,
        rule = AmbiguousFunctionCallRule,
        count = 3,
        code = indoc! {r#"
            <?php

            namespace App;

            clone_object($value);
            exit_now();
            die_hard();
        "#}
    }
}
