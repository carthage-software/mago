use indoc::indoc;

use mago_ast::*;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::Span;
use mago_walker::Walker;

use crate::context::LintContext;
use crate::definition::RuleDefinition;
use crate::definition::RuleUsageExample;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantNoopRule;

impl Rule for RedundantNoopRule {
    fn get_definition(&self) -> RuleDefinition {
        RuleDefinition::enabled("Redundant Noop", Level::Help)
            .with_description(indoc! {"
                Detects redundant `noop` statements.
            "})
            .with_example(RuleUsageExample::invalid(
                "A redundant `noop` statement",
                indoc! {r#"
                    <?php

                    ;
                "#},
            ))
    }
}

impl RedundantNoopRule {
    fn report(&self, noop: &Span, context: &mut LintContext<'_>) {
        let issue = Issue::new(context.level(), "Redundant noop statement.")
            .with_annotation(Annotation::primary(*noop).with_message("This is a redundant `noop` statement."))
            .with_help("Remove the redundant `;`");

        context.report_with_fix(issue, |plan| plan.delete(noop.to_range(), SafetyClassification::Safe));
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantNoopRule {
    fn walk_in_program<'ast>(&self, program: &'ast Program, context: &mut LintContext<'a>) {
        for statement in program.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_block<'ast>(&self, block: &'ast Block, context: &mut LintContext<'a>) {
        for statement in block.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_namespace<'ast>(&self, namespace: &'ast Namespace, context: &mut LintContext<'a>) {
        for statement in namespace.statements().iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_declare_colon_delimited_body<'ast>(
        &self,
        declare_colon_delimited_body: &'ast DeclareColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in declare_colon_delimited_body.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_switch_expression_case<'ast>(
        &self,
        switch_expression_case: &'ast SwitchExpressionCase,
        context: &mut LintContext<'a>,
    ) {
        for statement in switch_expression_case.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_switch_default_case<'ast>(
        &self,
        switch_default_case: &'ast SwitchDefaultCase,
        context: &mut LintContext<'a>,
    ) {
        for statement in switch_default_case.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_foreach_colon_delimited_body<'ast>(
        &self,
        foreach_colon_delimited_body: &'ast ForeachColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in foreach_colon_delimited_body.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_in_while_colon_delimited_body<'ast>(
        &self,
        while_colon_delimited_body: &'ast WhileColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in while_colon_delimited_body.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_for_colon_delimited_body<'ast>(
        &self,
        for_colon_delimited_body: &'ast ForColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in for_colon_delimited_body.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_if_colon_delimited_body<'ast>(
        &self,
        if_colon_delimited_body: &'ast IfColonDelimitedBody,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_if_colon_delimited_body_else_if_clause<'ast>(
        &self,
        if_colon_delimited_body_else_if_clause: &'ast IfColonDelimitedBodyElseIfClause,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body_else_if_clause.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }

    fn walk_if_colon_delimited_body_else_clause<'ast>(
        &self,
        if_colon_delimited_body_else_clause: &'ast IfColonDelimitedBodyElseClause,
        context: &mut LintContext<'a>,
    ) {
        for statement in if_colon_delimited_body_else_clause.statements.iter() {
            if let Statement::Noop(noop) = statement {
                self.report(noop, context);
            }
        }
    }
}
