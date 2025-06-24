use ahash::HashMap;
use mago_codex::ttype::template::TemplateResult;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::invocation::Invocation;
use crate::invocation::InvocationArgumentsSource;
use crate::invocation::InvocationTarget;
use crate::invocation::LanguageConstructKind;
use crate::invocation::analyzer::analyze_invocation;
use crate::invocation::return_type_fetcher::fetch_invocation_return_type;

impl Analyzable for Echo {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let target = InvocationTarget::for_language_construct(LanguageConstructKind::Echo, self.echo.span);
        let arguments = InvocationArgumentsSource::LanguageConstructExpressions(self.values.as_slice());
        let invocation = Invocation::new(target, arguments, self.span());

        let mut template_result = TemplateResult::default();
        let mut argument_types = HashMap::default();

        analyze_invocation(
            context,
            block_context,
            artifacts,
            &invocation,
            None,
            &mut template_result,
            &mut argument_types,
        )?;

        let return_type =
            fetch_invocation_return_type(context, artifacts, &invocation, &template_result, &argument_types);
        artifacts.set_expression_type(self, return_type);

        Ok(())
    }
}
