use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_syntax::ast::ClassLikeMember;
use mago_syntax::ast::FunctionLikeParameter;
use mago_syntax::ast::Statement;

use crate::context::Context;
use crate::statement::function_like::FunctionLikeBody;

mod avoid_catching_error;
mod override_attribute;
mod unused_parameter;

pub fn check_function_like<'ctx, 'ast, 'arena>(
    metadata: &'ctx FunctionLikeMetadata,
    params: &'ast [FunctionLikeParameter<'arena>],
    body: FunctionLikeBody<'ast, 'arena>,
    ctx: &mut Context<'ctx, 'arena>,
) {
    if !ctx.settings.perform_heuristic_checks {
        return;
    }

    unused_parameter::check_unused_params(metadata, params, body, ctx);
}

pub fn check_class_like<'ctx, 'arena>(
    metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
    ctx: &mut Context<'ctx, 'arena>,
) {
    if !ctx.settings.perform_heuristic_checks {
        return;
    }

    override_attribute::check_override_attribute(metadata, members, ctx);
}

pub fn check_statement<'ctx, 'ast, 'arena>(stmt: &'ast Statement<'arena>, ctx: &mut Context<'ctx, 'arena>) {
    if !ctx.settings.perform_heuristic_checks {
        return;
    }

    if let Statement::Try(r#try) = stmt {
        avoid_catching_error::check_for_caught_error(r#try, ctx);
    }
}
