use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;

impl Analyzable for StaticPropertyAccess {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        _block_context: &mut BlockContext<'a>,
        _artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        context.buffer.report(
            TypingIssueKind::UnsupportedFeature,
            Issue::warning("Analysis for static property access expression is not yet implemented.")
                .with_annotation(
                    Annotation::primary(self.span()).with_message("This expression will be skipped during analysis"),
                )
                .with_note("Support for static property access expression is planned for a future release.")
                .with_note("If you need this feature, please open an issue on the project's GitHub repository."),
        );

        Ok(())
    }
}
