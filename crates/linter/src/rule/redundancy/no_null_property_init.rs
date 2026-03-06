use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Literal;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::PropertyItem;
use mago_text_edit::TextEdit;
use mago_text_edit::TextRange;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNullPropertyInitRule {
    meta: &'static RuleMeta,
    cfg: NoNullPropertyInitConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoNullPropertyInitConfig {
    pub level: Level,
}

impl Default for NoNullPropertyInitConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoNullPropertyInitConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNullPropertyInitRule {
    type Config = NoNullPropertyInitConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Null Property Initialization",
            code: "no-null-property-init",
            description: indoc! {"
                Detects redundant `= null` initialization on untyped properties.

                Untyped properties already default to `null`, making an explicit
                `= null` initializer unnecessary.
            "},
            good_example: indoc! {r"
                <?php

                class Foo {
                    public $name;
                }
            "},
            bad_example: indoc! {r"
                <?php

                class Foo {
                    public $name = null;
                }
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::PlainProperty];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::PlainProperty(property) = node else {
            return;
        };

        // Typed properties become uninitialized without an explicit initializer,
        // so this rule only applies to untyped properties.
        if property.hint.is_some() {
            return;
        }

        for item in property.items.iter() {
            let PropertyItem::Concrete(concrete) = item else {
                continue;
            };

            // Check if the value is `null`
            let is_null = matches!(concrete.value, Expression::Literal(Literal::Null(_)));
            if !is_null {
                continue;
            }

            let issue = Issue::new(self.cfg.level(), "Redundant `= null` initialization on untyped property")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(concrete.value.span()).with_message("This `= null` is redundant"))
                .with_note("Untyped properties default to `null` without explicit initialization.")
                .with_help("Remove the `= null` assignment");

            ctx.collector.propose(issue, |edits| {
                // Remove from the equals sign through the null value
                edits.push(TextEdit::delete(TextRange::new(
                    concrete.equals.start_offset(),
                    concrete.value.span().end_offset(),
                )));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoNullPropertyInitRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = nullable_without_init_is_ok,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                private ?string $name;
            }
        "}
    }

    test_lint_success! {
        name = nullable_with_non_null_init_is_ok,
        rule = NoNullPropertyInitRule,
        code = indoc! {r#"
            <?php

            class Foo {
                private ?string $name = "default";
            }
        "#}
    }

    test_lint_success! {
        name = non_nullable_with_null_is_ok,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                private string $name = '';
            }
        "}
    }

    test_lint_failure! {
        name = untyped_nullable_name_with_null_init_is_bad,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                public $name = null;
            }
        "}
    }

    test_lint_success! {
        name = nullable_typed_property_with_null_init_is_ok,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                private ?string $name = null;
            }
        "}
    }

    test_lint_success! {
        name = union_typed_property_with_null_init_is_ok,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                private string|null $name = null;
            }
        "}
    }

    test_lint_failure! {
        name = untyped_with_null_init_is_bad,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                public $bar = null;
            }
        "}
    }

    test_lint_failure! {
        name = var_with_null_init_is_bad,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                var $bar = null;
            }
        "}
    }

    test_lint_failure! {
        name = static_untyped_with_null_init_is_bad,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                public static $bar = null;
            }
        "}
    }

    test_lint_success! {
        name = non_null_typed_property_not_flagged,
        rule = NoNullPropertyInitRule,
        code = indoc! {r"
            <?php

            class Foo {
                private string $name;
            }
        "}
    }
}
