use indoc::indoc;
use mago_allocator::Arena;
use schemars::JsonSchema;

use mago_bytes::BytesDisplay;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::Span;
use mago_twig_syntax::cst::BinaryOperator;
use mago_twig_syntax::cst::Expression;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule::utils::is_not_snake_case;
use crate::rule::utils::snake_case;
use crate::rule::utils::ungroup;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct VariableNameRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: VariableNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct VariableNameConfig {
    pub level: Level,
}

impl Default for VariableNameConfig {
    fn default() -> Self {
        Self { level: Level::Warning }
    }
}

impl Config for VariableNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for VariableNameRule {
    type Config = VariableNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Variable Name",
            code: "twig-variable-name",
            description: indoc! {r"
                Enforces snake_case for the names that `set`, `for`, `import`, `from`, arrow functions,
                assignments, and destructuring bind.

                Variables from the application, mapping keys, attribute names, and non-ASCII names are left alone.
            "},
            good_example: indoc! {r"
                {% set first_name = user.name %}
            "},
            bad_example: indoc! {r"
                {% set firstName = user.name %}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::For,
            NodeKind::Set,
            NodeKind::Import,
            NodeKind::ImportedMacro,
            NodeKind::ArrowFunction,
            NodeKind::Binary,
        ];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        match node {
            Node::For(r#for) => r#for.targets.iter().for_each(|name| self.check_name(ctx, name.value, name.span)),
            Node::Set(set) => set.names.iter().for_each(|name| self.check_name(ctx, name.value, name.span)),
            Node::Import(import) => self.check_name(ctx, import.alias.value, import.alias.span),
            Node::ImportedMacro(imported) => {
                let alias = imported.to.unwrap_or(imported.from);
                self.check_name(ctx, alias.value, alias.span);
            }
            Node::ArrowFunction(function) => {
                function.parameters.iter().for_each(|name| self.check_name(ctx, name.value, name.span));
            }
            Node::Binary(binary) if matches!(binary.operator, BinaryOperator::Assignment(_)) => {
                match ungroup(binary.lhs) {
                    Expression::Name(_) => self.check_binding(ctx, binary.lhs),
                    Expression::Array(array) => {
                        array
                            .elements
                            .iter()
                            .filter_map(|element| element.value())
                            .for_each(|value| self.check_binding(ctx, value));
                    }
                    Expression::HashMap(mapping) => {
                        mapping.entries.iter().for_each(|entry| self.check_binding(ctx, &entry.value));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

impl VariableNameRule {
    fn check_binding<A>(&self, ctx: &mut LintContext<'_, '_, A>, expression: &Expression<'_>)
    where
        A: Arena,
    {
        if let Expression::Name(name) = ungroup(expression) {
            self.check_name(ctx, name.name, name.span);
        }
    }

    fn check_name<A>(&self, ctx: &mut LintContext<'_, '_, A>, name: &[u8], span: Span)
    where
        A: Arena,
    {
        if !is_not_snake_case(name) {
            return;
        }

        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The variable name `{}` is not in snake_case.", BytesDisplay(name)))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(span).with_message("Not snake_case"))
                .with_note("The Twig coding standard spells variable, macro, and argument names in snake_case.")
                .with_help(format!(
                    "Rename the variable to `{}`, and update every use of it in this template.",
                    snake_case(name),
                )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = VariableNameRule,
        code = r"{% set first_name = firstName %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = VariableNameRule,
        code = r"{% for item_key, item_value in list %}{{ item_value }}{% endfor %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = VariableNameRule,
        code = r"{{ user.firstName }}",
    }

    test_lint_success! {
        name = valid_template_4,
        rule = VariableNameRule,
        code = r"{% set _internal = 1 %}",
    }

    test_lint_success! {
        name = valid_template_5,
        rule = VariableNameRule,
        code = r"{{ first_name = 1 }}",
    }

    test_lint_success! {
        name = valid_template_6,
        rule = VariableNameRule,
        code = r"{{ first_name = second_name = 1 }}",
    }

    test_lint_success! {
        name = valid_template_7,
        rule = VariableNameRule,
        code = r"{{ [first_name, , last_name] = values }}",
    }

    test_lint_success! {
        name = valid_template_8,
        rule = VariableNameRule,
        code = r"{{ {firstName: first_name} = row }}",
    }

    test_lint_success! {
        name = valid_template_9,
        rule = VariableNameRule,
        code = r"{% set userNamé = value %}",
    }

    test_lint_success! {
        name = valid_template_10,
        rule = VariableNameRule,
        code = r"{% set 用户Name = value %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = VariableNameRule,
        count = 2,
        code = r"{% set firstName, lastName = user.firstName, user.lastName %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = VariableNameRule,
        count = 1,
        code = r"{% for itemValue in list %}{% endfor %}",
    }

    test_lint_failure! {
        name = invalid_template_3,
        rule = VariableNameRule,
        count = 1,
        code = r"{% import 'macros' as htmlForm %}",
    }

    test_lint_failure! {
        name = invalid_template_4,
        rule = VariableNameRule,
        count = 1,
        code = r"{% from 'macros' import form as htmlForm %}",
    }

    test_lint_failure! {
        name = invalid_template_5,
        rule = VariableNameRule,
        count = 1,
        code = r"{{ list|map(itemValue => itemValue.name) }}",
    }

    test_lint_failure! {
        name = invalid_template_6,
        rule = VariableNameRule,
        count = 1,
        code = r"{{ firstName = 1 }}",
    }

    test_lint_failure! {
        name = invalid_template_7,
        rule = VariableNameRule,
        count = 2,
        code = r"{{ firstName = secondName = 1 }}",
    }

    test_lint_failure! {
        name = invalid_template_8,
        rule = VariableNameRule,
        count = 1,
        code = r"{% if firstName = true %}yes{% endif %}",
    }

    test_lint_failure! {
        name = invalid_template_9,
        rule = VariableNameRule,
        count = 2,
        code = r"{{ [firstName, , lastName] = values }}",
    }

    test_lint_failure! {
        name = invalid_template_10,
        rule = VariableNameRule,
        count = 1,
        code = r"{{ {name: firstName} = row }}",
    }

    test_lint_failure! {
        name = invalid_template_11,
        rule = VariableNameRule,
        count = 1,
        code = r"{{ [(firstName)] = values }}",
    }
}
