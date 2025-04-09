use indoc::indoc;

use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::directive::LintDirective;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantBlockRule;

impl Rule for RedundantBlockRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Block", Level::Help)
            .with_description(indoc! {"
                Detects redundant blocks around statements.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant block around a statement",
                indoc! {r#"
                    <?php

                    {
                        echo "Hello, world!";
                    }
                "#},
            ))
    }

    fn lint_node(&self, node: Node<'_>, context: &mut LintContext<'_>) -> LintDirective {
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
            _ => return LintDirective::default(),
        };

        for statement in statements {
            if let Statement::Block(block) = statement {
                let issue = Issue::new(context.level(), "Redundant block around statements")
                    .with_annotations([Annotation::primary(block.span())
                        .with_message("Statements do not need to be wrapped within a block.")])
                    .with_help("Remove the block to simplify the code.");

                context.propose(issue, |plan| {
                    plan.delete(block.left_brace.to_range(), SafetyClassification::Safe);
                    plan.delete(block.right_brace.to_range(), SafetyClassification::Safe);
                });
            }
        }

        LintDirective::default()
    }
}
