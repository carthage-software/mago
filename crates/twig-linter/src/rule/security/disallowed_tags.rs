use indoc::indoc;
use mago_allocator::Arena;
use mago_bytes::BytesDisplay;
use schemars::JsonSchema;

use foldhash::HashSet;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_twig_syntax::cst::Keyword;
use mago_twig_syntax::cst::Node;
use mago_twig_syntax::cst::NodeKind;
use mago_twig_syntax::cst::Statement;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct DisallowedTagsRule {
    meta: &'static RuleMeta,
    pub(crate) cfg: DisallowedTagsConfig,
    names: HashSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, JsonSchema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default, rename_all = "kebab-case", deny_unknown_fields))]
pub struct DisallowedTagsConfig {
    pub level: Level,
    /// The names to disallow.
    pub names: Vec<String>,
}

impl Default for DisallowedTagsConfig {
    fn default() -> Self {
        Self { level: Level::Warning, names: Vec::new() }
    }
}

impl Config for DisallowedTagsConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for DisallowedTagsRule {
    type Config = DisallowedTagsConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "Disallowed Tags",
            code: "twig-disallowed-tags",
            description: indoc! {r"
                Reports the tags listed in `names`, including custom tags.

                Names match exactly. An empty list reports nothing.
            "},
            good_example: indoc! {r"
                {{ title }}
            "},
            bad_example: indoc! {r"
                {% flush %}
            "},
            category: Category::Security,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Statement];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self {
            meta: Self::meta(),
            names: settings.config.names.iter().cloned().collect(),
            cfg: settings.config.clone(),
        }
    }

    fn check<'arena, A>(&self, ctx: &mut LintContext<'_, 'arena, A>, node: Node<'_, 'arena>)
    where
        A: Arena,
    {
        let Node::Statement(statement) = node else {
            return;
        };

        let Some(keyword) = tag_keyword(statement) else {
            return;
        };

        if !std::str::from_utf8(keyword.value).is_ok_and(|tag| self.names.contains(tag)) {
            return;
        }

        let tag = BytesDisplay(keyword.value);
        ctx.collector.report(
            Issue::new(self.cfg.level, format!("The `{tag}` tag is disallowed in this project."))
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(keyword.span).with_message(format!("Use of `{tag}`")))
                .with_note(format!("`{tag}` is listed in `[twig.linter.rules.disallowed-tags] names`."))
                .with_help(format!(
                    "Replace the `{tag}` tag with an approach the project allows, or remove `{tag}` from the `names` list if it is now permitted.",
                )),
        );
    }
}

fn tag_keyword<'arena>(statement: &Statement<'arena>) -> Option<Keyword<'arena>> {
    Some(match statement {
        Statement::Text(_) | Statement::Print(_) => return None,
        Statement::Verbatim(node) => node.keyword,
        Statement::If(node) => node.branches.first().map_or(node.end_keyword, |branch| branch.keyword),
        Statement::For(node) => node.keyword,
        Statement::Set(node) => node.keyword,
        Statement::Block(node) => node.keyword,
        Statement::Macro(node) => node.keyword,
        Statement::Extends(node) => node.keyword,
        Statement::Use(node) => node.keyword,
        Statement::Include(node) => node.keyword,
        Statement::Embed(node) => node.keyword,
        Statement::Import(node) => node.keyword,
        Statement::From(node) => node.keyword,
        Statement::With(node) => node.keyword,
        Statement::Apply(node) => node.keyword,
        Statement::Autoescape(node) => node.keyword,
        Statement::Sandbox(node) => node.keyword,
        Statement::Deprecated(node) => node.keyword,
        Statement::Do(node) => node.keyword,
        Statement::Flush(node) => node.keyword,
        Statement::Guard(node) => node.keyword,
        Statement::Cache(node) => node.keyword,
        Statement::Types(node) => node.keyword,
        Statement::Unknown(node) => Keyword { span: node.name.span, value: node.name.value },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = valid_template_1,
        rule = DisallowedTagsRule,
        settings = |s: &mut Settings| s.rules.disallowed_tags.config.names = vec!["flush".into(), "app_tag".into()],
        code = r"{% custom_tag value %}",
    }

    test_lint_success! {
        name = valid_template_2,
        rule = DisallowedTagsRule,
        settings = |s: &mut Settings| s.rules.disallowed_tags.config.names = vec!["flush".into(), "app_tag".into()],
        code = r"{{ flush() }}",
    }

    test_lint_failure! {
        name = invalid_template_1,
        rule = DisallowedTagsRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_tags.config.names = vec!["flush".into(), "app_tag".into()],
        code = r"{% flush %}",
    }

    test_lint_failure! {
        name = invalid_template_2,
        rule = DisallowedTagsRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_tags.config.names = vec!["flush".into(), "app_tag".into()],
        code = r"{% app_tag value %}",
    }

    test_lint_failure! {
        name = nested_tags_are_reported,
        rule = DisallowedTagsRule,
        count = 1,
        settings = |s: &mut Settings| s.rules.disallowed_tags.config.names = vec!["flush".into()],
        code = r"{% if flag %}{% for item in items %}{% flush %}{% endfor %}{% endif %}",
    }
}
