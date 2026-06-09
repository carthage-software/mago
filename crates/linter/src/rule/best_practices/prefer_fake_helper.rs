use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
use mago_syntax::ast::Variable;
use mago_text_edit::TextEdit;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::Integration;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::scope::ClassLikeScope;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct PreferFakeHelperRule {
    meta: &'static RuleMeta,
    cfg: PreferFakeHelperConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct PreferFakeHelperConfig {
    pub level: Level,
}

impl Default for PreferFakeHelperConfig {
    fn default() -> Self {
        Self { level: Level::Note }
    }
}

impl Config for PreferFakeHelperConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for PreferFakeHelperRule {
    type Config = PreferFakeHelperConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Prefer Fake Helper",
            code: "prefer-fake-helper",
            description: indoc! {"
                Detects use of the `$this->faker` property inside a Laravel model factory. The global
                `fake()` helper is the modern equivalent and does not depend on the property being set.
            "},
            good_example: indoc! {r"
                <?php

                class UserFactory
                {
                    public function definition(): array
                    {
                        return ['name' => fake()->name()];
                    }
                }
            "},
            bad_example: indoc! {r"
                <?php

                class UserFactory
                {
                    public function definition(): array
                    {
                        return ['name' => $this->faker->name()];
                    }
                }
            "},
            category: Category::BestPractices,
            requirements: RuleRequirements::Integration(Integration::Laravel),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::PropertyAccess];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::PropertyAccess(access) = node else {
            return;
        };

        let Expression::Variable(Variable::Direct(variable)) = access.object else {
            return;
        };

        if !variable.name.eq_ignore_ascii_case(b"$this") {
            return;
        }

        // Property names are case-sensitive in PHP.
        let ClassLikeMemberSelector::Identifier(property) = &access.property else {
            return;
        };

        if property.value != b"faker" {
            return;
        }

        // The `$this->faker` property only carries the Faker generator inside a factory.
        let Some(ClassLikeScope::Class(class_name)) = ctx.scope.get_class_like_scope() else {
            return;
        };

        if !class_name.ends_with(b"Factory") {
            return;
        }

        let inside_interpolation = ctx.is_child_of(NodeKind::CompositeString)
            || ctx.is_child_of(NodeKind::InterpolatedString)
            || ctx.is_child_of(NodeKind::ShellExecuteString)
            || ctx.is_child_of(NodeKind::BracedExpressionStringPart);

        let issue = Issue::new(self.cfg.level(), "Use the `fake()` helper instead of the `$this->faker` property.")
            .with_code(self.meta.code)
            .with_annotation(Annotation::primary(access.span()).with_message("This can be written as `fake()`"))
            .with_note("`fake()` is the modern helper for obtaining a Faker generator in a Laravel factory.")
            .with_help(if inside_interpolation {
                "Break the interpolation apart with concatenation or `sprintf`, then replace `$this->faker` with `fake()`."
            } else {
                "Replace `$this->faker` with `fake()`."
            });

        if inside_interpolation {
            ctx.collector.report(issue);
        } else {
            ctx.collector.propose(issue, |edits| {
                edits.push(TextEdit::replace(access.span(), "fake()"));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_failure! {
        name = faker_property_in_factory,
        rule = PreferFakeHelperRule,
        code = indoc! {r"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['name' => $this->faker->unique()->word()];
                }
            }
        "}
    }

    test_lint_success! {
        name = fake_helper_used,
        rule = PreferFakeHelperRule,
        code = indoc! {r"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['name' => fake()->word()];
                }
            }
        "}
    }

    test_lint_success! {
        name = faker_property_outside_factory,
        rule = PreferFakeHelperRule,
        code = indoc! {r"
            <?php

            class ReportService
            {
                public function build(): string
                {
                    return $this->faker->word();
                }
            }
        "}
    }

    test_lint_fix! {
        name = fix_faker_property,
        rule = PreferFakeHelperRule,
        code = indoc! {r"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['name' => $this->faker->unique()->word()];
                }
            }
        "},
        fixed = indoc! {r"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['name' => fake()->unique()->word()];
                }
            }
        "}
    }

    test_lint_failure! {
        name = faker_in_braced_interpolation_is_reported_but_not_fixed,
        rule = PreferFakeHelperRule,
        code = indoc! {r#"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['url' => "https://{$this->faker->domainName}"];
                }
            }
        "#}
    }

    test_lint_failure! {
        name = faker_in_bare_interpolation_is_reported_but_not_fixed,
        rule = PreferFakeHelperRule,
        code = indoc! {r#"
            <?php

            class UserFactory
            {
                public function definition(): array
                {
                    return ['url' => "host: $this->faker"];
                }
            }
        "#}
    }
}
