use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoClosingTagRule {
    meta: &'static RuleMeta,
    cfg: NoClosingTagConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoClosingTagConfig {
    pub level: Level,
}

impl Default for NoClosingTagConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoClosingTagConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoClosingTagRule {
    type Config = NoClosingTagConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Closing Tag",
            code: "no-closing-tag",
            description: indoc! {"
                Detects redundant closing tags ( `?>` ) at the end of a file.
            "},
            good_example: indoc! {r#"
                <?php

                echo "Hello, world!";
            "#},
            bad_example: indoc! {r#"
                <?php

                echo "Hello, world!";

                ?>
            "#},
            category: Category::Redundancy,

            requirements: RuleRequirements::None,
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[NodeKind::Program];

        TARGETS
    }

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let Node::Program(program) = node else {
            return;
        };

        self.check_statements(&program.statements, ctx);
    }
}

impl NoClosingTagRule {
    fn check_statements(&self, sequence: &Sequence<Statement>, ctx: &mut LintContext) {
        let Some(last_statement) = sequence.last() else {
            return;
        };

        if let Statement::ClosingTag(closing_tag) = last_statement {
            let issue = Issue::new(self.cfg.level(), "Redundant closing tag ( `?>` ).")
                .with_code(self.meta.code)
                .with_annotation(Annotation::primary(closing_tag.span()).with_message("This closing tag is redundant"))
                .with_help("Remove the redundant closing tag ( `?>` ).");

            ctx.collector.report(issue);

            return;
        }

        if let Statement::Inline(inline) = last_statement {
            let stmts_len = sequence.len();
            if stmts_len < 2 {
                return;
            }

            if inline.value.bytes().all(|b| b.is_ascii_whitespace()) {
                let Some(Statement::ClosingTag(tag)) = sequence.get(stmts_len - 2) else {
                    return;
                };

                let issue =
                    Issue::new(self.cfg.level(), "Redundant closing tag ( `?>` ) followed by trailing whitespace.")
                        .with_code(self.meta.code)
                        .with_annotation(Annotation::primary(tag.span()).with_message("This closing tag is redundant"))
                        .with_annotation(
                            Annotation::secondary(inline.span())
                                .with_message("This inline statement contains only whitespace"),
                        )
                        .with_help("Remove the redundant closing tag ( `?>` ) and trailing whitespace.");

                ctx.collector.report(issue);
            }
        }

        if let Statement::Namespace(namespace) = last_statement {
            match &namespace.body {
                NamespaceBody::Implicit(namespace_implicit_body) => {
                    self.check_statements(&namespace_implicit_body.statements, ctx);
                }
                NamespaceBody::BraceDelimited(_) => {}
            }
        }
    }
}
