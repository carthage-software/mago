use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_string;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::common::construct::ConstructInput;
use crate::common::construct::analyze_construct_inputs;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for IncludeConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.include.span,
            self.value,
            true,  // is_include
            false, // is_once
        )
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for IncludeOnceConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.include_once.span,
            self.value,
            true, // is_include
            true, // is_once
        )
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for RequireConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.require.span,
            self.value,
            false, // is_include
            false, // is_once
        )
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for RequireOnceConstruct<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_include(
            context,
            block_context,
            artifacts,
            self.span(),
            self.require_once.span,
            self.value,
            false, // is_include
            true,  // is_once
        )
    }
}

fn analyze_include<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    construct_span: Span,
    keyword_span: Span,
    included_file: &Expression<'arena>,
    is_include: bool,
    is_once: bool,
) -> Result<(), AnalysisError> {
    let was_inside_call = block_context.inside_call;
    block_context.inside_call = true;
    included_file.analyze(context, block_context, artifacts)?;
    block_context.inside_call = was_inside_call;

    let construct_kind = if is_include {
        if is_once { "include_once" } else { "include" }
    } else if is_once {
        "require_once"
    } else {
        "require"
    };

    analyze_construct_inputs(
        context,
        block_context,
        artifacts,
        construct_kind,
        keyword_span,
        ConstructInput::Expression(included_file),
        get_string(),
        false,
        false,
        true,
    )?;

    artifacts.set_expression_type(&construct_span, get_mixed());

    Ok(())
}
