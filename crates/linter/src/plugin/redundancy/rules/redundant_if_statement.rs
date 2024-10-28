use fennec_ast::*;
use fennec_fixer::SafetyClassification;
use fennec_reporting::*;
use fennec_span::HasSpan;
use fennec_walker::Walker;

use crate::context::LintContext;
use crate::rule::Rule;

#[derive(Clone, Debug)]
pub struct RedundantIfStatementRule;

impl Rule for RedundantIfStatementRule {
    fn get_name(&self) -> &'static str {
        "redundant-if-statement"
    }

    fn get_default_level(&self) -> Option<Level> {
        Some(Level::Help)
    }
}

impl<'a> Walker<LintContext<'a>> for RedundantIfStatementRule {
    fn walk_in_if<'ast>(&self, r#if: &'ast If, context: &mut LintContext<'a>) {
        if fennec_ast_utils::is_truthy(&r#if.condition) {
            // this condition always evaluates, given:
            //
            // if ($expr) { block } elseif ($expr2) { block2 } else { block3 }
            // if ($expr) { block } else { block2 }
            // if ($expr): block elseif ($expr2): block2 else: block3 endif
            //
            // reduce it to:
            //
            // block
            // block
            // block
            let issue = Issue::new(context.level(), "unnecessary `if` statement")
                .with_annotations([
                    Annotation::primary(r#if.condition.span()).with_message("this condition always evaluates to true"),
                    Annotation::secondary(r#if.span()),
                ])
                .with_note("this `if` statement's condition always evaluates to true.")
                .with_note("the `if` statement can be removed, and its body can be executed unconditionally.")
                .with_help("remove the unnecessary `if` statement and execute its body directly.");

            context.report_with_fix(issue, |plan| {
                let mut plan =
                    plan.delete(r#if.r#if.span.join(r#if.right_parenthesis).to_range(), SafetyClassification::Safe);

                match &r#if.body {
                    IfBody::Statement(if_statement_body) => {
                        for clause in if_statement_body.else_if_clauses.iter() {
                            plan = plan.delete(clause.span().to_range(), SafetyClassification::Safe);
                        }

                        if let Some(else_clause) = &if_statement_body.else_clause {
                            plan = plan.delete(else_clause.span().to_range(), SafetyClassification::Safe);
                        }

                        plan
                    }
                    IfBody::ColonDelimited(if_colon_delimited_body) => {
                        plan = plan.delete(if_colon_delimited_body.colon.to_range(), SafetyClassification::Safe);

                        for clause in if_colon_delimited_body.else_if_clauses.iter() {
                            plan = plan.delete(clause.span().to_range(), SafetyClassification::Safe);
                        }

                        if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                            plan = plan.delete(else_clause.span().to_range(), SafetyClassification::Safe);
                        }

                        plan.delete(if_colon_delimited_body.endif.span().to_range(), SafetyClassification::Safe)
                            .delete(if_colon_delimited_body.terminator.span().to_range(), SafetyClassification::Safe)
                    }
                }
            });

            return;
        }

        if fennec_ast_utils::is_falsy(&r#if.condition) {
            // if the `if` statement has no else if/else clauses, and the body contains only
            // definitions, then we should not report it as redundant.
            //
            // if (false) {
            //    class Foo {}
            // }
            //
            // this is a common pattern used by PHP libraries to provide a stub to be
            //  used by IDEs for code completion.
            match &r#if.body {
                IfBody::Statement(if_statement_body) => {
                    if if_statement_body.else_if_clauses.is_empty()
                        && if_statement_body.else_clause.is_none()
                        && fennec_ast_utils::statement_contains_only_definitions(&if_statement_body.statement)
                    {
                        return;
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    if if_colon_delimited_body.else_if_clauses.is_empty()
                        && if_colon_delimited_body.else_clause.is_none()
                        && fennec_ast_utils::statement_sequence_contains_only_definitions(
                            &if_colon_delimited_body.statements,
                        )
                    {
                        return;
                    }
                }
            }

            // this condition always skipped, given:
            //
            // if ($expr) { block } elseif ($expr2) { block2 } else { block3 }
            // if ($expr) { block } else { block2 }
            // if ($expr): block elseif ($expr2): block2 else: block3 endif
            // if ($expr): block else: block2 endif
            //
            // reduce it to:
            //
            // if ($expr2) { block2 } else { block3 }
            // block2
            // block2
            let issue = Issue::new(context.level(), "unnecessary `if` statement")
                .with_annotations([
                    Annotation::primary(r#if.condition.span())
                        .with_message("this condition always evaluates to false."),
                    Annotation::secondary(r#if.span()),
                ])
                .with_note("this `if` statement's condition always evaluates to false.")
                .with_note("the `if` statement can be removed, and its body can be skipped.")
                .with_help("remove the unnecessary `if` statement and skip its body.");

            context.report_with_fix(issue, |plan| match &r#if.body {
                IfBody::Statement(if_statement_body) => {
                    if let Some(else_if_clause) = if_statement_body.else_if_clauses.first() {
                        let span = r#if.r#if.span.join(else_if_clause.elseif.span());

                        plan.delete(span.start.offset..(span.end.offset - 2), SafetyClassification::Safe)
                    } else if let Some(else_clause) = &if_statement_body.else_clause {
                        let span = r#if.r#if.span.join(else_clause.r#else.span());

                        plan.delete(span.to_range(), SafetyClassification::Safe)
                    } else {
                        plan.delete(r#if.span().to_range(), SafetyClassification::Safe)
                    }
                }
                IfBody::ColonDelimited(if_colon_delimited_body) => {
                    if let Some(else_if_clause) = if_colon_delimited_body.else_if_clauses.first() {
                        let span = r#if.r#if.span.join(else_if_clause.elseif.span());

                        plan.delete(span.start.offset..(span.end.offset - 2), SafetyClassification::Safe)
                    } else if let Some(else_clause) = &if_colon_delimited_body.else_clause {
                        plan.delete(
                            r#if.r#if.span.join(else_clause.colon.span()).to_range(),
                            SafetyClassification::Safe,
                        )
                        .delete(if_colon_delimited_body.endif.span().to_range(), SafetyClassification::Safe)
                        .delete(if_colon_delimited_body.terminator.span().to_range(), SafetyClassification::Safe)
                    } else {
                        plan.delete(r#if.span().to_range(), SafetyClassification::Safe)
                    }
                }
            });
        }
    }
}
