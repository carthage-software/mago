use indoc::indoc;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersionRange;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::integration::IntegrationSet;
use crate::rule::Config;
use crate::rule::LintRule;
use crate::rule_meta::RuleMeta;
use crate::settings::RuleSettings;

#[derive(Debug, Clone)]
pub struct NoNoopRule {
    meta: &'static RuleMeta,
    cfg: NoNoopConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct NoNoopConfig {
    pub level: Level,
}

impl Default for NoNoopConfig {
    fn default() -> Self {
        Self { level: Level::Help }
    }
}

impl Config for NoNoopConfig {
    fn level(&self) -> Level {
        self.level
    }
}

impl LintRule for NoNoopRule {
    type Config = NoNoopConfig;

    fn meta() -> &'static RuleMeta {
        const META: RuleMeta = RuleMeta {
            name: "No Noop",
            code: "no-noop",
            description: indoc! {"
                Detects redundant `noop` statements.
            "},
            good_example: indoc! {r#"
                <?php

                echo "Hello, world!";
            "#},
            bad_example: indoc! {r#"
                <?php

                ;
            "#},
            category: Category::Redundancy,
            php: PHPVersionRange::any(),
            requires: IntegrationSet::empty(),
        };

        &META
    }

    fn targets() -> &'static [NodeKind] {
        const TARGETS: &[NodeKind] = &[
            NodeKind::Program,
            NodeKind::Block,
            NodeKind::Namespace,
            NodeKind::DeclareColonDelimitedBody,
            NodeKind::SwitchExpressionCase,
            NodeKind::SwitchDefaultCase,
            NodeKind::ForeachColonDelimitedBody,
            NodeKind::WhileColonDelimitedBody,
            NodeKind::ForColonDelimitedBody,
            NodeKind::IfColonDelimitedBody,
            NodeKind::IfColonDelimitedBodyElseIfClause,
            NodeKind::IfColonDelimitedBodyElseClause,
        ];

        TARGETS
    }

    fn build(settings: RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let statements = match node {
            Node::Program(program) => program.statements.as_slice(),
            Node::Block(block) => block.statements.as_slice(),
            Node::Namespace(namespace) => namespace.statements().as_slice(),
            Node::DeclareColonDelimitedBody(declare_colon_delimited_body) => {
                declare_colon_delimited_body.statements.as_slice()
            }
            Node::SwitchExpressionCase(switch_expression_case) => switch_expression_case.statements.as_slice(),
            Node::SwitchDefaultCase(switch_default_case) => switch_default_case.statements.as_slice(),
            Node::ForeachColonDelimitedBody(foreach_colon_delimited_body) => {
                foreach_colon_delimited_body.statements.as_slice()
            }
            Node::WhileColonDelimitedBody(while_colon_delimited_body) => {
                while_colon_delimited_body.statements.as_slice()
            }
            Node::ForColonDelimitedBody(for_colon_delimited_body) => for_colon_delimited_body.statements.as_slice(),
            Node::IfColonDelimitedBody(if_colon_delimited_body) => if_colon_delimited_body.statements.as_slice(),
            Node::IfColonDelimitedBodyElseIfClause(if_colon_delimited_body_else_if_clause) => {
                if_colon_delimited_body_else_if_clause.statements.as_slice()
            }
            Node::IfColonDelimitedBodyElseClause(if_colon_delimited_body_else_clause) => {
                if_colon_delimited_body_else_clause.statements.as_slice()
            }
            _ => return,
        };

        for statement in statements {
            if let Statement::Noop(noop) = statement {
                let issue = Issue::new(self.cfg.level(), "Redundant noop statement.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(*noop).with_message("This is a redundant `noop` statement"))
                    .with_help("Remove the redundant `;`.");

                ctx.collector.report(issue);
            }
        }
    }
}
