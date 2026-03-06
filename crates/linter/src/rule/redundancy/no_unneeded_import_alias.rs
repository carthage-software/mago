use indoc::indoc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::Node;
use mago_syntax::ast::NodeKind;
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
pub struct NoUnneededImportAliasRule {
    meta: &'static RuleMeta,
    cfg: NoUnneededImportAliasConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoUnneededImportAliasConfig {
    pub level: Level,
}

impl Default for NoUnneededImportAliasConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoUnneededImportAliasConfig {
    fn default_enabled() -> bool {
        false
    }

    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoUnneededImportAliasRule {
    type Config = NoUnneededImportAliasConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Unneeded Import Alias",
            code: "no-unneeded-import-alias",
            description: indoc! {"
                Detects `use` statements with an alias that matches the imported class name.

                `use Foo\\Bar as Bar` is identical to `use Foo\\Bar` and the alias is redundant.
            "},
            good_example: indoc! {r"
                <?php

                use Foo\Bar;
                use Baz\Qux as RenamedQux;
            "},
            bad_example: indoc! {r"
                <?php

                use Foo\Bar as Bar;
            "},
            category: Category::Redundancy,
            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::UseItem];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'_, 'arena>) {
        let Node::UseItem(use_item) = node else {
            return;
        };

        let Some(ref alias) = use_item.alias else {
            return;
        };

        let source_code = ctx.source_file.contents.as_ref();

        // Get the last segment of the imported name
        let name_span = use_item.name.span();
        let name_text = &source_code[name_span.start_offset() as usize..name_span.end_offset() as usize];
        let last_segment = name_text.rsplit('\\').next().unwrap_or(name_text);

        // Get the alias name
        let alias_span = alias.identifier.span();
        let alias_text = &source_code[alias_span.start_offset() as usize..alias_span.end_offset() as usize];

        if last_segment == alias_text {
            let issue = Issue::new(self.cfg.level(), format!("Unnecessary import alias `as {}`", alias_text))
                .with_code(self.meta.code)
                .with_annotation(
                    Annotation::primary(alias.r#as.span().join(alias_span))
                        .with_message("This alias matches the class name and is redundant"),
                )
                .with_help("Remove the `as` clause");

            ctx.collector.propose(issue, |edits| {
                // Remove the ` as Alias` part (from before `as` keyword to end of alias)
                edits.push(TextEdit::delete(TextRange::new(name_span.end_offset(), alias_span.end_offset())));
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::NoUnneededImportAliasRule;
    use crate::test_lint_failure;
    use crate::test_lint_success;

    test_lint_success! {
        name = no_alias_is_ok,
        rule = NoUnneededImportAliasRule,
        code = indoc! {r"
            <?php

            use Foo\Bar;
        "}
    }

    test_lint_success! {
        name = different_alias_is_ok,
        rule = NoUnneededImportAliasRule,
        code = indoc! {r"
            <?php

            use Foo\Bar as Baz;
        "}
    }

    test_lint_failure! {
        name = same_alias_is_bad,
        rule = NoUnneededImportAliasRule,
        code = indoc! {r"
            <?php

            use Foo\Bar as Bar;
        "}
    }
}
