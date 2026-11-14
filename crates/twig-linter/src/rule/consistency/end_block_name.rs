use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_text_edit::TextEdit;
use mago_twig_syntax::cst::BlockBody;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct EndBlockNameRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: EndBlockNameConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct EndBlockNameConfig {
    pub level: Level,
}

impl Default for EndBlockNameConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for EndBlockNameConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for EndBlockNameRule {
    type Config = EndBlockNameConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Named Closing Tag",
            code: "twig-end-block-name",
            description: indoc! {r"
                Requires the block or macro name on `endblock` and `endmacro` tags.

                Short blocks need no closing tag. The fix adds the name and leaves whitespace control intact.
            "},
            good_example: indoc! {r"
                {% block title %}Title{% endblock title %}
            "},
            bad_example: indoc! {r"
                {% block title %}Title{% endblock %}
            "},
            category: Category::Consistency,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Block, NodeKind::Macro];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let (kind, name, end_keyword) = match node {
            Node::Block(block) => match &block.body {
                BlockBody::Long(body) if body.end_name.is_none() => ("block", block.name, body.end_keyword),
                _ => return,
            },
            Node::Macro(r#macro) if r#macro.end_name.is_none() => ("macro", r#macro.name, r#macro.end_keyword),
            _ => return,
        };

        let end = BytesDisplay(end_keyword.value);
        let named = BytesDisplay(name.value);
        let mut replacement = Vec::with_capacity(name.value.len() + 1);
        replacement.push(b' ');
        replacement.extend_from_slice(name.value);

        ctx.collector.propose(
            Issue::new(self.cfg.level, format!("Add the {kind} name to this `{end}` tag."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(end_keyword.span).with_message(format!("Closes `{named}`")))
                .with_annotation(Annotation::secondary(name.span).with_message(format!("The {kind} opens here")))
                .with_note(format!(
                    "A named closing tag shows which {kind} it closes, so long templates read without scrolling back up.",
                ))
                .with_help(format!("Write `{{% {end} {named} %}}` instead of `{{% {end} %}}`.")),
            |edits| edits.push(TextEdit::insert(end_keyword.span.end_offset(), replacement)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_lint_failure;
    use crate::test_lint_fix;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = EndBlockNameRule,
        code = r"{% block title %}Title{% endblock title %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = EndBlockNameRule,
        code = r"{% macro input() %}{% endmacro input %}",
    }

    test_lint_success! {
        name = valid_template_3,
        rule = EndBlockNameRule,
        code = r"{% block title 'Title' %}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = EndBlockNameRule,
        count = 1,
        code = r"{% block title %}Title{% endblock %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = EndBlockNameRule,
        count = 1,
        code = r"{% macro input() %}{% endmacro %}",
    }

    test_lint_fix! {
        name = block_name_is_added,
        rule = EndBlockNameRule,
        code = r"{% block title %}Title{% endblock %}",
        fixed = r"{% block title %}Title{% endblock title %}",
    }

    test_lint_fix! {
        name = macro_name_is_added,
        rule = EndBlockNameRule,
        code = r"{% macro input() %}{% endmacro %}",
        fixed = r"{% macro input() %}{% endmacro input %}",
    }
}
