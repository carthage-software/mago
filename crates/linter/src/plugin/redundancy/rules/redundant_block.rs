use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
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
}

impl RedundantBlockRule {
    fn report(&self, block: &Block, context: &mut LintContext<'_>) {
        let issue = Issue::new(context.level(), "Redundant block around statements")
            .with_annotations([
                Annotation::primary(block.span()).with_message("Statements do not need to be wrapped within a block.")
            ])
            .with_help("Remove the block to simplify the code.");

        context.report_with_fix(issue, |plan| {
            plan.delete(block.left_brace.to_range(), SafetyClassification::Safe);
            plan.delete(block.right_brace.to_range(), SafetyClassification::Safe);
        });
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantBlockRule {
    fn walk_in_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        for statement in program.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_block<'ast>(&self, block: &'ast Block, context: &mut LintContext<'a>) {
        for statement in block.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_namespace<'ast>(&self, namespace: &'ast Namespace, context: &mut LintContext<'a>) {
        for statement in namespace.statements().iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_declare_colon_delimited_body<'ast>(
        &self,
        declare_colon_delimited_body: &'ast DeclareColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in declare_colon_delimited_body.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_switch_expression_case<'ast>(
        &self,
        switch_expression_case: &'ast SwitchExpressionCase,
        context: &mut LintContext<'a>,
    ) {
        for statement in switch_expression_case.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_switch_default_case<'ast>(
        &self,
        switch_default_case: &'ast SwitchDefaultCase,
        context: &mut LintContext<'a>,
    ) {
        for statement in switch_default_case.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_foreach_colon_delimited_body<'ast>(
        &self,
        foreach_colon_delimited_body: &'ast ForeachColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in foreach_colon_delimited_body.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_in_while_colon_delimited_body<'ast>(
        &self,
        while_colon_delimited_body: &'ast WhileColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in while_colon_delimited_body.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_for_colon_delimited_body<'ast>(
        &self,
        for_colon_delimited_body: &'ast ForColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in for_colon_delimited_body.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_if_colon_delimited_body<'ast>(
        &self,
        if_colon_delimited_body: &'ast IfColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_if_colon_delimited_body_else_if_clause<'ast>(
        &self,
        if_colon_delimited_body_else_if_clause: &'ast IfColonDelimitedBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body_else_if_clause.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }

    fn walk_if_colon_delimited_body_else_clause<'ast>(
        &self,
        if_colon_delimited_body_else_clause: &'ast IfColonDelimitedBodyElseClause,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body_else_clause.statements.iter() {
            if let Statement::Block(inner) = statement {
                self.report(inner, context);
            }
        }
    }
}
