use mago_atom::atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeConstant;
use mago_syntax::ast::ClassLikeConstantItem;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::statement::attributes::AttributeTarget;
use crate::statement::attributes::analyze_attributes;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstant<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_attributes(
            context,
            block_context,
            artifacts,
            self.attribute_lists.as_slice(),
            AttributeTarget::ClassLikeConstant,
        );

        for item in &self.items {
            item.analyze(context, block_context, artifacts)?;
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for ClassLikeConstantItem<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        self.value.analyze(context, block_context, artifacts)?;

        if let Some(class_metadata) = block_context.scope.get_class_like()
            && let Some(constant_metadata) = class_metadata.constants.get(&atom(self.name.value))
            && let Some(inferred_type) = constant_metadata.inferred_type.as_ref()
            && inferred_type.is_never()
        {
            context.collector.report_with_code(
                IssueCode::UnresolvableClassConstant,
                Issue::error(format!(
                    "Cannot resolve the value of class constant `{}::{}`.",
                    class_metadata.original_name, self.name.value,
                ))
                .with_annotation(
                    Annotation::primary(self.value.span()).with_message("This initializer could not be evaluated"),
                )
                .with_note("Mago could not determine a value for this constant from its initializer."),
            );
        }

        Ok(())
    }
}
