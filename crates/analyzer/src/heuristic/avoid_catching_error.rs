use mago_codex::is_instance_of;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::code::IssueCode;
use crate::context::Context;
use crate::statement::r#try::get_caught_classes;

const ERROR_CLASS: &str = "Error";

pub fn check_for_caught_error<'ctx, 'ast, 'arena>(r#try: &'ast Try<'arena>, context: &mut Context<'ctx, 'arena>) {
    for catch_clause in r#try.catch_clauses.iter() {
        let caught_classes = get_caught_classes(context, &catch_clause.hint);

        for caught in caught_classes {
            if !is_instance_of(context.codebase, &caught, ERROR_CLASS) {
                continue;
            }

            context.collector.report_with_code(
                IssueCode::AvoidCatchingError,
                Issue::warning("Avoid catching 'Error' class instances.")
                    .with_annotation(
                        Annotation::primary(catch_clause.hint.span()).with_message(
                            "This throwable is an instance of the `Error` class or one of its sub-classes.",
                        ),
                    )
                    .with_annotation(
                        Annotation::secondary(catch_clause.block.span())
                            .with_message("This catch clause intercepts a critical error."),
                    )
                    .with_note("Catching these errors hides issues that should crash your app.")
                    .with_help("Remove or adjust this catch clause so errors propagate naturally."),
            );
        }
    }
}
