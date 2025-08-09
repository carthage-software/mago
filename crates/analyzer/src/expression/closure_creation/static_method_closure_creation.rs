use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::get_mixed_closure;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::static_method::resolve_static_method_targets;

impl Analyzable for StaticMethodClosureCreation {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let method_resolution =
            resolve_static_method_targets(context, block_context, artifacts, &self.class, &self.method)?;

        let mut callables = vec![];
        for resolved_method in method_resolution.resolved_methods {
            callables.push(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Method(
                *resolved_method.method_identifier.get_class_name(),
                *resolved_method.method_identifier.get_method_name(),
            ))));
        }

        let resulting_type = if callables.is_empty() {
            if method_resolution.has_invalid_target { get_never() } else { get_mixed_closure() }
        } else {
            TUnion::new(callables)
        };

        artifacts.set_expression_type(self, resulting_type);

        Ok(())
    }
}
