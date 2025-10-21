use indoc::indoc;
use mago_fixer::SafetyClassification;
use serde::Deserialize;
use serde::Serialize;

use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_reporting::Level;
use mago_syntax::ast::*;

use crate::category::Category;
use crate::context::LintContext;
use crate::requirements::RuleRequirements;
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
            requirements: RuleRequirements::None,
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

    fn build(settings: &RuleSettings<Self::Config>) -> Self {
        Self { meta: Self::meta(), cfg: settings.config }
    }

    fn check<'ast, 'arena>(&self, ctx: &mut LintContext<'_, 'arena>, node: Node<'ast, 'arena>) {
        let statements = match node {
            Node::Program(node) => node.statements.as_slice(),
            Node::Block(node) => node.statements.as_slice(),
            Node::Namespace(node) => node.statements().as_slice(),
            Node::DeclareColonDelimitedBody(node) => node.statements.as_slice(),
            Node::SwitchExpressionCase(node) => node.statements.as_slice(),
            Node::SwitchDefaultCase(node) => node.statements.as_slice(),
            Node::ForeachColonDelimitedBody(node) => node.statements.as_slice(),
            Node::WhileColonDelimitedBody(node) => node.statements.as_slice(),
            Node::ForColonDelimitedBody(node) => node.statements.as_slice(),
            Node::IfColonDelimitedBody(node) => node.statements.as_slice(),
            Node::IfColonDelimitedBodyElseIfClause(node) => node.statements.as_slice(),
            Node::IfColonDelimitedBodyElseClause(node) => node.statements.as_slice(),
            _ => return,
        };

        for statement in statements {
            if let Statement::Noop(noop) = statement {
                let issue = Issue::new(self.cfg.level(), "Redundant noop statement.")
                    .with_code(self.meta.code)
                    .with_annotation(Annotation::primary(*noop).with_message("This is a redundant `noop` statement"))
                    .with_help("Remove the redundant `;`.");

                ctx.collector.propose(issue, |plan| {
                    plan.delete(noop.to_range(), SafetyClassification::Safe);
                });
            }
        }
    }
}
